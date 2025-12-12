use anyhow::{Context, Result};

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::indicator::Spinner;
use crate::branch::{BranchNaming, BranchType};
use crate::commands::check;
use crate::commands::pr::helpers::{
    copy_and_open_pull_request, create_branch_from_default, create_or_get_pull_request,
    ensure_jira_status, handle_stash_pop_result, resolve_description, resolve_title,
    select_change_types, update_jira_ticket,
};
use crate::git::{GitBranch, GitCommit, GitStash};
use crate::jira::helpers::validate_jira_ticket_format;
use crate::jira::Jira;
use crate::pr::helpers::{generate_commit_title, generate_pull_request_body};
use crate::pr::llm::CreateGenerator;
use crate::pr::{
    map_branch_type_to_change_type_index, map_branch_type_to_change_types, TYPES_OF_CHANGES,
};
use crate::repo::RepoConfig;
use crate::{log_break, log_info, log_success, log_warning};

/// PR 创建命令
#[allow(dead_code)]
pub struct PullRequestCreateCommand;

#[allow(dead_code)]
impl PullRequestCreateCommand {
    /// 创建 PR（完整流程）
    pub fn create(
        jira_ticket: Option<String>,
        title: Option<String>,
        description: Option<String>,
        dry_run: bool,
    ) -> Result<()> {
        // 0. 检查并确保仓库配置存在
        crate::commands::repo::setup::RepoSetupCommand::ensure()?;

        // 1. 运行检查
        if !dry_run {
            check::CheckCommand::run_all()?;
        }

        // 2. 获取或输入 Jira ticket
        let jira_ticket = Self::resolve_jira_ticket(jira_ticket)?;

        // 3. 如果有 Jira ticket，检查并配置状态
        let created_pull_request_status = ensure_jira_status(&jira_ticket)?;

        // 4. 获取或生成 PR 标题
        let title = resolve_title(title, &jira_ticket, false)?;

        // 4.5. 确定分支类型（优先使用 repository prefix）
        // 需要在生成分支名之前确定类型，以便使用模板系统
        let branch_type = BranchType::resolve_with_repo_prefix()?;

        // 5. 生成 commit_title、分支名和描述
        let (commit_title, branch_name, llm_description) =
            Self::generate_commit_title_and_branch_name(&jira_ticket, &title, branch_type)?;

        // 6. 获取描述（优先使用用户输入的描述，否则使用 LLM 生成的描述）
        let short_description = if let Some(desc) = &description {
            desc.clone()
        } else if let Some(desc) = &llm_description {
            desc.clone()
        } else {
            resolve_description(description.clone())?
        };

        // 7. 选择变更类型（智能选择：根据分支类型自动选择）
        let selected_types = Self::select_change_types_with_auto_select(Some(branch_type))?;

        // 8. 获取 JIRA issue 信息（如果存在）用于模板
        let jira_info = if let Some(ref ticket) = jira_ticket {
            Jira::get_ticket_info(ticket).ok()
        } else {
            None
        };

        // 9. 生成 PR body（使用模板系统）
        let pull_request_body = generate_pull_request_body(
            &selected_types,
            Some(&short_description),
            jira_ticket.as_deref(),
            None, // dependency 暂时为空
            jira_info.as_ref(),
        )?;

        if dry_run {
            log_info!("[DRY RUN] Would create branch: {}", branch_name);
            log_info!("[DRY RUN] Commit title: {}", commit_title);
            log_info!("[DRY RUN] PR body:\n{}", pull_request_body);
            return Ok(());
        }

        // 9. 创建或更新分支
        let (actual_branch_name, default_branch) =
            Self::create_or_update_branch(&branch_name, &commit_title)?;

        // 10. 创建或获取 PR
        let pull_request_url = create_or_get_pull_request(
            &actual_branch_name,
            &default_branch,
            &commit_title,
            &pull_request_body,
        )?;

        // 11. 更新 Jira（如果有 ticket）
        update_jira_ticket(
            &jira_ticket,
            &created_pull_request_status,
            &pull_request_url,
            &actual_branch_name,
        )?;

        // 12. 复制 PR URL 到剪贴板并打开浏览器
        copy_and_open_pull_request(&pull_request_url)?;

        log_success!("PR created successfully!");
        Ok(())
    }

