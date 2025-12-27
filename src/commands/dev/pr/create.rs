//! PR 创建命令
//!
//! 提供创建版本更新 PR 的功能。

use crate::git::{GitBranch, GitCommand};
use crate::pr::github::platform::GitHub;
use crate::pr::platform::PlatformProvider;
use crate::{log_break, log_info, log_success, log_warning};
use color_eyre::{eyre::WrapErr, Result};
use reqwest::header::HeaderMap;

/// PR 创建命令
pub struct PrCreateCommand {
    version: String,
    branch_name: Option<String>,
    base_branch: Option<String>,
    ci: bool,
}

impl PrCreateCommand {
    /// 创建新的 PR 创建命令
    pub fn new(
        version: String,
        branch_name: Option<String>,
        base_branch: Option<String>,
        ci: bool,
    ) -> Self {
        Self {
            version,
            branch_name,
            base_branch,
            ci,
        }
    }

    /// 创建版本更新 PR
    pub fn create(&self) -> Result<()> {
        log_break!('=');
        log_info!("创建版本更新 PR");
        log_break!('=');
        log_break!();

        let default_branch = format!("bump-version-{}", self.version);
        let branch_name = self.branch_name.as_deref().unwrap_or(&default_branch);

        log_info!("版本: {}", self.version);
        log_info!("分支: {}", branch_name);
        log_break!();

        // 第一步：创建新分支
        log_info!("创建分支 {}...", branch_name);
        GitBranch::checkout_branch(branch_name)?;
        log_success!("分支创建/切换成功");

        // 第二步：添加文件
        log_break!();
        log_info!("暂存 Cargo.toml 和 Cargo.lock...");
        GitCommand::new(["add", "Cargo.toml", "Cargo.lock"]).run()?;
        log_success!("文件已暂存");

        // 第三步：提交更改
        log_break!();
        log_info!("提交更改...");
        let commit_message = format!("chore: bump version to {}", self.version);
        GitCommand::new(["commit", "-m", &commit_message]).run()?;
        log_success!("提交成功");

        // 第四步：推送到新分支
        log_break!();
        log_info!("推送分支到远程...");
        GitBranch::push(branch_name, false)?;
        log_success!("分支推送成功");

        // 第五步：创建 Pull Request
        log_break!();
        log_info!("创建 Pull Request...");
        let base_branch = self.base_branch.as_deref().unwrap_or("master");
        let title = format!("chore: bump version to {}", self.version);
        let body = format!(
            "Automated version bump to {}\n\nThis PR was created automatically by the release workflow.\n\n**Changes:**\n- Updated version in Cargo.toml to {}\n- Updated version in Cargo.lock to {}",
            self.version, self.version, self.version
        );

        let github = GitHub;
        match <GitHub as PlatformProvider>::create_pull_request(
            &github,
            &title,
            &body,
            branch_name,
            Some(base_branch),
        ) {
            Ok(pr_url) => {
                log_success!("PR 创建成功");
                log_info!("   URL: {}", pr_url);

                // 提取 PR 编号
                if let Some(pr_number) = pr_url.split('/').next_back() {
                    log_info!("   PR 编号: {}", pr_number);
                    if self.ci {
                        self.output_ci_result(Some(pr_number), Some(&pr_url))?;
                    }
                }
            }
            Err(e) => {
                // 如果是 422 错误（已存在 PR），尝试查找现有的 PR
                if e.to_string().contains("422") {
                    log_warning!("PR 可能已存在，尝试查找现有 PR...");
                    if let Ok(existing_pr) = self.find_existing_pr(branch_name, base_branch) {
                        log_success!("找到现有 PR");
                        log_info!("   URL: {}", existing_pr.0);
                        log_info!("   PR 编号: {}", existing_pr.1);
                        if self.ci {
                            self.output_ci_result(Some(&existing_pr.1), Some(&existing_pr.0))?;
                        }
                        return Ok(());
                    }
                }
                return Err(e);
            }
        }

        log_break!();
        log_success!("PR 创建流程完成");
        log_break!();

        Ok(())
    }

    /// 查找现有的 PR
    fn find_existing_pr(&self, branch_name: &str, base_branch: &str) -> Result<(String, String)> {
        use crate::base::http::{HttpClient, RequestConfig};
        use crate::pr::github::errors::handle_github_error;
        use crate::pr::github::platform::GitHub;
        use crate::pr::github::responses::PullRequestInfo;
        use serde_json::Value;

        let (owner, repo_name) = GitHub::get_owner_and_repo()?;
        let url = format!(
            "{}/repos/{}/{}/pulls?head={}:{}&base={}&state=open",
            crate::git::github::API_BASE,
            owner,
            repo_name,
            owner,
            branch_name,
            base_branch
        );

        let client = HttpClient::global()?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("WORKFLOW_PAT"))?
            )
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

        let config = RequestConfig::<Value, Value>::new().headers(&headers);
        let response = client.get(&url, config)?;
        let prs: Vec<PullRequestInfo> =
            response.ensure_success_with(handle_github_error)?.as_json()?;

        if let Some(pr) = prs.first() {
            Ok((pr.html_url.clone(), pr.number.to_string()))
        } else {
            Err(color_eyre::eyre::eyre!("No existing PR found"))
        }
    }

    /// 输出 CI 模式结果到 GITHUB_OUTPUT
    fn output_ci_result(&self, pr_number: Option<&str>, pr_url: Option<&str>) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        if let Ok(output_file) = std::env::var("GITHUB_OUTPUT") {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&output_file)
                .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

            if let Some(number) = pr_number {
                writeln!(file, "pr_number={}", number).wrap_err("Failed to write pr_number")?;
            }
            if let Some(url) = pr_url {
                writeln!(file, "pr_url={}", url).wrap_err("Failed to write pr_url")?;
            }
        }

        Ok(())
    }
}
