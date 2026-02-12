//! 版本管理模块
//!
//! 提供版本获取和比较功能。

use http::{Authorization, HttpClient, Response};
use prompt::{info, success, Spinner};
use toolkit::log_debug;

// Re-export VersionComparison from types for convenience
pub use crate::commands::update::types::VersionComparison;
use crate::commands::update::types::{GITHUB_API_BASE, REPO_NAME, REPO_OWNER};

/// 获取当前安装的版本号
///
/// 从编译时嵌入的版本号获取。
pub fn get_current_version() -> Option<String> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Some(VERSION.to_string())
}

/// 获取目标版本号
///
/// 如果指定了版本，使用指定版本；否则从 GitHub API 获取最新版本。
pub fn get_target_version(
    version: Option<String>,
    github_token: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match version {
        Some(v) => {
            info!("Using specified version: v{}", v);
            Ok(v)
        }
        None => fetch_latest_version(github_token),
    }
}

/// 从 GitHub API 获取最新版本
fn fetch_latest_version(github_token: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/repos/{}/{}/releases/latest",
        GITHUB_API_BASE, REPO_OWNER, REPO_NAME
    );

    let spinner = Spinner::new("Fetching latest version...");
    let spinner_instance = spinner.start();

    let http_client = HttpClient::global()?;

    // 构建请求并发送
    let response = if let Some(token) = github_token {
        log_debug!("Using GitHub token for API request");
        http_client.get(&url).auth(Authorization::bearer(token)).send()
    } else {
        http_client.get(&url).send()
    }
    .map_err(|e| format!("Failed to fetch latest release from GitHub: {}", e));

    spinner_instance.stop();

    let response = response?;

    // 检查响应状态码
    handle_github_api_error(&response)?;

    // 解析响应（使用 serde_json::Value）
    let json: serde_json::Value = response.json()?;
    let tag_name = json.get("tag_name").and_then(|v| v.as_str()).ok_or_else(
        || -> Box<dyn std::error::Error> { "Missing tag_name in GitHub release response".into() },
    )?;

    let version = tag_name.trim_start_matches('v').to_string();

    success!("Latest version: v{}", version);
    Ok(version)
}

/// 处理 GitHub API 错误响应
fn handle_github_api_error(response: &Response) -> Result<(), Box<dyn std::error::Error>> {
    let status = response.status;

    if (200..300).contains(&status) {
        return Ok(());
    }

    let error_msg = match status {
        403 => {
            let rate_limit_remaining =
                response.header("x-ratelimit-remaining").and_then(|s| s.parse::<u32>().ok());

            if rate_limit_remaining == Some(0) {
                "Failed to fetch latest version: HTTP 403 (Rate limit exceeded)\n\
                Tip: Configure a GitHub token to increase rate limit from 60/hour to 5000/hour.\n\
                Run 'workflow setup' to configure your GitHub token."
                    .to_string()
            } else {
                "Failed to fetch latest version: HTTP 403 (Forbidden)\n\
                This may be due to repository access restrictions or network issues.\n\
                Tip: Configure a GitHub token to improve reliability.\n\
                Run 'workflow setup' to configure your GitHub token."
                    .to_string()
            }
        }
        404 => "Failed to fetch latest version: HTTP 404 (Not Found)\n\
            The repository or release may not exist, or you may not have access to it."
            .to_string(),
        429 => "Failed to fetch latest version: HTTP 429 (Too Many Requests)\n\
            Tip: Configure a GitHub token to increase rate limit from 60/hour to 5000/hour.\n\
            Run 'workflow setup' to configure your GitHub token."
            .to_string(),
        _ => {
            format!(
                "Failed to fetch latest version: HTTP {}\n\
                Please check your network connection and try again.",
                status
            )
        }
    };

    Err(error_msg.into())
}

/// 比较两个版本号
///
/// 返回版本比较结果。
pub fn compare_versions(current: impl AsRef<str>, target: impl AsRef<str>) -> VersionComparison {
    let current_parts: Vec<u32> =
        current.as_ref().split('.').filter_map(|s| s.parse().ok()).collect();
    let target_parts: Vec<u32> =
        target.as_ref().split('.').filter_map(|s| s.parse().ok()).collect();

    // 补齐到相同长度
    let max_len = current_parts.len().max(target_parts.len());
    let mut current_padded = current_parts;
    let mut target_padded = target_parts;
    current_padded.resize(max_len, 0);
    target_padded.resize(max_len, 0);

    // 逐级比较
    for (c, t) in current_padded.iter().zip(target_padded.iter()) {
        if c < t {
            return VersionComparison::NeedsUpdate;
        } else if c > t {
            return VersionComparison::Downgrade;
        }
    }

    VersionComparison::UpToDate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions_equal() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.3"),
            VersionComparison::UpToDate
        );
    }

    #[test]
    fn test_compare_versions_needs_update() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.1"),
            VersionComparison::NeedsUpdate
        );
        assert_eq!(
            compare_versions("1.0.0", "2.0.0"),
            VersionComparison::NeedsUpdate
        );
        assert_eq!(
            compare_versions("1.2.3", "1.3.0"),
            VersionComparison::NeedsUpdate
        );
    }

    #[test]
    fn test_compare_versions_downgrade() {
        assert_eq!(
            compare_versions("1.0.1", "1.0.0"),
            VersionComparison::Downgrade
        );
        assert_eq!(
            compare_versions("2.0.0", "1.0.0"),
            VersionComparison::Downgrade
        );
    }

    #[test]
    fn test_compare_versions_different_lengths() {
        assert_eq!(
            compare_versions("1.0", "1.0.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.0.0", "1.0"),
            VersionComparison::UpToDate
        );
        assert_eq!(
            compare_versions("1.0", "1.0.1"),
            VersionComparison::NeedsUpdate
        );
    }
}