    /// 获取或输入 Jira ticket
    ///
    /// 步骤 2：如果提供了 ticket，验证其格式；如果没有提供，提示用户输入并验证。
    fn resolve_jira_ticket(jira_ticket: Option<String>) -> Result<Option<String>> {
        let ticket = if let Some(t) = jira_ticket {
            let trimmed = t.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        } else {
            let input = InputDialog::new("Jira ticket (optional)")
                .allow_empty(true)
                .prompt()
                .context("Failed to get Jira ticket")?;
            let trimmed = input.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        };

        // 统一验证逻辑
        if let Some(ref ticket) = ticket {
            validate_jira_ticket_format(ticket)?;
        }

        Ok(ticket)
    }

    /// 生成 commit title 和分支名
    ///
    /// 步骤 5：使用与 branch create 相同的流程生成分支名。
    /// 1. 准备公共数据（分支列表、git diff）
    /// 2. 统一调用 LLM 生成分支名、PR 标题、描述和 scope
    /// 3. LLM 成功时统一处理，失败时根据是否有 JIRA ticket 选择不同的回退策略
    /// 4. 使用模板系统根据分支类型和 slug 生成分支名
    /// 5. 应用 repository prefix
    /// 6. 生成 commit title（使用 LLM 提取的 scope，如果可用）
    ///
    /// 返回 (commit_title, branch_name, description) 元组。
    fn generate_commit_title_and_branch_name(
        jira_ticket: &Option<String>,
        title: &str,
        branch_type: BranchType,
    ) -> Result<(String, String, Option<String>)> {
        // Step 1: 准备公共数据（不管是否有 jira_ticket）
        let exists_branches = GitBranch::get_all_branches(true).ok();
        let git_diff = GitCommit::get_diff();

        // Step 2: 先尝试 LLM（统一处理）
        let (pr_title, branch_name_slug, description, llm_content) =
            match CreateGenerator::generate(title, exists_branches, git_diff) {
                Ok(content) => {
                    // LLM 成功：统一处理（不管是否有 jira_ticket）
                    log_success!("Generated branch name using LLM: {}", content.branch_name);
                    if let Some(ref scope) = content.scope {
                        log_info!("Extracted scope: {}", scope);
                    }
                    let slug = BranchNaming::sanitize(&content.branch_name);
                    let pr_title = content.pr_title.clone();
                    let description = content.description.clone();
                    (pr_title, slug, description, Some(content))
                }
                Err(e) => {
                    // LLM 失败：根据是否有 jira_ticket 选择不同的回退策略
                    if let Some(ticket) = jira_ticket {
                        // 有 JIRA ticket：从 JIRA 获取 summary
                        log_warning!(
                            "Failed to generate branch name using LLM: {}, using JIRA summary slug",
                            e
                        );
                        let issue =
                            Spinner::with(format!("Getting ticket info for {}...", ticket), || {
                                Jira::get_ticket_info(ticket)
                            })
                            .with_context(|| format!("Failed to get ticket info for {}", ticket))?;
                        let slug = BranchNaming::slugify(&issue.fields.summary);
                        (title.to_string(), slug, None, None)
                    } else {
                        // 无 JIRA ticket：翻译并清理 title
                        log_warning!(
                            "Failed to generate branch name using LLM: {}, using sanitized title",
                            e
                        );
                        let slug = BranchNaming::sanitize_and_translate_branch_name(title)?;
                        (title.to_string(), slug, None, None)
                    }
                }
            };

        // Step 2: 使用模板系统根据分支类型和 slug 生成分支名
        let branch_name = BranchNaming::from_type_and_slug(
            branch_type.as_str(),
            &branch_name_slug,
            jira_ticket.as_deref(),
        )?;

        // Step 3: 生成 commit title（使用模板系统）
        // 如果使用了 LLM 生成内容，尝试使用 LLM 提取的 scope
        let scope = llm_content.as_ref().and_then(|c| c.scope.as_deref());

        let commit_title = generate_commit_title(
            jira_ticket.as_deref(),
            &pr_title,
            None,  // commit_type - can be extracted from context if needed
            scope, // scope - 使用 LLM 提取的 scope（如果可用）
            None,  // body - optional
        )
        .unwrap_or_else(|_| {
            // Fallback to simple format if template fails
            match jira_ticket.as_deref() {
                Some(ticket) => format!("{}: {}", ticket, pr_title),
                None => format!("# {}", pr_title),
            }
        });

        Ok((commit_title, branch_name, description))
    }

