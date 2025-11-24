//! Prompt 管理模块
//!
//! 本模块提供了统一的 Prompt 管理功能，支持：
//! - 从文件加载 Prompt（便于维护和版本控制）
//! - 默认值回退（如果文件不存在，使用代码中的默认值）
//! - Prompt 缓存（避免重复读取文件）
//!
//! ## 使用示例
//!
//! ```rust
//! use workflow::base::prompt::PromptManager;
//!
//! // 加载 Prompt，如果文件不存在则使用默认值
//! let prompt = PromptManager::load_or_default("generate_branch.system.md", || {
//!     "Default system prompt".to_string()
//! })?;
//!
//! // 直接加载 Prompt（如果文件不存在会返回错误）
//! let prompt = PromptManager::load("generate_branch.system.md")?;
//! ```

pub mod manager;

// 重新导出公共 API
pub use manager::PromptManager;

