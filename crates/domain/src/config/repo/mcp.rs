//! MCP 配置类型定义

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_mcp_config_is_empty() {
        let config = MCPConfig::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_mcp_config_serialize_rename_and_roundtrip() {
        let mut servers = HashMap::new();
        servers.insert(
            "example".to_string(),
            MCPServerConfig {
                command: "npx".to_string(),
                args: vec!["server".to_string()],
                env: HashMap::new(),
            },
        );

        let config = MCPConfig {
            mcp_servers: servers,
        };
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("mcpServers"));

        let deserialized: MCPConfig = toml::from_str(&toml).unwrap();
        assert!(deserialized.mcp_servers.contains_key("example"));
    }
}
