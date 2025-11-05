use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::git::Git;
use crate::http::{HttpClient, HttpResponse};
use crate::log_info;
use crate::settings::Settings;

use super::helpers::extract_github_repo_from_url;
use super::provider::PlatformProvider;

/// GitHub API 模块
pub struct GitHub;

#[derive(Debug, Serialize)]
struct CreatePullRequestRequest {
    title: String,
    body: String,
    head: String,
    base: String,
}

#[derive(Debug, Deserialize)]
struct CreatePullRequestResponse {
    html_url: String,
}

#[derive(Debug, Serialize)]
struct MergePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_message: Option<String>,
    merge_method: String,
}

#[derive(Debug, Deserialize)]
struct PullRequestInfo {
    number: u64,
    title: String,
    body: Option<String>,
    state: String,
    html_url: String,
    head: PullRequestBranch,
    base: PullRequestBranch,
}

#[derive(Debug, Deserialize)]
struct PullRequestBranch {
    #[serde(rename = "ref")]
    ref_name: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryInfo {
    default_branch: String,
    #[serde(rename = "allow_squash_merge")]
    allow_squash_merge: Option<bool>,
    #[serde(rename = "allow_merge_commit")]
    allow_merge_commit: Option<bool>,
    #[serde(rename = "allow_rebase_merge")]
    allow_rebase_merge: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct GitHubErrorResponse {
    message: String,
    errors: Option<Vec<GitHubError>>,
}

#[derive(Debug, Deserialize)]
struct GitHubError {
    resource: Option<String>,
    field: Option<String>,
    code: Option<String>,
}

impl PlatformProvider for GitHub {
    /// 创建 Pull Request
    fn create_pull_request(
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;

        // 如果没有指定目标分支，获取仓库的默认分支
        let base_branch = if let Some(branch) = target_branch {
            branch.to_string()
        } else {
            Self::get_default_branch(&owner, &repo_name)?
        };

        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: source_branch.to_string(),
            base: base_branch.clone(),
        };

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;

        // 先尝试解析为通用 JSON 以检查状态码
        let response: HttpResponse<serde_json::Value> = client
            .post(&url, &request, None, Some(headers))
            .context("Failed to create PR via GitHub API")?;

        // 如果请求失败，尝试解析错误响应
        if !response.is_success() {
            let error_msg = if let Ok(error) =
                serde_json::from_value::<GitHubErrorResponse>(response.data.clone())
            {
                let mut msg = format!(
                    "GitHub API error: {} (Status: {})",
                    error.message, response.status
                );
                if let Some(errors) = error.errors {
                    for err in errors {
                        if let (Some(resource), Some(field), Some(code)) =
                            (err.resource, err.field, err.code)
                        {
                            msg.push_str(&format!(
                                "\n  - {}: {} field is invalid ({})",
                                resource, field, code
                            ));
                        }
                    }
                }
                msg
            } else {
                format!(
                    "GitHub API request failed: {} - {}",
                    response.status,
                    serde_json::to_string_pretty(&response.data)
                        .unwrap_or_else(|_| "Unknown error".to_string())
                )
            };
            anyhow::bail!("{}", error_msg);
        }

        // 解析成功响应
        let response_data: CreatePullRequestResponse =
            serde_json::from_value(response.data).context("Failed to parse success response")?;

        Ok(response_data.html_url)
    }

    /// 合并 Pull Request
    fn merge_pull_request(pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        // 检测仓库支持的合并方法：优先使用 squash，否则使用 merge
        let merge_method = Self::get_preferred_merge_method(&owner, &repo_name)?;
        log_info!("Using merge method: {}", merge_method);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/merge",
            owner, repo_name, pr_number
        );

        let request = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method,
        };

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<serde_json::Value> = client
            .put(&url, &request, None, Some(headers))
            .context(format!("Failed to merge PR: {}", pull_request_id))?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub merge request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        // 如果需要删除分支，调用删除分支 API
        if delete_branch {
            // 先获取 PR 信息以获取源分支名
            let pr_info_url = format!(
                "https://api.github.com/repos/{}/{}/pulls/{}",
                owner, repo_name, pr_number
            );
            let client = Self::get_client()?;
            let headers = Self::get_headers()?;
            let pr_response: HttpResponse<PullRequestInfo> = client
                .get(&pr_info_url, None, Some(headers))
                .context("Failed to get PR info for branch deletion")?;

            if pr_response.is_success() {
                let branch_name = pr_response.data.head.ref_name;
                let branch_url = format!(
                    "https://api.github.com/repos/{}/{}/git/refs/heads/{}",
                    owner, repo_name, branch_name
                );
                let delete_response: HttpResponse<serde_json::Value> = client
                    .delete(&branch_url, None, Some(headers))
                    .context("Failed to delete branch")?;
                // 忽略删除分支的错误（分支可能已经被删除）
                if !delete_response.is_success() && delete_response.status != 404 {
                    // 404 表示分支不存在，这是正常的，可以忽略
                }
            }
        }

