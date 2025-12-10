use std::fmt::Write;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde_json::Value;

use crate::base::http::{HttpClient, RequestConfig};
use crate::base::settings::Settings;
use crate::git::{GitBranch, GitRepo};
use crate::jira::history::JiraWorkHistory;
use crate::pr::github::errors::handle_github_error;
use crate::pr::helpers::extract_github_repo_from_url;
use crate::pr::platform::{PlatformProvider, PullRequestStatus};
use crate::pr::PullRequestRow;

use super::requests::{
    CreatePullRequestRequest, MergePullRequestRequest, UpdatePullRequestRequest,
};
use super::responses::{CreatePullRequestResponse, GitHubUser, PullRequestInfo, RepositoryInfo};

/// GitHub 平台实现
///
/// 实现 `PlatformProvider` trait，提供 GitHub 平台的 PR 操作功能
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

        let url = format!("{}/repos/{}/{}/pulls", Self::base_url(), owner, repo_name);

        // 对于包含 `/` 的分支名，使用 `owner:branch_name` 格式以确保 GitHub API 正确处理
        // 即使分支在同一个仓库中，使用这种格式也更安全
        let head_branch = format!("{}:{}", owner, source_branch);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: head_branch,
            base: base_branch,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;
        let response_data: CreatePullRequestResponse =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(response_data.html_url)
    }

    /// 合并 Pull Request
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        // 检测仓库支持的合并方法：优先使用 squash，否则使用 merge
        let merge_method = Self::get_preferred_merge_method(&owner, &repo_name)?;
        crate::trace_debug!("Using merge method: {}", merge_method);

        let url = format!(
            "{}/repos/{}/{}/pulls/{}/merge",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.put(&url, config)?;
        // GitHub API 返回合并结果，但我们不需要使用响应
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        // 如果需要删除分支，调用删除分支 API
        if delete_branch {
            // 先获取 PR 信息以获取源分支名
            let pr_info_url = format!(
                "{}/repos/{}/{}/pulls/{}",
                Self::base_url(),
                owner,
                repo_name,
                pr_number
            );
            let client = HttpClient::global()?;
            let headers = Self::get_headers(None)?;
            let config = RequestConfig::<Value, Value>::new().headers(&headers);

            let response = client.get(&pr_info_url, config)?;
            // 获取 PR 信息以获取源分支名
            if let Ok(pr_info) = response
                .ensure_success_with(handle_github_error)
                .and_then(|r| r.as_json::<PullRequestInfo>())
            {
                let branch_name = pr_info.head.ref_name;
                let branch_url = format!(
                    "{}/repos/{}/{}/git/refs/heads/{}",
                    Self::base_url(),
                    owner,
                    repo_name,
                    branch_name
                );
                // 尝试删除分支，忽略 404 错误（分支可能已经被删除）
                let delete_config = RequestConfig::<Value, Value>::new().headers(&headers);
                let _ = client.delete(&branch_url, delete_config);
            }
        }

        Ok(())
    }

    /// 获取 PR 信息
    fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;
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
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.html_url)
    }

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.title)
    }

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>> {
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(pr.body)
    }

    /// 获取 PR 状态
    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus> {
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;
        let pr = Self::fetch_pr_info_internal(pr_number)?;
        Ok(PullRequestStatus {
            state: pr.state,
            merged: pr.merged,
            merged_at: pr.merged_at,
        })
    }

    /// 列出 PR
    fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<PullRequestRow>> {
        let prs = Self::get_pull_requests_raw(state, limit)?;
        let rows: Vec<PullRequestRow> = prs
            .into_iter()
            .map(|pr| PullRequestRow {
                number: pr.number.to_string(),
                state: pr.state,
                branch: pr.head.ref_name,
                title: pr.title,
                author: pr
                    .user
                    .as_ref()
                    .map(|u| u.login.clone())
                    .unwrap_or_else(|| "N/A".to_string()),
                url: pr.html_url,
            })
            .collect();
        Ok(rows)
    }

    /// 获取当前分支的 PR
    fn get_current_branch_pull_request(&self) -> Result<Option<String>> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let current_branch = GitBranch::current_branch()?;

        // 使用 head 参数查找当前分支的 PR
        let url = format!(
            "{}/repos/{}/{}/pulls?head={}:{}&state=open",
            Self::base_url(),
            owner,
            repo_name,
            owner,
            current_branch
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        // 如果 API 查询成功，返回结果
        let prs: Vec<PullRequestInfo> =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        if let Some(pr) = prs.first() {
            return Ok(Some(pr.number.to_string()));
        }

        // 如果 API 查询没有找到 open 状态的 PR，尝试从 work-history 文件中查找
        // 这可以处理已关闭或已合并的 PR
        let remote_url = GitRepo::get_remote_url().ok();
        if let Some(pr_id) =
            JiraWorkHistory::find_pr_id_by_branch(&current_branch, remote_url.as_deref())?
        {
            crate::trace_debug!(
                "Found PR #{} for branch '{}' from work-history",
                pr_id,
                current_branch
            );
            return Ok(Some(pr_id));
        }

        Ok(None)
    }

    /// 获取 PR 的 diff 内容
    fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        // 使用 GitHub API 获取 PR diff
        // 格式: GET /repos/{owner}/{repo}/pulls/{pr_number}.diff
        // 注意：需要设置 Accept header 为 diff 格式，否则会返回 JSON
        let url = format!(
            "{}/repos/{}/{}/pulls/{}.diff",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let client = HttpClient::global()?;
        // 获取基础 headers（包含认证信息）
        let mut headers = Self::get_headers(None)?;

        // 覆盖 Accept header，设置为 diff 格式
        // 注意：GitHub API 的 .diff 端点需要设置正确的 Accept header 才能返回纯文本 diff
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3.diff"),
        );

        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        let diff = response.ensure_success_with(handle_github_error)?.as_text()?;

        Ok(diff)
    }

    /// 关闭 Pull Request
    fn close_pull_request(&self, pull_request_id: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = UpdatePullRequestRequest {
            state: "closed".to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.patch(&url, config)?;
        // GitHub API 返回更新后的 PR 对象，但我们不需要使用响应
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 添加评论到 Pull Request
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        // GitHub API: POST /repos/{owner}/{repo}/issues/{issue_number}/comments
        // 注意：PR 在 GitHub API 中也是 issue，所以使用 issues 端点
        let url = format!(
            "{}/repos/{}/{}/issues/{}/comments",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        #[derive(serde::Serialize)]
        struct CommentRequest {
            body: String,
        }

        let request = CommentRequest {
            body: comment.to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 批准 Pull Request
    fn approve_pull_request(&self, pull_request_id: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        // 先获取 PR 信息以检查是否是自己的 PR
        let pr_info = Self::fetch_pr_info_internal(pr_number)?;
        let current_user = Self::get_user_info(None)?;

        // 检查是否是自己的 PR
        if let Some(ref pr_user) = pr_info.user {
            if pr_user.login == current_user.login {
                anyhow::bail!(
                    "Cannot approve your own pull request. GitHub does not allow users to approve their own PRs."
                );
            }
        }

        // GitHub API: POST /repos/{owner}/{repo}/pulls/{pull_number}/reviews
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/reviews",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        #[derive(serde::Serialize)]
        struct ReviewRequest {
            event: String,
            body: String,
        }

        let request = ReviewRequest {
            event: "APPROVE".to_string(),
            body: "👍".to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;

        // 处理可能的错误（例如，如果 API 仍然返回错误，提供更友好的消息）
        match response.ensure_success_with(handle_github_error) {
            Ok(_) => Ok(()),
            Err(e) => {
                // 检查是否是"不能批准自己的 PR"的错误
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("can not approve your own pull request")
                    || error_msg.contains("cannot approve your own")
                {
                    anyhow::bail!(
                        "Cannot approve your own pull request. GitHub does not allow users to approve their own PRs."
                    );
                }
                Err(e)
            }
        }
    }

    /// 更新 PR 的 base 分支
    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id.parse::<u64>().context("Invalid PR number")?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = serde_json::json!({
            "base": new_base
        });

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        let response = client.patch(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }
}

impl GitHub {
    /// 获取 GitHub API 基础 URL
    fn base_url() -> &'static str {
        "https://api.github.com"
    }

    /// 创建 GitHub API 请求的 headers（内部方法）
    ///
    /// # 参数
    ///
    /// * `token` - 可选的 GitHub API token。如果为 `None`，则从 settings 获取当前激活账号的 token。
    fn get_headers(token: Option<&str>) -> Result<HeaderMap> {
        let token = if let Some(token) = token {
            token
        } else {
            let settings = Settings::get();
            settings.github.get_current_token().context(
                "GitHub API token is not configured. Please run 'workflow setup' to configure it",
            )?
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .context("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json".parse().context("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().context("Failed to parse X-GitHub-Api-Version header")?,
        );
        headers.insert(
            "User-Agent",
            "workflow-cli".parse().context("Failed to parse User-Agent header")?,
        );

        Ok(headers)
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
        let url = format!("{}/repos/{}/{}", Self::base_url(), owner, repo_name);
        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        let repo_info: RepositoryInfo =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        Ok(repo_info)
    }

    /// 获取 PR 列表原始数据（不格式化）
    ///
    /// # 参数
    ///
    /// * `state` - PR 状态筛选（如 "open", "closed"）
    /// * `limit` - 返回数量限制
    ///
    /// # 返回
    ///
    /// PR 信息列表
    pub fn get_pull_requests_raw(
        state: Option<&str>,
        limit: Option<u32>,
    ) -> Result<Vec<PullRequestInfo>> {
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
            "{}/repos/{}/{}/pulls?state={}&per_page={}",
            Self::base_url(),
            owner,
            repo_name,
            state,
            per_page
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        let prs: Vec<PullRequestInfo> =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(prs)
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
            "{}/repos/{}/{}/pulls/{}",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        let pr_info: PullRequestInfo =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        Ok(pr_info)
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
        let url = format!("{}/user", Self::base_url());
        let client = HttpClient::global()?;

        // 如果提供了 token，使用该 token 创建 headers；否则使用当前账号的 headers
        let headers = if let Some(token) = token {
            Self::get_headers(Some(token))?
        } else {
            Self::get_headers(None)?
        };

        let config = RequestConfig::<Value, Value>::new().headers(&headers);
        let response = client.get(&url, config)?;
        let user: GitHubUser = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(user)
    }
}
