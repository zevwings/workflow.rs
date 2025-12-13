//! TOML 配置管理器
//!
//! 本模块提供了统一的 TOML 配置文件读写功能，支持：
//! - 自动创建默认配置（文件不存在时）
//! - 统一的错误处理
//! - Unix 系统下的文件权限设置（600）

use color_eyre::{eyre::WrapErr, Result};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::marker::PhantomData;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use super::status::ProjectStatusConfig;

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
/// use workflow::base::settings::paths::Paths;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// struct MyConfig {
///     some_field: String,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config_path = Paths::jira_config()?;
/// let manager = ConfigManager::<MyConfig>::new(config_path);
///
/// // 读取配置
/// let config = manager.read()?;
///
/// // 更新配置
/// let new_value = "new_value".to_string();
/// manager.update(|config| {
///     config.some_field = new_value;
/// })?;
///
/// // 写入配置
/// let updated_config = manager.read()?;
/// manager.write(&updated_config)?;
/// # Ok(())
/// # }
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
            .wrap_err(format!("Failed to read config file: {:?}", self.path))?;
        toml::from_str(&content).wrap_err(format!("Failed to parse TOML config: {:?}", self.path))
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
        let toml_content =
            toml::to_string_pretty(config).wrap_err("Failed to serialize config to TOML")?;
        fs::write(&self.path, toml_content)
            .wrap_err(format!("Failed to write config file: {:?}", self.path))?;
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
            .wrap_err("Failed to set config file permissions")?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_permissions(&self) -> Result<()> {
        Ok(())
    }
}

// ==================== Jira 配置结构体 ====================

/// Jira 用户条目（TOML）
///
/// 用于存储单个用户的 Jira 信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUserEntry {
    /// 用户邮箱（必需，用于查找）
    pub email: String,
    /// 账户 ID
    pub account_id: String,
    /// 显示名称
    pub display_name: String,
}

/// 合并后的 Jira 配置
///
/// 将 `jira-users.toml` 和 `jira-status.toml` 合并为 `jira.toml`。
///
/// TOML 格式示例：
/// ```toml
/// [[users]]
/// email = "user@example.com"
/// account_id = "628d9616269a9a0068f27e0c"
/// display_name = "User Name"
///
/// [status.WEW]
/// created-pr = "In Progress"
/// merged-pr = "In Review"
///
/// [status.NA]
/// created-pr = "In Progress"
/// merged-pr = "In Review"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JiraConfig {
    /// 用户列表
    #[serde(default)]
    pub users: Vec<JiraUserEntry>,

    /// 项目状态配置映射
    /// 使用 `[status.PROJECT_KEY]` 格式存储每个项目的状态配置
    #[serde(default, rename = "status")]
    pub status: HashMap<String, ProjectStatusConfig>,
}
