//! PR 重写命令：基于三阶段提交分析重新生成 PR 描述并更新到远端。
//!
//! 流程：运行分析 → 展示摘要 → 用户确认 → 从当前分支解析 branch_type，
//! 使用与 pr create 相同的模板生成 PR body，并调用 API 仅更新 description。

use domain::{
    extract_jira_ticket_id, get_change_types_by_branch_type, BranchType, PullRequestService,
};
use prompt::{br, confirm, info, spinner, success, warning};

use crate::commands::pr::utils::generate_pull_request_body;
use crate::{
    bootstrap::{get_commit_summary_service, get_git_repository, get_pull_request_service},
    util::branch::branch_type_from_branch_name,
};

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
        // 1. 解析当前分支与 PR
        let git_repo = get_git_repository();
        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 2. 获取当前分支的 PR 信息（单次 API 调用，避免重复）
        let pr_service: std::sync::Arc<dyn PullRequestService> = get_pull_request_service();

        let pr_info = spinner!("Fetching PR information...")
            .with(|| pr_service.get_pr_info())
            .map_err(|e| format!("Failed to get PR info: {}", e))?;

        if pr_info.id.is_empty() {
            if self.dry_run {
                warning!("[DRY RUN] No PR found for current branch.");
            } else {
                return Err("No PR found for current branch".into());
            }
        }

        let summary_service = get_commit_summary_service();
        let summary = spinner!("Analyzing commit summary...")
            .with(|| summary_service.run_analysis(None))
            .map_err(|e| format!("Failed to analyze commit summary: {}", e))?;

        success!("Pull request reword analysis completed.");
        br!();

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
            info!("[DRY RUN] Would update PR #{} description:", pr_info.id);
            if !pr_body.is_empty() {
                info!("{}", pr_body);
            }
            success!("[DRY RUN] Pull request reword skipped (no changes made).");
        } else {
            spinner!("Updating PR description...")
                .with(|| pr_service.update_pull_request(&pr_info.id, None, Some(&pr_body)))
                .map_err(|e| format!("Failed to update PR description: {}", e))?;
            success!("Pull request reworded successfully.");
        }
        Ok(())
    }
}
