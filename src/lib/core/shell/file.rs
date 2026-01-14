//! Shell 配置文件操作工具
//!
//! 提供 shell 配置文件的读取和写入功能。

use color_eyre::{eyre::WrapErr, Result};
use std::fs;

/// 读取配置文件内容
///
/// # 参数
///
/// * `path` - 配置文件路径
///
/// # 返回
///
/// 返回配置文件内容。如果文件不存在，返回空字符串。
///
/// # 错误
///
/// 如果读取文件失败，返回相应的错误信息。
pub fn read_config_file(path: &std::path::Path) -> Result<String> {
    if path.exists() {
        fs::read_to_string(path).wrap_err("Failed to read shell config file")
    } else {
        Ok(String::new())
    }
}

/// 写入配置文件内容
///
/// # 参数
///
/// * `path` - 配置文件路径
/// * `content` - 要写入的内容
///
/// # 错误
///
/// 如果写入文件失败，返回相应的错误信息。
pub fn write_config_file(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .wrap_err_with(|| format!("Failed to create parent directory: {:?}", parent))?;
    }
    fs::write(path, content).wrap_err("Failed to write to shell config file")?;
    Ok(())
}
