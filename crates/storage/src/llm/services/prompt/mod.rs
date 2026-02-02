//! LLM Prompt 模块
//!
//! 本模块提供了所有 LLM prompt 的便捷访问方法。
//!
//! ## 使用方式
//!
//! ```rust
//! use storage::llm::prompt;
//!
//! // 获取基础 prompt 内容
//! let create_prompt = prompt::create();
//! let summarize_prompt = prompt::summarize();
//!
//! // 获取带语言增强的 prompt
//! let summarize_prompt_with_lang = prompt::summarize_with_language("en");
//! ```

use domain::SupportedLanguage;

/// PR 创建 prompt
///
/// 用于根据 commit 标题和 git 变更生成分支名、PR 标题和描述。
pub const fn create() -> &'static str {
    include_str!("create.md")
}

/// PR 重写 prompt
///
/// 用于根据当前 PR 标题和 PR diff 生成更新的 PR 标题和描述。
pub const fn reword() -> &'static str {
    include_str!("reword.md")
}

/// PR 总结 prompt（基础版本）
///
/// 用于根据 PR 的 diff 内容生成总结文档。
/// 注意：此版本不包含语言增强，如需语言增强请使用 `summarize_with_language()`。
pub const fn summarize() -> &'static str {
    include_str!("summarize.md")
}

/// PR 总结 prompt（带语言增强）
///
/// 用于根据 PR 的 diff 内容生成总结文档，并根据语言代码自动添加语言要求。
///
/// # 参数
///
/// * `language_code` - 语言代码（如 "en", "zh"）
///
/// # 返回
///
/// 返回增强后的 system prompt，包含强化的语言要求
pub fn summarize_with_language(language_code: &str) -> String {
    SupportedLanguage::get_requirement(summarize(), language_code)
}

/// 文件修改总结 prompt（基础版本）
///
/// 用于根据文件的 diff 内容生成该文件的修改总结。
/// 注意：此版本不包含语言增强，如需语言增强请使用 `file_summary_with_language()`。
pub const fn file_summary() -> &'static str {
    include_str!("file_summary.md")
}

/// 文件修改总结 prompt（带语言增强）
///
/// 用于根据文件的 diff 内容生成该文件的修改总结，并根据语言代码自动添加语言要求。
///
/// # 参数
///
/// * `language_code` - 语言代码（如 "en", "zh"）
///
/// # 返回
///
/// 返回增强后的 system prompt，包含强化的语言要求
pub fn file_summary_with_language(language_code: &str) -> String {
    SupportedLanguage::get_requirement(file_summary(), language_code)
}

/// 翻译 prompt
///
/// 用于将非英文文本（中文、俄文等）翻译为英文。
pub const fn translate() -> &'static str {
    include_str!("translate.md")
}
