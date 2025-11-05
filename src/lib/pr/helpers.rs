use crate::settings::Settings;
use anyhow::{Context, Result};

use super::constants::TYPES_OF_CHANGES;

/// 从 PR URL 提取 PR ID
///
/// # 示例
/// ```
/// assert_eq!(extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123"), Ok("123".to_string()));
/// assert_eq!(extract_pull_request_id_from_url("https://codeup.aliyun.com/xxx/project/xxx/code_reviews/12345"), Ok("12345".to_string()));
/// ```
pub fn extract_pull_request_id_from_url(url: &str) -> Result<String> {
    use regex::Regex;
    let re = Regex::new(r"/(\d+)(?:/|$)").context("Invalid regex pattern")?;
    let caps = re
        .captures(url)
        .context("Failed to extract PR ID from URL")?;

    Ok(caps.get(1).unwrap().as_str().to_string())
}

/// 从 Git remote URL 提取 GitHub 仓库的 owner/repo
///
/// # 示例
/// ```
/// assert_eq!(extract_github_repo_from_url("git@github.com:owner/repo.git"), Ok("owner/repo".to_string()));
/// assert_eq!(extract_github_repo_from_url("https://github.com/owner/repo.git"), Ok("owner/repo".to_string()));
/// ```
pub fn extract_github_repo_from_url(url: &str) -> Result<String> {
    use regex::Regex;

    // 匹配 SSH 格式: git@github.com:owner/repo.git
    let ssh_re =
        Regex::new(r"git@github\.com:(.+?)(?:\.git)?$").context("Invalid regex pattern")?;
    if let Some(caps) = ssh_re.captures(url) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    // 匹配 HTTPS 格式: https://github.com/owner/repo.git
    let https_re = Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$")
        .context("Invalid regex pattern")?;
    if let Some(caps) = https_re.captures(url) {
        return Ok(caps.get(1).unwrap().as_str().to_string());
    }

    anyhow::bail!("Failed to extract GitHub repo from URL: {}", url)
}

/// 生成 PR body
///
/// # Arguments
/// * `selected_change_types` - 选中的变更类型数组
/// * `short_description` - 简短描述（可选）
/// * `jira_ticket` - Jira ticket ID（可选）
pub fn generate_pull_request_body(
    selected_change_types: &[bool],
    short_description: Option<&str>,
    jira_ticket: Option<&str>,
) -> Result<String> {
    let mut body = String::from("\n# PR Ready\n\n## Types of changes\n\n");

    // 生成变更类型复选框
    for (i, change_type) in TYPES_OF_CHANGES.iter().enumerate() {
        let checked = if i < selected_change_types.len() && selected_change_types[i] {
            "[x]"
        } else {
            "[ ]"
        };
        body.push_str(&format!("{} {}\n", checked, change_type));
    }

    // 添加简短描述
    if let Some(desc) = short_description {
        if !desc.trim().is_empty() {
            body.push_str("\n#### Short description:\n\n");
            body.push_str(desc);
            body.push('\n');
        }
    }

    // 添加 Jira 链接
    if let Some(ticket) = jira_ticket {
        let settings = Settings::get();
        let jira_service = &settings.jira_service_address;
        if !jira_service.is_empty() {
            body.push_str(&format!(
                "\n#### Jira Link:\n\n{}/browse/{}\n",
                jira_service, ticket
            ));
        }
    }

    Ok(body)
}

/// 生成分支名
///
/// # Arguments
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `title` - PR 标题
pub fn generate_branch_name(jira_ticket: Option<&str>, title: &str) -> Result<String> {
    let mut branch_name = String::new();

    // 如果有 Jira ticket，添加到分支名前缀
    if let Some(ticket) = jira_ticket {
        branch_name.push_str(ticket);
        branch_name.push_str("--");
    }

    // 清理标题作为分支名
    let cleaned_title = transform_to_branch_name(title);
    branch_name.push_str(&cleaned_title);

    // 如果有 GITHUB_BRANCH_PREFIX，添加前缀
    let settings = Settings::get();
    if let Some(prefix) = &settings.github_branch_prefix {
        if !prefix.trim().is_empty() {
            branch_name = format!("{}/{}", prefix.trim(), branch_name);
        }
    }

    Ok(branch_name)
}

/// 生成 commit 标题
///
/// # Arguments
/// * `jira_ticket` - Jira ticket ID（可选）
/// * `title` - PR 标题
pub fn generate_commit_title(jira_ticket: Option<&str>, title: &str) -> String {
    match jira_ticket {
        Some(ticket) => format!("{}: {}", ticket, title),
        None => format!("# {}", title),
    }
}

/// 将字符串转换为分支名（替换特殊字符为连字符，去除重复连字符）
/// 只保留 ASCII 字母数字字符，过滤掉中文字符等其他非 ASCII 字符
pub fn transform_to_branch_name(s: &str) -> String {
    let mut result = String::new();
    let mut last_was_dash = false;

    for c in s.chars() {
        // 只保留 ASCII 字母数字字符
        if c.is_ascii_alphanumeric() {
            result.push(c.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            result.push('-');
            last_was_dash = true;
        }
    }

    result.trim_matches('-').to_string()
}
