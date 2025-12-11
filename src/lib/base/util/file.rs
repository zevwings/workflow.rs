//! 文件操作工具
//!
//! 提供文件读取相关的工具函数。

use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// 文件读取器
///
/// 提供文件打开和读取功能。
pub struct FileReader;

impl FileReader {
    /// 打开文件并返回 BufReader
    ///
    /// 打开指定路径的文件，并返回一个缓冲读取器。
    ///
    /// # 参数
    ///
    /// * `file_path` - 文件路径
    ///
    /// # 返回
    ///
    /// 返回 `BufReader<File>`，用于读取文件内容。
    ///
    /// # 错误
    ///
    /// 如果文件无法打开，返回相应的错误信息。
    pub fn open(file_path: &Path) -> Result<BufReader<File>> {
        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;
        Ok(BufReader::new(file))
    }
}
