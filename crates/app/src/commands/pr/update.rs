//! 提交本地更改并推送到 PR 命令
//!
//! 将更改加入暂存区，用 AI 根据暂存内容生成 commit message，
//! 若有 Jira key 则以「jira_key: ai_message」形式提交并推送。

use domain::{extract_jira_ticket_id, GitRepository};
use prompt::{error, info, spinner, success};
use toolkit::{log_debug, log_info, log_info_with_fields};

use crate::bootstrap::{get_commit_message_service, get_git_repository, get_pull_request_service};
use crate::util::safe_push;

/// Pull Request Update 命令
///
/// 基于暂存区 → AI 生成 message → 以 jira_key: message 提交并推送
pub struct PullRequestUpdateCommand {
    dry_run: bool,
}

impl PullRequestUpdateCommand {
    /// 创建新的 PullRequestUpdateCommand
    ///
    /// # 参数
    /// * `pr_id` - PR ID（可选，不提供时自动检测当前分支的 PR）
    /// * `message` - 自定义 commit message（可选，不提供时由 AI 根据暂存内容生成）
    /// * `dry_run` - 是否仅预览不提交不推送
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// 运行 `workflow pr update` 命令
    ///
    /// 工作流程：
    /// 1. 获取当前分支关联的 PR 及从标题解析 jira_key
    /// 2. 检查暂存区有变更；无则报错
    /// 3. 生成 commit message：自定义或 AI 根据暂存内容生成
    /// 4. 若有 jira_key，格式为「jira_key: message」
    /// 5. 提交并推送到远端（dry_run 则仅打印不执行）
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let pr_service = get_pull_request_service();
        let git_repo = get_git_repository();

        // 1. 获取 PR 状态并从标题解析 Jira key
        let pr_status = spinner!("Fetching PR information...")
            .with(|| pr_service.get_pr_info())
            .map_err(|e| format!("Failed to get PR status: {}", e))?;

        success!("Found PR #{}", pr_status.id);
        log_info!("Found PR #{}: {}", pr_status.id, pr_status.title);

        let jira_key = extract_jira_ticket_id(pr_status.title.as_str());

        // 2. 添加所有更改到暂存区
        git_repo
            .add_all()
            .map_err(|e| format!("Failed to add all files to staging area: {}", e))?;

        // 3. 检查暂存区是否有变更
        let staged_files = git_repo
            .get_staged_files()
            .map_err(|e| format!("Failed to get staged files: {}", e))?;

        if staged_files.is_empty() {
            error!("No staged changes to commit. Use 'git add' to stage files first.");
            return Err("No staged changes".into());
        }

        log_debug!("Found {} staged file(s) to commit", staged_files.len());

        // 4. 生成或使用 commit message
        log_info!("Analyzing staged changes and generating commit message...");

        let commit_message_service = get_commit_message_service();
        let analysis =
            spinner!("Analyzing changes and generating commit message...").with(|| {
                commit_message_service
                    .generate_for_staged()
                    .map_err(|e| format!("Failed to generate commit message: {}", e))
            })?;

        log_info_with_fields!(
            title = % analysis.commit_message.title,
            body = % analysis.commit_message.body,
            footer = % analysis.commit_message.footer,
            "Generated commit message"
        );

        let mut full = analysis.commit_message.title.clone();
        if !analysis.commit_message.body.is_empty() {
            full.push_str("\n\n");
            full.push_str(&analysis.commit_message.body);
        }
        if !analysis.commit_message.footer.is_empty() {
            full.push_str("\n\n");
            full.push_str(&analysis.commit_message.footer);
        }

        // 5. 若有 jira_key，格式为「jira_key: message」
        let commit_message = match &jira_key {
            Some(jk) => {
                let first_line = full.lines().next().unwrap_or("");
                let rest: String = full.lines().skip(1).collect::<Vec<_>>().join("\n");
                if rest.is_empty() {
                    format!("{}: {}", jk, first_line)
                } else {
                    format!("{}: {}\n{}", jk, first_line, rest)
                }
            }
            None => full,
        };

        if self.dry_run {
            info!(
                "[DRY RUN] Would commit with message: {}",
                commit_message.lines().next().unwrap_or("")
            );
            return Ok(());
        }

        // 6. 提交（暂存区已就绪）
        self.commit_changes(&*git_repo, &commit_message)?;

        // 7. 推送到远端
        safe_push(None, false)?;

        success!(
            "Successfully updated PR #{} with commit: {}",
            pr_status.id,
            commit_message.lines().next().unwrap_or("")
        );

        Ok(())
    }

    /// 提交暂存区更改
    fn commit_changes(
        &self,
        git_repo: &dyn GitRepository,
        commit_message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        log_debug!(
            "Committing changes with message: {}",
            commit_message.lines().next().unwrap_or("")
        );

        let commit_sha = git_repo.commit(commit_message, false).map_err(|e| {
            let err_msg = e.to_string();
            if err_msg.contains("nothing to commit") {
                return "No changes to commit".into();
            }
            format!("Failed to commit changes: {}", e)
        })?;

        success!(
            "Committed changes: {}",
            &commit_sha[..7.min(commit_sha.len())]
        );

        Ok(())
    }
}
