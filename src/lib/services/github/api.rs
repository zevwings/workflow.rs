//! GitHub API 客户端
//!
//! 提供与 GitHub API 交互的完整功能，包括 Pull Request 操作、用户信息查询等。

use std::fmt::Write;
use std::sync::OnceLock;

use color_eyre::{
    eyre::{eyre, ContextCompat, WrapErr},
    Result,
};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde_json::Value;

use crate::config::settings::Settings;
use crate::core::constants::errors;
use crate::core::http::{HttpClient, RequestConfig};
use crate::git::{self, GitBranch, GitRepo};

use super::errors::handle_github_error;
use super::types::{
    CreatePullRequestRequest, CreatePullRequestResponse, GitHubUser, MergePullRequestRequest,
    PullRequestFile, PullRequestInfo, RepositoryInfo, UpdatePullRequestRequest,
};

/// GitHub API 客户端
///
/// 提供与 GitHub API 交互的方法，包括 Pull Request 操作、用户信息查询等。
pub struct GitHub;

impl GitHub {
    /// 获取 GitHub API 基础 URL
    pub fn base_url() -> &'static str {
        git::github::API_BASE
    }

    /// 创建 GitHub API 请求的 headers
    ///
    /// # 参数
    ///
    /// * `token` - 可选的 GitHub API token。如果为 `None`，则从 settings 获取当前激活账号的 token。
    pub fn get_headers(token: Option<&str>) -> Result<HeaderMap> {
        let token = if let Some(token) = token {
            token
        } else {
            let settings = Settings::get();
            settings.github.get_current_token().wrap_err(
                "GitHub API token is not configured. Please run 'workflow setup' to configure it",
            )?
        };

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token)
                .parse()
                .wrap_err("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json"
                .parse()
                .wrap_err("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().wrap_err("Failed to parse X-GitHub-Api-Version header")?,
        );
        headers.insert(
            "User-Agent",
            "workflow-cli".parse().wrap_err("Failed to parse User-Agent header")?,
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
            Err(e) => Err(eyre!("{}", e)),
        }
    }

    /// 获取 GitHub 仓库信息（owner/repo）
    fn get_repo() -> Result<String> {
        let remote_url = GitRepo::get_remote_url().wrap_err("Failed to get remote URL")?;
        Self::extract_github_repo_from_url(&remote_url)
            .wrap_err("Failed to extract GitHub repo from remote URL")
    }

    /// 从 Git remote URL 提取 GitHub 仓库的 owner/repo
    ///
    /// 支持标准格式和 SSH host 别名格式（如 github-brainim）
    pub fn extract_github_repo_from_url(url: &str) -> Result<String> {
        use regex::Regex;

        // 匹配 ssh:// 协议格式: ssh://git@github.com/owner/repo.git
        let ssh_protocol_re = Regex::new(r"ssh://git@github\.com/(.+?)(?:\.git)?/?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = ssh_protocol_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| {
                    eyre!(
                        "Failed to extract repo name from GitHub ssh:// URL: {}",
                        url
                    )
                })?
                .as_str()
                .to_string());
        }

        // 匹配 SSH 格式: git@github.com:owner/repo.git 或 git@github-xxx:owner/repo.git (支持 SSH host 别名)
        let ssh_re =
            Regex::new(r"git@github[^:]*:(.+?)(?:\.git)?$").wrap_err("Invalid regex pattern")?;
        if let Some(caps) = ssh_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from GitHub SSH URL: {}", url))?
                .as_str()
                .to_string());
        }

        // 匹配 HTTPS 格式: https://github.com/owner/repo.git
        let https_re = Regex::new(r"https?://(?:www\.)?github\.com/(.+?)(?:\.git)?/?$")
            .wrap_err("Invalid regex pattern")?;
        if let Some(caps) = https_re.captures(url) {
            return Ok(caps
                .get(1)
                .ok_or_else(|| eyre!("Failed to extract repo name from GitHub HTTPS URL: {}", url))?
                .as_str()
                .to_string());
        }

        color_eyre::eyre::bail!("Failed to extract GitHub repo from URL: {}", url)
    }

    /// 解析仓库字符串为 owner 和 repo_name
    fn parse_repo(repo: &str) -> Result<(String, String)> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            color_eyre::eyre::bail!(
                "{}: {}",
                errors::validation_error::VALIDATION_INVALID_REPO_FORMAT,
                repo
            );
        }
        Ok((parts[0].to_string(), parts[1].to_string()))
    }

    /// 创建 Pull Request
    ///
    /// # 参数
    ///
    /// * `title` - PR 标题
    /// * `body` - PR 描述
    /// * `source_branch` - 源分支名
    /// * `target_branch` - 目标分支名（可选，默认使用仓库的默认分支）
    ///
    /// # 返回
    ///
    /// PR URL 字符串
    pub fn create_pull_request(
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
        let head_branch = format!("{}:{}", owner, source_branch);

        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: head_branch,
            base: base_branch,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;
        let response_data: CreatePullRequestResponse =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(response_data.html_url)
    }

    /// 合并 Pull Request
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    /// * `delete_branch` - 是否删除源分支
    pub fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        // 检测仓库支持的合并方法：优先使用 squash，否则使用 merge
        let merge_method = Self::get_preferred_merge_method(&owner, &repo_name)?;
        crate::log_debug!("Using merge method: {}", merge_method);

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
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.put(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        // 如果需要删除分支，调用删除分支 API
        if delete_branch {
            let pr_info = Self::fetch_pr_info(pr_number)?;
            let branch_name = pr_info.head.ref_name;
            let branch_url = format!(
                "{}/repos/{}/{}/git/refs/heads/{}",
                Self::base_url(),
                owner,
                repo_name,
                branch_name
            );
            // 尝试删除分支，忽略 404 错误（分支可能已经被删除）
            let delete_config = RequestConfig::new().headers(&headers);
            let _ = client.delete(&branch_url, delete_config);
        }

        Ok(())
    }

    /// 获取 PR 信息
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR 信息的格式化字符串
    pub fn get_pull_request_info(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;
        let pr = Self::fetch_pr_info(pr_number)?;

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
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR URL 字符串
    pub fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;
        let pr = Self::fetch_pr_info(pr_number)?;
        Ok(pr.html_url)
    }

    /// 获取 PR 标题
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR 标题字符串
    pub fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;
        let pr = Self::fetch_pr_info(pr_number)?;
        Ok(pr.title)
    }

    /// 获取 PR body 内容
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR body 字符串（如果存在），否则返回 None
    pub fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;
        let pr = Self::fetch_pr_info(pr_number)?;
        Ok(pr.body)
    }

    /// 获取 PR 状态
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR 状态信息（state, merged, merged_at）
    pub fn get_pull_request_status(
        &self,
        pull_request_id: &str,
    ) -> Result<(String, bool, Option<String>)> {
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;
        let pr = Self::fetch_pr_info(pr_number)?;
        Ok((pr.state, pr.merged, pr.merged_at))
    }

    /// 获取 PR 列表
    ///
    /// # 参数
    ///
    /// * `state` - PR 状态筛选（如 "open", "closed"）
    /// * `limit` - 返回数量限制
    ///
    /// # 返回
    ///
    /// PR 信息列表
    pub fn get_pull_requests(
        &self,
        state: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PullRequestInfo>> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;

        // 转换 state 参数：GitHub API 支持 "open", "closed", "all"
        let state = match state {
            Some("open") => "open",
            Some("closed") => "closed",
            Some("merged") => "closed", // GitHub API 中 merged 是 closed 状态的一种
            Some("all") | None => "all",
            _ => "all",
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
        let config = RequestConfig::new().headers(&headers);

        let response = client.get(&url, config)?;
        let prs: Vec<PullRequestInfo> =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(prs)
    }

    /// 获取当前分支的 PR ID
    ///
    /// # 返回
    ///
    /// PR ID（如果存在），否则返回 None
    pub fn get_current_branch_pull_request(&self) -> Result<Option<String>> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let current_branch = GitBranch::current_branch()?;

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;

        // 首先尝试查找 open 状态的 PR
        let url = format!(
            "{}/repos/{}/{}/pulls?head={}:{}&state=open",
            Self::base_url(),
            owner,
            repo_name,
            owner,
            current_branch
        );

        let config = RequestConfig::new().headers(&headers);
        let response = client.get(&url, config)?;
        let prs: Vec<PullRequestInfo> =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        if let Some(pr) = prs.first() {
            return Ok(Some(pr.number.to_string()));
        }

        // 如果找不到 open 状态的 PR，尝试查找所有状态的 PR
        let url_all = format!(
            "{}/repos/{}/{}/pulls?head={}:{}&state=all",
            Self::base_url(),
            owner,
            repo_name,
            owner,
            current_branch
        );

        let config_all = RequestConfig::new().headers(&headers);
        let response_all = client.get(&url_all, config_all)?;
        let prs_all: Vec<PullRequestInfo> =
            response_all.ensure_success_with(handle_github_error)?.as_json()?;
        if let Some(pr) = prs_all.first() {
            crate::log_debug!(
                "Found PR #{} for branch '{}' (state: {})",
                pr.number,
                current_branch,
                pr.state
            );
            return Ok(Some(pr.number.to_string()));
        }

        Ok(None)
    }

    /// 获取 PR 的 diff 内容
    ///
    /// 如果 PR diff 超过 GitHub API 的限制（20000 行），会返回 406 错误。
    /// 此时会使用替代方案：通过 files API 获取文件列表，然后获取部分文件的 diff。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    ///
    /// # 返回
    ///
    /// PR 的 diff 内容（字符串格式）
    pub fn get_pull_request_diff(&self, pull_request_id: &str) -> Result<String> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}.diff",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let client = HttpClient::global()?;
        let mut headers = Self::get_headers(None)?;
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github.v3.diff"),
        );

        let config = RequestConfig::new().headers(&headers);
        let response = client.get(&url, config)?;

        // 检查是否是 406 错误（diff too large）
        if response.status == 406 {
            let is_too_large = if let Ok(data) = response.as_json::<Value>() {
                if let Some(errors) = data.get("errors").and_then(|v| v.as_array()) {
                    errors.iter().any(|err| {
                        err.get("code")
                            .and_then(|c| c.as_str())
                            .map(|c| c == "too_large")
                            .unwrap_or(false)
                    })
                } else {
                    false
                }
            } else {
                false
            };

            if is_too_large {
                crate::log_debug!(
                    "PR diff exceeds GitHub API limit (20000 lines), using fallback method"
                );
                return Self::get_pull_request_diff_fallback(owner, repo_name, pr_number);
            }
        }

        // 正常情况：返回完整的 diff
        let diff = response.ensure_success_with(handle_github_error)?.as_text()?;
        Ok(diff)
    }

    /// 关闭 Pull Request
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    pub fn close_pull_request(&self, pull_request_id: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = UpdatePullRequestRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
            base: None,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.patch(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 添加评论到 Pull Request
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    /// * `comment` - 评论内容
    pub fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<()> {
        #[derive(serde::Serialize)]
        struct CommentRequest {
            body: String,
        }

        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        // GitHub API: POST /repos/{owner}/{repo}/issues/{issue_number}/comments
        let url = format!(
            "{}/repos/{}/{}/issues/{}/comments",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = CommentRequest {
            body: comment.to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 批准 Pull Request
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    pub fn approve_pull_request(&self, pull_request_id: &str) -> Result<()> {
        use crate::core::constants::messages;

        #[derive(serde::Serialize)]
        struct ReviewRequest {
            event: String,
            body: String,
        }

        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        // 先获取 PR 信息以检查是否是自己的 PR
        let pr_info = Self::fetch_pr_info(pr_number)?;
        let current_user = Self::get_user_info(None)?;

        // 检查是否是自己的 PR
        if let Some(ref pr_user) = pr_info.user {
            if pr_user.login == current_user.login {
                color_eyre::eyre::bail!(
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

        let request = ReviewRequest {
            event: messages::PR_APPROVE_EVENT.to_string(),
            body: messages::PR_APPROVE_EMOJI.to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.post(&url, config)?;

        // 处理可能的错误
        match response.ensure_success_with(handle_github_error) {
            Ok(_) => Ok(()),
            Err(e) => {
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("can not approve your own pull request")
                    || error_msg.contains("cannot approve your own")
                {
                    color_eyre::eyre::bail!(
                        "Cannot approve your own pull request. GitHub does not allow users to approve their own PRs."
                    );
                }
                Err(e)
            }
        }
    }

    /// 更新 PR 的 base 分支
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    /// * `new_base` - 新的 base 分支名称
    pub fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

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
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.patch(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 更新 Pull Request 的标题和/或描述
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID
    /// * `title` - 新的标题（可选）
    /// * `body` - 新的描述（可选）
    pub fn update_pull_request(
        &self,
        pull_request_id: &str,
        title: Option<&str>,
        body: Option<&str>,
    ) -> Result<()> {
        let (owner, repo_name) = Self::get_owner_and_repo()?;
        let pr_number = pull_request_id
            .parse::<u64>()
            .wrap_err(errors::validation_error::VALIDATION_INVALID_PR_NUMBER)?;

        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let request = UpdatePullRequestRequest {
            title: title.map(|s| s.to_string()),
            body: body.map(|s| s.to_string()),
            state: None,
            base: None,
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().body(&request).headers(&headers);

        let response = client.patch(&url, config)?;
        let _: serde_json::Value = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(())
    }

    /// 获取仓库信息
    fn get_repository_info(owner: &str, repo_name: &str) -> Result<RepositoryInfo> {
        let url = format!("{}/repos/{}/{}", Self::base_url(), owner, repo_name);
        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().headers(&headers);

        let response = client.get(&url, config)?;
        let repo_info: RepositoryInfo =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        Ok(repo_info)
    }

    /// 获取首选的合并方法：优先使用 squash，其次 rebase，最后 merge
    fn get_preferred_merge_method(owner: &str, repo_name: &str) -> Result<String> {
        let repo_info = Self::get_repository_info(owner, repo_name)?;

        // 优先级：squash > rebase > merge
        if repo_info.allow_squash_merge.unwrap_or(false) {
            return Ok("squash".to_string());
        }

        if repo_info.allow_rebase_merge.unwrap_or(false) {
            return Ok("rebase".to_string());
        }

        if repo_info.allow_merge_commit.unwrap_or(false) {
            return Ok("merge".to_string());
        }

        color_eyre::eyre::bail!(
            "Repository does not support squash, rebase, or merge commit methods"
        );
    }

    /// 获取 PR 信息（内部方法）
    pub fn fetch_pr_info(pr_number: u64) -> Result<PullRequestInfo> {
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
        let config = RequestConfig::new().headers(&headers);

        let response = client.get(&url, config)?;
        let pr_info: PullRequestInfo =
            response.ensure_success_with(handle_github_error)?.as_json()?;
        Ok(pr_info)
    }

    /// 获取 GitHub 用户信息
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

        let headers = if let Some(token) = token {
            Self::get_headers(Some(token))?
        } else {
            Self::get_headers(None)?
        };

        let config = RequestConfig::new().headers(&headers);
        let response = client.get(&url, config)?;
        let user: GitHubUser = response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(user)
    }

    /// 获取 PR diff 的替代方案（当 diff 超过 20000 行时）
    fn get_pull_request_diff_fallback(
        owner: String,
        repo_name: String,
        pr_number: u64,
    ) -> Result<String> {
        const MAX_FILES: usize = 50;
        const MAX_LINES: usize = 15000;

        let files = Self::get_pull_request_files(&owner, &repo_name, pr_number)?;

        if files.is_empty() {
            color_eyre::eyre::bail!("No files found in PR");
        }

        let files_to_process = if files.len() > MAX_FILES {
            crate::log_debug!(
                "PR has {} files, limiting to first {} files",
                files.len(),
                MAX_FILES
            );
            &files[..MAX_FILES]
        } else {
            &files
        };

        let mut diff_parts = Vec::new();
        let mut total_lines = 0;

        for file in files_to_process {
            if let Some(ref patch) = file.patch {
                let patch_lines: Vec<&str> = patch.lines().collect();
                if total_lines + patch_lines.len() > MAX_LINES {
                    let remaining_lines = MAX_LINES.saturating_sub(total_lines);
                    if remaining_lines > 0 {
                        let partial_patch =
                            patch_lines[..remaining_lines.min(patch_lines.len())].join("\n");
                        diff_parts.push(format!(
                            "diff --git a/{} b/{}\n{}",
                            file.filename, file.filename, partial_patch
                        ));
                    }
                    diff_parts.push(format!(
                        "\n... (diff truncated: {} files processed, {} total files in PR)",
                        files_to_process.len(),
                        files.len()
                    ));
                    break;
                }

                diff_parts.push(format!(
                    "diff --git a/{} b/{}\n{}",
                    file.filename, file.filename, patch
                ));
                total_lines += patch_lines.len();
            } else {
                let diff_header = match file.status.as_str() {
                    "added" => format!(
                        "diff --git a/{} b/{}\nnew file mode 100644\n--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n... (file too large, {} additions)",
                        file.filename, file.filename, file.filename, file.additions, file.additions
                    ),
                    "removed" => format!(
                        "diff --git a/{} b/{}\ndeleted file mode 100644\n--- a/{}\n+++ /dev/null\n@@ -1,{} +0,0 @@\n... (file too large, {} deletions)",
                        file.filename, file.filename, file.filename, file.deletions, file.deletions
                    ),
                    _ => format!(
                        "diff --git a/{} b/{}\nindex 0000000..0000000\n--- a/{}\n+++ b/{}\n@@ -1,{} +1,{} @@\n... (file too large, {} additions, {} deletions)",
                        file.filename,
                        file.filename,
                        file.filename,
                        file.filename,
                        file.deletions,
                        file.additions,
                        file.additions,
                        file.deletions
                    ),
                };
                diff_parts.push(diff_header);
            }
        }

        if files.len() > files_to_process.len() {
            diff_parts.push(format!(
                "\n... ({} more files not included due to size limit)",
                files.len() - files_to_process.len()
            ));
        }

        Ok(diff_parts.join("\n"))
    }

    /// 获取 PR 文件列表
    fn get_pull_request_files(
        owner: &str,
        repo_name: &str,
        pr_number: u64,
    ) -> Result<Vec<PullRequestFile>> {
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/files",
            Self::base_url(),
            owner,
            repo_name,
            pr_number
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(None)?;
        let config = RequestConfig::new().headers(&headers);

        let response = client.get(&url, config)?;
        let files: Vec<PullRequestFile> =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        Ok(files)
    }
}
