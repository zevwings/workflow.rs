use domain::jira::validate_jira_ticket_format;
use prompt::input;

/// 交互式获取 JIRA ID（必填）
///
/// 如果提供了 JIRA ID，直接返回；否则提示用户输入。
///
/// # 参数
///
/// * `jira_id` - 可选的 JIRA ID
///
/// # 返回
///
/// 返回验证后的 JIRA ID 字符串
pub fn get_jira_id_interactive(
    jira_id: Option<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(id) = jira_id {
        // 验证格式
        validate_jira_ticket_format(&id).map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
        Ok(id)
    } else {
        // 交互式输入
        let id = input!("Please enter your JIRA ticket ID (e.g., PROJ-123):")
            .validator(|input: &str| {
                validate_jira_ticket_format(input)
                    .map_err(|e| format!("Invalid JIRA ID format: {}", e))
            })
            .prompt()
            .map_err(|e| format!("Failed to get JIRA ID: {}", e))?;
        Ok(id)
    }
}

/// 交互式获取 JIRA ID（可选）
///
/// 如果提供了 JIRA ID，直接返回；否则提示用户输入。
/// 用户可以跳过输入（按 Enter），返回 None。
///
/// # 参数
///
/// * `jira_id` - 可选的 JIRA ID
///
/// # 返回
///
/// 返回验证后的 JIRA ID 字符串，如果用户跳过则返回 None
pub fn get_jira_id_interactive_optional(
    jira_id: Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(id) = jira_id {
        // 验证格式
        validate_jira_ticket_format(&id).map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
        Ok(Some(id))
    } else {
        // 交互式输入
        let id = input!("Please enter your JIRA ticket ID (optional, press Enter to skip):")
            .prompt()
            .ok();

        if let Some(id) = id {
            let trimmed = id.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                // 验证格式
                validate_jira_ticket_format(trimmed)
                    .map_err(|e| format!("Invalid JIRA ID format: {}", e))?;
                Ok(Some(trimmed.to_string()))
            }
        } else {
            Ok(None)
        }
    }
}
