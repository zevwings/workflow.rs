//! MCP 配置文件管理
//!
//! 提供 `.cursor/mcp.json` 配置文件的读写和管理功能。

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    /// 命令（如 "npx"）
    pub command: String,
    /// 命令参数
    pub args: Vec<String>,
    /// 环境变量
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// MCP 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    /// MCP 服务器配置
    #[serde(default, rename = "mcpServers")]
    pub mcp_servers: HashMap<String, MCPServerConfig>,
}

impl Default for MCPConfig {
    fn default() -> Self {
        Self {
            mcp_servers: HashMap::new(),
        }
    }
}

/// MCP 配置管理器
pub struct MCPConfigManager {
    config_path: PathBuf,
}

impl MCPConfigManager {
    /// 创建新的配置管理器
    ///
    /// 自动检测配置文件位置：
    /// 1. 项目目录下的 `.cursor/mcp.json`（优先）
    /// 2. 全局配置 `~/.cursor/mcp.json`（Windows: `%APPDATA%\Cursor\mcp.json`）
    pub fn new() -> Result<Self> {
        let config_path = Self::detect_config_path()?;
        Ok(Self { config_path })
    }

    /// 创建配置管理器，优先使用项目目录
    ///
    /// 如果项目目录下不存在 `.cursor/mcp.json`，会使用项目目录创建新配置。
    /// 这样可以让配置保存在项目中，便于团队协作。
    pub fn new_with_project_priority() -> Result<Self> {
        // 优先使用项目目录
        if let Ok(current_dir) = std::env::current_dir() {
            let project_config = current_dir.join(".cursor").join("mcp.json");
            // 如果项目目录下已存在配置，使用它
            // 如果不存在，也使用项目目录（创建新配置）
            return Ok(Self {
                config_path: project_config,
            });
        }

        // 如果无法获取当前目录，回退到全局配置
        let config_path = Self::detect_config_path()?;
        Ok(Self { config_path })
    }

    /// 检测配置文件路径
    ///
    /// 优先级：
    /// 1. 项目目录下的 `.cursor/mcp.json`
    /// 2. 全局配置目录下的 `mcp.json`
    ///
    /// 跨平台支持：
    /// - Windows: `%APPDATA%\Cursor\mcp.json`
    /// - Unix-like: `~/.cursor/mcp.json`
    fn detect_config_path() -> Result<PathBuf> {
        // 1. 检查项目目录下的 .cursor/mcp.json
        if let Ok(current_dir) = std::env::current_dir() {
            let project_config = current_dir.join(".cursor").join("mcp.json");
            if project_config.exists() {
                return Ok(project_config);
            }
        }

        // 2. 使用全局配置目录（跨平台）
        let global_config = if cfg!(target_os = "windows") {
            // Windows: 使用 %APPDATA%\Cursor\mcp.json
            let app_data = std::env::var("APPDATA")
                .context("APPDATA environment variable not set")?;
            PathBuf::from(app_data).join("Cursor").join("mcp.json")
        } else {
            // Unix-like: 使用 ~/.cursor/mcp.json
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            PathBuf::from(home).join(".cursor").join("mcp.json")
        };

        Ok(global_config)
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// 读取配置文件
    ///
    /// 如果文件不存在，返回默认配置（空的 mcp_servers）。
    pub fn read(&self) -> Result<MCPConfig> {
        if !self.config_path.exists() {
            return Ok(MCPConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .context(format!("Failed to read MCP config file: {:?}", self.config_path))?;

        let config: MCPConfig = serde_json::from_str(&content)
            .context(format!("Failed to parse MCP config JSON: {:?}", self.config_path))?;

        Ok(config)
    }

    /// 写入配置文件
    ///
    /// 自动创建目录和文件，并设置适当的权限。
    /// 跨平台支持：Windows 不设置权限，Unix-like 系统设置 600 权限。
    pub fn write(&self, config: &MCPConfig) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context(format!("Failed to create config directory: {:?}", parent))?;

            // 设置目录权限（仅 Unix，700）
            #[cfg(unix)]
            {
                fs::set_permissions(parent, fs::Permissions::from_mode(0o700))
                    .context("Failed to set config directory permissions")?;
            }
        }

        // 序列化为 JSON
        let content = serde_json::to_string_pretty(config)
            .context("Failed to serialize MCP config to JSON")?;

        // 写入文件
        fs::write(&self.config_path, content)
            .context(format!("Failed to write MCP config file: {:?}", self.config_path))?;

        // 设置文件权限（仅 Unix，600）
        #[cfg(unix)]
        {
            fs::set_permissions(&self.config_path, fs::Permissions::from_mode(0o600))
                .context("Failed to set config file permissions")?;
        }

        Ok(())
    }

    /// 更新配置文件
    ///
    /// 读取现有配置，应用更新函数，然后写回文件。
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut MCPConfig),
    {
        let mut config = self.read()?;
        f(&mut config);
        self.write(&config)
    }

    /// 合并配置
    ///
    /// 将新配置合并到现有配置中，不覆盖已有的 MCP 服务器配置。
    pub fn merge(&self, new_config: &MCPConfig) -> Result<()> {
        self.update(|existing| {
            for (name, server_config) in &new_config.mcp_servers {
                // 如果已存在，合并环境变量（不覆盖已有）
                if let Some(existing_server) = existing.mcp_servers.get_mut(name) {
                    for (key, value) in &server_config.env {
                        existing_server.env.entry(key.clone()).or_insert_with(|| value.clone());
                    }
                } else {
                    // 如果不存在，直接添加
                    existing.mcp_servers.insert(name.clone(), server_config.clone());
                }
            }
        })
    }

    /// 检测已配置的 MCP 服务器
    ///
    /// 返回已配置的 MCP 服务器名称集合。
    pub fn detect_configured_servers(&self) -> Result<std::collections::HashSet<String>> {
        let config = self.read()?;
        Ok(config.mcp_servers.keys().cloned().collect())
    }

    /// 检查特定 MCP 服务器是否已配置
    pub fn is_configured(&self, server_name: &str) -> Result<bool> {
        let config = self.read()?;
        Ok(config.mcp_servers.contains_key(server_name))
    }
}

