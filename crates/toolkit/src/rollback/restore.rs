//! 恢复功能模块
//!
//! 提供二进制文件和补全脚本的恢复和清理功能。

use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::process::Command;

use crate::shell::{config_file_path, detect_shell};

use super::error::RollbackError;
use super::reload::reload_shell;

/// 清理备份
///
/// 删除备份目录及其所有内容。
///
/// # 参数
///
/// * `backup_info` - 备份信息
pub fn cleanup_backup(backup_dir: PathBuf) -> Result<(), RollbackError> {
    if backup_dir.exists() {
        tracing::debug!("Cleaning up backup directory: {}", backup_dir.display());
        fs::remove_dir_all(&backup_dir).map_err(|e| RollbackError::CleanupFailed {
            path: backup_dir.display().to_string(),
            reason: e.to_string(),
        })?;
        tracing::debug!("Backup directory cleaned up");
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
///
/// # 返回
///
/// 返回 `RollbackResult` 结构体，包含恢复的文件列表和状态信息。
pub fn rollback(
    binary_name: &str,
    backup_dir: PathBuf,
    install_dir: PathBuf,
) -> Result<(), RollbackError> {
    tracing::info!("Starting rollback operation");
    // 恢复二进制文件
    tracing::info!("Restoring binary files");
    // for (binary_name, backup_path) in backups {
    let target = install_dir.join(binary_name);
    let source = backup_dir.join(binary_name);
    // 如果备份文件不存在，跳过
    if !source.exists() {
        tracing::warn!("Backup file does not exist: {}", source.display());
        return Err(RollbackError::RestoreBinaryFailed {
            src: source.display().to_string(),
            dest: target.display().to_string(),
            message: "Backup file does not exist".to_string(),
        });
    }

    // 复制文件（Unix 使用 sudo，Windows 直接复制）
    let result = (|| -> Result<(), String> {
        #[cfg(unix)]
        {
            let status =
                Command::new("sudo").arg("cp").arg(&source).arg(&target).status().map_err(|e| {
                    format!(
                        "Failed to restore binary file: {} -> {}: {}",
                        source.display(),
                        target.display(),
                        e
                    )
                })?;

            if !status.success() {
                return Err(format!(
                    "Failed to restore binary file: {} -> {} (exit code: {})",
                    source.display(),
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
            fs::copy(&source, &target).map_err(|e| {
                format!(
                    "Failed to restore binary file: {} -> {}: {}",
                    source.display(),
                    target.display(),
                    e
                )
            })?;
        }

        // 不传 --shell/--output，由 workflow completion generate 自动检测当前 shell 并使用默认目录
        Command::new("workflow")
            .arg("completion")
            .arg("generate")
            .status()
            .map_err(|e| format!("Failed to generate workflow completion: {}", e))?;

        cleanup_backup(backup_dir).map_err(|e| format!("Failed to cleanup backup: {}", e))?;

        Ok(())
    })();

    result.map_err(|error_msg| RollbackError::RestoreBinaryFailed {
        src: source.display().to_string(),
        dest: target.display().to_string(),
        message: error_msg,
    })?;

    tracing::info!("Restored binary file: {}", binary_name);

    // 尝试重新加载当前 shell 的配置（需要检测 shell，但这是可选操作）
    match detect_shell() {
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

    Ok(())
}
