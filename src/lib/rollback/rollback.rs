//! 回滚工具模块
//!
//! 本模块提供了更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件

use crate::completion::get_all_completion_files;
use crate::{base::settings::paths::Paths, log_debug, log_info, log_success, log_warning, Detect};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 备份信息
///
/// 存储备份的文件路径和备份目录。
#[derive(Debug)]
pub struct BackupInfo {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份的二进制文件路径
    binary_backups: Vec<(String, PathBuf)>, // (binary_name, backup_path)
    /// 备份的补全脚本路径
    completion_backups: Vec<(String, PathBuf)>, // (completion_name, backup_path)
}

impl BackupInfo {
    // 这些方法目前未使用，但保留以备将来扩展
    #[allow(dead_code)]
    /// 创建新的备份信息
    fn new(backup_dir: PathBuf) -> Self {
        Self {
            backup_dir,
            binary_backups: Vec::new(),
            completion_backups: Vec::new(),
        }
    }

    #[allow(dead_code)]
    /// 添加二进制文件备份
    fn add_binary_backup(&mut self, name: String, path: PathBuf) {
        self.binary_backups.push((name, path));
    }

    #[allow(dead_code)]
    /// 添加补全脚本备份
    fn add_completion_backup(&mut self, name: String, path: PathBuf) {
        self.completion_backups.push((name, path));
    }
}

/// 回滚管理器
///
/// 提供备份和恢复功能，用于更新失败时的回滚操作。
pub struct RollbackManager;

impl RollbackManager {
    /// 创建备份目录
    ///
    /// 在临时目录中创建一个唯一的备份目录。
    ///
    /// # 返回
    ///
    /// 返回备份目录路径。
    fn create_backup_dir() -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let backup_dir = temp_dir.join(format!(
            "workflow-backup-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

        fs::create_dir_all(&backup_dir).with_context(|| {
            format!(
                "Failed to create backup directory: {}",
                backup_dir.display()
            )
        })?;

        log_debug!("创建备份目录: {}", backup_dir.display());
        Ok(backup_dir)
    }

    /// 备份二进制文件
    ///
    /// 备份 `/usr/local/bin` 中的二进制文件到备份目录。
    ///
    /// # 参数
    ///
    /// * `backup_dir` - 备份目录
    /// * `binaries` - 要备份的二进制文件名称列表
    ///
    /// # 返回
    ///
    /// 返回备份的文件路径列表。
    fn backup_binaries(backup_dir: &Path, binaries: &[&str]) -> Result<Vec<(String, PathBuf)>> {
        let mut backups = Vec::new();

        for binary in binaries {
            let source = PathBuf::from(format!("/usr/local/bin/{}", binary));

            // 如果文件不存在，跳过
            if !source.exists() {
                log_debug!("二进制文件不存在，跳过备份: {}", source.display());
                continue;
            }

            let backup_path = backup_dir.join(binary);

            // 使用 sudo 复制文件
            let status = Command::new("sudo")
                .arg("cp")
                .arg(&source)
                .arg(&backup_path)
                .status()
                .with_context(|| format!("Failed to backup binary: {}", source.display()))?;

            if !status.success() {
                anyhow::bail!("备份二进制文件失败: {}", source.display());
            }

            // 设置执行权限
            Command::new("chmod")
                .arg("+x")
                .arg(&backup_path)
                .status()
                .with_context(|| {
                    format!(
                        "Failed to set executable permission for backup: {}",
                        backup_path.display()
                    )
                })?;

            log_debug!(
                "已备份二进制文件: {} -> {}",
                source.display(),
                backup_path.display()
            );
            backups.push((binary.to_string(), backup_path));
        }

        Ok(backups)
    }

    /// 备份补全脚本
    ///
    /// 备份补全脚本目录中的文件到备份目录。
    ///
    /// # 参数
    ///
    /// * `backup_dir` - 备份目录
    /// * `completion_dir` - 补全脚本目录
    ///
    /// # 返回
    ///
    /// 返回备份的文件路径列表。
    fn backup_completions(
        backup_dir: &Path,
        completion_dir: &Path,
    ) -> Result<Vec<(String, PathBuf)>> {
        let mut backups = Vec::new();

        // 如果补全脚本目录不存在，返回空列表
        if !completion_dir.exists() {
            log_debug!("补全脚本目录不存在，跳过备份: {}", completion_dir.display());
            return Ok(backups);
        }

        // 要备份的补全脚本文件（所有 shell 类型）
        let commands = ["workflow", "pr", "qk"];
        let completion_files = get_all_completion_files(&commands);

        for file_name in &completion_files {
            let source = completion_dir.join(file_name);

            // 如果文件不存在，跳过
            if !source.exists() {
                log_debug!("补全脚本不存在，跳过备份: {}", source.display());
                continue;
            }

            let backup_path = backup_dir.join(file_name);

            // 复制文件
            fs::copy(&source, &backup_path).with_context(|| {
                format!("Failed to backup completion script: {}", source.display())
            })?;

            log_debug!(
                "已备份补全脚本: {} -> {}",
                source.display(),
                backup_path.display()
            );
            backups.push((file_name.to_string(), backup_path));
        }

        Ok(backups)
    }

    /// 创建备份
    ///
    /// 备份当前版本的二进制文件和补全脚本。
    ///
    /// # 返回
    ///
    /// 返回 `BackupInfo` 结构体，包含备份信息。
    pub fn create_backup() -> Result<BackupInfo> {
        log_info!("正在创建备份...");

        // 创建备份目录
        let backup_dir = Self::create_backup_dir()?;

        // 备份二进制文件
        let binaries = ["workflow", "pr", "qk"];
        let binary_backups =
            Self::backup_binaries(&backup_dir, &binaries).context("Failed to backup binaries")?;

        // 备份补全脚本
        let _shell = match Detect::shell() {
            Ok(shell) => shell,
            Err(e) => {
                log_warning!("无法检测 shell 类型，跳过补全脚本备份: {}", e);
                return Ok(BackupInfo {
                    backup_dir,
                    binary_backups,
                    completion_backups: Vec::new(),
                });
            }
        };

        let completion_dir = Paths::completion_dir()?;
        let completion_backups = Self::backup_completions(&backup_dir, &completion_dir)
            .context("Failed to backup completions")?;

        let backup_info = BackupInfo {
            backup_dir,
            binary_backups,
            completion_backups,
        };

        log_success!("  备份完成");
        log_debug!(
            "备份了 {} 个二进制文件，{} 个补全脚本",
            backup_info.binary_backups.len(),
            backup_info.completion_backups.len()
        );

        Ok(backup_info)
    }

    /// 恢复二进制文件
    ///
    /// 从备份恢复二进制文件到 `/usr/local/bin`。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    fn restore_binaries(backups: &[(String, PathBuf)]) -> Result<()> {
        for (binary_name, backup_path) in backups {
            let target = PathBuf::from(format!("/usr/local/bin/{}", binary_name));

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                log_warning!("备份文件不存在，跳过恢复: {}", backup_path.display());
                continue;
            }

            // 使用 sudo 复制文件
            let status = Command::new("sudo")
                .arg("cp")
                .arg(backup_path)
                .arg(&target)
                .status()
                .with_context(|| format!("Failed to restore binary: {}", target.display()))?;

            if !status.success() {
                anyhow::bail!("恢复二进制文件失败: {}", target.display());
            }

            // 设置执行权限
            Command::new("sudo")
                .arg("chmod")
                .arg("+x")
                .arg(&target)
                .status()
                .with_context(|| {
                    format!(
                        "Failed to set executable permission for restored binary: {}",
                        target.display()
                    )
                })?;

            log_success!("  已恢复二进制文件: {}", binary_name);
        }

