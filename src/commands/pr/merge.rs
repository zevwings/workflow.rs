use crate::commands::check::CheckCommand;
use crate::jira::status::JiraStatus;
use crate::{
    extract_jira_ticket_id, log_success, log_warning, Codeup, Git, GitHub, Jira, PlatformProvider,
    RepoType,
};
use anyhow::{Context, Result};

/// PR 合并命令
#[allow(dead_code)]
pub struct PullRequestMergeCommand;

impl PullRequestMergeCommand {
    /// 合并 PR
    #[allow(dead_code)]
    pub fn merge(pull_request_id: Option<String>, _force: bool) -> Result<()> {
        // 1. 运行检查
        CheckCommand::run_all()?;

        // 2. 获取 PR ID（从参数或当前分支）
        let repo_type = Git::detect_repo_type()?;
        let pull_request_id = if let Some(id) = pull_request_id {
            id
        } else {
            // 从当前分支获取 PR
            match repo_type {
                RepoType::GitHub => {
                    match <GitHub as PlatformProvider>::get_current_branch_pull_request()? {
                        Some(id) => {
                            log_success!("Found PR for current branch: #{}", id);
                            id
                        }
                        None => {
                            anyhow::bail!("No PR found for current branch. Please specify PR ID.");
                        }
                    }
                }
                RepoType::Codeup => {
                    match <Codeup as PlatformProvider>::get_current_branch_pull_request()? {
                        Some(id) => {
                            log_success!("Found PR for current branch: #{}", id);
                            id
                        }
                        None => {
                            anyhow::bail!("No PR found for current branch. Please specify PR ID or branch name.");
                        }
                    }
                }
                _ => {
                    anyhow::bail!("Auto-detection of PR ID is only supported for GitHub and Codeup repositories.");
                }
            }
        };

        log_success!("\nMerging PR: #{}", pull_request_id);

        // 3. 获取当前分支名（合并前保存）
        let current_branch = Git::current_branch()?;

        // 4. 获取默认分支
        let default_branch = match repo_type {
            RepoType::GitHub => {
                let (owner, repo_name) = GitHub::get_owner_and_repo()?;
                GitHub::get_default_branch(&owner, &repo_name)?
            }
            RepoType::Codeup => {
                // Codeup 默认分支通常是 develop 或 main
                // 尝试从远程获取默认分支
                Self::get_default_branch_from_remote()?
            }
            _ => {
                // 尝试从远程获取默认分支
                Self::get_default_branch_from_remote()?
            }
        };

        // 5. 合并 PR
        match repo_type {
            RepoType::GitHub => {
                <GitHub as PlatformProvider>::merge_pull_request(&pull_request_id, true)?; // delete-branch
                log_success!("PR merged successfully");
            }
            RepoType::Codeup => {
                <Codeup as PlatformProvider>::merge_pull_request(&pull_request_id, true)?; // delete-branch
                log_success!("PR merged successfully");
            }
            _ => {
                anyhow::bail!(
                    "PR merge is currently only supported for GitHub and Codeup repositories."
                );
            }
        }

        // 6. 合并后清理：切换到默认分支并删除当前分支
        // 注意：远程分支已经通过 API 删除（在 merge_pull_request 中）
        Self::cleanup_after_merge(&current_branch, &default_branch)?;

        // 7. 更新 Jira 状态（如果关联了 ticket）
        // 尝试从历史记录读取
        let mut jira_ticket = JiraStatus::read_work_history(&pull_request_id)?;

        // 如果历史记录中没有，尝试从 PR 标题提取
        if jira_ticket.is_none() {
            match repo_type {
                RepoType::GitHub => {
                    if let Ok(title) =
                        <GitHub as PlatformProvider>::get_pull_request_title(&pull_request_id)
                    {
                        jira_ticket = extract_jira_ticket_id(&title);
                    }
                }
                RepoType::Codeup => {
                    if let Ok(title) =
                        <Codeup as PlatformProvider>::get_pull_request_title(&pull_request_id)
                    {
                        jira_ticket = extract_jira_ticket_id(&title);
                    }
                }
                _ => {}
            }
        }

        if let Some(ticket) = jira_ticket {
            // 读取合并时的状态
            if let Ok(Some(status)) = JiraStatus::read_pull_request_merged_status(&ticket) {
                log_success!("Updating Jira ticket: {} to status: {}", ticket, status);
                Jira::move_ticket(&ticket, &status)?;
                log_success!("Jira ticket updated");
            } else {
                log_warning!("No Jira status configuration found for ticket: {}", ticket);
            }

            // 更新工作历史记录的合并时间
            JiraStatus::update_work_history_merged(&pull_request_id)?;
        } else {
            log_warning!("No Jira ticket associated with this PR");
        }

        Ok(())
    }

