//! PR 重写命令：基于三阶段提交分析重新生成 PR 描述并更新到远端。
//!
//! 流程：运行分析 → 展示摘要 → 用户确认 → 从当前分支解析 branch_type，
//! 使用与 pr create 相同的模板生成 PR body，并调用 API 仅更新 description。

use domain::{
    extract_jira_ticket_id, get_change_types_by_branch_type, BranchType, PullRequestService,
};
use prompt::{confirm, info, spinner, success, warning};

use crate::registry::{get_commit_summary_service, get_git_repository, get_pull_request_service};
use crate::workflows::utils::branch::branch_type_from_branch_name;
use crate::workflows::utils::pull_request::generate_pull_request_body;

/// PR 重写命令（基于三阶段分析 + 模板更新 description）
pub struct PullRequestRewordCommand {
    dry_run: bool,
}

impl Default for PullRequestRewordCommand {
    fn default() -> Self {
        Self::new(false)
    }
}

impl PullRequestRewordCommand {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("starting 3-stage commit summary analysis...");

        // 1. 解析当前分支与 PR
        let git_repo = get_git_repository();
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 2. 获取当前分支的 PR ID
        let pr_service: std::sync::Arc<dyn PullRequestService> = get_pull_request_service();

        let pr_id = pr_service.get_current_branch_pull_request(&current_branch)?;
        if pr_id.is_none() {
            if self.dry_run {
                warning!("[DRY RUN] No PR found for current branch.");
            } else {
                return Err("No PR found for current branch".into());
            }
        }

        let summary_service = get_commit_summary_service();
        let summary = summary_service.run_analysis(None)?;
        info!("Pull request reword analysis completed.");
        info!("Pull request reword: \n{}", summary.to_markdown());

        if self.dry_run {
            info!("[DRY RUN] Pull request reword skipped (no changes made).");
            return Ok(());
        }

        let confirm_reword = confirm!("Are you sure you want to reword the pull request?")
            .default(true)
            .prompt()?;
        if !confirm_reword {
            info!("Pull request reword cancelled.");
            return Err("Pull request reword cancelled.".into());
        }

        let pr_status = spinner!("Fetching PR information...")
            .with(|| pr_service.get_pr_status(None))
            .map_err(|e| format!("No PR for current branch or API error: {}", e))?;

        // 2. 从分支名得到 branch_type，生成模板所需的 selected_change_types
        let branch_type =
            branch_type_from_branch_name(current_branch.as_str()).unwrap_or(BranchType::Feature);
        let selected_change_types = get_change_types_by_branch_type(branch_type);

        // 3. 使用与 pr create 相同的模板生成 PR body（传入完整 LLM 正文，与 pr create 一致）
        let llm_body = summary.to_markdown();
        let jira_ticket = extract_jira_ticket_id(current_branch.as_str());
        let pr_body = generate_pull_request_body(
            &selected_change_types,
            Some(llm_body.as_str()),
            jira_ticket.as_deref(),
            None,
            None,
        )?;

        // 4. 仅更新 PR 描述（不改标题）
        if self.dry_run {
            info!("[DRY RUN] Would update PR #{} description:", pr_status.id);
            if !pr_body.is_empty() {
                info!("{}", pr_body);
            }
            success!("[DRY RUN] Pull request reword skipped (no changes made).");
        } else {
            spinner!("Updating PR description...")
                .with(|| pr_service.update_pull_request(&pr_status.id, None, Some(&pr_body)))
                .map_err(|e| format!("Failed to update PR description: {}", e))?;
            success!("Pull request reworded successfully.");
        }
        Ok(())
    }
}
