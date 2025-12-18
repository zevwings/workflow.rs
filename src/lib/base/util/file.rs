//! 文件操作工具
//!
//! 提供文件读取和写入的工具类型：
//! - `FileReader`：围绕路径的读取助手
//! - `FileWriter`：围绕路径的写入助手

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
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

/// 文件写入器，基于路径提供常用写入操作。
pub struct FileWriter {
    path: PathBuf,
}

impl FileWriter {
    /// 创建一个新的文件写入器。
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 确保父目录存在。
    ///
    /// 如果文件的父目录不存在，会自动创建所有必要的父目录。
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果创建目录失败，返回错误。
    pub fn ensure_parent_dir(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .wrap_err_with(|| format!("Failed to create parent directory: {:?}", parent))?;
        }
        Ok(())
    }

    /// 设置文件权限（仅 Unix 系统）。
    ///
    /// # 参数
    ///
    /// * `mode` - 文件权限模式（八进制，如 `0o600`）
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果设置权限失败，返回错误。
    #[cfg(unix)]
    pub fn set_permissions(&self, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&self.path, fs::Permissions::from_mode(mode))
            .wrap_err_with(|| format!("Failed to set file permissions: {:?}", self.path))?;
        Ok(())
    }

    /// 将字符串内容写入文件。
    pub fn write_str(&self, content: &str) -> Result<()> {
        fs::write(&self.path, content)
            .wrap_err_with(|| format!("Failed to write file: {:?}", self.path))
    }

    /// 将字符串内容写入文件（自动创建父目录）。
    ///
    /// 在写入前会自动创建所有必要的父目录。
    ///
    /// # 参数
    ///
    /// * `content` - 要写入的字符串内容
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_str_with_dir(&self, content: &str) -> Result<()> {
        self.ensure_parent_dir()?;
        self.write_str(content)
    }

    /// 将字节内容写入文件。
    pub fn write_bytes(&self, content: &[u8]) -> Result<()> {
        fs::write(&self.path, content)
            .wrap_err_with(|| format!("Failed to write file: {:?}", self.path))
    }

    /// 将字节内容写入文件（自动创建父目录）。
    ///
    /// 在写入前会自动创建所有必要的父目录。
    ///
    /// # 参数
    ///
    /// * `content` - 要写入的字节内容
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_bytes_with_dir(&self, content: &[u8]) -> Result<()> {
        self.ensure_parent_dir()?;
        self.write_bytes(content)
    }

    /// 将类型 `T` 序列化为 TOML 并写入文件。
    pub fn write_toml<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let toml_content = toml::to_string_pretty(data)
            .wrap_err_with(|| format!("Failed to serialize config to TOML: {:?}", self.path))?;
        self.write_str(&toml_content)
    }

    /// 将类型 `T` 序列化为 TOML 并写入文件（自动创建目录和设置权限）。
    ///
    /// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
    ///
    /// # 参数
    ///
    /// * `data` - 要序列化和写入的数据
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_toml_secure<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ensure_parent_dir()?;
        self.write_toml(data)?;
        #[cfg(unix)]
        self.set_permissions(0o600)?;
        Ok(())
    }

    /// 将类型 `T` 序列化为 JSON 并写入文件。
    pub fn write_json<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        let json_content = serde_json::to_string_pretty(data)
            .wrap_err_with(|| format!("Failed to serialize config to JSON: {:?}", self.path))?;
        self.write_str(&json_content)
    }

    /// 将类型 `T` 序列化为 JSON 并写入文件（自动创建目录和设置权限）。
    ///
    /// 在写入前会自动创建所有必要的父目录，并在 Unix 系统上设置文件权限为 `0o600`。
    ///
    /// # 参数
    ///
    /// * `data` - 要序列化和写入的数据
    ///
    /// # 返回
    ///
    /// 如果成功，返回 `Ok(())`；如果失败，返回错误。
    pub fn write_json_secure<T>(&self, data: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.ensure_parent_dir()?;
        self.write_json(data)?;
        #[cfg(unix)]
        self.set_permissions(0o600)?;
        Ok(())
    }
}
