//! MCP 配置类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MCPConfig {
    /// MCP 服务器配置
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub mcp_servers: HashMap<String, MCPServerConfig>,
}

impl MCPConfig {
    /// 检查 MCP 配置是否为空
    pub fn is_empty(&self) -> bool {
        self.mcp_servers.is_empty()
    }
}
