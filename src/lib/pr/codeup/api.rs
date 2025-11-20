use anyhow::{Context, Result};
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::fmt::Write;
use std::sync::OnceLock;

use crate::base::settings::Settings;
use crate::git::GitBranch;

use super::requests::{ClosePullRequestRequest, CreatePullRequestRequest, MergePullRequestRequest};
use super::responses::{CodeupUser, CreatePullRequestResponse, PullRequestInfo};
use crate::pr::http_client::PRHttpClient;
use crate::pr::provider::{PlatformProvider, PullRequestStatus};
use regex::Regex;

/// Codeup API 模块
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
            "https://codeup.aliyun.com/api/v4/projects/{}/code_reviews?_csrf={}&_input_charset=utf-8",
            project_id, csrf_token
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers(cookie, Some("application/json"))?;

        // 使用 PRHttpClient 发送请求，自动处理错误和响应解析
        let response_data: CreatePullRequestResponse =
            http_client.post(&url, &request, &headers)?;
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
            "https://codeup.aliyun.com/api/v4/projects/{}/code_reviews/{}/merge?_csrf={}&_input_charset=utf-8",
            project_id, actual_pull_request_id, csrf_token
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;

        // 使用 PRHttpClient 发送 PUT 请求，自动处理错误
        // Codeup API 返回合并结果，但我们不需要使用响应
        let _: serde_json::Value = http_client.put(&url, &merge_request, &headers)?;

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
            "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&project_ids={}&sub_state_list={}&per_page={}",
            project_id, sub_state_list, per_page
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::create_headers(&cookie, Some("application/x-www-form-urlencoded"))?;

        let pull_request_list: Vec<PullRequestInfo> = http_client.get(&url, &headers)?;

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
            "https://codeup.aliyun.com/api/v4/projects/{}/code_reviews/{}/close?_csrf={}&_input_charset=utf-8",
            project_id, actual_pull_request_id, csrf_token
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers(&cookie, Some("application/json"))?;

        // 使用 PRHttpClient 发送 PUT 请求，自动处理错误
        // Codeup API 返回关闭结果，但我们不需要使用响应
        let _: serde_json::Value = http_client.put(&url, &request, &headers)?;

        Ok(())
    }
}

impl Codeup {
    /// 获取 PR HTTP 客户端实例
    fn get_http_client() -> Result<&'static PRHttpClient> {
        static CLIENT: OnceLock<Result<PRHttpClient>> = OnceLock::new();
        CLIENT
            .get_or_init(PRHttpClient::new)
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Failed to create PR HTTP client: {}", e))
    }

    /// 获取 headers（每次调用都创建新的，因为 cookie 可能不同）
    fn get_headers(cookie: &str, content_type: Option<&str>) -> Result<HeaderMap> {
        Self::create_headers(cookie, content_type)
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

    /// 创建 Codeup API 请求的 headers（内部方法）
    fn create_headers(cookie: &str, content_type: Option<&str>) -> Result<HeaderMap> {
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

    /// 通过分支名查找 PR（内部方法）
    pub(crate) fn get_pull_request_by_branch(branch_name: &str) -> Result<Option<PullRequestInfo>> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let url = format!(
            "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review&per_page=50",
            project_id
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::create_headers(&cookie, Some("application/x-www-form-urlencoded"))?;

        // 通过分支名查找 PR
        let prs: Vec<PullRequestInfo> = http_client.get(&url, &headers)?;
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
            "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&project_ids={}&sub_state_list=wip%2Cunder_review%2Cmerged%2Cclosed&per_page=100",
            project_id
        );

        let http_client = Self::get_http_client()?;
        let headers = Self::create_headers(cookie, Some("application/x-www-form-urlencoded"))?;

        // 查找匹配的 PR ID
        let prs: Vec<PullRequestInfo> = http_client.get(&url, &headers)?;
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
        let url = "https://codeup.aliyun.com/api/v4/user?_input_charset=utf-8";

        let http_client = Self::get_http_client()?;
        let headers = Self::get_headers(&cookie, Some("application/x-www-form-urlencoded"))?;

        // 使用 PRHttpClient 发送请求，自动处理错误和响应解析
        // 注意：Codeup API 返回的是 Value，需要手动解析
        let data: Value = http_client.get(url, &headers)?;
        let user: CodeupUser =
            serde_json::from_value(data).context("Failed to parse Codeup user response")?;

        Ok(user)
    }
}
