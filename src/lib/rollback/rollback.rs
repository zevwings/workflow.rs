//! 回滚工具模块
//!
//! 本模块提供了更新失败时的回滚机制，包括：
//! - 备份当前版本的二进制文件和补全脚本
//! - 在更新失败时恢复备份的文件
//! - 清理备份文件

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::process::Command;

use color_eyre::{eyre::WrapErr, Result};

use crate::completion::get_all_completion_files;
use crate::{
    base::settings::paths::Paths, base::shell::Reload, trace_debug, trace_info, trace_warn, Detect,
};

// ==================== 类型别名 ====================

/// 恢复结果类型：成功列表和失败列表（文件名，错误信息）
type RestoreResult = (Vec<String>, Vec<(String, String)>);

// ==================== 返回结构体 ====================

/// 备份结果
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// 备份信息
    pub backup_info: BackupInfo,
    /// 备份的二进制文件数量
    pub binary_count: usize,
    /// 备份的补全脚本数量
    pub completion_count: usize,
}

/// 回滚结果
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// 恢复的二进制文件列表
    pub restored_binaries: Vec<String>,
    /// 恢复的补全脚本列表
    pub restored_completions: Vec<String>,
    /// 失败的二进制文件列表（文件名称和错误信息）
    pub failed_binaries: Vec<(String, String)>,
    /// 失败的补全脚本列表（文件名称和错误信息）
    pub failed_completions: Vec<(String, String)>,
    /// 是否成功重新加载 shell 配置
    pub shell_reload_success: Option<bool>,
    /// Shell 配置文件路径（如果检测到）
    pub shell_config_file: Option<PathBuf>,
}

