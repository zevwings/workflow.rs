//! Jira 命令公共帮助函数
//!
//! 提供 Jira 命令之间共享的公共功能，避免代码重复。

use crate::base::constants::errors::input_reading;
use crate::base::dialog::InputDialog;
use chrono::{DateTime, FixedOffset};
use color_eyre::{eyre::WrapErr, Result};

/// 输出格式选项
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
    Markdown,
}

impl OutputFormat {
    /// 从命令行参数确定输出格式（优先级：json > yaml > markdown > table（默认））
    pub fn from_args(table: bool, json: bool, yaml: bool, markdown: bool) -> Self {
        // table 参数用于显式指定，但默认就是 table
        let _ = table; // 避免未使用警告
        if json {
            Self::Json
        } else if yaml {
            Self::Yaml
        } else if markdown {
            Self::Markdown
        } else {
            Self::Table
        }
    }
}

// 为 OutputFormatArgs 添加转换方法
impl From<&crate::cli::OutputFormatArgs> for OutputFormat {
    fn from(args: &crate::cli::OutputFormatArgs) -> Self {
        OutputFormat::from_args(args.table, args.json, args.yaml, args.markdown)
    }
}

/// 获取 JIRA ID（从参数或交互式输入）
///
/// # 参数
///
/// * `jira_id` - 可选的 JIRA ID 参数
/// * `prompt_message` - 交互式提示消息（如果 jira_id 为 None）
///
/// # 返回
///
/// 返回 JIRA ID 字符串
pub fn get_jira_id(jira_id: Option<String>, prompt_message: Option<&str>) -> Result<String> {
    if let Some(id) = jira_id {
        Ok(id)
    } else {
        let message = prompt_message.unwrap_or("Enter Jira ticket ID (e.g., PROJ-123)");
        InputDialog::new(message)
            .prompt()
            .wrap_err(input_reading::READ_JIRA_TICKET_ID_FAILED)
    }
}

/// 格式化日期时间字符串
///
/// 支持 RFC3339 格式和其他常见格式的日期时间解析。
///
/// # 参数
///
/// * `date_str` - 日期时间字符串
///
/// # 返回
///
/// 格式化后的日期时间字符串（格式：YYYY-MM-DD HH:MM:SS）
pub fn format_date(date_str: &str) -> Result<String> {
    DateTime::<FixedOffset>::parse_from_rfc3339(date_str)
        .or_else(|_| {
            // 尝试其他格式
            DateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.3f%z")
        })
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .or_else(|_| Ok(date_str.to_string()))
}
