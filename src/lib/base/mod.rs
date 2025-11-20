//! Base 基础设施模块
//!
//! 本模块包含所有不关联业务的通用基础设施代码，包括：
//! - HTTP 客户端和网络工具
//! - 工具函数（logger, string, browser, clipboard, checksum, unzip, confirm）
//! - 配置管理
//! - Shell 检测和管理
//! - LLM 客户端（通用 LLM 接口）

pub mod http;
pub mod llm;
pub mod settings;
pub mod shell;
pub mod util;

// 重新导出常用类型，方便使用
pub use http::{Authorization, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig};
pub use settings::{LLMSettings, Paths, Settings};
pub use shell::{Detect, Reload, ShellConfigManager};
pub use util::{
    confirm, mask_sensitive_value, Browser, Checksum, Clipboard, LogLevel, Logger, Unzip,
};
