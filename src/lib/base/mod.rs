//! Base 基础设施模块
//!
//! 本模块包含所有不关联业务的通用基础设施代码，包括：
//! - HTTP 客户端和网络工具
//! - 日志功能（LogLevel, Logger, Tracer）
//! - 格式化工具（format - MessageFormatter, DisplayFormatter）
//! - 工具函数（string, platform, browser, clipboard, checksum, unzip）
//! - 交互式对话框（FormBuilder）
//!   - InputDialog、ConfirmDialog、SelectDialog、MultiSelectDialog 已迁移到 `base::interactive::dialog`
//!   - FormBuilder、FormResult 已迁移到 `base::interactive::form`
//! - 进度指示器（Spinner, Progress）- 位于 `interactive::output`
//! - 表格输出工具（TableBuilder, TableStyle）- 位于 `interactive::output::table`
//! - 配置管理
//! - Shell 检测和管理
//! - LLM 客户端（通用 LLM 接口）
//! - Prompt 管理（统一管理 LLM Prompt）

pub mod alias;
pub mod concurrent;
pub mod constants;
// pub mod dialog; // 已完全迁移到 base::interactive::dialog
pub mod format;
pub mod http;
pub mod interactive;
pub mod llm;
pub mod logger;
pub mod mcp;
pub mod prompt;
pub mod settings;
pub mod shell;
pub mod util;

// 重新导出常用类型，方便使用
pub use alias::{AliasManager, CommandsConfig};
pub use concurrent::{ConcurrentExecutor, TaskResult};
// 注意：InputDialog、ConfirmDialog、SelectDialog、MultiSelectDialog 已迁移到 base::interactive::dialog
// 请使用：
//   - base::interactive::dialog::input / crate::input!
//   - base::interactive::dialog::confirm / crate::confirm!
//   - base::interactive::dialog::select / crate::select!
//   - base::interactive::dialog::multiselect / crate::multiselect!
// FormBuilder 和 FormResult 已迁移到 base::interactive::form
// 请使用 base::interactive::form::{FormBuilder, FormResult}
pub use format::DisplayFormatter;
pub use http::{Authorization, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig};
pub use logger::{LogLevel, Tracer};
pub use prompt::GENERATE_BRANCH_SYSTEM_PROMPT;
pub use settings::{LLMSettings, Paths, Settings};
pub use shell::{Detect, Reload, ShellConfigManager};
pub use util::{mask_sensitive_value, Browser, Checksum, Clipboard, Unzip};
