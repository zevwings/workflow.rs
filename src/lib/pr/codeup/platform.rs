use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::fmt::Write;

use crate::base::settings::Settings;
use crate::git::GitBranch;

use super::requests::{ClosePullRequestRequest, CreatePullRequestRequest, MergePullRequestRequest};
use super::responses::{CodeupUser, CreatePullRequestResponse, PullRequestInfo};
use crate::base::http::{HttpClient, RequestConfig};
use crate::pr::codeup::errors::handle_codeup_error;
use crate::pr::platform::{PlatformProvider, PullRequestStatus};
use regex::Regex;

/// Codeup 平台实现
///
/// 实现 `PlatformProvider` trait，提供 Codeup 平台的 PR 操作功能
pub struct Codeup;

impl PlatformProvider for Codeup {
    /// 创建 Pull Request（通过 HTTP API）
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        let settings = Settings::get();
        let project_id = settings.codeup.project_id.context(
            "Codeup project ID is not configured. Please run 'workflow setup' to configure it",
        )?;

        let csrf_token = settings.codeup.csrf_token.as_ref().context(
            "Codeup CSRF token is not configured. Please run 'workflow setup' to configure it",
        )?;

        let cookie = settings.codeup.cookie.as_ref().context(
            "Codeup cookie is not configured. Please run 'workflow setup' to configure it",
        )?;

        let target_branch = target_branch.unwrap_or("develop");

        let request = CreatePullRequestRequest {
            source_project_id: project_id,
            target_project_id: project_id,
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            title: title.to_string(),
            description: body.to_string(),
            tb_user_ids: vec![],
            reviewer_user_ids: vec![],
            create_from: "WEB".to_string(),
        };

        let url = format!(
            "{}/projects/{}/code_reviews?_csrf={}&_input_charset=utf-8",
            Self::base_url(),
            project_id,
            csrf_token
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(cookie, Some("application/json"))?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.post(&url, config)?;
        let response_data: CreatePullRequestResponse = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;
        let pull_request_url = response_data
            .detail_url
            .context("Failed to get PR URL from Codeup API response")?;

        Ok(pull_request_url)
    }

    /// 合并 PR（Codeup 暂不支持 gh CLI）
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let settings = Settings::get();
        let csrf_token = settings.codeup.csrf_token.as_ref().context(
            "Codeup CSRF token is not configured. Please run 'workflow setup' to configure it",
        )?;

