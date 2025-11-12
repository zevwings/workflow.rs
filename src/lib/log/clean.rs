//! 清理模块
//! 清理日志目录

use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::path::{Path, PathBuf};

use crate::{log_break, log_info, log_success, Settings};

use super::utils::{calculate_dir_info, expand_path, format_size, list_dir_contents};

/// 显示目录信息
fn display_dir_info(dir_name: &str, dir: &Path, size: u64, file_count: usize) -> Result<()> {
    // 根据 dir_name 判断显示格式
    if dir_name.starts_with("the directory for") {
        // JIRA 目录格式：提取 JIRA ID
        if let Some(jira_id) = dir_name.strip_prefix("the directory for ") {
            log_info!("JIRA ID: {}", jira_id);
        }
    } else {
        // 基础目录格式
        log_info!("{}: {:?}", dir_name, dir);
    }
    log_info!("Directory: {:?}", dir);
    log_info!("Total size: {}", format_size(size));
    log_info!("Total files: {}", file_count);
    log_break!();
    log_info!("Contents:");
    let contents = list_dir_contents(dir)?;
    for path in contents {
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                log_info!("  📄 {} ({})", path.display(), format_size(metadata.len()));
            } else {
                log_info!("  📄 {}", path.display());
            }
        } else if path.is_dir() {
            log_info!("  📁 {}", path.display());
        }
    }
    Ok(())
}

/// 清理目录的通用实现
pub fn clean_dir(dir: &Path, dir_name: &str, dry_run: bool, list_only: bool) -> Result<bool> {
    if !dir.exists() {
        log_info!("Directory does not exist: {:?}", dir);
        return Ok(false);
    }

    let (size, file_count) = calculate_dir_info(dir)?;

    if list_only {
        display_dir_info(dir_name, dir, size, file_count)?;
        return Ok(false);
    }

    if dry_run {
        log_info!("[DRY RUN] Would delete {}: {:?}", dir_name, dir);
        log_info!("[DRY RUN] Total size: {}", format_size(size));
        log_info!("[DRY RUN] Total files: {}", file_count);
        return Ok(false);
    }

    display_dir_info(dir_name, dir, size, file_count)?;

    let confirmed = Confirm::new()
        .with_prompt(format!(
            "Are you sure you want to delete {}? This will remove {} files ({}).",
            dir_name,
            file_count,
            format_size(size)
        ))
        .default(false)
        .interact()
        .context("Failed to get confirmation")?;

    if !confirmed {
        log_info!("Clean operation cancelled.");
        return Ok(false);
    }

    std::fs::remove_dir_all(dir)
        .with_context(|| format!("Failed to delete {}: {:?}", dir_name, dir))?;

    log_success!("{} deleted successfully: {:?}", dir_name, dir);
    Ok(true)
}

/// 获取基础目录路径
/// 展开 ~ 路径并返回完整的基础目录路径
pub fn get_base_dir_path() -> Result<PathBuf> {
    let settings = Settings::load();
    expand_path(&settings.log_download_base_dir)
}
