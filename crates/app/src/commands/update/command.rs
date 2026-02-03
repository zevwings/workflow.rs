//! 更新命令实现
//!
//! 提供从 GitHub Releases 更新 Workflow CLI 的功能。

use color_eyre::{eyre::WrapErr, Result};
use prompt::{br, error, info, print, success, warning, ConfirmBuilder};
use toolkit::{BackupResult, Platform, RollbackManager};

use super::download::{build_download_url, download_file, extract_archive, verify_checksum};
use super::types::TempDirManager;
use super::verify::{run_installer, verify_installation};
use super::version::{
    compare_versions, get_current_version, get_target_version, VersionComparison,
};

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
    pub fn run(&self) -> Result<()> {
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
                .wrap_err("Failed to get confirmation")?;

            if !confirmed {
                print!("Update cancelled");
                return Ok(());
            }
            br!();
        }

        // 创建备份
        let backup_info = self.create_backup();
        br!();

        // 准备临时目录
        let temp_manager = TempDirManager::new(&target_version, &platform)?;
        let download_url = build_download_url(&target_version, &platform);
        info!("Download URL: {}", download_url);
        br!();

        // 执行更新操作
        let update_result = self.perform_update(&temp_manager, &download_url, &target_version);

        // 处理更新结果
        self.handle_update_result(update_result, backup_info.as_ref(), &temp_manager)
    }

    /// 创建备份
    fn create_backup(&self) -> Option<BackupResult> {
        match RollbackManager::create_backup() {
            Ok(backup) => {
                info!(
                    "Backup created: {} binary(ies), {} completion(s)",
                    backup.binary_count, backup.completion_count
                );
                Some(backup)
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
    ) -> Result<()> {
        // 下载文件
        download_file(download_url, &temp_manager.archive_path)?;
        br!();

        // 验证文件完整性
        verify_checksum(&temp_manager.archive_path, download_url)?;
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
            color_eyre::eyre::bail!("Installation verification failed, some checks did not pass");
        }

        Ok(())
    }

    /// 处理更新结果
    fn handle_update_result(
        &self,
        update_result: Result<()>,
        backup_info: Option<&BackupResult>,
        temp_manager: &TempDirManager,
    ) -> Result<()> {
        match update_result {
            Ok(()) => {
                // 更新成功，清理资源
                self.cleanup_resources(temp_manager, backup_info);
                success!("Workflow CLI update complete! All verifications passed.");
                Ok(())
            }
            Err(e) => {
                // 更新失败，执行回滚
                error!("Update failed: {}", e);
                br!();

                if let Some(backup) = backup_info {
                    self.perform_rollback(backup);
                } else {
                    error!("Unable to rollback: no available backup");
                    error!("Please manually check and restore files");
                }

                // 清理临时资源
                self.cleanup_resources(temp_manager, backup_info);
                Err(e.wrap_err("Update failed"))
            }
        }
    }

    /// 执行回滚
    fn perform_rollback(&self, backup: &BackupResult) {
        warning!("Update failed, rolling back to previous version...");
        br!();

        match RollbackManager::rollback(&backup.backup_info) {
            Ok(rollback_result) => {
                // 显示恢复的二进制文件
                if !rollback_result.restored_binaries.is_empty() {
                    info!("Restoring binary files...");
                    for binary in &rollback_result.restored_binaries {
                        info!("  Restored: {}", binary);
                    }
                }

                // 显示失败的二进制文件
                if !rollback_result.failed_binaries.is_empty() {
                    warning!("Failed to restore some binary files:");
                    for (binary, err) in &rollback_result.failed_binaries {
                        warning!("  {}: {}", binary, err);
                    }
                }

                // 显示恢复的补全脚本
                if !rollback_result.restored_completions.is_empty() {
                    info!("Restoring completion scripts...");
                    for completion in &rollback_result.restored_completions {
                        info!("  Restored: {}", completion);
                    }
                }

                // 显示失败的补全脚本
                if !rollback_result.failed_completions.is_empty() {
                    warning!("Failed to restore some completion scripts:");
                    for (completion, err) in &rollback_result.failed_completions {
                        warning!("  {}: {}", completion, err);
                    }
                }

                // 处理 shell 重新加载
                if let Some(reload_success) = rollback_result.shell_reload_success {
                    if reload_success {
                        info!("Note: Configuration has been reloaded in subprocess");
                        if let Some(ref config_file) = rollback_result.shell_config_file {
                            info!(
                                "If completion is not working, please run manually: source {}",
                                config_file.display()
                            );
                        }
                    } else {
                        warning!("Failed to reload shell configuration");
                        if let Some(ref config_file) = rollback_result.shell_config_file {
                            info!("Please run manually: source {}", config_file.display());
                        }
                    }
                } else {
                    info!("Please manually reload shell config file to enable completion");
                }

                success!("Rollback completed");
                br!();

                // 回滚成功后清理备份
                if let Err(cleanup_err) = RollbackManager::cleanup_backup(&backup.backup_info) {
                    warning!("Failed to clean up backup: {}", cleanup_err);
                }
            }
            Err(rollback_err) => {
                error!("Rollback failed: {}", rollback_err);
                error!("System may be in an inconsistent state");
                error!("Please manually check and restore files");
                error!(
                    "Backup location: {}",
                    backup.backup_info.backup_dir.display()
                );
            }
        }
    }

    /// 清理资源
    fn cleanup_resources(&self, temp_manager: &TempDirManager, backup_info: Option<&BackupResult>) {
        // 清理临时文件
        if let Err(e) = temp_manager.cleanup() {
            warning!("Failed to clean up temporary files: {}", e);
        }

        // 清理备份（成功时）
        if let Some(backup) = backup_info {
            if let Err(e) = RollbackManager::cleanup_backup(&backup.backup_info) {
                warning!("Failed to clean up backup: {}", e);
            }
        }
    }
}
