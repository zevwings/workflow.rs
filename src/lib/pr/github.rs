use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};

use crate::git::Git;
use crate::http::{HttpClient, HttpResponse};
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
        let base_branch = target_branch.unwrap_or("main");

        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: source_branch.to_string(),
            base: base_branch.to_string(),
        };

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<CreatePullRequestResponse> = client
            .post(&url, &request, None, Some(&headers))
            .context("Failed to create PR via GitHub API")?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(response.data.html_url)
    }

    /// 合并 Pull Request
    fn merge_pull_request(pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/merge",
            owner, repo_name, pr_number
        );

        let request = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method: "merge".to_string(),
        };

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<serde_json::Value> = client
            .put(&url, &request, None, Some(&headers))
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
            let client = HttpClient::new()?;
            let headers = Self::create_headers()?;
            let pr_response: HttpResponse<PullRequestInfo> = client
                .get(&pr_info_url, None, Some(&headers))
                .context("Failed to get PR info for branch deletion")?;

            if pr_response.is_success() {
                let branch_name = pr_response.data.head.ref_name;
                let branch_url = format!(
                    "https://api.github.com/repos/{}/{}/git/refs/heads/{}",
                    owner, repo_name, branch_name
                );
                let delete_response: HttpResponse<serde_json::Value> = client
                    .delete(&branch_url, None, Some(&headers))
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
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<PullRequestInfo> = client
            .get(&url, None, Some(&headers))
            .context(format!("Failed to get PR info: {}", pull_request_id))?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        let pr = response.data;
        let mut info = String::new();
        info.push_str(&format!("Title: {}\n", pr.title));
        if let Some(body) = pr.body {
            info.push_str(&format!("Description: {}\n", body));
        }
        info.push_str(&format!("State: {}\n", pr.state));
        info.push_str(&format!("Source Branch: {}\n", pr.head.ref_name));
        info.push_str(&format!("Target Branch: {}\n", pr.base.ref_name));
        info.push_str(&format!("URL: {}\n", pr.html_url));

        Ok(info)
    }

    /// 获取 PR URL
    #[allow(dead_code)]
    fn get_pull_request_url(pull_request_id: &str) -> Result<String> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<PullRequestInfo> = client
            .get(&url, None, Some(&headers))
            .context(format!("Failed to get PR URL: {}", pull_request_id))?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(response.data.html_url)
    }

    /// 获取 PR 标题
    fn get_pull_request_title(pull_request_id: &str) -> Result<String> {
        let repo = Self::get_repo()?;
        let (owner, repo_name) = Self::parse_repo(&repo)?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<PullRequestInfo> = client
            .get(&url, None, Some(&headers))
            .context(format!("Failed to get PR title: {}", pull_request_id))?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        Ok(response.data.title)
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

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<Vec<PullRequestInfo>> =
            client
                .get(&url, None, Some(&headers))
                .context("Failed to list PRs via GitHub API")?;

        if !response.is_success() {
            anyhow::bail!(
                "GitHub API request failed: {} - {}",
                response.status,
                response.status_text
            );
        }

        let mut output = String::new();
        for pr in response.data {
            output.push_str(&format!(
                "#{}  {}  [{}]  {}\n    {}\n",
                pr.number, pr.state, pr.head.ref_name, pr.title, pr.html_url
            ));
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

        let client = HttpClient::new()?;
        let headers = Self::create_headers()?;
        let response: HttpResponse<Vec<PullRequestInfo>> =
            client
                .get(&url, None, Some(&headers))
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

    /// 创建 GitHub API 请求的 headers
    fn create_headers() -> Result<HeaderMap> {
        let settings = Settings::load();
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
