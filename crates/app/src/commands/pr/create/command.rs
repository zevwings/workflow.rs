use domain::{get_change_types_by_branch_type, BranchType, GitRepository, JiraIssue};
use prompt::{error, info, input, spinner, success, warning};
use toolkit::{log_debug, log_error};

use crate::registry;
use crate::utils::{
    generate_branch_name_from_jira, generate_branch_name_from_template, select_branch_type, to_slug,
};

use crate::utils::{
    ensure_jira_status_config, generate_pull_request_body, generate_pull_request_title,
    get_jira_id_interactive_optional,
};

use crate::commands::pr::create::branch::{handle_default_branch, handle_non_default_branch};
use crate::commands::pr::create::commit::commit_changes;
use crate::commands::pr::create::pr::{create_pull_request, format_pr_title, generate_pr_summary};
use crate::commands::pr::create::types::BranchHandleContext;

/// 创建分支并创建 PR 时的 JIRA/描述上下文
struct CreatePullRequrestContext<'a> {
    jira_id: &'a Option<String>,
    jira_created_status: &'a Option<String>,
    description: Option<&'a str>,
    jira_info: Option<&'a JiraIssue>,
}

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
        let git_repo = registry::get_git_repository();
        let jira_repo = registry::get_jira_repository();

        // 获取 JIRA ID（交互式或从参数）
        let jira_id = get_jira_id_interactive_optional(self.jira_id.clone())?;

        // 如果有 JIRA ID，确保状态配置存在
        let jira_created_status = if !self.dry_run {
            ensure_jira_status_config(jira_repo.as_ref(), &jira_id)?
        } else {
            None
        };

        // 第一步：创建分支，并保存 description、jira_issue 用于后续提交和 PR body
        let (branch_name, branch_type, description_for_commit, jira_issue_opt) =
            if let Some(ref jira_id) = jira_id {
                // 从 JIRA ID 生成分支名
                let result = generate_branch_name_from_jira(jira_id)?;
                let description_for_commit =
                    Some(format!("{}: {}", jira_id, result.jira_issue.fields.summary));
                (
                    result.branch_name,
                    result.branch_type,
                    description_for_commit,
                    Some(result.jira_issue),
                )
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

                // 先让用户选择分支类型（在 Spinner 之外，避免 raw mode 冲突）
                let branch_type = select_branch_type()
                    .map_err(|e| format!("Failed to select branch type: {}", e))?;

                // 然后使用 Spinner 生成分支名称
                let base_branch_name = {
                    let branch_repo = registry::get_git_repository();
                    let exists_branches: Vec<String> = branch_repo
                        .list_branches(false, true)
                        .map(|branches| branches.iter().map(|b| b.name.clone()).collect())
                        .unwrap_or_default();

                    let branch_service = registry::get_branch_service();
                    match spinner!("Generating branch name...").with(|| {
                        branch_service
                            .generate_branch_name(Some(description.as_str()), &exists_branches)
                    }) {
                        Ok(name) => name,
                        Err(e) => {
                            warning!("LLM generation failed: {}, using fallback method", e);
                            to_slug(description.as_str())
                        }
                    }
                };

                // 使用模板将基础分支名与 branch_type 组合成完整分支名（不包含 JIRA ID）
                let branch_name =
                    generate_branch_name_from_template(branch_type, &base_branch_name, None)?;
                (branch_name, branch_type, Some(description), None)
            };

        // 获取默认分支和当前分支
        let default_branch = git_repo
            .get_default_branch()
            .map_err(|e| format!("Failed to get default branch: {}", e))?;

        let current_branch = git_repo
            .get_current_branch()
            .map_err(|e| format!("Failed to get current branch: {}", e))?;

        git_repo.add_all().map_err(|e| format!("Failed to add all files: {}", e))?;
        // 根据当前分支和默认分支的关系，处理不同的逻辑
        // 返回: (新分支名, 目标分支)
        let (final_branch_name, target_branch) = if default_branch != current_branch {
            // 情况1: 不是默认分支，询问用户如何处理
            let ctx = BranchHandleContext {
                branch_repo: git_repo.as_ref(),
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
            let ctx = CreatePullRequrestContext {
                jira_id: &jira_id,
                jira_created_status: &jira_created_status,
                description: description_for_commit.as_deref(),
                jira_info: jira_issue_opt.as_ref(),
            };
            self.create_branch_and_pr(
                git_repo.as_ref(),
                &new_branch_name,
                branch_type,
                target_branch.as_deref(),
                &ctx,
            )?;
        }

        Ok(())
    }

    /// 创建新分支并创建 PR
    fn create_branch_and_pr(
        &self,
        branch_repo: &dyn GitRepository,
        new_branch_name: &str,
        branch_type: BranchType,
        target_branch: Option<&str>,
        ctx: &CreatePullRequrestContext<'_>,
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

        // 生成 commit message（用于提交和 PR 标题）
        let commit_message = build_commit_message(ctx.jira_id, ctx.description)?;

        // 先提交代码，确保 merge diff 可用
        if !self.dry_run {
            commit_changes(branch_repo, ctx.jira_id, ctx.description)?;
        } else {
            let status = branch_repo
                .get_working_tree_status()
                .map_err(|e| format!("Failed to check working tree status: {}", e))?;

            if !status.is_clean() {
                info!("[DRY RUN] Would commit changes");
                info!("[DRY RUN] Commit message: {}", commit_message);
                info!(
                    "[DRY RUN] Would push branch '{}' to remote",
                    new_branch_name
                );
            } else {
                info!("[DRY RUN] No changes to commit");
            }
        }

        // 生成 PR 摘要（三阶段分析，用于 type/scope 和标题）
        let pr_summary = generate_pr_summary(target_branch)?;

        // // 根据分支名解析分支类型，生成 PR body（模板 + 变更类型与分支类型一致）
        // let branch_type = branch_type_from_branch_name(new_branch_name)
        //     .unwrap_or(BranchType::Feature);
        let selected_change_types = get_change_types_by_branch_type(branch_type);
        let pr_body = generate_pull_request_body(
            &selected_change_types,
            Some(&pr_summary.pr_body),
            ctx.jira_id.as_deref(),
            None,
            ctx.jira_info,
        )?;

        // 组合 PR 标题：优先使用模板，否则使用内置格式
        let pr_title = generate_pull_request_title(
            &pr_summary.type_,
            pr_summary.scope.as_deref(),
            ctx.jira_id.as_deref(),
            &commit_message,
        )
        .unwrap_or_else(|| {
            format_pr_title(
                &pr_summary.type_,
                pr_summary.scope.as_deref(),
                ctx.jira_id.as_deref(),
                &commit_message,
            )
        });

        // 创建 PR
        {
            let pr_result = create_pull_request(
                branch_repo,
                new_branch_name,
                &pr_title,
                &pr_body,
                target_branch,
                self.dry_run,
            )?;

            // 如果 PR 创建成功，更新 Jira ticket
            if let Some(pr_result) = pr_result {
                if ctx.jira_id.is_some() && ctx.jira_created_status.is_some() {
                    // 获取仓库 URL
                    let repo_info = branch_repo.get_repo_info();
                    let repository_url = repo_info.origin_url.as_deref().unwrap_or("");

                    self.update_jira_after_pr_created(
                        ctx.jira_id,
                        ctx.jira_created_status,
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
                log_debug!("Jira: updating issue {} to status {}", ticket, status);
                jira_repo.update_issue_status(ticket, status).map_err(|e| {
                    log_error!(
                        "Jira update_issue_status failed: ticket={}, status={}, error={}",
                        ticket,
                        status,
                        e
                    );
                    format!("Failed to update issue status: {}", e)
                })?;
                log_debug!("Jira: issue status updated successfully");

                // 添加评论（PR URL）
                log_debug!(
                    "Jira: adding comment to {} (pr_url len={})",
                    ticket,
                    pr_url.len()
                );
                jira_repo.add_comment(ticket, pr_url).map_err(|e| {
                    log_error!("Jira add_comment failed: ticket={}, error={}", ticket, e);
                    format!("Failed to add comment: {}", e)
                })?;
                log_debug!("Jira: comment added successfully");

                Ok(())
            },
        )?;

        success!("Updated Jira ticket {} to status: {}", ticket, status);

        // 写入工作历史记录
        log_debug!(
            "Jira: writing work history ticket={}, pr_id={}, branch={}",
            ticket,
            pr_id,
            branch_name
        );
        work_history_repo
            .write_work_history(
                ticket,
                pr_id,
                Some(pr_url),
                repository_url,
                Some(branch_name),
            )
            .map_err(|e| {
                log_error!(
                    "Jira write_work_history failed: ticket={}, pr_id={}, error={}",
                    ticket,
                    pr_id,
                    e
                );
                format!("Failed to write work history: {}", e)
            })?;
        log_debug!("Jira: work history written successfully");

        info!("Work history recorded for PR #{}", pr_id);

        Ok(())
    }
}

/// 生成 commit message 字符串
///
/// 与 `commit_changes` 中的逻辑一致，用于组合 PR 标题。
///
/// - 有 JIRA ID → `"{jira_id}: {JIRA summary}"`
/// - 无 JIRA ID → 使用 description
/// - 都没有 → 返回 "Update"
fn build_commit_message(
    jira_id: &Option<String>,
    description: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    if let Some(jira_id) = jira_id {
        if jira_id.trim().is_empty() {
            return Ok(description.unwrap_or("Update").to_string());
        }
        let jira_repo = registry::get_jira_repository();
        match spinner!("Fetching JIRA ticket '{}'...", jira_id)
            .with(|| jira_repo.get_issue_info(jira_id))
        {
            Ok(issue) => Ok(format!("{}: {}", jira_id, issue.fields.summary)),
            Err(e) => {
                warning!("Failed to fetch JIRA ticket '{}': {}", jira_id, e);
                Ok(jira_id.clone())
            }
        }
    } else {
        Ok(description.unwrap_or("Update").to_string())
    }
}
