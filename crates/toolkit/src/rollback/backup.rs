//! 备份功能模块
//!
//! 提供二进制文件和补全脚本的备份功能。

#[cfg(windows)]
use std::fs;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;

use crate::rollback::error::RollbackError;
use crate::util::fs::directory;

/// 创建备份目录
///
/// 在临时目录中创建一个唯一的备份目录。
///
/// # 返回
///
/// 返回备份目录路径。
pub(super) fn create_backup_dir() -> Result<PathBuf, RollbackError> {
    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let backup_dir = temp_dir.join(format!("workflow-backup-{}", timestamp));

    directory::ensure_exists(&backup_dir)?;

    tracing::debug!("Created backup directory: {}", backup_dir.display());
    Ok(backup_dir)
}

/// 创建备份
///
/// 备份当前版本的二进制文件和补全脚本。
///
/// # 返回
///
/// 返回 `BackupResult` 结构体，包含备份信息和统计。
pub fn backup(bin_name: &str, install_dir: PathBuf) -> Result<PathBuf, RollbackError> {
    tracing::info!("Creating backup");

    let backup_dir = create_backup_dir()?;

    let source = install_dir.join(bin_name);

    // 如果文件不存在，返回 None
    if !source.exists() {
        tracing::debug!(
            "Binary file does not exist, skipping backup: {}",
            source.display()
        );
        return Err(RollbackError::BackupBinaryFailed {
            src: source.display().to_string(),
            dest: backup_dir.display().to_string(),
            message: "Binary file does not exist".to_string(),
        });
    }

    let backup_path = backup_dir.join(bin_name);

    // 复制文件（Unix 使用 sudo，Windows 直接复制）
    #[cfg(unix)]
    {
        let status =
            Command::new("sudo").arg("cp").arg(&source).arg(&backup_path).status().map_err(
                |e| RollbackError::BackupBinaryFailed {
                    src: source.display().to_string(),
                    dest: backup_path.display().to_string(),
                    message: e.to_string(),
                },
            )?;

        if !status.success() {
            return Err(RollbackError::BackupBinaryFailed {
                src: source.display().to_string(),
                dest: backup_path.display().to_string(),
                message: format!("exit code: {}", status.code().unwrap_or(-1)),
            });
        }

        // 设置执行权限（仅 Unix）
        let chmod_status =
            Command::new("chmod").arg("+x").arg(&backup_path).status().map_err(|e| {
                RollbackError::BackupBinaryFailed {
                    src: source.display().to_string(),
                    dest: backup_path.display().to_string(),
                    message: format!("Failed to set executable permission: {}", e),
                }
            })?;

        if !chmod_status.success() {
            return Err(RollbackError::BackupBinaryFailed {
                src: source.display().to_string(),
                dest: backup_path.display().to_string(),
                message: format!(
                    "Failed to set executable permission (exit code: {})",
                    chmod_status.code().unwrap_or(-1)
                ),
            });
        }
    }
    #[cfg(windows)]
    {
        fs::copy(&source, &backup_path).map_err(|e| RollbackError::BackupBinaryFailed {
            src: source.display().to_string(),
            dest: backup_path.display().to_string(),
            message: e.to_string(),
        })?;
    }

    tracing::debug!(
        "Backed up binary file: {} -> {}",
        source.display(),
        backup_path.display()
    );

    Ok(backup_dir)
}
