use client::{Authorization, HttpClientHolder, HttpResponse};
use di::Container;
use prompt::Spinner;
use toolkit::log_debug;

// ============================================================================
// 常量定义
// ============================================================================

/// GitHub API 基础 URL
pub const GITHUB_API_BASE: &str = "https://api.github.com";

/// 仓库所有者
pub const REPO_OWNER: &str = "zevwings";

/// 仓库名称
pub const REPO_NAME: &str = "workflow.rs";

/// 获取目标版本号
///
/// 如果指定了版本，使用指定版本；否则从 GitHub API 获取最新版本。
pub fn get_target_version(
    version: Option<String>,
    github_token: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    match version {
        Some(v) => {
            log_debug!("Using specified version: v{}", v);
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

    let spinner = Spinner::new("Fetching...");
    let spinner_instance = spinner.start();

    // 获取 HTTP 客户端
    let http_client = Container::global()
        .get()
        .map_err(|e| format!("Failed to get HTTP client: {}", e))?;
    let client = HttpClientHolder::new(http_client);

    // 构建请求并发送
    let response = if let Some(token) = github_token {
        log_debug!("Using GitHub token for API request");
        client.get(&url).auth(Authorization::bearer(token)).send()
    } else {
        client.get(&url).send()
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
    log_debug!("Latest version: v{}", version);
    Ok(version)
}

/// 处理 GitHub API 错误响应
fn handle_github_api_error(response: &HttpResponse) -> Result<(), Box<dyn std::error::Error>> {
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
