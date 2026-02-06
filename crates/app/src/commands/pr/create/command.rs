use domain::GitRepository;
use prompt::{error, info, input, spinner, success, warning};

use crate::registry;
use crate::workflows::utils::branch::{
    generate_branch_name_from_jira, generate_branch_name_from_template,
};
use crate::workflows::utils::jira::{ensure_jira_status_config, get_jira_id_interactive_optional};

use crate::commands::pr::create::branch::{handle_default_branch, handle_non_default_branch};
use crate::commands::pr::create::commit::commit_changes;
use crate::commands::pr::create::pr::{create_pull_request, generate_pr_summary};
use crate::commands::pr::create::types::BranchHandleContext;

/// Pull Request Create 命令
pub struct PullRequestCreateCommand {
    jira_id: Option<String>,
    dry_run: bool,
}

impl PullRequestCreateCommand {
    /// 创建新的 PullRequestCreateCommand
    pub fn new(jira_id: Option<String>, dry_run: bool) -> Self {
        Self { jira_id, dry_run }
    }

    /// 运行 `workflow pr create` 命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let branch_repo = registry::get_git_repository();
        let jira_repo = registry::get_jira_repository();

        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive_optional(self.jira_id.clone())?;

        // 如果有 JIRA ID，确保状态配置存在
        let jira_created_status = if !self.dry_run {
            ensure_jira_status_config(jira_repo.as_ref(), &jira_id)?
        } else {
            None
        };

        // 保存 description 用于后续提交
        let mut description_for_commit = None;

