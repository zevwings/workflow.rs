//! 更新命令实现
//!
//! 提供从 GitHub Releases 更新 Workflow CLI 的功能。
//! 备份与回滚路径从 pathService 获取，参考 toolkit rollback 单文件流程。

use std::path::PathBuf;

use prompt::{br, error, info, print, success, warning, ConfirmBuilder};
use toolkit::{backup, cleanup_backup, rollback, Platform};

use super::{
    download::{build_download_url, download_file, extract_archive, verify_file_checksum},
    types::TempDirManager,
    verify::{run_installer, verify_installation},
    version::{compare_versions, get_current_version, get_target_version, VersionComparison},
};
use crate::bootstrap::get_path_service;

/// 更新命令
pub struct UpdateCommand {
    /// 目标版本（None 表示最新版本）
    target_version: Option<String>,
    /// 是否跳过确认
    force: bool,
    /// GitHub token（用于提高 API 速率限制）
    github_token: Option<String>,
}

impl UpdateCommand {
    /// 创建新的 UpdateCommand 实例
    pub fn new(target_version: Option<String>, force: bool, github_token: Option<String>) -> Self {
        Self {
            target_version,
            force,
            github_token,
        }
    }

    /// 运行更新命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Workflow CLI update...");
        br!();

        // 获取当前版本
        let current_version = get_current_version();
        if let Some(ref current) = current_version {
            info!("Current version: v{}", current);
        } else {
            warning!("Unable to detect current version, will continue update process");
        }
        br!();

        // 检测平台
        let platform = Platform::detect().release_identifier()?;
        info!("Detected platform: {}", platform);
        br!();

        // 获取目标版本号
        let target_version =
            get_target_version(self.target_version.clone(), self.github_token.as_deref())?;

        // 比较版本
        if let Some(ref current) = current_version {
            match compare_versions(current, &target_version) {
                VersionComparison::UpToDate => {
                    success!("Already at latest version (v{}), no update needed", current);
                    return Ok(());
                }
                VersionComparison::NeedsUpdate => {
                    info!("New version found: v{} -> v{}", current, target_version);
                }
                VersionComparison::Downgrade => {
                    warning!(
                        "Target version (v{}) is lower than current version (v{})",
                        target_version,
                        current
                    );
                    warning!("This will perform a downgrade operation");
                }
            }
        } else {
            info!("Target version: v{}", target_version);
        }
        br!();

        // 获取用户确认
        if !self.force {
            let confirm_message = if let Some(ref current) = current_version {
                format!(
                    "Are you sure you want to update Workflow CLI?\n  Current version: v{}\n  Target version: v{}",
                    current, target_version
                )
            } else {
                format!(
                    "Are you sure you want to update Workflow CLI to v{}?",
                    target_version
                )
            };

            let confirmed = ConfirmBuilder::new(&confirm_message)
                .default(true)
                .prompt()
                .map_err(|e| format!("Failed to get confirmation: {}", e))?;

            if !confirmed {
                print!("Update cancelled");
                return Ok(());
            }
            br!();
        }

        // 创建备份
        let backup_dir = self.create_backup();
        br!();

        // 准备临时目录
        let temp_manager = TempDirManager::new(&target_version, &platform)?;
        let download_url = build_download_url(&target_version, &platform);
        info!("Download URL: {}", download_url);
        br!();

        // 执行更新操作
        let update_result = self.perform_update(&temp_manager, &download_url, &target_version);

        if let Some(backup_dir) = backup_dir {
            match update_result {
                Ok(()) => {
                    // 更新成功，清理资源
                    success!("Workflow CLI update complete! All verifications passed.");
                    Ok(())
                }
                Err(e) => {
                    // 更新失败，执行回滚
                    error!("Update failed: {}", e);
                    br!();

                    self.perform_rollback(backup_dir);

                    Err(format!("Update failed: {}", e).into())
                }
            }
        } else {
            error!("Failed to create backup");
            Err("Failed to create backup".into())
        }
    }

    /// 创建备份（从 pathService 获取二进制路径，单文件备份）
    fn create_backup(&self) -> Option<PathBuf> {
        let path_service = get_path_service();
        let install_dir = match path_service.get_binary_install_dir() {
            Ok(d) => d,
            Err(e) => {
                warning!("Failed to get install dir: {}", e);
                return None;
            }
        };
        let bin_name = match path_service.get_binary_name() {
            Ok(n) => n,
            Err(e) => {
                warning!("Failed to get binary name: {}", e);
                return None;
            }
        };
        match backup(bin_name.as_str(), install_dir.clone()) {
            Ok(backup_dir) => {
                info!(
                    "Backup created: {} -> {}",
                    install_dir.display(),
                    backup_dir.display()
                );
                Some(backup_dir)
            }
            Err(e) => {
                warning!("Failed to create backup: {}", e);
                warning!("Will continue update, but cannot rollback on failure");
                warning!("If update fails, manual recovery may be required");
                None
            }
        }
    }

    /// 执行更新操作
    fn perform_update(
        &self,
        temp_manager: &TempDirManager,
        download_url: &str,
        _target_version: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 下载文件
        download_file(download_url, &temp_manager.archive_path)?;
        br!();

        // 验证文件完整性
        verify_file_checksum(&temp_manager.archive_path, download_url)?;
        br!();

        // 解压文件
        extract_archive(&temp_manager.archive_path, &temp_manager.extract_dir)?;
        br!();

        // 安装
        run_installer(&temp_manager.extract_dir)?;
        br!();

        // 验证安装结果
        let verification_result = verify_installation()?;
        br!();

        if !verification_result.all_checks_passed {
            return Err("Installation verification failed, some checks did not pass".into());
        }

        Ok(())
    }

    /// 执行回滚（使用 pathService 的 install_dir，单文件恢复）
    fn perform_rollback(&self, backup_dir: PathBuf) {
        warning!("Update failed, rolling back to previous version...");
        br!();

        let install_dir = match get_path_service().get_binary_install_dir() {
            Ok(d) => d,
            Err(e) => {
                error!("Cannot get install dir for rollback: {}", e);
                error!("Backup location: {}", backup_dir.display());
                return;
            }
        };

        let bin_name = match get_path_service().get_binary_name() {
            Ok(n) => n,
            Err(e) => {
                error!("Cannot get binary name for rollback: {}", e);
                error!("Backup location: {}", backup_dir.display());
                return;
            }
        };

        match rollback(bin_name.as_str(), backup_dir.clone(), install_dir) {
            Ok(()) => {
                info!("  Restored: {}", bin_name);
                success!("Rollback completed");
                br!();
                if let Err(cleanup_err) = cleanup_backup(backup_dir) {
                    warning!("Failed to clean up backup: {}", cleanup_err);
                }
            }
            Err(rollback_err) => {
                error!("Rollback failed: {}", rollback_err);
                error!("System may be in an inconsistent state");
                error!("Please manually check and restore files");
                error!("Backup location: {}", backup_dir.display());
            }
        }
    }
}