    /// 选择变更类型（智能选择：根据分支类型自动选择）
    ///
    /// 步骤 7：根据分支类型自动选择对应的 PR 变更类型，显示确认对话框。
    /// 用户可以确认使用自动选择，或修改选择。
    ///
    /// # Arguments
    /// * `branch_type` - 可选的分支类型，如果提供则用于自动选择
    ///
    /// # Returns
    /// 返回布尔向量，表示每个 PR 变更类型是否被选中
    fn select_change_types_with_auto_select(branch_type: Option<BranchType>) -> Result<Vec<bool>> {
        // Step 1: 尝试自动选择
        let auto_selected = if let Some(ty) = branch_type {
            let change_types = map_branch_type_to_change_types(ty);
            // 检查是否有自动选择的结果
            if change_types.iter().any(|&selected| selected) {
                Some((ty, change_types))
            } else {
                None
            }
        } else {
            None
        };

        // Step 2: 如果有自动选择，检查是否需要跳过确认
        if let Some((ty, selected_types)) = auto_selected {
            // 获取对应的变更类型名称
            if let Some(index) = map_branch_type_to_change_type_index(ty) {
                let change_type_name = TYPES_OF_CHANGES[index];

                // 检查项目配置中是否设置了自动接受
                let should_auto_accept = match RepoConfig::load() {
                    Ok(config) => {
                        let auto_accept = config.auto_accept_change_type.unwrap_or(false);
                        if auto_accept {
                            log_info!("Auto-accept change type is enabled in project config");
                        }
                        auto_accept
                    }
                    Err(e) => {
                        log_warning!(
                            "Failed to load project config for auto_accept_change_type: {}. Using default (false).",
                            e
                        );
                        false
                    }
                };

                // 如果配置为自动接受，直接使用自动选择的结果
                if should_auto_accept {
                    log_success!(
                        "Using auto-selected change type: {} (auto-accept enabled)",
                        change_type_name
                    );
                    return Ok(selected_types);
                }

                // 否则显示确认对话框
                // 特殊提示（Hotfix 和 Chore）
                let prompt_message = match ty {
                    BranchType::Hotfix => format!(
                        "Auto-selected change type: {}\n\nNote: Hotfix is a type of bug fix. You may want to add other types if this PR also includes other changes.\n\nUse this?",
                        change_type_name
                    ),
                    BranchType::Chore => {
                        // Chore 没有自动选择，直接进入手动选择
                        return select_change_types();
                    }
                    _ => format!(
                        "Auto-selected change type: {}\n\nUse this?",
                        change_type_name
                    ),
                };

                let confirmed = ConfirmDialog::new(&prompt_message)
                    .with_default(true)
                    .prompt()
                    .context("Failed to confirm change type")?;

                if confirmed {
                    log_success!("Using auto-selected change type: {}", change_type_name);
                    return Ok(selected_types);
                }
            }
        }

        // Step 3: 用户手动选择（原有逻辑）
        select_change_types()
    }

    /// 在当前分支上提交并推送
    ///
    /// 步骤 9 的辅助方法：当用户不在默认分支上且有未提交修改时，在当前分支上
    /// 提交更改并推送（如需要）。
    ///
    /// # 流程
    /// 1. 检查分支是否已推送到远程
    /// 2. 提交更改
    /// 3. 推送分支（如果未推送则设置上游，如果已推送则只推送更改）
    fn commit_and_push_current_branch(
        current_branch: &str,
        commit_title: &str,
        default_branch: &str,
    ) -> Result<(String, String)> {
        log_info!(
            "Will commit and create PR on current branch '{}'...",
            current_branch
        );

        // 检查是否已推送
        let exists_remote = GitBranch::has_remote_branch(current_branch)
            .context("Failed to check if branch exists on remote")?;

        // 提交
        Spinner::with("Committing changes...", || {
            GitCommit::commit(commit_title, true) // no-verify
        })?;

        // 推送（如需要）
        if !exists_remote {
            log_break!();
            log_info!("Pushing to remote...");
            log_break!();
            GitBranch::push(current_branch, true)?; // set-upstream
        } else {
            log_info!("Branch '{}' already exists on remote.", current_branch);
            log_info!("Pushing latest changes...");
            GitBranch::push(current_branch, false)?; // 不使用 -u，因为已经设置过
        }

        Ok((current_branch.to_string(), default_branch.to_string()))
    }