    /// 从远程获取默认分支
    fn get_default_branch_from_remote() -> Result<String> {
        use duct::cmd;

        // 尝试获取远程默认分支
        let output = cmd("git", &["symbolic-ref", "refs/remotes/origin/HEAD"])
            .read()
            .or_else(|_| {
                // 如果上面失败，尝试从 remote show origin 获取
                let remote_info = cmd("git", &["remote", "show", "origin"]).read()?;
                for line in remote_info.lines() {
                    if line.contains("HEAD branch:") {
                        if let Some(branch) = line.split("HEAD branch:").nth(1) {
                            return Ok(branch.trim().to_string());
                        }
                    }
                }
                anyhow::bail!("Could not determine default branch")
            })?;

        // 提取分支名：refs/remotes/origin/main -> main
        let branch = output
            .trim()
            .strip_prefix("refs/remotes/origin/")
            .unwrap_or(output.trim())
            .to_string();

        Ok(branch)
    }

    /// 合并后清理：切换到默认分支并删除当前分支
    fn cleanup_after_merge(current_branch: &str, default_branch: &str) -> Result<()> {
        use crate::log_info;
        use duct::cmd;

        // 如果当前分支已经是默认分支，不需要清理
        if current_branch == default_branch {
            log_info!("Already on default branch: {}", default_branch);
            return Ok(());
        }

        log_info!("Switching to default branch: {}", default_branch);
        log_info!(
            "Note: Remote branch '{}' has already been deleted via API",
            current_branch
        );

        // 1. 先更新远程分支信息（这会同步远程删除的分支）
        cmd("git", &["fetch", "origin"])
            .run()
            .context("Failed to fetch from origin")?;

        // 2. 切换到默认分支
        cmd("git", &["checkout", default_branch])
            .run()
            .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

        // 3. 更新本地默认分支
        cmd("git", &["pull", "origin", default_branch])
            .run()
            .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

        // 4. 删除本地分支（如果还存在）
        if Self::branch_exists_locally(current_branch)? {
            log_info!("Deleting local branch: {}", current_branch);
            cmd("git", &["branch", "-d", current_branch])
                .run()
                .or_else(|_| {
                    // 如果分支未完全合并，尝试强制删除
                    log_info!("Branch may not be fully merged, trying force delete...");
                    cmd("git", &["branch", "-D", current_branch])
                        .run()
                        .with_context(|| {
                            format!("Failed to delete local branch: {}", current_branch)
                        })
                })
                .context("Failed to delete local branch")?;
            log_success!("Local branch deleted: {}", current_branch);
        } else {
            log_info!("Local branch already deleted: {}", current_branch);
        }

        // 5. 清理远程分支引用（prune，移除已删除的远程分支引用）
        cmd("git", &["remote", "prune", "origin"])
            .run()
            .context("Failed to prune remote references")?;

        log_success!(
            "Cleanup completed: switched to {} and deleted local branch {}",
            default_branch,
            current_branch
        );
        log_success!(
            "Remote branch '{}' was already deleted via API",
            current_branch
        );
        Ok(())
    }

    /// 检查本地分支是否存在
    fn branch_exists_locally(branch_name: &str) -> Result<bool> {
        use duct::cmd;

        let output = cmd("git", &["branch", "--list", branch_name])
            .read()
            .context("Failed to list branches")?;

        Ok(output
            .trim()
            .lines()
            .any(|line| line.trim().trim_start_matches('*').trim() == branch_name))
    }
}
