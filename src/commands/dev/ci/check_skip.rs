//! CI 跳过检查实现

use color_eyre::{eyre::WrapErr, Result};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

use crate::git::GitBranch;
use crate::{log_error, log_info, log_success};

/// CI 跳过检查命令
pub struct CiSkipCommand {
    branch: Option<String>,
    pr_creator: Option<String>,
    expected_user: Option<String>,
    ci_mode: bool,
}

impl CiSkipCommand {
    /// 创建新的 CI 跳过检查命令
    pub fn new(
        branch: Option<String>,
        pr_creator: Option<String>,
        expected_user: Option<String>,
        ci_mode: bool,
    ) -> Self {
        Self {
            branch,
            pr_creator,
            expected_user,
            ci_mode,
        }
    }

    /// 检查是否应该跳过 CI
    pub fn check(&self) -> Result<bool> {
        // 获取分支名称
        let branch_name = if let Some(ref branch) = self.branch {
            branch.clone()
        } else {
            // 尝试从环境变量获取（GitHub Actions）
            env::var("GITHUB_HEAD_REF")
                .or_else(|_| env::var("GITHUB_REF_NAME"))
                .or_else(|_| GitBranch::current_branch())
                .wrap_err("Failed to get branch name")?
        };

        log_info!("🔍 Checking branch: {}", branch_name);
        log_info!(
            "   Event: {}",
            env::var("GITHUB_EVENT_NAME").unwrap_or_else(|_| "unknown".to_string())
        );

        // 检查是否是版本更新分支（bump-version-*）
        if branch_name.starts_with("bump-version-") {
            log_success!("Detected bump-version-* branch: {}", branch_name);

            // 对于 PR 事件，验证分支是否由 CI 创建
            let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
            if event_name == "pull_request" {
                let pr_creator = if let Some(ref creator) = self.pr_creator {
                    creator.clone()
                } else {
                    // 尝试从环境变量获取
                    env::var("GITHUB_PR_CREATOR").unwrap_or_default()
                };

                let expected_user = if let Some(ref user) = self.expected_user {
                    user.clone()
                } else {
                    env::var("WORKFLOW_USER_NAME").unwrap_or_default()
                };

                log_info!(
                    "   PR creator: {}",
                    if pr_creator.is_empty() {
                        "unknown"
                    } else {
                        &pr_creator
                    }
                );
                log_info!(
                    "   Expected user: {}",
                    if expected_user.is_empty() {
                        "unknown"
                    } else {
                        &expected_user
                    }
                );

                if !pr_creator.is_empty()
                    && !expected_user.is_empty()
                    && pr_creator != expected_user
                {
                    log_error!("bump-version-* branches can only be created by authorized user");
                    log_error!("   PR creator: {}, expected: {}", pr_creator, expected_user);
                    return Err(color_eyre::eyre::eyre!(
                        "Unauthorized PR creator: {} (expected: {})",
                        pr_creator,
                        expected_user
                    ));
                }
            }

            log_success!("Setting should_skip=true (bump-version-* branch detected)");

            if self.ci_mode {
                self.output_github_actions(true)?;
            }

            return Ok(true);
        }

        log_info!("Not a bump-version-* branch, CI will run normally");

        if self.ci_mode {
            self.output_github_actions(false)?;
        }

        Ok(false)
    }

    /// 输出到 GitHub Actions GITHUB_OUTPUT
    fn output_github_actions(&self, should_skip: bool) -> Result<()> {
        let output_file = env::var("GITHUB_OUTPUT")
            .ok()
            .ok_or_else(|| color_eyre::eyre::eyre!("GITHUB_OUTPUT not set"))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
            .wrap_err_with(|| format!("Failed to open GITHUB_OUTPUT: {}", output_file))?;

        writeln!(file, "should_skip={}", should_skip)
            .wrap_err("Failed to write should_skip to GITHUB_OUTPUT")?;

        log_success!("Output should_skip={} to GITHUB_OUTPUT", should_skip);

        Ok(())
    }
}
