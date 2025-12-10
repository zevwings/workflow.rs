//! 配置命令的辅助函数
//!
//! 提供可复用的交互式选择函数，用于配置设置。
//! 同时提供配置解析和提取的共享函数，减少代码冗余。

use crate::base::dialog::SelectDialog;
use crate::base::llm::{get_supported_language_display_names, SUPPORTED_LANGUAGES};
use crate::base::settings::settings::Settings;
use anyhow::{Context, Result};
use std::path::Path;

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
/// ```rust,no_run
/// use workflow::commands::config::helpers::select_language;
///
/// let language = select_language(Some("zh-CN"))?;
/// // 用户从列表中选择后，返回 "zh-CN"
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn select_language(current_language: Option<&str>) -> Result<String> {
    // 获取所有支持的语言显示名称
    let language_display_names = get_supported_language_display_names();

    // 查找当前语言的索引
    let current_idx = current_language
        .and_then(|code| SUPPORTED_LANGUAGES.iter().position(|lang| lang.code == code))
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

/// 解析配置文件（支持 TOML、JSON、YAML）
///
/// 根据文件扩展名或内容自动检测配置格式并解析。
///
/// # 参数
///
/// * `content` - 配置文件内容
/// * `path` - 配置文件路径（用于确定格式）
///
/// # 返回
///
/// 解析后的 `Settings` 实例
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::commands::config::helpers::parse_config;
/// use std::path::Path;
///
/// let content = r#"[jira]
/// api_token = "test"
/// "#;
/// let path = Path::new("config.toml");
/// let settings = parse_config(content, path)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_config(content: &str, path: &Path) -> Result<Settings> {
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("toml").to_lowercase();

    match extension.as_str() {
        "toml" => toml::from_str::<Settings>(content).context("Failed to parse TOML config file"),
        "json" => {
            serde_json::from_str::<Settings>(content).context("Failed to parse JSON config file")
        }
        "yaml" | "yml" => {
            serde_yaml::from_str::<Settings>(content).context("Failed to parse YAML config file")
        }
        _ => {
            // 尝试自动检测格式
            if content.trim_start().starts_with('{') {
                serde_json::from_str::<Settings>(content)
                    .context("Failed to parse JSON config file")
            } else if content.trim_start().starts_with("---") || content.contains(':') {
                serde_yaml::from_str::<Settings>(content)
                    .context("Failed to parse YAML config file")
            } else {
                toml::from_str::<Settings>(content).context("Failed to parse TOML config file")
            }
        }
    }
}

/// 提取特定配置段
///
/// 从完整的配置中提取指定配置段（如 jira、github、log、llm）。
///
/// # 参数
///
/// * `settings` - 完整的配置对象
/// * `section` - 要提取的配置段名称
///
/// # 返回
///
/// 只包含指定配置段的 `Settings` 实例
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::commands::config::helpers::extract_section;
/// use workflow::base::settings::settings::Settings;
///
/// let settings = Settings::default();
/// let jira_config = extract_section(&settings, "jira")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn extract_section(settings: &Settings, section: &str) -> Result<Settings> {
    let mut extracted = Settings::default();

    match section.to_lowercase().as_str() {
        "jira" => {
            extracted.jira = settings.jira.clone();
        }
        "github" => {
            extracted.github = settings.github.clone();
        }
        "log" => {
            extracted.log = settings.log.clone();
        }
        "llm" => {
            extracted.llm = settings.llm.clone();
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown section: '{}'. Valid sections: jira, github, log, llm",
                section
            ));
        }
    }

    Ok(extracted)
}