        // 先获取 PR 信息以确定实际的 PR ID（可能是从 URL 提取的数字）
        let actual_pull_request_id = if pull_request_id.parse::<u64>().is_ok() {
            pull_request_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pull_request_info = Self::get_pull_request_by_branch(pull_request_id)?;
            match pull_request_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pull_request_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pull_request_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pull_request_id),
            }
        };

        let merge_request = MergePullRequestRequest {
            merge_method: "merge".to_string(), // Codeup 可能支持 "merge", "squash", "rebase"
            delete_source_branch: delete_branch,
        };

        let url = format!(
            "{}/projects/{}/code_reviews/{}/merge?_csrf={}&_input_charset=utf-8",
            Self::base_url(),
            project_id,
            actual_pull_request_id,
            csrf_token
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;
        let config = RequestConfig::<_, Value>::new()
            .body(&merge_request)
            .headers(&headers);

        let response = client.put(&url, config)?;
        // Codeup API 返回合并结果，但我们不需要使用响应
        let _: serde_json::Value = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;

        Ok(())
    }

    /// 获取 PR 信息（Codeup 暂不支持 gh CLI，需要通过 API）
    fn get_pull_request_info(&self, pull_request_id_or_branch: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 尝试解析为数字，如果是数字则当作 PR ID，否则当作分支名
        let pull_request_info = if pull_request_id_or_branch.parse::<u64>().is_ok() {
            // 作为 PR ID，使用 get_pull_request_by_id（支持已合并的 PR）
            Some(Self::get_pull_request_by_id(
                pull_request_id_or_branch,
                project_id,
                &cookie,
            )?)
        } else {
            // 作为分支名查找
            Self::get_pull_request_by_branch(pull_request_id_or_branch)?
        };

        match pull_request_info {
            Some(pr) => {
                let mut info = String::new();
                writeln!(info, "Title: {}", pr.title.as_deref().unwrap_or("N/A"))?;
                if let Some(desc) = pr.description {
                    writeln!(info, "Description: {}", desc)?;
                }
                writeln!(
                    info,
                    "Source Branch: {}",
                    pr.source_branch.as_deref().unwrap_or("N/A")
                )?;
                writeln!(
                    info,
                    "Target Branch: {}",
                    pr.target_branch.as_deref().unwrap_or("N/A")
                )?;
                writeln!(info, "State: {}", pr.state.as_deref().unwrap_or("N/A"))?;
                if let Some(url) = pr.detail_url {
                    writeln!(info, "URL: {}", url)?;
                }
                Ok(info)
            }
            None => {
                anyhow::bail!("PR not found: {}", pull_request_id_or_branch)
            }
        }
    }

    /// 获取 PR URL
    #[allow(dead_code)]
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 通过 PR ID 获取 PR 信息（支持已合并的 PR）
        let pr = Self::get_pull_request_by_id(pull_request_id, project_id, &cookie)?;
        pr.detail_url.context("PR URL not found in response")
    }

    /// 获取 PR 标题
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 通过 PR ID 获取 PR 信息（支持已合并的 PR）
        let pr = Self::get_pull_request_by_id(pull_request_id, project_id, &cookie)?;
        pr.title.context("PR title not found in response")
    }

    /// 获取 PR body 内容
    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 通过 PR ID 获取 PR 信息（支持已合并的 PR）
        let pr = Self::get_pull_request_by_id(pull_request_id, project_id, &cookie)?;
        Ok(pr.description)
    }

    /// 获取当前分支的 PR ID
    fn get_current_branch_pull_request(&self) -> Result<Option<String>> {
        let current_branch = GitBranch::current_branch()?;

        match Self::get_pull_request_by_branch(&current_branch)? {
            Some(pr) => {
                // 从 detail_url 提取 PR ID，或使用 iid
                if let Some(ref detail_url) = pr.detail_url {
                    if let Some(id) = Self::extract_pull_request_id_from_url(detail_url) {
                        return Ok(Some(id));
                    }
                }
                if let Some(iid) = pr.pull_request_number {
                    return Ok(Some(iid.to_string()));
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// 获取 PR 状态
    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 通过 PR ID 获取 PR 信息（支持已合并的 PR）
        let pr_info = Self::get_pull_request_by_id(pull_request_id, project_id, &cookie)?;

        let state = pr_info.state.as_deref().unwrap_or("unknown").to_string();
        let merged = state == "merged";

        Ok(PullRequestStatus {
            state,
            merged,
            merged_at: None, // Codeup API 不返回 merged_at 字段
        })
    }

    /// 列出 PR
    fn get_pull_requests(&self, state: Option<&str>, limit: Option<u32>) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 根据 state 参数确定 sub_state_list
        let sub_state_list = match state {
            Some("open") => "wip%2Cunder_review",
            Some("closed") => "merged%2Cclosed",
            Some("merged") => "merged",
            Some("all") | None => "wip%2Cunder_review%2Cmerged%2Cclosed",
            _ => "wip%2Cunder_review%2Cmerged%2Cclosed", // 默认显示所有状态
        };

        let per_page = limit.unwrap_or(50);
        let url = format!(
            "{}/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&project_ids={}&sub_state_list={}&per_page={}",
            Self::base_url(), project_id, sub_state_list, per_page
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/x-www-form-urlencoded"))?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        let pull_request_list: Vec<PullRequestInfo> = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;

        // 格式化输出
        let mut output = String::new();
        for pr in pull_request_list {
            let pull_request_id = if let Some(iid) = pr.pull_request_number {
                iid.to_string()
            } else if let Some(ref detail_url) = pr.detail_url {
                Self::extract_pull_request_id_from_url(detail_url)
                    .unwrap_or_else(|| "N/A".to_string())
            } else {
                "N/A".to_string()
            };

            let title = pr.title.as_deref().unwrap_or("N/A");
            let state_str = pr.state.as_deref().unwrap_or("N/A");
            let source_branch = pr.source_branch.as_deref().unwrap_or("N/A");
            let url_str = pr.detail_url.as_deref().unwrap_or("N/A");

            writeln!(
                output,
                "#{}  {}  [{}]  {}\n    {}",
                pull_request_id, state_str, source_branch, title, url_str
            )?;
        }

        if output.is_empty() {
            output.push_str("No PRs found.");
        }

        Ok(output)
    }

    /// 关闭 Pull Request
    fn close_pull_request(&self, pull_request_id: &str) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;
        let settings = Settings::get();
        let csrf_token = settings.codeup.csrf_token.as_ref().context(
            "Codeup CSRF token is not configured. Please run 'workflow setup' to configure it",
        )?;

        // 先获取 PR 信息以确定实际的 PR ID
        let actual_pull_request_id = if pull_request_id.parse::<u64>().is_ok() {
            pull_request_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pull_request_info = Self::get_pull_request_by_branch(pull_request_id)?;
            match pull_request_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pull_request_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pull_request_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pull_request_id),
            }
        };

        let request = ClosePullRequestRequest {
            state: "closed".to_string(),
        };

        let url = format!(
            "{}/projects/{}/code_reviews/{}/close?_csrf={}&_input_charset=utf-8",
            Self::base_url(),
            project_id,
            actual_pull_request_id,
            csrf_token
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.put(&url, config)?;
        // Codeup API 返回关闭结果，但我们不需要使用响应
        let _: serde_json::Value = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;

        Ok(())
    }

    /// 添加评论到 Pull Request
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;
        let settings = Settings::get();
        let csrf_token = settings.codeup.csrf_token.as_ref().context(
            "Codeup CSRF token is not configured. Please run 'workflow setup' to configure it",
        )?;

        // 先获取 PR 信息以确定实际的 PR ID
        let actual_pull_request_id = if pull_request_id.parse::<u64>().is_ok() {
            pull_request_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pull_request_info = Self::get_pull_request_by_branch(pull_request_id)?;
            match pull_request_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pull_request_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pull_request_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pull_request_id),
            }
        };

        // Codeup API: POST /projects/{project_id}/code_reviews/{pr_id}/comments
        let url = format!(
            "{}/projects/{}/code_reviews/{}/comments?_csrf={}&_input_charset=utf-8",
            Self::base_url(),
            project_id,
            actual_pull_request_id,
            csrf_token
        );

        #[derive(serde::Serialize)]
        struct CommentRequest {
            content: String,
        }

        let request = CommentRequest {
            content: comment.to_string(),
        };

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.post(&url, config)?;
        let _: serde_json::Value = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;

        Ok(())
    }

    /// 批准 Pull Request
    fn approve_pull_request(&self, pull_request_id: &str) -> Result<()> {
        // Codeup 的批准操作实际上是通过添加一个 👍 评论来实现的
        // 因为 Codeup API 可能没有专门的批准接口，我们使用 add_comment 来实现
        self.add_comment(pull_request_id, "👍")
    }

    /// 更新 PR 的 base 分支
    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let settings = Settings::get();
        let csrf_token = settings.codeup.csrf_token.as_ref().context(
            "Codeup CSRF token is not configured. Please run 'workflow setup' to configure it",
        )?;

        // 先获取 PR 信息以确定实际的 PR ID（可能是从 URL 提取的数字）
        let actual_pull_request_id = if pull_request_id.parse::<u64>().is_ok() {
            pull_request_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pull_request_info = Self::get_pull_request_by_branch(pull_request_id)?;
            match pull_request_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pull_request_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pull_request_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pull_request_id),
            }
        };

        let url = format!(
            "{}/projects/{}/code_reviews/{}?_csrf={}&_input_charset=utf-8",
            Self::base_url(),
            project_id,
            actual_pull_request_id,
            csrf_token
        );

        let request = serde_json::json!({
            "target_branch": new_base
        });

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;
        let config = RequestConfig::<_, Value>::new()
            .body(&request)
            .headers(&headers);

        let response = client.put(&url, config)?;
        let _: serde_json::Value = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;

        Ok(())
    }
}

