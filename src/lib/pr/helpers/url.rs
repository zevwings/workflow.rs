//! URL 相关辅助函数
//!
//! 提供从 URL 中提取 PR ID 和仓库信息的函数。

use anyhow::{Context, Result};
use regex::Regex;

/// 从 PR URL 提取 PR ID
///
/// # 示例
/// ```
/// use workflow::pr::helpers::url::extract_pull_request_id_from_url;
/// assert_eq!(extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(), "123".to_string());
/// ```
pub fn extract_pull_request_id_from_url(url: &str) -> Result<String> {
    let re = Regex::new(r"/(\d+)(?:/|$)").context("Invalid regex pattern")?;
    let caps = re.captures(url).context("Failed to extract PR ID from URL")?;

    Ok(caps.get(1).unwrap().as_str().to_string())
}

/// 从 Git remote URL 提取 GitHub 仓库的 owner/repo
///
/// 支持标准格式和 SSH host 别名格式（如 github-brainim）
///
/// # 示例
/// ```
/// use workflow::pr::helpers::url::extract_github_repo_from_url;
/// assert_eq!(extract_github_repo_from_url("git@github.com:owner/repo.git").unwrap(), "owner/repo");
/// assert_eq!(extract_github_repo_from_url("git@github-brainim:owner/repo.git").unwrap(), "owner/repo");
/// assert_eq!(extract_github_repo_from_url("https://github.com/owner/repo.git").unwrap(), "owner/repo");
/// ```
pub fn extract_github_repo_from_url(url: &str) -> Result<String> {
    // 匹配 SSH 格式: git@github.com:owner/repo.git 或 git@github-xxx:owner/repo.git (支持 SSH host 别名)
    // 使用 git@github[^:]*: 来匹配 git@github 开头的所有 SSH host（包括别名）
    let ssh_re =
        Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$").context("Invalid regex pattern")?;
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
