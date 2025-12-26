//! PR 合并命令
//!
//! 提供检查 PR 状态、等待 CI 完成和合并 PR 的功能。

use color_eyre::{eyre::WrapErr, Result};
use reqwest::header::HeaderMap;
use std::time::Duration;
use crate::{log_info, log_success, log_error, log_warning, log_break};

/// PR 合并命令
pub struct PrMergeCommand {
    pr_number: u64,
    max_wait: u64,
    initial_check_interval: u64,
    normal_check_interval: u64,
    commit_title: Option<String>,
    commit_message: Option<String>,
    ci: bool,
}

impl PrMergeCommand {
    /// 创建新的 PR 合并命令
    pub fn new(
        pr_number: u64,
        max_wait: Option<u64>,
        initial_check_interval: Option<u64>,
        normal_check_interval: Option<u64>,
        commit_title: Option<String>,
        commit_message: Option<String>,
        ci: bool,
    ) -> Self {
        Self {
            pr_number,
            max_wait: max_wait.unwrap_or(300),
            initial_check_interval: initial_check_interval.unwrap_or(3),
            normal_check_interval: normal_check_interval.unwrap_or(5),
            commit_title,
            commit_message,
            ci,
        }
    }

    /// 检查并合并 PR
    pub fn merge(&self) -> Result<()> {
        log_break!('=');
        log_info!("合并 PR");
        log_break!('=');
        log_break!();

        log_info!("PR 编号: {}", self.pr_number);
        log_break!();

        // 等待 PR 状态更新
        log_info!("等待 PR 状态更新和 CI 完成...");
        std::thread::sleep(Duration::from_secs(30));
        log_success!("等待完成");
        log_break!();

        // 检查 PR 状态
        let mut elapsed = 30u64;

        while elapsed < self.max_wait {
            let check_interval = if elapsed < 60 {
                self.initial_check_interval
            } else {
                self.normal_check_interval
            };

            log_info!("检查 PR 状态 (已等待 {}s)...", elapsed);

            let pr_status = self.check_pr_status()?;

            if pr_status.merged {
                log_success!("PR 已合并");
                if self.ci {
                    self.output_ci_result(true, None)?;
                }
                return Ok(());
            }

            if let Some(true) = pr_status.mergeable {
                log_success!("PR 可以合并，开始合并...");
                log_break!();
                return self.perform_merge();
            }

            if pr_status.mergeable == Some(false) {
                log_warning!("PR 暂不可合并（可能有冲突或待检查），继续等待...");
            } else {
                log_info!("PR 合并状态计算中，继续等待...");
            }

            std::thread::sleep(Duration::from_secs(check_interval));
            elapsed += check_interval;
        }

        log_error!("超时: PR 在 {}s 内未变为可合并状态", self.max_wait);
        if self.ci {
            self.output_ci_result(false, Some("timeout"))?;
        }
        Err(color_eyre::eyre::eyre!("PR did not become mergeable within {}s", self.max_wait))
    }

