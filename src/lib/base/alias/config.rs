//! 常用命令配置模块
//!
//! 管理用于交互式添加别名时的常用命令列表。

use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

/// 常用命令配置
///
/// 用于存储交互式添加别名时的常用命令列表。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandsConfig {
    /// 常用命令列表（用于交互式添加别名时的命令选择）
    #[serde(default)]
    pub common_commands: Vec<String>,
}

/// 常用命令配置管理器类型别名
pub type CommandsConfigManager = ConfigManager<CommandsConfig>;

impl CommandsConfig {
    /// 加载常用命令配置
    ///
    /// 从 `commands.toml` 配置文件加载常用命令列表。
    /// 如果文件不存在，返回默认配置（空列表）。
    ///
    /// # 返回
    ///
    /// 返回 `CommandsConfig` 实例。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取或解析失败，返回相应的错误信息。
    pub fn load() -> Result<Self> {
        let config_path = Paths::commands_config()?;
        let manager = CommandsConfigManager::new(config_path);
        manager.read()
    }

    /// 获取常用命令列表（混合方式）
    ///
    /// 优先级：
    /// 1. `commands.toml` 配置文件中的常用命令列表
    /// 2. 硬编码的默认常用命令列表
    ///
    /// # 返回
    ///
    /// 返回常用命令列表。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取失败，返回相应的错误信息。
    pub fn get_common_commands() -> Result<Vec<String>> {
        // 1. 优先从 commands.toml 配置文件读取
        if let Ok(config) = Self::load() {
            if !config.common_commands.is_empty() {
                return Ok(config.common_commands);
            }
        }

        // 2. 使用硬编码的默认常用命令列表
        const DEFAULT_COMMON_COMMANDS: &[&str] = &[
            "pr create",
            "pr merge",
            "pr status",
            "pr summarize",
            "pr approve",
            "jira info",
            "jira search",
            "branch create",
            "branch switch",
            "repo clean",
            "branch rename",
            "branch sync",
            "commit amend",
            "commit reword",
        ];

        Ok(DEFAULT_COMMON_COMMANDS.iter().map(|s| s.to_string()).collect())
    }
}