        Ok(())
    }

    /// 获取 PR 信息
    fn get_pull_request_info(pull_request_id: &str) -> Result<String> {
        use std::fmt::Write;

        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;

        let mut info = String::new();
        writeln!(info, "Title: {}", pr.title)?;
        if let Some(body) = pr.body {
            writeln!(info, "Description: {}", body)?;
        }
        writeln!(info, "State: {}", pr.state)?;
        writeln!(info, "Source Branch: {}", pr.head.ref_name)?;
        writeln!(info, "Target Branch: {}", pr.base.ref_name)?;
        writeln!(info, "URL: {}", pr.html_url)?;

        Ok(info)
    }

    /// 获取 PR URL
    #[allow(dead_code)]
    fn get_pull_request_url(pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.html_url)
    }

    /// 获取 PR 标题
    fn get_pull_request_title(pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.title)
    }

    /// 列出 PR
    fn get_pull_requests(state: Option<&str>, limit: Option<u32>) -> Result<String> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;

        // 转换 state 参数：GitHub API 支持 "open", "closed", "all"
        let state = match state {
            Some("open") => "open",
            Some("closed") => "closed",
            Some("merged") => "closed", // GitHub API 中 merged 是 closed 状态的一种
            Some("all") | None => "all",
            _ => "all", // 默认显示所有状态
        };
        let per_page = limit.unwrap_or(30).min(100); // GitHub API 限制最多 100

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state={}&per_page={}",
            owner, repo_name, state, per_page
        );

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<Vec<PullRequestInfo>> = client
            .get(&url, None, Some(headers))
            .context("Failed to list PRs via GitHub API")?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        use std::fmt::Write;

        let mut output = String::new();
        for pr in response.data {
            writeln!(
                output,
                "#{}  {}  [{}]  {}\n    {}",
                pr.number, pr.state, pr.head.ref_name, pr.title, pr.html_url
            )?;
        }

        if output.is_empty() {
            output.push_str("No PRs found.");
        }

        Ok(output)
    }

    /// 获取当前分支的 PR
    fn get_current_branch_pull_request() -> Result<Option<String>> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let current_branch = Git::current_branch()?;

        // 使用 head 参数查找当前分支的 PR
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=open",
            owner, repo_name, owner, current_branch
        );

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<Vec<PullRequestInfo>> = client
            .get(&url, None, Some(headers))
            .context("Failed to get current branch PR via GitHub API")?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        match response.data.first() {
            Some(pr) => Ok(Some(pr.number.to_string())),
            None => Ok(None),
        }
    }
}

impl GitHub {
    /// 获取 GitHub 仓库信息（owner/repo）
    fn get_repo() -> Result<String> {
        let remote_url = Git::get_remote_url().context("Failed to get remote URL")?;
        extract_github_repo_from_url(&remote_url)
            .context("Failed to extract GitHub repo from remote URL")
    }

    /// 解析仓库字符串为 owner 和 repo_name
    fn parse_repo(repo: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repo format: {}", repo);
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 获取仓库的默认分支
    fn get_default_branch(owner: &str, repo_name: &str) -> Result<String> {
        let repo_info = Self::get_repository_info(owner, repo_name)?;
        Ok(repo_info.default_branch)
    }

    /// 获取仓库信息
    fn get_repository_info(owner: &str, repo_name: &str) -> Result<RepositoryInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo_name);
        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<RepositoryInfo> = client
            .get(&url, None, Some(headers))
            .context("Failed to get repository info")?;

        if !response.is_success() {
            anyhow::bail!(
                "Failed to get repository info: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(response.data)
    }

    /// 获取首选的合并方法：优先使用 squash，其次 rebase，最后 merge
    fn get_preferred_merge_method(owner: &str, repo_name: &str) -> Result<String> {
        let repo_info = Self::get_repository_info(owner, repo_name)?;

        // 优先级：squash > rebase > merge
        // 1. 优先使用 squash，如果支持的话
        if repo_info.allow_squash_merge.unwrap_or(false) {
            return Ok("squash".to_string());
        }

        // 2. 其次使用 rebase，如果支持的话
        if repo_info.allow_rebase_merge.unwrap_or(false) {
            return Ok("rebase".to_string());
        }

        // 3. 最后使用 merge，如果支持的话
        if repo_info.allow_merge_commit.unwrap_or(false) {
            return Ok("merge".to_string());
        }

        // 如果都不支持，返回错误
        anyhow::bail!("Repository does not support squash, rebase, or merge commit methods");
    }

    /// 内部方法：获取 PR 信息（不缓存，避免数据不一致）
    fn fetch_pr_info_internal(pr_number: u64) -> Result<PullRequestInfo> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let client = Self::get_client()?;
        let headers = Self::get_headers()?;
        let response: HttpResponse<PullRequestInfo> = client
            .get(&url, None, Some(headers))
            .context(format!("Failed to get PR info: {}", pr_number))?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(response.data)
    }

    /// 获取缓存的 HTTP 客户端
    fn get_client() -> Result<&'static HttpClient> {
        static CLIENT: OnceLock<Result<HttpClient>> = OnceLock::new();
        CLIENT
            .get_or_init(HttpClient::new)
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))
    }

    /// 获取缓存的 headers
    fn get_headers() -> Result<&'static HeaderMap> {
        static HEADERS: OnceLock<Result<HeaderMap>> = OnceLock::new();
        HEADERS
            .get_or_init(Self::create_headers)
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to create headers: {}", e))
    }

    /// 创建 GitHub API 请求的 headers（内部方法）
    fn create_headers() -> Result<HeaderMap> {
        let settings = Settings::get();
        let token = settings
            .github_api_token
            .as_ref()
            .context("GITHUB_API_TOKEN environment variable not set")?;

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .context("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json"
                .parse()
                .context("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28"
                .parse()
                .context("Failed to parse X-GitHub-Api-Version header")?,
        );
        headers.insert(
            "User-Agent",
            "workflow-cli"
                .parse()
                .context("Failed to parse User-Agent header")?,
        );

        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::super::helpers::extract_pull_request_id_from_url;

    #[test]
    fn test_extract_pull_request_id_from_url() {
        assert_eq!(
            extract_pull_request_id_from_url("https://github.com/owner/repo/pull/123").unwrap(),
            "123"
        );
        assert_eq!(
            extract_pull_request_id_from_url("https://github.com/owner/repo/pull/456/").unwrap(),
            "456"
        );
    }
}
