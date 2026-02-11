//! Commit create 命令实现
//!
//! 智能生成 commit message 并提交代码。

use domain::GitRepository;
use prompt::{error, info, spinner, success};
use toolkit::{log_info, log_info_with_fields};

use crate::bootstrap;

/// Commit Create 命令
pub struct CommitCreateCommand {
    /// 是否自动添加所有更改
    all: bool,
    /// 是否自动推送到远端
    push: bool,
    /// 是否为 dry-run 模式
    dry_run: bool,
    /// 自定义 commit message（如果提供则跳过 AI 生成）
    message: Option<String>,
}

impl CommitCreateCommand {
    /// 创建新的 CommitCreateCommand
    ///
    /// # 参数
    /// - `all`: 是否添加所有更改到暂存区
    /// - `push`: 是否在提交后自动推送到远端
    /// - `dry_run`: 是否为 dry-run 模式（仅预览不实际提交）
    /// - `message`: 可选的自定义 commit message
    pub fn new(all: bool, push: bool, dry_run: bool, message: Option<String>) -> Self {
        Self {
            all,
            push,
            dry_run,
            message,
        }
    }

    /// 运行 `workflow commit create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = bootstrap::get_git_repository();

        // Step 1: Stage 代码（如果需要）
        if self.all {
            info!("Adding all files to staging area...");
            git_repo
                .add_all()
                .map_err(|e| format!("Failed to add files to staging area: {}", e))?;
        }

        // Step 2: 检查暂存区是否有变更
        let staged_files = git_repo
            .get_staged_files()
            .map_err(|e| format!("Failed to get staged files: {}", e))?;

        if staged_files.is_empty() {
            error!("No staged changes to commit. Use 'git add' to stage files first.");
            return Err("No staged changes".into());
        }

        info!("Found {} staged file(s) to commit", staged_files.len());

        // Step 3: 生成或使用 commit message
        let commit_message = if let Some(msg) = &self.message {
            // 使用用户提供的 message
            msg.clone()
        } else {
            // 使用 AI 生成 commit message
            log_info!("Analyzing changes and generating commit message...");

            let commit_message_service = bootstrap::get_commit_message_service();
            let analysis =
                spinner!("Analyzing changes and generating commit message...").with(|| {
                    commit_message_service
                        .generate_for_staged()
                        .map_err(|e| format!("Failed to generate commit message: {}", e))
                })?;

            // 结构化输出生成的 commit message（便于日志采集与检索）
            log_info_with_fields!(
                title = % analysis.commit_message.title,
                body = % analysis.commit_message.body,
                footer = % analysis.commit_message.footer,
                "Generated commit message"
            );

            if self.dry_run {
                info!(
                    "[DRY RUN] Commit message: {}",
                    analysis.commit_message.title
                );
                return Ok(());
            }

            // 构建完整的 commit message
            let mut full_message = analysis.commit_message.title.clone();
            if !analysis.commit_message.body.is_empty() {
                full_message.push_str("\n\n");
                full_message.push_str(&analysis.commit_message.body);
            }
            if !analysis.commit_message.footer.is_empty() {
                full_message.push_str("\n\n");
                full_message.push_str(&analysis.commit_message.footer);
            }

            full_message
        };

        // dry_run 时仅预览，不实际提交
        if self.dry_run {
            info!(
                "[DRY RUN] Would commit with message: {}",
                commit_message.lines().next().unwrap_or("")
            );
            return Ok(());
        }

        info!(
            "Committing changes with message: {}",
            commit_message.lines().next().unwrap_or("")
        );

        // Step 4: 提交代码
        let oid = spinner!("Creating commit...")
            .with(|| git_repo.commit(&commit_message, false)) // false 因为文件已经在 Step 1 中添加到暂存区
            .map_err(|e| format!("Failed to create commit: {}", e))?;

        success!("✓ Created commit: {}", oid);

        // Step 5: 询问是否推送到远端
        if self.push {
            self.push_to_remote(&git_repo)?;
        } else {
            let should_push = prompt::confirm!("\nPush to remote?").default(false).prompt()?;
            if should_push {
                self.push_to_remote(&git_repo)?;
            }
        }

        Ok(())
    }

    /// 推送到远端
    fn push_to_remote(
        &self,
        git_repo: &std::sync::Arc<dyn GitRepository>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let branch_name = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        spinner!("Pushing to origin/{}...", branch_name)
            .with(|| git_repo.push(&branch_name, false))
            .map_err(|e| format!("Failed to push: {}", e))?;

        success!("✓ Pushed to origin/{}", branch_name);

        Ok(())
    }
}