impl Codeup {
    /// 获取 Codeup API 基础 URL
    fn base_url() -> &'static str {
        "https://codeup.aliyun.com/api/v4"
    }

    /// 创建 Codeup API 请求的 headers（内部方法）
    fn get_headers(cookie: &str, content_type: Option<&str>) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Requested-With",
            "XMLHttpRequest"
                .parse()
                .context("Failed to parse X-Requested-With header")?,
        );
        headers.insert(
            "Cookie",
            cookie.parse().context("Failed to parse Cookie header")?,
        );
        if let Some(ct) = content_type {
            headers.insert(
                "Content-Type",
                ct.parse().context("Failed to parse Content-Type header")?,
            );
        }
        Ok(headers)
    }

    /// 获取 Codeup 配置（辅助函数）
    /// 从 TOML 配置文件读取 project_id 和 cookie
    fn get_env_vars() -> Result<(u64, String)> {
        let settings = Settings::get();
        let project_id = settings.codeup.project_id.context(
            "Codeup project ID is not configured. Please run 'workflow setup' to configure it",
        )?;

        let cookie = settings
            .codeup
            .cookie
            .as_ref()
            .context(
                "Codeup cookie is not configured. Please run 'workflow setup' to configure it",
            )?
            .clone();

        Ok((project_id, cookie))
    }

    /// 通过分支名查找 PR（内部方法）
    pub(crate) fn get_pull_request_by_branch(branch_name: &str) -> Result<Option<PullRequestInfo>> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let url = format!(
            "{}/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review&per_page=50",
            Self::base_url(), project_id
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/x-www-form-urlencoded"))?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        // 通过分支名查找 PR
        let prs: Vec<PullRequestInfo> = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;
        for pr in prs {
            if let Some(ref source_branch) = pr.source_branch {
                if source_branch == branch_name {
                    return Ok(Some(pr));
                }
            }
        }

        Ok(None)
    }

    /// 通过 PR ID 获取 PR 信息（内部方法，支持已合并的 PR）
    fn get_pull_request_by_id(
        pull_request_id: &str,
        project_id: u64,
        cookie: &str,
    ) -> Result<PullRequestInfo> {
        // 搜索所有状态的 PR（包括已合并的）
        let url = format!(
            "{}/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&project_ids={}&sub_state_list=wip%2Cunder_review%2Cmerged%2Cclosed&per_page=100",
            Self::base_url(), project_id
        );

        let client = HttpClient::global()?;
        let headers = Self::get_headers(cookie, Some("application/x-www-form-urlencoded"))?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        // 查找匹配的 PR ID
        let prs: Vec<PullRequestInfo> = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;
        for pr in prs {
            // 通过 iid 匹配
            if let Some(iid) = pr.pull_request_number {
                if iid.to_string() == pull_request_id {
                    return Ok(pr);
                }
            }
            // 通过 detail_url 中的 ID 匹配
            if let Some(ref detail_url) = pr.detail_url {
                if let Some(id) = Self::extract_pull_request_id_from_url(detail_url) {
                    if id == pull_request_id {
                        return Ok(pr);
                    }
                }
            }
        }

        anyhow::bail!("PR not found: {}", pull_request_id)
    }

    /// 从 PR URL 提取 PR ID（内部方法）
    fn extract_pull_request_id_from_url(url: &str) -> Option<String> {
        // Codeup PR URL 格式: https://codeup.aliyun.com/xxx/project/xxx/code_reviews/12345
        let re = Regex::new(r"/code_reviews/(\d+)").ok()?;
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// 获取 Codeup 用户信息
    ///
    /// 调用 Codeup API 获取当前用户信息。
    ///
    /// # 返回
    ///
    /// 返回 `CodeupUser` 结构体，包含用户的 `name`、`email` 等信息。
    pub fn get_user_info() -> Result<CodeupUser> {
        let (_, cookie) = Self::get_env_vars()?;

        // Codeup API 用户信息端点
        let url = format!("{}/user?_input_charset=utf-8", Self::base_url());

        let client = HttpClient::global()?;
        let headers = Self::get_headers(&cookie, Some("application/x-www-form-urlencoded"))?;
        let config = RequestConfig::<Value, Value>::new().headers(&headers);

        let response = client.get(&url, config)?;
        // 注意：Codeup API 返回的是 Value，需要手动解析
        let data: Value = response
            .ensure_success_with(handle_codeup_error)?
            .as_json()?;
        let user: CodeupUser =
            serde_json::from_value(data).context("Failed to parse Codeup user response")?;

        Ok(user)
    }
}
