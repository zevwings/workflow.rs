//! Base 基础设施模块
//!
//! 本模块包含所有不关联业务的通用基础设施代码，包括：
//! - HTTP 客户端和网络工具
//! - 日志功能（LogLevel, Logger, Tracer）
//! - 工具函数（string, format, platform, browser, clipboard, checksum, unzip）
//! - 交互式对话框（InputDialog, SelectDialog, MultiSelectDialog, ConfirmDialog）
//! - 进度指示器（Spinner, Progress）
//! - 配置管理
//! - Shell 检测和管理
//! - LLM 客户端（通用 LLM 接口）
//! - Prompt 管理（统一管理 LLM Prompt）

pub mod concurrent;
pub mod dialog;
pub mod http;
pub mod indicator;
pub mod llm;
pub mod logger;
pub mod prompt;
pub mod settings;
pub mod shell;
pub mod util;

// 重新导出常用类型，方便使用
pub use concurrent::{ConcurrentExecutor, TaskResult};
pub use dialog::{ConfirmDialog, InputDialog, MultiSelectDialog, SelectDialog};
pub use http::{Authorization, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig};
pub use indicator::{Progress, Spinner};
pub use logger::{LogLevel, Logger, Tracer};
pub use prompt::GENERATE_BRANCH_SYSTEM_PROMPT;
pub use settings::{LLMSettings, Paths, Settings};
pub use shell::{Detect, Reload, ShellConfigManager};
pub use util::{format_size, mask_sensitive_value, Browser, Checksum, Clipboard, Unzip};
