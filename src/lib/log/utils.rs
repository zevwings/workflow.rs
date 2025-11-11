//! 通用工具函数模块
//! 提供与业务逻辑无关的通用工具函数

use anyhow::{Context, Result};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 展开路径字符串（支持 ~ 和 ~/ 格式）
pub fn expand_path(path_str: &str) -> Result<PathBuf> {
    if let Some(rest) = path_str.strip_prefix("~/") {
        let home = env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(rest))
    } else if path_str == "~" {
        let home = env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home))
    } else {
        Ok(PathBuf::from(path_str))
    }
}

/// 打开日志文件并返回 BufReader
pub fn open_log_file(log_file: &Path) -> Result<BufReader<File>> {
    let file =
        File::open(log_file).with_context(|| format!("Failed to open log file: {:?}", log_file))?;
    Ok(BufReader::new(file))
}

/// 格式化文件大小
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// 计算目录大小和文件数量
pub fn calculate_dir_info(dir: &Path) -> Result<(u64, usize)> {
    let mut total_size = 0u64;
    let mut file_count = 0usize;

    if !dir.exists() {
        return Ok((0, 0));
    }

    for entry in WalkDir::new(dir) {
        let entry = entry.context("Failed to read directory entry")?;
        let metadata = entry.metadata().context("Failed to get file metadata")?;

        if metadata.is_file() {
            total_size += metadata.len();
            file_count += 1;
        }
    }

    Ok((total_size, file_count))
}

/// 列出目录内容
pub fn list_dir_contents(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut contents = Vec::new();

    if !dir.exists() {
        return Ok(contents);
    }

    for entry in WalkDir::new(dir).max_depth(3) {
        let entry = entry.context("Failed to read directory entry")?;
        contents.push(entry.path().to_path_buf());
    }

    Ok(contents)
}
