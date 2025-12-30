use crate::commands::check;
use crate::commands::pr::helpers;
use crate::git::{GitBranch, GitRepo};
use crate::jira::status::JiraStatus;
use crate::jira::{extract_jira_ticket_id, Jira, JiraWorkHistory};
use crate::pr::create_provider_auto;
use crate::pr::helpers::resolve_pull_request_id;
use crate::{log_break, log_info, log_success, log_warning};
use color_eyre::Result;

/// PR 合并命令
pub struct PullRequestMergeCommand;

impl PullRequestMergeCommand {
    /// 合并 PR
    pub fn merge(pull_request_id: Option<String>, _force: bool) -> Result<()> {
        // 1. 运行环境检查
        check::CheckCommand::run_all()?;

        // 2. 获取 PR ID
        let pull_request_id = resolve_pull_request_id(pull_request_id)?;

        log_break!();
        log_success!("Merging PR: #{}", pull_request_id);

        // 3. 获取当前分支名（合并前保存）
        let current_branch = GitBranch::current_branch()?;

        // 4. 获取默认分支
        let default_branch = GitBranch::get_default_branch()?;

        // 5. 获取 PR 的目标分支（合并到的分支）
        let target_branch =
            Self::get_pr_target_branch(&pull_request_id).unwrap_or_else(|_| default_branch.clone());

        // 6. 合并 PR（如果已合并，跳过合并步骤但继续执行后续步骤）
        Self::merge_pull_request(&pull_request_id)?;

        // 7. 合并后清理：切换到目标分支并删除当前分支
        // 注意：如果 PR 已合并，远程分支可能已经被删除
        Self::cleanup_after_merge(&current_branch, &target_branch, &default_branch)?;

        // 8. 更新 Jira 状态（如果关联了 ticket）
        Self::update_jira_status(&pull_request_id)?;

        Ok(())
    }

    /// 合并 PR（根据仓库类型调用对应的实现）
    /// 返回 true 表示新合并，false 表示已经合并
    fn merge_pull_request(pull_request_id: &str) -> Result<bool> {
        let provider = create_provider_auto()?;

        // 先检查 PR 状态
        let status = provider.get_pull_request_status(pull_request_id)?;

        // 如果已经合并，跳过合并步骤
        if status.merged {
            log_warning!("PR #{} has already been merged", pull_request_id);
            if let Some(merged_at) = status.merged_at {
                log_info!("Merged at: {}", merged_at);
            }
            log_info!("Skipping merge step, continuing with cleanup...");
            return Ok(false);
        }

        // 执行合并操作
        match provider.merge_pull_request(pull_request_id, true) {
            Ok(()) => {
                log_success!("PR merged successfully");
                Ok(true)
            }
            Err(e) => {
                // 检查是否是"已合并"错误
                if helpers::is_pr_already_merged_error(&e) {
                    log_warning!(
                        "PR #{} has already been merged (detected from merge error)",
                        pull_request_id
                    );
                    log_info!("Skipping merge step, continuing with cleanup...");
                    Ok(false)
                } else {
                    // 其他错误，返回错误
                    Err(e)
                }
            }
        }
    }

    /// 更新 Jira 状态（如果关联了 ticket）
    fn update_jira_status(pull_request_id: &str) -> Result<()> {
        // 获取当前仓库 URL
        let repository = GitRepo::get_remote_url().ok();

        // 尝试从历史记录读取
        let mut jira_ticket =
            JiraWorkHistory::read_work_history(pull_request_id, repository.as_deref())?;

        // 如果历史记录中没有，尝试从 PR 标题提取
        if jira_ticket.is_none() {
            jira_ticket = Self::extract_jira_ticket_from_pr_title(pull_request_id)?;
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
        } else {
            log_warning!("No Jira ticket associated with this PR");
        }

        // 删除工作历史记录中的 PR ID 条目
        let delete_result =
            JiraWorkHistory::delete_work_history_entry(pull_request_id, repository.as_deref())?;

        // 显示删除消息
        for message in &delete_result.messages {
            log_info!("{}", message);
        }

        // 显示警告信息
        for warning in &delete_result.warnings {
            log_warning!("{}", warning);
        }

        Ok(())
    }

    /// 从 PR 标题提取 Jira ticket ID
    fn extract_jira_ticket_from_pr_title(pull_request_id: &str) -> Result<Option<String>> {
        let provider = create_provider_auto()?;
        let title = provider.get_pull_request_title(pull_request_id).ok();
        Ok(title.and_then(|t| extract_jira_ticket_id(&t)))
    }

    /// 获取 PR 的目标分支（合并到的分支）
    fn get_pr_target_branch(pull_request_id: &str) -> Result<String> {
        let provider = create_provider_auto()?;

        // 从 PR 信息中获取目标分支名
        let info = provider.get_pull_request_info(pull_request_id)?;

        // 解析 PR 信息，提取目标分支名
        // PR 信息格式包含 "Target Branch: branch_name"
        for line in info.lines() {
            if let Some(branch_line) = line.strip_prefix("Target Branch: ") {
                return Ok(branch_line.trim().to_string());
            }
        }

        color_eyre::eyre::bail!(
            "Failed to extract target branch from PR #{}",
            pull_request_id
        )
    }

    /// 合并后清理：切换到目标分支并删除当前分支
    fn cleanup_after_merge(
        current_branch: &str,
        target_branch: &str,
        default_branch: &str,
    ) -> Result<()> {
        log_info!(
            "Note: Remote branch '{}' may have already been deleted via API",
            current_branch
        );

        // 确定要切换到的分支：如果目标分支存在且不是默认分支，切换到目标分支；否则切换到默认分支
        let switch_to_branch = if target_branch != default_branch {
            // 检查目标分支是否存在
            if GitBranch::has_local_branch(target_branch).unwrap_or(false)
                || GitBranch::has_remote_branch(target_branch).unwrap_or(false)
            {
                log_info!(
                    "PR merged to '{}', switching to target branch instead of default branch",
                    target_branch
                );
                target_branch
            } else {
                log_warning!(
                    "Target branch '{}' not found, falling back to default branch '{}'",
                    target_branch,
                    default_branch
                );
                default_branch
            }
        } else {
            default_branch
        };

        helpers::cleanup_branch(current_branch, switch_to_branch, "PR merge")?;
        Ok(())
    }
}
