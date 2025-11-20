//! TOML 配置管理器
//!
//! 本模块提供了统一的 TOML 配置文件读写功能，支持：
//! - 自动创建默认配置（文件不存在时）
//! - 统一的错误处理
//! - Unix 系统下的文件权限设置（600）

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// TOML 配置管理器
///
/// 提供统一的 TOML 配置文件读写功能，支持：
/// - 自动创建默认配置（文件不存在时）
/// - 统一的错误处理
/// - Unix 系统下的文件权限设置（600）
///
/// # 类型参数
///
/// * `T` - 配置类型，必须实现 `Serialize`、`DeserializeOwned` 和 `Default` trait
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::jira::config::ConfigManager;
/// use crate::base::settings::paths::Paths;
///
/// let config_path = Paths::jira_users_config()?;
/// let manager = ConfigManager::<MyConfig>::new(config_path);
///
/// // 读取配置
/// let config = manager.read()?;
///
/// // 更新配置
/// manager.update(|config| {
///     config.some_field = new_value;
/// })?;
///
/// // 写入配置
/// manager.write(&config)?;
/// ```
pub struct ConfigManager<T> {
    path: PathBuf,
    _phantom: PhantomData<T>,
}

impl<T> ConfigManager<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// 创建新的配置管理器
    ///
    /// # 参数
    ///
    /// * `path` - 配置文件路径
    ///
    /// # 返回
    ///
    /// 返回 `ConfigManager` 实例。
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }

    /// 读取配置文件
    ///
    /// 如果文件不存在，返回默认配置。
    ///
    /// # 返回
    ///
    /// 返回解析后的配置对象。
    ///
    /// # 错误
    ///
    /// 如果文件存在但读取或解析失败，返回相应的错误信息。
    pub fn read(&self) -> Result<T> {
        if !self.path.exists() {
            return Ok(T::default());
        }
        let content = fs::read_to_string(&self.path)
            .context(format!("Failed to read config file: {:?}", self.path))?;
        toml::from_str(&content)
            .context(format!("Failed to parse TOML config: {:?}", self.path))
    }

    /// 写入配置文件
    ///
    /// 将配置对象序列化为 TOML 格式并写入文件。
    /// 在 Unix 系统上会自动设置文件权限为 600。
    ///
    /// # 参数
    ///
    /// * `config` - 要写入的配置对象
    ///
    /// # 错误
    ///
    /// 如果序列化或写入失败，返回相应的错误信息。
    pub fn write(&self, config: &T) -> Result<()> {
        let toml_content = toml::to_string_pretty(config)
            .context("Failed to serialize config to TOML")?;
        fs::write(&self.path, toml_content)
            .context(format!("Failed to write config file: {:?}", self.path))?;
        self.set_permissions()?;
        Ok(())
    }

    /// 更新配置文件
    ///
    /// 读取现有配置，应用更新函数，然后写回文件。
    ///
    /// # 参数
    ///
    /// * `f` - 更新函数，接收可变的配置对象引用
    ///
    /// # 错误
    ///
    /// 如果读取、更新或写入失败，返回相应的错误信息。
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut T),
    {
        let mut config = self.read()?;
        f(&mut config);
        self.write(&config)
    }

    #[cfg(unix)]
    fn set_permissions(&self) -> Result<()> {
        fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))
            .context("Failed to set config file permissions")?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_permissions(&self) -> Result<()> {
        Ok(())
    }
}
