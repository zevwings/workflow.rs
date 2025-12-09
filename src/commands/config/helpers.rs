//! 配置命令的辅助函数
//!
//! 提供可复用的交互式选择函数，用于配置设置。

use crate::base::dialog::SelectDialog;
use crate::base::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use anyhow::{Context, Result};

/// 交互式选择语言
///
/// 显示所有支持的语言列表，让用户选择。
///
/// # 参数
///
/// * `current_language` - 当前配置的语言代码（可选，用于显示当前值）
///
/// # 返回
///
/// 返回用户选择的语言代码（如 "en", "zh-CN" 等）
///
/// # 示例
///
/// ```rust
/// let language = select_language(Some("zh-CN"))?;
/// // 用户从列表中选择后，返回 "zh-CN"
/// ```
pub fn select_language(current_language: Option<&str>) -> Result<String> {
    // 获取所有支持的语言显示名称
    let language_display_names = get_supported_language_display_names();

    // 查找当前语言的索引
    let current_idx = current_language
        .and_then(|code| {
            SUPPORTED_LANGUAGES
                .iter()
                .position(|lang| lang.code == code)
        })
        .unwrap_or(0); // 如果没有找到或没有当前值，默认选择第一个（英文）

    // 构建提示信息
    let prompt = if let Some(code) = current_language {
        format!("Select LLM output language [current: {}]", code)
    } else {
        "Select LLM output language".to_string()
    };

    // 显示选择列表
    let language_display_names_vec: Vec<String> = language_display_names.to_vec();
    let selected_display_name = SelectDialog::new(&prompt, language_display_names_vec)
        .with_default(current_idx)
        .prompt()
        .context("Failed to select language")?;

    // 查找选中的语言代码
    let selected_idx = language_display_names
        .iter()
        .position(|name| name == &selected_display_name)
        .context("Selected language not found")?;

    // 返回选中的语言代码
    Ok(SUPPORTED_LANGUAGES[selected_idx].code.to_string())
}
