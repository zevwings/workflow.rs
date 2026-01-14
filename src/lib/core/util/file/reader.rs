//! 文件读取器
//!
//! 提供基于路径的文件读取操作。

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;

/// 文件读取器，基于路径提供常用读取操作。
pub struct FileReader {
    path: PathBuf,
}

impl FileReader {
    /// 创建一个新的文件读取器。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 打开文件并返回 `BufReader<File>`。
    pub fn open(&self) -> Result<BufReader<File>> {
        let file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        Ok(BufReader::new(file))
    }

    /// 读取文件内容为字符串。
    pub fn to_string(&self) -> Result<String> {
        fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read file: {:?}", self.path))
    }

    /// 读取文件的所有行。
    pub fn lines(&self) -> Result<Vec<String>> {
        let file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line =
                line.wrap_err_with(|| format!("Failed to read line from file: {:?}", self.path))?;
            lines.push(line);
        }
        Ok(lines)
    }

    /// 读取文件内容为字节向量。
    pub fn bytes(&self) -> Result<Vec<u8>> {
        let mut file = File::open(&self.path)
            .wrap_err_with(|| format!("Failed to open file: {:?}", self.path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .wrap_err_with(|| format!("Failed to read file: {:?}", self.path))?;
        Ok(buffer)
    }

    /// 读取 TOML 文件并解析为类型 `T`。
    pub fn toml<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read config file: {:?}", self.path))?;
        toml::from_str(&content)
            .wrap_err_with(|| format!("Failed to parse TOML config: {:?}", self.path))
    }

    /// 读取 JSON 文件并解析为类型 `T`。
    pub fn json<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(&self.path)
            .wrap_err_with(|| format!("Failed to read JSON file: {:?}", self.path))?;
        serde_json::from_str(&content)
            .wrap_err_with(|| format!("Failed to parse JSON file: {:?}", self.path))
    }
}