        Ok(())
    }

    /// 恢复补全脚本
    ///
    /// 从备份恢复补全脚本到补全脚本目录。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    /// * `completion_dir` - 补全脚本目录
    fn restore_completions(backups: &[(String, PathBuf)], completion_dir: &Path) -> Result<()> {
        // 确保补全脚本目录存在
        fs::create_dir_all(completion_dir).with_context(|| {
            format!(
                "Failed to create completion directory: {}",
                completion_dir.display()
            )
        })?;

        for (file_name, backup_path) in backups {
            let target = completion_dir.join(file_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                log_warning!("备份文件不存在，跳过恢复: {}", backup_path.display());
                continue;
            }

            // 复制文件
            fs::copy(backup_path, &target).with_context(|| {
                format!("Failed to restore completion script: {}", target.display())
            })?;

            log_success!("  已恢复补全脚本: {}", file_name);
        }

        Ok(())
    }

    /// 执行回滚
    ///
    /// 从备份恢复所有文件。
    ///
    /// # 参数
    ///
    /// * `backup_info` - 备份信息
    pub fn rollback(backup_info: &BackupInfo) -> Result<()> {
        log_warning!("更新失败，正在回滚到之前的版本...");
        crate::log_break!();

        // 恢复二进制文件
        if !backup_info.binary_backups.is_empty() {
            log_info!("正在恢复二进制文件...");
            Self::restore_binaries(&backup_info.binary_backups)
                .context("Failed to restore binaries")?;
        }

        // 恢复补全脚本
        if !backup_info.completion_backups.is_empty() {
            if let Ok(_shell) = Detect::shell() {
                let completion_dir = Paths::completion_dir()?;
                log_info!("正在恢复补全脚本...");
                Self::restore_completions(&backup_info.completion_backups, &completion_dir)
                    .context("Failed to restore completions")?;
            } else {
                log_warning!("无法检测 shell 类型，跳过补全脚本恢复");
            }
        }

        log_success!("回滚完成");
        Ok(())
    }

    /// 清理备份
    ///
    /// 删除备份目录及其所有内容。
    ///
    /// # 参数
    ///
    /// * `backup_info` - 备份信息
    pub fn cleanup_backup(backup_info: &BackupInfo) -> Result<()> {
        if backup_info.backup_dir.exists() {
            log_debug!("清理备份目录: {}", backup_info.backup_dir.display());
            fs::remove_dir_all(&backup_info.backup_dir).with_context(|| {
                format!(
                    "Failed to remove backup directory: {}",
                    backup_info.backup_dir.display()
                )
            })?;
            log_debug!("备份目录已清理");
        }
        Ok(())
    }
}