    /// 使用 stash 创建新分支并提交
    ///
    /// 步骤 9 的辅助方法：当用户不在默认分支上且有未提交修改，但选择创建新分支时，
    /// 使用 stash 暂存修改，切换到默认分支，拉取最新代码，创建新分支，恢复修改，
    /// 然后提交并推送。
    ///
    /// # 流程
    /// 1. 使用 stash 暂存未提交的修改
    /// 2. 切换到默认分支
    /// 3. 拉取最新代码
    /// 4. 检查目标分支是否存在（如果存在则报错）
    /// 5. 创建新分支
    /// 6. 恢复 stash 中的修改
    /// 7. 提交更改
    /// 8. 推送到远程
    fn create_new_branch_with_stash(
        _current_branch: &str,
        branch_name: &str,
        commit_title: &str,
        default_branch: &str,
    ) -> Result<(String, String)> {
        log_info!(
            "Will create new branch '{}' and commit changes...",
            branch_name
        );

        // 使用 stash 暂存修改
        log_success!("Stashing uncommitted changes...");
        GitStash::stash_push(Some(&format!("WIP: {}", commit_title)))?;

        // 切换到默认分支
        log_info!("Switching to default branch '{}'...", default_branch);
        GitBranch::checkout_branch(default_branch)?;

        // 拉取最新的代码
        Spinner::with(
            format!("Pulling latest changes from '{}'...", default_branch),
            || GitBranch::pull(default_branch),
        )?;

        // 检查目标分支是否存在，如果存在则报错（此方法应该创建新分支）
        let (exists_local, exists_remote) =
            GitBranch::is_branch_exists(branch_name).context("Failed to check if branch exists")?;

        if exists_local || exists_remote {
            anyhow::bail!(
                "Branch '{}' already exists. Cannot create new branch with existing name.",
                branch_name
            );
        }

        // 创建新分支
        log_success!("Creating branch: {}", branch_name);
        GitBranch::checkout_branch(branch_name)?;

        // 恢复 stash
        log_info!("Restoring stashed changes...");
        handle_stash_pop_result(GitStash::stash_pop(None));

        // 提交并推送
        Spinner::with("Committing changes...", || {
            GitCommit::commit(commit_title, true) // no-verify
        })?;
        log_break!();
        log_info!("Pushing to remote...");
        log_break!();
        GitBranch::push(branch_name, true)?; // set-upstream

        Ok((branch_name.to_string(), default_branch.to_string()))
    }

    /// 处理已推送到远程的分支
    ///
    /// 步骤 9 的辅助方法：当用户不在默认分支上且无未提交修改，但分支已推送到远程时，
    /// 询问用户是否在当前分支创建 PR。
    ///
    /// # 流程
    /// 1. 询问用户是否在当前分支创建 PR
    /// 2. 如果用户同意，返回当前分支名和默认分支名
    fn handle_existing_remote_branch(
        current_branch: &str,
        default_branch: &str,
    ) -> Result<(String, String)> {
        log_info!("Branch '{}' already exists on remote.", current_branch);
        ConfirmDialog::new(format!(
            "Create PR for current branch '{}'?",
            current_branch
        ))
        .with_default(true)
        .with_cancel_message("Operation cancelled.")
        .prompt()?;

        Ok((current_branch.to_string(), default_branch.to_string()))
    }

