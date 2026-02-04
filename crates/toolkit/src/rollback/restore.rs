//! 恢复功能模块
//!
//! 提供二进制文件和补全脚本的恢复和清理功能。

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::process::Command;

use crate::paths::{binary_install_dir, completion_dir};
use crate::shell::{config_file_path, detect_shell};
use crate::util::fs::directory;

use super::error::RollbackError;
use super::reload::reload_shell;
use super::types::{BackupInfo, RestoreResult, RollbackResult};

/// 清理备份
///
/// 删除备份目录及其所有内容。
///
/// # 参数
///
/// * `backup_info` - 备份信息
pub fn cleanup_backup(backup_info: &BackupInfo) -> Result<(), RollbackError> {
    if backup_info.backup_dir.exists() {
        tracing::debug!(
            "Cleaning up backup directory: {}",
            backup_info.backup_dir.display()
        );
        fs::remove_dir_all(&backup_info.backup_dir).map_err(|e| RollbackError::CleanupFailed {
            path: backup_info.backup_dir.display().to_string(),
            reason: e.to_string(),
        })?;
        tracing::debug!("Backup directory cleaned up");
    }
    Ok(())
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
pub(super) fn restore_binaries(backups: &[(String, PathBuf)]) -> RestoreResult {
    let install_dir = binary_install_dir();
    let install_path = PathBuf::from(&install_dir);
    let mut restored = Vec::new();
    let mut failed = Vec::new();

    for (binary_name, backup_path) in backups {
        let target = install_path.join(binary_name);

        // 如果备份文件不存在，跳过
        if !backup_path.exists() {
            let error_msg = format!("Backup file does not exist: {}", backup_path.display());
            tracing::warn!("{}", error_msg);
            failed.push((binary_name.clone(), error_msg));
            continue;
        }

        // 复制文件（Unix 使用 sudo，Windows 直接复制）
        let result = (|| -> Result<(), String> {
            #[cfg(unix)]
            {
                let status =
                    Command::new("sudo").arg("cp").arg(backup_path).arg(&target).status().map_err(
                        |e| {
                            format!(
                                "Failed to restore binary file: {} -> {}: {}",
                                backup_path.display(),
                                target.display(),
                                e
                            )
                        },
                    )?;

                if !status.success() {
                    return Err(format!(
                        "Failed to restore binary file: {} -> {} (exit code: {})",
                        backup_path.display(),
                        target.display(),
                        status.code().unwrap_or(-1)
                    ));
                }

                // 设置执行权限（仅 Unix）
                Command::new("sudo").arg("chmod").arg("+x").arg(&target).status().map_err(|e| {
                    format!(
                        "Failed to set executable permission for restored binary file: {}: {}",
                        target.display(),
                        e
                    )
                })?;
            }
            #[cfg(windows)]
            {
                fs::copy(backup_path, &target).map_err(|e| {
                    format!(
                        "Failed to restore binary file: {} -> {}: {}",
                        backup_path.display(),
                        target.display(),
                        e
                    )
                })?;
            }
            Ok(())
        })();

        match result {
            Ok(_) => {
                tracing::info!("Restored binary file: {}", binary_name);
                restored.push(binary_name.clone());
            }
            Err(error_msg) => {
                tracing::warn!(
                    "Failed to restore binary file {}: {}",
                    binary_name,
                    error_msg
                );
                failed.push((binary_name.clone(), error_msg));
            }
        }
    }

    (restored, failed)
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
pub(super) fn restore_completions(
    backups: &[(String, PathBuf)],
    completion_dir: &Path,
) -> RestoreResult {
    // 确保补全脚本目录存在
    if let Err(e) = directory::ensure_exists(completion_dir) {
        let error_msg = format!("Failed to ensure completion directory exists: {}", e);
        return (
            Vec::new(),
            backups.iter().map(|(name, _)| (name.clone(), error_msg.clone())).collect(),
        );
    }

    let mut restored = Vec::new();
    let mut failed = Vec::new();

    for (file_name, backup_path) in backups {
        let target = completion_dir.join(file_name);

        // 如果备份文件不存在，跳过
        if !backup_path.exists() {
            let error_msg = format!("Backup file does not exist: {}", backup_path.display());
            tracing::warn!("{}", error_msg);
            failed.push((file_name.clone(), error_msg));
            continue;
        }

        // 复制文件
        match fs::copy(backup_path, &target) {
            Ok(_) => {
                tracing::info!("Restored completion script: {}", file_name);
                restored.push(file_name.clone());
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to restore completion script: {} -> {}: {}",
                    backup_path.display(),
                    target.display(),
                    e
                );
                tracing::warn!("{}", error_msg);
                failed.push((file_name.clone(), error_msg));
            }
        }
    }

    (restored, failed)
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
pub fn rollback(backup_info: &BackupInfo) -> Result<RollbackResult, RollbackError> {
    tracing::info!("Starting rollback operation");

    let mut restored_binaries = Vec::new();
    let mut restored_completions = Vec::new();
    let mut failed_binaries = Vec::new();
    let mut failed_completions = Vec::new();

    // 恢复二进制文件
    if !backup_info.binary_backups.is_empty() {
        tracing::info!("Restoring binary files");
        let (restored, failed) = restore_binaries(&backup_info.binary_backups);
        restored_binaries = restored;
        failed_binaries = failed;
    }

    // 恢复补全脚本（不依赖 shell 检测）
    if !backup_info.completion_backups.is_empty() {
        tracing::info!("Restoring completion scripts");
        let comp_dir = match completion_dir() {
            Ok(dir) => dir,
            Err(e) => {
                tracing::warn!("Failed to get completion directory: {}", e);
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

        let (restored, failed) = restore_completions(&backup_info.completion_backups, &comp_dir);
        restored_completions = restored;
        failed_completions = failed;
    }

    // 尝试重新加载当前 shell 的配置（需要检测 shell，但这是可选操作）
    let (shell_reload_success, shell_config_file) = match detect_shell() {
        Ok(shell) => {
            tracing::info!("Reloading shell configuration for {}", shell);
            let config_file = config_file_path(&shell);

            match reload_shell(&shell) {
                Ok(result) => {
                    if result.reloaded {
                        tracing::info!("Shell configuration reloaded successfully");
                    }
                    (Some(result.reloaded), config_file)
                }
                Err(e) => {
                    tracing::warn!("Failed to reload shell configuration: {}", e);
                    (Some(false), config_file)
                }
            }
        }
        Err(e) => {
            tracing::debug!("Failed to detect shell type, skipping reload: {}", e);
            (None, None)
        }
    };

    tracing::info!(
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