    /// 检查 PR 状态
    fn check_pr_status(&self) -> Result<PrStatus> {
        use crate::pr::github::platform::GitHub;
        use crate::base::http::{HttpClient, RequestConfig};
        use crate::pr::github::responses::PullRequestInfo;
        use crate::pr::github::errors::handle_github_error;
        use serde_json::Value;

        let (owner, repo_name) = GitHub::get_owner_and_repo()?;
        let url = format!(
            "{}/repos/{}/{}/pulls/{}",
            crate::git::github::API_BASE,
            owner,
            repo_name,
            self.pr_number
        );

        let client = HttpClient::global()?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("WORKFLOW_PAT"))?)
                .parse()
                .wrap_err("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json".parse().wrap_err("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().wrap_err("Failed to parse X-GitHub-Api-Version header")?,
        );

        let config = RequestConfig::<Value, Value>::new().headers(&headers);
        let response = client.get(&url, config)?;
        let pr_info: PullRequestInfo = response
            .ensure_success_with(handle_github_error)?
            .as_json()?;

        Ok(PrStatus {
            merged: pr_info.merged.unwrap_or(false),
            mergeable: pr_info.mergeable,
        })
    }

    /// 执行合并
    fn perform_merge(&self) -> Result<()> {
        use crate::pr::github::platform::GitHub;
        use crate::pr::github::requests::MergePullRequestRequest;
        use crate::base::http::{HttpClient, RequestConfig};
        use serde_json::Value;

        let (owner, repo_name) = GitHub::get_owner_and_repo()?;
        let url = format!(
            "{}/repos/{}/{}/pulls/{}/merge",
            crate::git::github::API_BASE,
            owner,
            repo_name,
            self.pr_number
        );

        // 检测仓库支持的合并方法（使用 GitHub 的静态方法）
        let merge_method = Self::get_preferred_merge_method(&owner, &repo_name)?;
        log_info!("使用合并方法: {}", merge_method);

        let request = MergePullRequestRequest {
            commit_title: self.commit_title.clone(),
            commit_message: self.commit_message.clone(),
            merge_method: merge_method.clone(),
        };

        let client = HttpClient::global()?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("WORKFLOW_PAT"))?)
                .parse()
                .wrap_err("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json".parse().wrap_err("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().wrap_err("Failed to parse X-GitHub-Api-Version header")?,
        );

        let config = RequestConfig::<_, Value>::new().body(&request).headers(&headers);

        match client.put(&url, config) {
            Ok(response) => {
                let _: Value = response
                    .ensure_success_with(crate::pr::github::errors::handle_github_error)?
                    .as_json()?;
                log_success!("PR 合并成功");
                if self.ci {
                    self.output_ci_result(true, None)?;
                }
                Ok(())
            }
            Err(e) => {
                if e.to_string().contains("405") {
                    log_warning!("PR 暂不可合并 (HTTP 405)，但继续...");
                    if self.ci {
                        self.output_ci_result(false, Some("405"))?;
                    }
                    return Ok(());
                }
                Err(e)
            }
        }
    }

    /// 获取首选的合并方法
    fn get_preferred_merge_method(owner: &str, repo_name: &str) -> Result<String> {
        use crate::pr::github::platform::GitHub;
        use crate::base::http::{HttpClient, RequestConfig};
        use crate::pr::github::responses::RepositoryInfo;
        use crate::pr::github::errors::handle_github_error;
        use serde_json::Value;

        let url = format!(
            "{}/repos/{}/{}",
            crate::git::github::API_BASE,
            owner,
            repo_name
        );

        let client = HttpClient::global()?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("WORKFLOW_PAT"))?)
                .parse()
                .wrap_err("Failed to parse Authorization header")?,
        );
        headers.insert(
            "Accept",
            "application/vnd.github+json".parse().wrap_err("Failed to parse Accept header")?,
        );
        headers.insert(
            "X-GitHub-Api-Version",
            "2022-11-28".parse().wrap_err("Failed to parse X-GitHub-Api-Version header")?,
        );

        let config = RequestConfig::<Value, Value>::new().headers(&headers);
        let response = client.get(&url, config)?;
        let repo_info: RepositoryInfo = response
            .ensure_success_with(handle_github_error)?
            .as_json()?;

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

        Err(color_eyre::eyre::eyre!(
            "Repository does not support squash, rebase, or merge commit methods"
        ))
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, merged: bool, error: Option<&str>) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            writeln!(file, "pr_merged={}", merged)
                .wrap_err("Failed to write pr_merged")?;
            if let Some(e) = error {
                writeln!(file, "pr_merge_error={}", e)
                    .wrap_err("Failed to write pr_merge_error")?;
            }
        }

        Ok(())
    }
}

/// PR 状态
struct PrStatus {
    merged: bool,
    mergeable: Option<bool>,
}

