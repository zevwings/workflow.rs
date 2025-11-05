use crate::commands::check::CheckCommand;
use crate::jira::status::JiraStatus;
use crate::{
    extract_jira_ticket_id, log_success, log_warning, Codeup, Git, GitHub, Jira, PlatformProvider,
    RepoType,
};
use anyhow::Result;

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

        // 3. 合并 PR
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

        // 4. 更新 Jira 状态（如果关联了 ticket）
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
}
