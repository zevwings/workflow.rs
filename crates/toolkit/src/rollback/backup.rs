//! 备份功能模块
//!
//! 提供二进制文件和补全脚本的备份功能。

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::process::Command;

use crate::paths::{binary_install_dir, binary_name, command_names, completion_dir};
use crate::util::fs::directory;

use super::error::RollbackError;
use super::helpers::get_all_completion_files;
use super::types::{BackupInfo, BackupResult};

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
pub(super) fn backup_binaries(
    backup_dir: &Path,
    binaries: &[&str],
) -> Result<Vec<(String, PathBuf)>, RollbackError> {
    let install_dir = binary_install_dir();
    let install_path = PathBuf::from(&install_dir);
    let mut backups = Vec::new();

    for binary in binaries {
        let bin_name = binary_name(binary);
        let source = install_path.join(&bin_name);

        // 如果文件不存在，跳过
        if !source.exists() {
            tracing::debug!(
                "Binary file does not exist, skipping backup: {}",
                source.display()
            );
            continue;
        }

        let backup_path = backup_dir.join(binary);

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
pub(super) fn backup_completions(
    backup_dir: &Path,
    completion_dir: &Path,
) -> Result<Vec<(String, PathBuf)>, RollbackError> {
    let mut backups = Vec::new();

    // 如果补全脚本目录不存在，返回空列表
    if !completion_dir.exists() {
        tracing::debug!(
            "Completion directory does not exist, skipping backup: {}",
            completion_dir.display()
        );
        return Ok(backups);
    }

    // 要备份的补全脚本文件（所有 shell 类型）
    let commands = command_names();
    let completion_files = get_all_completion_files(commands);

    for file_name in &completion_files {
        let source = completion_dir.join(file_name);

        // 如果文件不存在，跳过
        if !source.exists() {
            tracing::debug!(
                "Completion script does not exist, skipping backup: {}",
                source.display()
            );
            continue;
        }

        let backup_path = backup_dir.join(file_name);

        // 复制文件
        fs::copy(&source, &backup_path).map_err(|e| RollbackError::BackupCompletionFailed {
            src: source.display().to_string(),
            dest: backup_path.display().to_string(),
            message: e.to_string(),
        })?;

        tracing::debug!(
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
pub fn create_backup() -> Result<BackupResult, RollbackError> {
    tracing::info!("Creating backup");

    // 创建备份目录
    let backup_dir = create_backup_dir()?;

    // 备份二进制文件
    let binaries = command_names();
    let binary_backups = backup_binaries(&backup_dir, binaries)?;

    // 备份补全脚本（不依赖 shell 检测）
    let comp_dir = completion_dir()?;
    let completion_backups = backup_completions(&backup_dir, &comp_dir)?;

    // 先获取计数，再转移所有权，避免不必要的 clone
    let binary_count = binary_backups.len();
    let completion_count = completion_backups.len();

    let backup_info = BackupInfo {
        backup_dir,
        binary_backups,
        completion_backups,
    };

    tracing::info!(
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