    /// 处理未推送的分支
    ///
    /// 步骤 9 的辅助方法：当用户不在默认分支上且无未提交修改，但分支未推送到远程时，
    /// 询问用户是否推送并创建 PR，然后推送分支。
    ///
    /// # 流程
    /// 1. 询问用户是否推送并创建 PR
    /// 2. 如果用户同意，推送分支到远程
    /// 3. 返回当前分支名和默认分支名
    fn handle_unpushed_branch(
        current_branch: &str,
        default_branch: &str,
    ) -> Result<(String, String)> {
        log_info!(
            "Branch '{}' has commits but not pushed to remote.",
            current_branch
        );
        ConfirmDialog::new(format!(
            "Push and create PR for current branch '{}'?",
            current_branch
        ))
        .with_default(true)
        .with_cancel_message("Operation cancelled.")
        .prompt()?;

        // 推送
        log_break!();
        log_info!("Pushing to remote...");
        log_break!();
        GitBranch::push(current_branch, true)?; // set-upstream

        Ok((current_branch.to_string(), default_branch.to_string()))
    }

    /// 创建或更新分支
    ///
    /// 步骤 9：根据当前 Git 状态决定分支策略，创建或更新分支：
    /// 1. 检查是否有未提交的修改
    /// 2. 获取当前分支和默认分支
    /// 3. 根据分支状态和修改情况决定处理策略：
    ///    - 在默认分支上：
    ///      - 有未提交修改 → 创建新分支并提交
    ///      - 无未提交修改 → 报错（无法创建 PR）
    ///    - 不在默认分支上：
    ///      - 有未提交修改 → 询问用户是否在当前分支创建 PR，或创建新分支
    ///      - 无未提交修改 → 检查分支是否已推送、是否有提交，然后处理
    /// 4. 执行相应的分支操作（创建分支、提交更改、推送到远程）
    ///
    /// 返回实际使用的分支名和默认分支名。
    fn create_or_update_branch(branch_name: &str, commit_title: &str) -> Result<(String, String)> {
        // 1. 检查是否有未提交的修改
        let has_uncommitted =
            GitCommit::has_commit().context("Failed to check uncommitted changes")?;

        // 2. 获取当前分支和默认分支
        let current_branch = GitBranch::current_branch()?;
        let default_branch = GitBranch::get_default_branch()?;
        let is_default_branch = current_branch == default_branch;

        // 3. 判断当前是否是默认分支
        if is_default_branch {
            // 在默认分支上
            if has_uncommitted {
                // 有未提交修改 → 创建新分支并提交
                create_branch_from_default(branch_name, commit_title, &default_branch)
            } else {
                // 无未提交修改 → 提示：默认分支上没有更改，无法创建 PR
                anyhow::bail!(
                    "No changes on default branch '{}'. Cannot create PR without changes.",
                    default_branch
                );
            }
        } else {
            // 不在默认分支上
            if has_uncommitted {
                // 有未提交的代码 → 询问用户是否需要在当前分支创建 PR
                log_info!(
                    "You are on branch '{}' with uncommitted changes.",
                    current_branch
                );
                let should_use_current = ConfirmDialog::new(format!(
                    "Create PR for current branch '{}'? (otherwise will create new branch '{}')",
                    current_branch, branch_name
                ))
                .with_default(true)
                .prompt()?;

                if should_use_current {
                    // 用户期望在当前分支提交并创建 PR
                    Self::commit_and_push_current_branch(
                        &current_branch,
                        commit_title,
                        &default_branch,
                    )
                } else {
                    // 用户不期望在当前分支提交并创建 → 使用 stash 创建新分支
                    Self::create_new_branch_with_stash(
                        &current_branch,
                        branch_name,
                        commit_title,
                        &default_branch,
                    )
                }
            } else {
                // 无未提交的代码 → 判断当前分支是否在远程分支上
                let exists_remote = GitBranch::has_remote_branch(&current_branch)
                    .context("Failed to check if branch exists on remote")?;

                // 检查当前分支是否有提交（相对于默认分支）
                let has_commits = GitBranch::is_branch_ahead(&current_branch, &default_branch)
                    .context("Failed to check if current branch has commits")?;

                if !has_commits {
                    anyhow::bail!("Current branch '{}' has no commits compared to '{}'. Cannot create PR without commits.", current_branch, default_branch);
                }

                if exists_remote {
                    // 当前已经推送到远程 → 询问是否在当前分支创建 PR
                    Self::handle_existing_remote_branch(&current_branch, &default_branch)
                } else {
                    // 当前没有推送到远程 → 询问用户是否需要在当前分支创建 PR
                    Self::handle_unpushed_branch(&current_branch, &default_branch)
                }
            }
        }
    }
}
