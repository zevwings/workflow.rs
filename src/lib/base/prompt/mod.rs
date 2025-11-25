//! Prompt 管理模块
//!
//! 本模块提供了 Prompt 常量定义。
//! Prompt 内容作为编译时常量直接嵌入到二进制文件中。
//!
//! ## 使用示例
//!
//! ```rust
//! use workflow::base::prompt::GENERATE_BRANCH_SYSTEM_PROMPT;
//!
//! // 直接使用编译时嵌入的 prompt
//! let system_prompt = GENERATE_BRANCH_SYSTEM_PROMPT.to_string();
//! ```

#[path = "generate_branch.system.rs"]
pub mod generate_branch_system;
pub mod languages;
#[path = "summarize_pr.system.rs"]
pub mod summarize_pr_system;

// 重新导出公共 API
pub use generate_branch_system::GENERATE_BRANCH_SYSTEM_PROMPT;
pub use languages::{
    find_language, get_language_instruction, get_supported_language_codes,
    get_supported_language_display_names, SupportedLanguage, SUPPORTED_LANGUAGES,
};
pub use summarize_pr_system::{generate_summarize_pr_system_prompt, SUMMARIZE_PR_SYSTEM_PROMPT};