        // 第一步：创建分支
        let branch_name = if let Some(ref jira_id) = jira_id {
            // 从 JIRA ID 生成分支名
            generate_branch_name_from_jira(jira_id)?
        } else {
            // 如果没有 JIRA ID，让用户输入描述
            let description = input!("Please enter ticket description:")
                .validator(|input: &str| {
                    let trimmed = input.trim();
                    if trimmed.is_empty() {
                        Err("Description cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .map(|s: String| s.trim().to_string())
                .map_err(|e| format!("Failed to get description: {}", e))?;

            // 保存 description 用于后续提交
            description_for_commit = Some(description.clone());

            // 先让用户选择分支类型（在 Spinner 之外，避免 raw mode 冲突）
            let branch_type = crate::workflows::utils::branch::select_branch_type()
                .map_err(|e| format!("Failed to select branch type: {}", e))?;

            // 然后使用 Spinner 生成分支名称
            let base_branch_name = {
                let branch_repo = registry::get_git_repository();
                let exists_branches: Option<Vec<String>> = branch_repo
                    .list_branches(false, true)
                    .map(|branches| branches.iter().map(|b| b.name.clone()).collect())
                    .ok();

                let llm_repo = registry::get_llm_repository();
                match spinner!("Generating branch name...").with(|| {
                    llm_repo.generate_branch_name(Some(description.as_str()), exists_branches)
                }) {
                    Ok(name) => name,
                    Err(e) => {
                        warning!("LLM generation failed: {}, using fallback method", e);
                        crate::workflows::utils::branch::to_slug(description.as_str())
                    }
                }
            };

            // 使用模板将基础分支名与 branch_type 组合成完整分支名（不包含 JIRA ID）
            generate_branch_name_from_template(branch_type, &base_branch_name, None)?
        };

        // 获取默认分支和当前分支
        let default_branch = branch_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;

        let current_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 根据当前分支和默认分支的关系，处理不同的逻辑
        // 返回: (新分支名, 目标分支)
        let (final_branch_name, target_branch) = if default_branch != current_branch {
            // 情况1: 不是默认分支，询问用户如何处理
            let ctx = BranchHandleContext {
                branch_repo: branch_repo.as_ref(),
                current_branch: &current_branch,
                default_branch: &default_branch,
                generated_branch_name: &branch_name,
                jira_id: &jira_id,
                description: description_for_commit.as_deref(),
            };
            handle_non_default_branch(&ctx, self.dry_run)?
        } else {
            // 情况2: 是默认分支，提交代码并创建新 PR
            handle_default_branch(&default_branch, &branch_name)
        };

        // 如果返回了分支名，说明需要创建新分支并创建 PR
        if let Some(new_branch_name) = final_branch_name {
            self.create_branch_and_pr(
                branch_repo.as_ref(),
                &new_branch_name,
                target_branch.as_deref(),
                &jira_id,
                &jira_created_status,
                description_for_commit.as_deref(),
            )?;
        }

        Ok(())
    }

    /// 创建新分支并创建 PR
    fn create_branch_and_pr(
        &self,
        branch_repo: &dyn GitRepository,
        new_branch_name: &str,
        target_branch: Option<&str>,
        jira_id: &Option<String>,
        jira_created_status: &Option<String>,
        description: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // 检查分支是否已存在
        let (exists_local, exists_remote) = branch_repo
            .has_branch(new_branch_name)
            .map_err(|e| format!("Failed to check branch existence: {}", e))?;

        if exists_local || exists_remote {
            error!("Branch '{}' already exists", new_branch_name);
            if exists_local {
                info!("  Local branch exists");
            }
            if exists_remote {
                info!("  Remote branch exists");
            }
            return Err(format!("Branch '{}' already exists", new_branch_name).into());
        }

        // 获取当前分支作为源分支（可能在 handle_non_default_branch 中已经切换）
        let source_branch = branch_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        // 创建新分支
        if self.dry_run {
            info!(
                "[DRY RUN] Would create branch '{}' from '{}'",
                new_branch_name, source_branch
            );
            info!("[DRY RUN] Would switch to branch '{}'", new_branch_name);
        } else {
            info!(
                "Creating branch '{}' from '{}'...",
                new_branch_name, source_branch
            );
            branch_repo
                .create_branch(new_branch_name)
                .map_err(|e| format!("Failed to create branch: {}", e))?;

            branch_repo
                .checkout_branch(new_branch_name)
                .map_err(|e| format!("Failed to checkout branch: {}", e))?;

            success!("Created and switched to branch '{}'", new_branch_name);
        }

        // 生成 PR 详细总结（在提交之前）
        let pr_content = generate_pr_summary(branch_repo, new_branch_name, jira_id, description)?;

        // 提交代码（如果有更改）
        if !self.dry_run {
            commit_changes(branch_repo, jira_id, description)?;
        } else {
            let status = branch_repo
                .get_working_tree_status()
                .map_err(|e| format!("Failed to check working tree status: {}", e))?;

            if !status.is_clean() {
                info!("[DRY RUN] Would commit changes");
                let commit_message = if let Some(jira_id) = jira_id {
                    format!(
                        "[DRY RUN] Commit message would be: {}: <JIRA summary>",
                        jira_id
                    )
                } else if let Some(desc) = description {
                    format!("[DRY RUN] Commit message would be: {}", desc)
                } else {
                    "[DRY RUN] Commit message would be generated".to_string()
                };
                info!("{}", commit_message);
                info!(
                    "[DRY RUN] Would push branch '{}' to remote",
                    new_branch_name
                );
            } else {
                info!("[DRY RUN] No changes to commit");
            }
        }

        // 创建 PR（如果有 PR 内容）
        if let Some(pr_content) = pr_content {
            let pr_result = create_pull_request(
                branch_repo,
                new_branch_name,
                &pr_content,
                target_branch,
                self.dry_run,
            )?;

            // 如果 PR 创建成功，更新 Jira ticket
            if let Some(pr_result) = pr_result {
                if jira_id.is_some() && jira_created_status.is_some() {
                    // 获取仓库 URL
                    let repo_info = branch_repo.get_repo_info();
                    let repository_url = repo_info.origin_url.as_deref().unwrap_or("");

                    self.update_jira_after_pr_created(
                        jira_id,
                        jira_created_status,
                        &pr_result.pr_id,
                        &pr_result.pr_url,
                        repository_url,
                        new_branch_name,
                    )?;
                }
            }
        }

        Ok(())
    }

    /// PR 创建后更新 Jira ticket
    ///
    /// 如果有 Jira ticket 和状态配置，更新 ticket：
    /// - 更新状态到 "PR 创建" 状态
    /// - 添加评论（PR URL）
    /// - 写入工作历史记录
    ///
    /// # 参数
    ///
    /// * `jira_repo` - Jira 仓储
    /// * `work_history_repo` - 工作历史记录仓储
    /// * `jira_ticket` - 可选的 Jira ticket ID
    /// * `created_status` - 可选的 PR 创建状态
    /// * `pr_id` - PR ID
    /// * `pr_url` - PR URL
    /// * `repository_url` - 仓库 URL
    /// * `branch_name` - 分支名称
    pub fn update_jira_after_pr_created(
        &self,
        jira_ticket: &Option<String>,
        created_status: &Option<String>,
        pr_id: &str,
        pr_url: &str,
        repository_url: &str,
        branch_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(ref ticket) = jira_ticket else {
            return Ok(());
        };

        let Some(ref status) = created_status else {
            return Ok(());
        };

        let jira_repo = registry::get_jira_repository();
        let work_history_repo = registry::get_jira_work_history_repository();

        // 更新 Jira ticket
        spinner!("Updating Jira ticket {}...", ticket).with(
            || -> Result<(), Box<dyn std::error::Error>> {
                // 更新状态
                jira_repo
                    .update_issue_status(ticket, status)
                    .map_err(|e| format!("Failed to update issue status: {}", e))?;

                // 添加评论（PR URL）
                jira_repo
                    .add_comment(ticket, pr_url)
                    .map_err(|e| format!("Failed to add comment: {}", e))?;

                Ok(())
            },
        )?;

        success!("Updated Jira ticket {} to status: {}", ticket, status);

        // 写入工作历史记录
        work_history_repo
            .write_work_history(
                ticket,
                pr_id,
                Some(pr_url),
                repository_url,
                Some(branch_name),
            )
            .map_err(|e| format!("Failed to write work history: {}", e))?;

        info!("Work history recorded for PR #{}", pr_id);

        Ok(())
    }
}
