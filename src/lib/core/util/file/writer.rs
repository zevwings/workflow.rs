//! 文件写入器
//!
//! 提供基于路径的文件写入操作。

use color_eyre::{eyre::WrapErr, Result};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

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
