use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use std::fmt::Write;
use std::sync::OnceLock;

use crate::base::settings::Settings;
use crate::git::{GitBranch, GitRepo};
use crate::jira::history::JiraWorkHistory;
use crate::log_debug;

use super::requests::{
    CreatePullRequestRequest, MergePullRequestRequest, UpdatePullRequestRequest,
};
use super::responses::{CreatePullRequestResponse, GitHubUser, PullRequestInfo, RepositoryInfo};
use crate::pr::helpers::extract_github_repo_from_url;
use crate::pr::http_client::PRHttpClient;
use crate::pr::provider::{PlatformProvider, PullRequestStatus};

/// GitHub API 模块
pub struct GitHub;

impl PlatformProvider for GitHub {
    /// 创建 Pull Request
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;

        // 如果没有指定目标分支，获取仓库的默认分支
        let base_branch = if let Some(branch) = target_branch {
            branch.to_string()
        } else {
            GitBranch::get_default_branch()?
        };

        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name);

        // 对于包含 `/` 的分支名，使用 `owner:branch_name` 格式以确保 GitHub API 正确处理
        // 即使分支在同一个仓库中，使用这种格式也更安全
        let head_branch = format!("{}:{}", owner, source_branch);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: head_branch,
            base: base_branch,
        };

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        // 使用 PRHttpClient 发送请求，自动处理错误和响应解析
        let response_data: CreatePullRequestResponse = http_client.post(&url, &request, headers)?;

        Ok(response_data.html_url)
    }

    /// 合并 Pull Request
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        // 检测仓库支持的合并方法：优先使用 squash，否则使用 merge
        let merge_method = Self::get_preferred_merge_method(&owner, &repo_name)?;
        log_debug!("Using merge method: {}", merge_method);

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}/merge",
            owner, repo_name, pr_number
        );

        let request = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method,
        };

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        // 使用 PRHttpClient 发送 PUT 请求，自动处理错误
        // GitHub API 返回合并结果，但我们不需要使用响应
        let _: serde_json::Value = http_client.put(&url, &request, headers)?;

        // 如果需要删除分支，调用删除分支 API
        if delete_branch {
            // 先获取 PR 信息以获取源分支名
            let pr_info_url = format!(
                "https://api.github.com/repos/{}/{}/pulls/{}",
                owner, repo_name, pr_number
            );
            let http_client = Self::get_http_client()?;
            let headers = Self::get_headers()?;

            // 获取 PR 信息以获取源分支名
            if let Ok(pr_info) = http_client.get::<PullRequestInfo>(&pr_info_url, headers) {
                let branch_name = pr_info.head.ref_name;
                let branch_url = format!(
                    "https://api.github.com/repos/{}/{}/git/refs/heads/{}",
                    owner, repo_name, branch_name
                );
                // 尝试删除分支，忽略 404 错误（分支可能已经被删除）
                let _ = http_client.delete(&branch_url, headers);
            }
        }

        Ok(())
    }

    /// 获取 PR 信息
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String> {
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
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.html_url)
    }

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.title)
    }

    /// 获取 PR 状态
    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(PullRequestStatus {
            state: pr.state,
            merged: pr.merged,
            merged_at: pr.merged_at,
        })
    }

    /// 列出 PR
    fn get_pull_requests(&self, state: Option<&str>, limit: Option<u32>) -> Result<String> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;

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

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        let prs: Vec<PullRequestInfo> = http_client.get(&url, headers)?;
        let mut output = String::new();
        for pr in prs {
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
    fn get_current_branch_pull_request(&self) -> Result<Option<String>> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let current_branch = GitBranch::current_branch()?;

        // 使用 head 参数查找当前分支的 PR
        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?head={}:{}&state=open",
            owner, repo_name, owner, current_branch
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        // 如果 API 查询成功，返回结果
        let prs: Vec<PullRequestInfo> = http_client.get(&url, headers)?;
        if let Some(pr) = prs.first() {
            return Ok(Some(pr.number.to_string()));
        }

        // 如果 API 查询没有找到 open 状态的 PR，尝试从 work-history 文件中查找
        // 这可以处理已关闭或已合并的 PR
        let remote_url = GitRepo::get_remote_url().ok();
        if let Some(pr_id) =
            JiraWorkHistory::find_pr_id_by_branch(&current_branch, remote_url.as_deref())?
        {
            log_debug!(
                "Found PR #{} for branch '{}' from work-history",
                pr_id,
                current_branch
            );
            return Ok(Some(pr_id));
        }

        Ok(None)
    }

    /// 关闭 Pull Request
    fn close_pull_request(&self, pull_request_id: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .context("Invalid PR number")?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let request = UpdatePullRequestRequest {
            state: "closed".to_string(),
        };

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        // 使用 PRHttpClient 发送 PATCH 请求，自动处理错误
        // GitHub API 返回更新后的 PR 对象，但我们不需要使用响应
        let _: serde_json::Value = http_client.patch(&url, &request, headers)?;

        Ok(())
    }
}

impl GitHub {
    /// 获取 PR HTTP 客户端实例
    fn get_http_client() -> Result<&'static PRHttpClient> {
        static CLIENT: OnceLock<Result<PRHttpClient>> = OnceLock::new();
        CLIENT
            .get_or_init(PRHttpClient::new)
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to create PR HTTP client: {}", e))
    }

    /// 获取缓存的 owner 和 repo_name
    pub fn get_owner_and_repo() -> Result<(String, String)> {
        static OWNER_REPO: OnceLock<Result<(String, String)>> = OnceLock::new();
        match OWNER_REPO.get_or_init(|| {
            let repo = Self::get_repo()?;
            Self::parse_repo(&repo)
        }) {
            Ok((owner, repo)) => Ok((owner.clone(), repo.clone())),
            Err(e) => Err(anyhow::anyhow!("{}", e)),
        }
    }

    /// 获取 GitHub 仓库信息（owner/repo）
    fn get_repo() -> Result<String> {
        let remote_url = GitRepo::get_remote_url().context("Failed to get remote URL")?;
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

    /// 获取仓库信息
    fn get_repository_info(owner: &str, repo_name: &str) -> Result<RepositoryInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo_name);
        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        let repo_info: RepositoryInfo = http_client.get(&url, headers)?;
        Ok(repo_info)
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
        let (owner, repo_name) = Self::get_owner_and_repo()?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo_name, pr_number
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers()?;

        let pr_info: PullRequestInfo = http_client.get(&url, headers)?;
        Ok(pr_info)
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
        let token = settings.github.get_current_token().context(
            "GitHub API token is not configured. Please run 'workflow setup' to configure it",
        )?;

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

    /// 获取 GitHub 用户信息
    ///
    /// 调用 GitHub API 的 /user 端点获取用户信息。
    ///
    /// # 参数
    ///
    /// * `token` - 可选的 GitHub API token。如果为 `None`，则使用当前激活账号的 token。
    ///
    /// # 返回
    ///
    /// 返回 `GitHubUser` 结构体，包含用户的 `login`、`name` 和 `email`。
    pub fn get_user_info(token: Option<&str>) -> Result<GitHubUser> {
        let url = "https://api.github.com/user";
        let http_client = Self::get_http_client()?;

        // 如果提供了 token，使用该 token 创建 headers；否则使用当前账号的 headers
        let headers = if let Some(token) = token {
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
            headers
        } else {
            Self::get_headers()?.clone()
        };

        // 使用 PRHttpClient 发送请求，自动处理错误和响应解析
        let user: GitHubUser = http_client.get(url, &headers)?;

        Ok(user)
    }
}
