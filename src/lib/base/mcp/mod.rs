//! MCP (Model Context Protocol) 配置管理模块
//!
//! 本模块提供 MCP 配置文件的读写和管理功能，支持：
//! - 读取和写入 `.cursor/mcp.json` 配置文件
//! - 检测已配置的 MCP 服务器
//! - 合并配置（不覆盖已有配置）
//! - 验证配置格式

pub mod config;
