//! 回滚工具模块
//!
//! 本模块提供了更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件

use crate::completion::get_all_completion_files;
use crate::{
    base::settings::paths::Paths, base::shell::Reload, log_break, log_debug, log_info, log_success,
    log_warning, Detect,
};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
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

        log_debug!("Created backup directory: {}", backup_dir.display());
        Ok(backup_dir)
    }

    /// 备份二进制文件
    ///
    /// 备份系统目录（通常是 /usr/local/bin）中的二进制文件到备份目录。
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
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let mut backups = Vec::new();

        for binary in binaries {
            let binary_name = Paths::binary_name(binary);
            let source = install_path.join(&binary_name);

            // 如果文件不存在，跳过
            if !source.exists() {
                log_debug!(
                    "Binary file does not exist, skipping backup: {}",
                    source.display()
                );
                continue;
            }

            let backup_path = backup_dir.join(binary);

            // 复制文件（Unix 使用 sudo，Windows 直接复制）
            #[cfg(unix)]
            {
                let status = Command::new("sudo")
                    .arg("cp")
                    .arg(&source)
                    .arg(&backup_path)
                    .status()
                    .with_context(|| {
                        format!(
                            "Failed to backup binary file: {} -> {}",
                            source.display(),
                            backup_path.display()
                        )
                    })?;

                if !status.success() {
                    anyhow::bail!(
                        "Failed to backup binary file: {} -> {} (exit code: {})",
                        source.display(),
                        backup_path.display(),
                        status.code().unwrap_or(-1)
                    );
                }

                // 设置执行权限（仅 Unix）
                Command::new("chmod")
                    .arg("+x")
                    .arg(&backup_path)
                    .status()
                    .with_context(|| {
                        format!(
                            "Failed to set executable permission for backup file: {}",
                            backup_path.display()
                        )
                    })?;
            }
            #[cfg(windows)]
            {
                fs::copy(&source, &backup_path).with_context(|| {
                    format!(
                        "Failed to backup binary file: {} -> {}",
                        source.display(),
                        backup_path.display()
                    )
                })?;
            }

            log_debug!(
                "Backed up binary file: {} -> {}",
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
            log_debug!(
                "Completion directory does not exist, skipping backup: {}",
                completion_dir.display()
            );
            return Ok(backups);
        }

        // 要备份的补全脚本文件（所有 shell 类型）
        let commands = Paths::command_names();
        let completion_files = get_all_completion_files(commands);

        for file_name in &completion_files {
            let source = completion_dir.join(file_name);

            // 如果文件不存在，跳过
            if !source.exists() {
                log_debug!(
                    "Completion script does not exist, skipping backup: {}",
                    source.display()
                );
                continue;
            }

            let backup_path = backup_dir.join(file_name);

            // 复制文件
            fs::copy(&source, &backup_path).with_context(|| {
                format!(
                    "Failed to backup completion script: {} -> {}",
                    source.display(),
                    backup_path.display()
                )
            })?;

            log_debug!(
                "Backed up completion script: {} -> {}",
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
        log_info!("Creating backup...");

        // 创建备份目录
        let backup_dir = Self::create_backup_dir()?;

        // 备份二进制文件
        let binaries = Paths::command_names();
        let binary_backups =
            Self::backup_binaries(&backup_dir, binaries).context("Failed to backup binaries")?;

        // 备份补全脚本（不依赖 shell 检测）
        let completion_dir = Paths::completion_dir()?;
        let completion_backups = Self::backup_completions(&backup_dir, &completion_dir)
            .context("Failed to backup completions")?;

        let backup_info = BackupInfo {
            backup_dir,
            binary_backups,
            completion_backups,
        };

        log_success!("  Backup completed");
        log_debug!(
            "Backed up {} binary file(s), {} completion script(s)",
            backup_info.binary_backups.len(),
            backup_info.completion_backups.len()
        );

        Ok(backup_info)
    }

    /// 恢复二进制文件
    ///
    /// 从备份恢复二进制文件到系统目录（通常是 /usr/local/bin）。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    fn restore_binaries(backups: &[(String, PathBuf)]) -> Result<()> {
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        for (binary_name, backup_path) in backups {
            let target = install_path.join(binary_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                log_warning!(
                    "Backup file does not exist, skipping restore: {}",
                    backup_path.display()
                );
                continue;
            }

            // 复制文件（Unix 使用 sudo，Windows 直接复制）
            #[cfg(unix)]
            {
                let status = Command::new("sudo")
                    .arg("cp")
                    .arg(backup_path)
                    .arg(&target)
                    .status()
                    .with_context(|| {
                        format!(
                            "Failed to restore binary file: {} -> {}",
                            backup_path.display(),
                            target.display()
                        )
                    })?;

                if !status.success() {
                    anyhow::bail!(
                        "Failed to restore binary file: {} -> {} (exit code: {})",
                        backup_path.display(),
                        target.display(),
                        status.code().unwrap_or(-1)
                    );
                }

                // 设置执行权限（仅 Unix）
                Command::new("sudo")
                    .arg("chmod")
                    .arg("+x")
                    .arg(&target)
                    .status()
                    .with_context(|| {
                        format!(
                            "Failed to set executable permission for restored binary file: {}",
                            target.display()
                        )
                    })?;
            }
            #[cfg(windows)]
            {
                fs::copy(backup_path, &target).with_context(|| {
                    format!(
                        "Failed to restore binary file: {} -> {}",
                        backup_path.display(),
                        target.display()
                    )
                })?;
            }

            log_success!("  Restored binary file: {}", binary_name);
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
                log_warning!(
                    "Backup file does not exist, skipping restore: {}",
                    backup_path.display()
                );
                continue;
            }

            // 复制文件
            fs::copy(backup_path, &target).with_context(|| {
                format!(
                    "Failed to restore completion script: {} -> {}",
                    backup_path.display(),
                    target.display()
                )
            })?;

            log_success!("  Restored completion script: {}", file_name);
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
        log_warning!("Update failed, rolling back to previous version...");
        log_break!();

        // 恢复二进制文件
        if !backup_info.binary_backups.is_empty() {
            log_info!("Restoring binary files...");
            Self::restore_binaries(&backup_info.binary_backups)
                .context("Failed to restore binaries")?;
        }

        // 恢复补全脚本（不依赖 shell 检测）
        if !backup_info.completion_backups.is_empty() {
            let completion_dir = Paths::completion_dir()?;
            log_info!("Restoring completion scripts...");
            Self::restore_completions(&backup_info.completion_backups, &completion_dir)
                .context("Failed to restore completions")?;
        }

        // 尝试重新加载当前 shell 的配置（需要检测 shell，但这是可选操作）
        if let Ok(shell) = Detect::shell() {
            log_info!("Reloading shell configuration...");
            if let Err(e) = Reload::shell(&shell) {
                log_warning!("Failed to reload shell configuration: {}", e);
                let config_file = Paths::config_file(&shell)?;
                log_info!("Please run manually: source {}", config_file.display());
            } else {
                log_info!("Note: Configuration has been reloaded in subprocess");
                if let Ok(config_file) = Paths::config_file(&shell) {
                    log_info!(
                        "  If completion is not working, please run manually: source {}",
                        config_file.display()
                    );
                }
            }
        } else {
            log_debug!("Failed to detect shell type, skipping reload");
            log_info!("Please manually reload shell config file to enable completion");
        }

        log_success!("Rollback completed");
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
            log_debug!(
                "Cleaning up backup directory: {}",
                backup_info.backup_dir.display()
            );
            fs::remove_dir_all(&backup_info.backup_dir).with_context(|| {
                format!(
                    "Failed to remove backup directory: {}",
                    backup_info.backup_dir.display()
                )
            })?;
            log_debug!("Backup directory cleaned up");
        }
        Ok(())
    }
}