/// 备份信息
///
/// 存储备份的文件路径和备份目录。
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// 备份目录
    pub backup_dir: PathBuf,
    /// 备份的二进制文件路径
    pub binary_backups: Vec<(String, PathBuf)>, // (binary_name, backup_path)
    /// 备份的补全脚本路径
    pub completion_backups: Vec<(String, PathBuf)>, // (completion_name, backup_path)
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

        fs::create_dir_all(&backup_dir).wrap_err_with(|| {
            format!(
                "Failed to create backup directory: {}",
                backup_dir.display()
            )
        })?;

        trace_debug!("Created backup directory: {}", backup_dir.display());
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
                trace_debug!(
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
                    .wrap_err_with(|| {
                        format!(
                            "Failed to backup binary file: {} -> {}",
                            source.display(),
                            backup_path.display()
                        )
                    })?;

                if !status.success() {
                    color_eyre::eyre::bail!(
                        "Failed to backup binary file: {} -> {} (exit code: {})",
                        source.display(),
                        backup_path.display(),
                        status.code().unwrap_or(-1)
                    );
                }

                // 设置执行权限（仅 Unix）
                Command::new("chmod").arg("+x").arg(&backup_path).status().wrap_err_with(|| {
                    format!(
                        "Failed to set executable permission for backup file: {}",
                        backup_path.display()
                    )
                })?;
            }
            #[cfg(windows)]
            {
                fs::copy(&source, &backup_path).wrap_err_with(|| {
                    format!(
                        "Failed to backup binary file: {} -> {}",
                        source.display(),
                        backup_path.display()
                    )
                })?;
            }

            trace_debug!(
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
            trace_debug!(
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
                trace_debug!(
                    "Completion script does not exist, skipping backup: {}",
                    source.display()
                );
                continue;
            }

            let backup_path = backup_dir.join(file_name);

            // 复制文件
            fs::copy(&source, &backup_path).wrap_err_with(|| {
                format!(
                    "Failed to backup completion script: {} -> {}",
                    source.display(),
                    backup_path.display()
                )
            })?;

            trace_debug!(
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
    /// 返回 `BackupResult` 结构体，包含备份信息和统计。
    pub fn create_backup() -> Result<BackupResult> {
        trace_info!("Creating backup");

        // 创建备份目录
        let backup_dir = Self::create_backup_dir()?;

        // 备份二进制文件
        let binaries = Paths::command_names();
        let binary_backups =
            Self::backup_binaries(&backup_dir, binaries).wrap_err("Failed to backup binaries")?;

        // 备份补全脚本（不依赖 shell 检测）
        let completion_dir = Paths::completion_dir()?;
        let completion_backups = Self::backup_completions(&backup_dir, &completion_dir)
            .wrap_err("Failed to backup completions")?;

        let backup_info = BackupInfo {
            backup_dir,
            binary_backups: binary_backups.clone(),
            completion_backups: completion_backups.clone(),
        };

        let binary_count = binary_backups.len();
        let completion_count = completion_backups.len();

        trace_info!(
            "Backed up {} binary file(s), {} completion script(s)",
            binary_count,
            completion_count
        );

        Ok(BackupResult {
            backup_info,
            binary_count,
            completion_count,
        })
    }

    /// 恢复二进制文件
    ///
    /// 从备份恢复二进制文件到系统目录（通常是 /usr/local/bin）。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    ///
    /// # 返回
    ///
    /// 返回恢复成功的文件列表和失败的文件列表。
    fn restore_binaries(backups: &[(String, PathBuf)]) -> Result<RestoreResult> {
        let install_dir = Paths::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        let mut restored = Vec::new();
        let mut failed = Vec::new();

        for (binary_name, backup_path) in backups {
            let target = install_path.join(binary_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                let error_msg = format!("Backup file does not exist: {}", backup_path.display());
                trace_warn!("{}", error_msg);
                failed.push((binary_name.clone(), error_msg));
                continue;
            }

            // 复制文件（Unix 使用 sudo，Windows 直接复制）
            let result = (|| -> Result<()> {
                #[cfg(unix)]
                {
                    let status = Command::new("sudo")
                        .arg("cp")
                        .arg(backup_path)
                        .arg(&target)
                        .status()
                        .wrap_err_with(|| {
                            format!(
                                "Failed to restore binary file: {} -> {}",
                                backup_path.display(),
                                target.display()
                            )
                        })?;

                    if !status.success() {
                        color_eyre::eyre::bail!(
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
                        .wrap_err_with(|| {
                            format!(
                                "Failed to set executable permission for restored binary file: {}",
                                target.display()
                            )
                        })?;
                }
                #[cfg(windows)]
                {
                    fs::copy(backup_path, &target).wrap_err_with(|| {
                        format!(
                            "Failed to restore binary file: {} -> {}",
                            backup_path.display(),
                            target.display()
                        )
                    })?;
                }
                Ok(())
            })();

            match result {
                Ok(_) => {
                    trace_info!("Restored binary file: {}", binary_name);
                    restored.push(binary_name.clone());
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    trace_warn!(
                        "Failed to restore binary file {}: {}",
                        binary_name,
                        error_msg
                    );
                    failed.push((binary_name.clone(), error_msg));
                }
            }
        }

        Ok((restored, failed))
    }

    /// 恢复补全脚本
    ///
    /// 从备份恢复补全脚本到补全脚本目录。
    ///
    /// # 参数
    ///
    /// * `backups` - 备份的文件路径列表
    /// * `completion_dir` - 补全脚本目录
    ///
    /// # 返回
    ///
    /// 返回恢复成功的文件列表和失败的文件列表。
    fn restore_completions(
        backups: &[(String, PathBuf)],
        completion_dir: &Path,
    ) -> Result<RestoreResult> {
        // 确保补全脚本目录存在
        fs::create_dir_all(completion_dir).wrap_err_with(|| {
            format!(
                "Failed to create completion directory: {}",
                completion_dir.display()
            )
        })?;

        let mut restored = Vec::new();
        let mut failed = Vec::new();

        for (file_name, backup_path) in backups {
            let target = completion_dir.join(file_name);

            // 如果备份文件不存在，跳过
            if !backup_path.exists() {
                let error_msg = format!("Backup file does not exist: {}", backup_path.display());
                trace_warn!("{}", error_msg);
                failed.push((file_name.clone(), error_msg));
                continue;
            }

            // 复制文件
            match fs::copy(backup_path, &target) {
                Ok(_) => {
                    trace_info!("Restored completion script: {}", file_name);
                    restored.push(file_name.clone());
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to restore completion script: {} -> {}: {}",
                        backup_path.display(),
                        target.display(),
                        e
                    );
                    trace_warn!("{}", error_msg);
                    failed.push((file_name.clone(), error_msg));
                }
            }
        }

        Ok((restored, failed))
    }

    /// 执行回滚
    ///
    /// 从备份恢复所有文件。
    ///
    /// # 参数
    ///
    /// * `backup_info` - 备份信息
    ///
    /// # 返回
    ///
    /// 返回 `RollbackResult` 结构体，包含恢复的文件列表和状态信息。
    pub fn rollback(backup_info: &BackupInfo) -> Result<RollbackResult> {
        trace_info!("Starting rollback operation");

        let mut restored_binaries = Vec::new();
        let mut restored_completions = Vec::new();
        let mut failed_binaries = Vec::new();
        let mut failed_completions = Vec::new();

        // 恢复二进制文件
        if !backup_info.binary_backups.is_empty() {
            trace_info!("Restoring binary files");
            match Self::restore_binaries(&backup_info.binary_backups) {
                Ok((restored, failed)) => {
                    restored_binaries = restored;
                    failed_binaries = failed;
                }
                Err(e) => {
                    trace_warn!("Failed to restore binaries: {}", e);
                    // 将所有二进制文件标记为失败
                    for (binary_name, _) in &backup_info.binary_backups {
                        failed_binaries.push((binary_name.clone(), format!("{}", e)));
                    }
                }
            }
        }

        // 恢复补全脚本（不依赖 shell 检测）
        if !backup_info.completion_backups.is_empty() {
            trace_info!("Restoring completion scripts");
            let completion_dir = match Paths::completion_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    trace_warn!("Failed to get completion directory: {}", e);
                    // 将所有补全脚本标记为失败
                    for (file_name, _) in &backup_info.completion_backups {
                        failed_completions.push((file_name.clone(), format!("{}", e)));
                    }
                    return Ok(RollbackResult {
                        restored_binaries,
                        restored_completions,
                        failed_binaries,
                        failed_completions,
                        shell_reload_success: None,
                        shell_config_file: None,
                    });
                }
            };

            match Self::restore_completions(&backup_info.completion_backups, &completion_dir) {
                Ok((restored, failed)) => {
                    restored_completions = restored;
                    failed_completions = failed;
                }
                Err(e) => {
                    trace_warn!("Failed to restore completions: {}", e);
                    // 将所有补全脚本标记为失败
                    for (file_name, _) in &backup_info.completion_backups {
                        if !restored_completions.contains(file_name) {
                            failed_completions.push((file_name.clone(), format!("{}", e)));
                        }
                    }
                }
            }
        }

        // 尝试重新加载当前 shell 的配置（需要检测 shell，但这是可选操作）
        let (shell_reload_success, shell_config_file) = match Detect::shell() {
            Ok(shell) => {
                trace_info!("Reloading shell configuration for {}", shell);
                let config_file_result = Paths::config_file(&shell);
                let config_file = config_file_result.ok();

                match Reload::shell(&shell) {
                    Ok(_) => {
                        trace_info!("Shell configuration reloaded successfully");
                        (Some(true), config_file)
                    }
                    Err(e) => {
                        trace_warn!("Failed to reload shell configuration: {}", e);
                        (Some(false), config_file)
                    }
                }
            }
            Err(e) => {
                trace_debug!("Failed to detect shell type, skipping reload: {}", e);
                (None, None)
            }
        };

        trace_info!(
            "Rollback completed: {} binary(ies), {} completion(s) restored",
            restored_binaries.len(),
            restored_completions.len()
        );

        Ok(RollbackResult {
            restored_binaries,
            restored_completions,
            failed_binaries,
            failed_completions,
            shell_reload_success,
            shell_config_file,
        })
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
            trace_debug!(
                "Cleaning up backup directory: {}",
                backup_info.backup_dir.display()
            );
            fs::remove_dir_all(&backup_info.backup_dir).wrap_err_with(|| {
                format!(
                    "Failed to remove backup directory: {}",
                    backup_info.backup_dir.display()
                )
            })?;
            trace_debug!("Backup directory cleaned up");
        }
        Ok(())
    }
}
