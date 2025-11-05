use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::provider::Platform;
use crate::settings::Settings;

/// Codeup API 模块
pub struct Codeup;

#[derive(Debug, Serialize)]
struct CreatePRRequest {
    source_project_id: u64,
    target_project_id: u64,
    source_branch: String,
    target_branch: String,
    title: String,
    description: String,
    tb_user_ids: Vec<u64>,
    reviewer_user_ids: Vec<u64>,
    create_from: String,
}

#[derive(Debug, Deserialize)]
struct CreatePRResponse {
    detail_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PRInfo {
    #[allow(dead_code)]
    id: Option<u64>,
    title: Option<String>,
    description: Option<String>,
    source_branch: Option<String>,
    target_branch: Option<String>,
    state: Option<String>,
    detail_url: Option<String>,
    #[serde(rename = "iid")]
    pr_number: Option<u64>,
}

#[derive(Debug, Serialize)]
struct MergePRRequest {
    merge_method: String,
    delete_source_branch: bool,
}

impl Platform for Codeup {
    /// 创建 Pull Request（通过 HTTP API）
    fn create_pr(
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String> {
        let settings = Settings::load();
        let project_id = settings
            .codeup_project_id
            .context("CODEUP_PROJECT_ID environment variable not set")?;

        let csrf_token = settings
            .codeup_csrf_token
            .as_ref()
            .context("CODEUP_CSRF_TOKEN environment variable not set")?;

        let cookie = settings
            .codeup_cookie
            .as_ref()
            .context("CODEUP_COOKIE environment variable not set")?;

        let target_branch = target_branch.unwrap_or("develop");

        let request = CreatePRRequest {
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

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Cookie", cookie)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .context("Failed to send Codeup API request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("Codeup API request failed: {} - {}", status, error_text);
        }

        let response_data: CreatePRResponse = response
            .json()
            .context("Failed to parse Codeup API response")?;

        let pr_url = response_data
            .detail_url
            .context("Failed to get PR URL from Codeup API response")?;

        Ok(pr_url)
    }

    /// 合并 PR（Codeup 暂不支持 gh CLI）
    fn merge_pr(pr_id: &str, delete_branch: bool) -> Result<()> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let settings = Settings::load();
        let csrf_token = settings
            .codeup_csrf_token
            .as_ref()
            .context("CODEUP_CSRF_TOKEN environment variable not set")?;

        // 先获取 PR 信息以确定实际的 PR ID（可能是从 URL 提取的数字）
        let actual_pr_id = if pr_id.parse::<u64>().is_ok() {
            pr_id.to_string()
        } else {
            // 可能是分支名或 URL，先查找 PR
            let pr_info = Self::get_pr_by_branch(pr_id)?;
            match pr_info {
                Some(pr) => {
                    if let Some(ref detail_url) = pr.detail_url {
                        Self::extract_pr_id_from_url(detail_url)
                            .context("Failed to extract PR ID from URL")?
                    } else if let Some(iid) = pr.pr_number {
                        iid.to_string()
                    } else {
                        anyhow::bail!("Cannot determine PR ID")
                    }
                }
                None => anyhow::bail!("PR not found: {}", pr_id),
            }
        };

        let merge_request = MergePRRequest {
            merge_method: "merge".to_string(), // Codeup 可能支持 "merge", "squash", "rebase"
            delete_source_branch: delete_branch,
        };

        let url = format!(
            "https://codeup.aliyun.com/api/v4/projects/{}/code_reviews/{}/merge?_csrf={}&_input_charset=utf-8",
            project_id, actual_pr_id, csrf_token
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .put(&url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Cookie", cookie)
            .header("Content-Type", "application/json")
            .json(&merge_request)
            .send()
            .context("Failed to send Codeup merge request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("Codeup merge request failed: {} - {}", status, error_text);
        }

        Ok(())
    }

    /// 获取 PR 信息（Codeup 暂不支持 gh CLI，需要通过 API）
    fn get_pr_info(pr_id_or_branch: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 尝试解析为数字，如果是数字则当作 PR ID，否则当作分支名
        let pr_info = if pr_id_or_branch.parse::<u64>().is_ok() {
            // 作为 PR ID，需要直接获取 PR 详情（尝试通过搜索 API 找到）
            // Codeup API 可能不支持直接通过 ID 获取，我们先通过搜索找到
            let url = format!(
                "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review%2Cmerged%2Cclosed&per_page=50",
                project_id
            );

            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&url)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Cookie", cookie)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send()
                .context("Failed to send Codeup API request")?;

            let status = response.status();
            if !status.is_success() {
                let error_text = response.text().unwrap_or_default();
                anyhow::bail!("Codeup API request failed: {} - {}", status, error_text);
            }

            let pr_list: Vec<PRInfo> = response
                .json()
                .context("Failed to parse Codeup API response")?;

            // 查找匹配的 PR ID
            pr_list.into_iter().find(|pr| {
                if let Some(iid) = pr.pr_number {
                    iid.to_string() == pr_id_or_branch
                } else if let Some(ref detail_url) = pr.detail_url {
                    Self::extract_pr_id_from_url(detail_url)
                        .map(|id| id == pr_id_or_branch)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        } else {
            // 作为分支名查找
            Self::get_pr_by_branch(pr_id_or_branch)?
        };

        match pr_info {
            Some(pr) => {
                let mut info = String::new();
                info.push_str(&format!(
                    "Title: {}\n",
                    pr.title.as_deref().unwrap_or("N/A")
                ));
                if let Some(desc) = pr.description {
                    info.push_str(&format!("Description: {}\n", desc));
                }
                info.push_str(&format!(
                    "Source Branch: {}\n",
                    pr.source_branch.as_deref().unwrap_or("N/A")
                ));
                info.push_str(&format!(
                    "Target Branch: {}\n",
                    pr.target_branch.as_deref().unwrap_or("N/A")
                ));
                info.push_str(&format!(
                    "State: {}\n",
                    pr.state.as_deref().unwrap_or("N/A")
                ));
                if let Some(url) = pr.detail_url {
                    info.push_str(&format!("URL: {}\n", url));
                }
                Ok(info)
            }
            None => {
                anyhow::bail!("PR not found: {}", pr_id_or_branch)
            }
        }
    }

    /// 获取 PR URL
    #[allow(dead_code)]
    fn get_pr_url(pr_id: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        // 尝试解析为数字
        let pr_info = if pr_id.parse::<u64>().is_ok() {
            let url = format!(
                "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review%2Cmerged%2Cclosed&per_page=50",
                project_id
            );

            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&url)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Cookie", cookie)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send()
                .context("Failed to send Codeup API request")?;

            let status = response.status();
            if !status.is_success() {
                anyhow::bail!("Codeup API request failed: {}", status);
            }

            let pr_list: Vec<PRInfo> = response
                .json()
                .context("Failed to parse Codeup API response")?;

            pr_list.into_iter().find(|pr| {
                if let Some(iid) = pr.pr_number {
                    iid.to_string() == pr_id
                } else if let Some(ref detail_url) = pr.detail_url {
                    Self::extract_pr_id_from_url(detail_url)
                        .map(|id| id == pr_id)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        } else {
            Self::get_pr_by_branch(pr_id)?
        };

        match pr_info {
            Some(pr) => pr.detail_url.context("PR URL not found in response"),
            None => {
                anyhow::bail!("PR not found: {}", pr_id)
            }
        }
    }

    /// 获取 PR 标题
    fn get_pr_title(pr_id: &str) -> Result<String> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let pr_info = if pr_id.parse::<u64>().is_ok() {
            let url = format!(
                "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review%2Cmerged%2Cclosed&per_page=50",
                project_id
            );

            let client = reqwest::blocking::Client::new();
            let response = client
                .get(&url)
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Cookie", cookie)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .send()
                .context("Failed to send Codeup API request")?;

            let status = response.status();
            if !status.is_success() {
                anyhow::bail!("Codeup API request failed: {}", status);
            }

            let pr_list: Vec<PRInfo> = response
                .json()
                .context("Failed to parse Codeup API response")?;

            pr_list.into_iter().find(|pr| {
                if let Some(iid) = pr.pr_number {
                    iid.to_string() == pr_id
                } else if let Some(ref detail_url) = pr.detail_url {
                    Self::extract_pr_id_from_url(detail_url)
                        .map(|id| id == pr_id)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        } else {
            Self::get_pr_by_branch(pr_id)?
        };

        match pr_info {
            Some(pr) => pr.title.context("PR title not found in response"),
            None => {
                anyhow::bail!("PR not found: {}", pr_id)
            }
        }
    }

    /// 获取当前分支的 PR ID
    fn get_current_branch_pr() -> Result<Option<String>> {
        use crate::git::Git;
        let current_branch = Git::current_branch()?;

        match Self::get_pr_by_branch(&current_branch)? {
            Some(pr) => {
                // 从 detail_url 提取 PR ID，或使用 iid
                if let Some(ref detail_url) = pr.detail_url {
                    if let Some(id) = Self::extract_pr_id_from_url(detail_url) {
                        return Ok(Some(id));
                    }
                }
                if let Some(iid) = pr.pr_number {
                    return Ok(Some(iid.to_string()));
                }
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// 列出 PR
    fn list_prs(state: Option<&str>, limit: Option<u32>) -> Result<String> {
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

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Cookie", cookie)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .context("Failed to send Codeup API request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("Codeup API request failed: {} - {}", status, error_text);
        }

        let pr_list: Vec<PRInfo> = response
            .json()
            .context("Failed to parse Codeup API response")?;

        // 格式化输出
        let mut output = String::new();
        for pr in pr_list {
            let pr_id = if let Some(iid) = pr.pr_number {
                iid.to_string()
            } else if let Some(ref detail_url) = pr.detail_url {
                Self::extract_pr_id_from_url(detail_url).unwrap_or_else(|| "N/A".to_string())
            } else {
                "N/A".to_string()
            };

            let title = pr.title.as_deref().unwrap_or("N/A");
            let state_str = pr.state.as_deref().unwrap_or("N/A");
            let source_branch = pr.source_branch.as_deref().unwrap_or("N/A");
            let url_str = pr.detail_url.as_deref().unwrap_or("N/A");

            output.push_str(&format!(
                "#{}  {}  [{}]  {}\n    {}\n",
                pr_id, state_str, source_branch, title, url_str
            ));
        }

        if output.is_empty() {
            output.push_str("No PRs found.");
        }

        Ok(output)
    }
}

impl Codeup {
    /// 获取环境变量（辅助函数）
    fn get_env_vars() -> Result<(u64, String)> {
        let settings = Settings::load();
        let project_id = settings
            .codeup_project_id
            .context("CODEUP_PROJECT_ID environment variable not set")?;

        let cookie = settings
            .codeup_cookie
            .as_ref()
            .context("CODEUP_COOKIE environment variable not set")?
            .clone();

        Ok((project_id, cookie))
    }

    /// 通过分支名查找 PR（内部方法）
    pub(crate) fn get_pr_by_branch(branch_name: &str) -> Result<Option<PRInfo>> {
        let (project_id, cookie) = Self::get_env_vars()?;

        let url = format!(
            "https://codeup.aliyun.com/api/v4/projects/code_reviews/advanced_search_cr?_input_charset=utf-8&page=1&search=&order_by=updated_at&state=opened&project_ids={}&sub_state_list=wip%2Cunder_review&per_page=50",
            project_id
        );

        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Cookie", cookie)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .context("Failed to send Codeup API request")?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().unwrap_or_default();
            anyhow::bail!("Codeup API request failed: {} - {}", status, error_text);
        }

        let pr_list: Vec<PRInfo> = response
            .json()
            .context("Failed to parse Codeup API response")?;

        // 通过分支名查找 PR
        for pr in pr_list {
            if let Some(ref source_branch) = pr.source_branch {
                if source_branch == branch_name {
                    return Ok(Some(pr));
                }
            }
        }

        Ok(None)
    }

    /// 从 PR URL 提取 PR ID（内部方法）
    fn extract_pr_id_from_url(url: &str) -> Option<String> {
        use regex::Regex;
        // Codeup PR URL 格式: https://codeup.aliyun.com/xxx/project/xxx/code_reviews/12345
        let re = Regex::new(r"/code_reviews/(\d+)").ok()?;
        re.captures(url)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}
