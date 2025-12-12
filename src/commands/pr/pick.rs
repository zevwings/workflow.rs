use color_eyre::{eyre::WrapErr, Result};

use crate::base::dialog::{ConfirmDialog, InputDialog};
use crate::base::indicator::Spinner;
use crate::branch::{BranchNaming, BranchType};
use crate::commands::check;
use crate::commands::pr::helpers::{
    copy_and_open_pull_request, create_branch_from_default, create_or_get_pull_request,
    detect_base_branch, ensure_jira_status, handle_stash_pop_result, resolve_description,
    resolve_title, select_change_types, update_jira_ticket,
};
use crate::git::{GitBranch, GitCherryPick, GitCommit, GitRepo, GitStash};
use crate::jira::helpers::validate_jira_ticket_format;
use crate::jira::Jira;
use crate::pr::body_parser::{extract_info_from_source_pr, ExtractedPrInfo, SourcePrInfo};
use crate::pr::create_provider_auto;
use crate::pr::helpers::{
    generate_commit_title, generate_pull_request_body, get_current_branch_pr_id,
};
use crate::pr::llm::CreateGenerator;
use crate::pr::TYPES_OF_CHANGES;
use crate::{log_break, log_error, log_info, log_success, log_warning};

/// PR Pick 命令
///
/// 从源分支 cherry-pick 提交到目标分支，然后交互式创建新 PR。
///
/// # 完整执行流程
///
/// ```text
/// 1. 预检查
/// 2. 验证分支存在
/// 3. 拉取最新代码
/// 4. 检测新提交
/// 5. 保存当前状态
/// 6. 检查工作区状态
/// 7. 切换到 TO_BRANCH
/// 8. Cherry-pick (--no-commit) 所有提交
/// 9. 获取源 PR 信息
/// 10. 询问是否创建 PR
/// 11. 交互式 PR 创建流程（复用 create 的逻辑）
/// 12. 恢复原分支和 stash
/// ```
pub struct PullRequestPickCommand;

impl PullRequestPickCommand {
    /// 执行 pick 操作
    ///
    /// # 参数
    ///
    /// * `from_branch` - 源分支名称
    /// * `to_branch` - 目标分支名称
    /// * `dry_run` - 预览模式
    pub fn pick(from_branch: String, to_branch: String, dry_run: bool) -> Result<()> {
        // 0. 检查并确保仓库配置存在
        crate::commands::repo::setup::RepoSetupCommand::ensure()?;

        // 1. 运行预检查
        if !dry_run {
            log_info!("Running pre-flight checks...");
            if let Err(e) = check::CheckCommand::run_all() {
                log_warning!("Pre-flight checks failed: {}", e);
                ConfirmDialog::new("Continue anyway?")
                    .with_default(false)
                    .with_cancel_message("Operation cancelled by user")
                    .prompt()?;
            }
        }

        // 2. 验证分支存在
        Self::validate_branches(&from_branch, &to_branch)?;

        // 3. 拉取最新代码
        Spinner::with("Fetching latest changes...", GitRepo::fetch)?;

        // 4. 检测新提交
        let commits = Self::get_new_commits(&from_branch, &to_branch)?;
        if commits.is_empty() {
            log_info!(
                "No new commits to cherry-pick from '{}' to '{}'",
                from_branch,
                to_branch
            );
            return Ok(());
        }

        log_success!("Found {} commit(s) to cherry-pick", commits.len());

        // 5. 保存当前分支
        let current_branch = GitBranch::current_branch()?;

        // 6. 检查工作区状态
        let has_stashed = Self::check_working_directory()?;

        // 7. 切换到 TO_BRANCH
        log_info!("Switching to target branch '{}'...", to_branch);
        GitBranch::checkout_branch(&to_branch)?;
        log_success!("Switched to branch: {}", to_branch);

        // 8. Cherry-pick (--no-commit) 所有提交
        log_info!(
            "Cherry-picking {} commit(s) (without committing)...",
            commits.len()
        );
        let cherry_pick_result = Self::cherry_pick_commits_no_commit(&commits);

        // 9. 处理 cherry-pick 结果
        match cherry_pick_result {
            Ok(()) => {
                log_success!(
                    "Cherry-picked {} commit(s) to working directory",
                    commits.len()
                );
            }
            Err(e) => {
                // 检查是否是冲突
                if Self::is_cherry_pick_conflict(&e) {
                    Self::handle_cherry_pick_conflict(
                        &from_branch,
                        &to_branch,
                        &current_branch,
                        has_stashed,
                    )?;
                    return Ok(());
                }
                // 其他错误，恢复原分支
                GitBranch::checkout_branch(&current_branch)?;
                if has_stashed {
                    handle_stash_pop_result(GitStash::stash_pop(None));
                }
                return Err(e);
            }
        }

        // 10. 预览模式
        if dry_run {
            Self::dry_run(&from_branch, &to_branch, &commits)?;
            // 恢复原分支
            GitBranch::checkout_branch(&current_branch)?;
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop(None));
            }
            return Ok(());
        }

        // 11. 获取源 PR 信息（如果存在）
        let source_pr_info = Self::get_source_pr_info(&from_branch)?;

        // 12. 询问是否创建 PR
        let should_create_pr = ConfirmDialog::new("Create PR for cherry-picked commits?")
            .with_default(true)
            .prompt()?;

        if !should_create_pr {
            // 用户选择不创建 PR
            let keep_changes =
                ConfirmDialog::new("Keep cherry-picked changes in working directory?")
                    .with_default(false)
                    .prompt()?;

            if !keep_changes {
                // 清理工作区并恢复原分支
                log_info!("Cleaning up working directory...");
                GitCommit::reset_hard(None)?;
            }
            // 恢复原分支
            GitBranch::checkout_branch(&current_branch)?;
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop(None));
            }
            log_info!("Pick operation completed (PR not created)");
            return Ok(());
        }

        // 13. 交互式 PR 创建流程（复用 create 的逻辑）
        // 如果失败，需要恢复原分支和 stash
        let pr_result = Self::create_pr_interactively(&from_branch, &source_pr_info);

        // 14. 恢复原分支和 stash（无论 PR 创建是否成功）
        let restore_branch_result = GitBranch::checkout_branch(&current_branch);
        if let Err(e) = &restore_branch_result {
            log_error!("Failed to restore original branch: {}", e);
            log_error!(
                "You may need to manually checkout: git checkout {}",
                current_branch
            );
        }

        if has_stashed {
            log_info!("Restoring stashed changes...");
            match GitStash::stash_pop(None) {
                Ok(result) => {
                    if result.restored {
                        if let Some(ref msg) = result.message {
                            log_success!("{}", msg);
                        }
                    }
                    // 显示警告信息
                    for warning in &result.warnings {
                        log_warning!("{}", warning);
                    }
                }
                Err(e) => {
                    log_warning!("Failed to restore stashed changes: {}", e);
                    log_info!("You may need to manually restore: git stash pop");
                }
            }
        }

        // 如果分支恢复失败，返回错误
        restore_branch_result?;

        // 如果 PR 创建失败，返回错误
        pr_result?;

        log_break!();
        log_success!("Pick operation completed successfully!");
        Ok(())
    }

    /// 验证分支存在
    fn validate_branches(from_branch: &str, to_branch: &str) -> Result<()> {
        let (from_local, from_remote) = GitBranch::is_branch_exists(from_branch)?;
        if !from_local && !from_remote {
            color_eyre::eyre::bail!(
                "Source branch '{}' does not exist. Please check the branch name.",
                from_branch
            );
        }

        let (to_local, to_remote) = GitBranch::is_branch_exists(to_branch)?;
        if !to_local && !to_remote {
            color_eyre::eyre::bail!(
                "Target branch '{}' does not exist. Please check the branch name.",
                to_branch
            );
        }

        Ok(())
    }

    /// 获取新提交列表（支持智能检测基础分支）
    ///
    /// 首先尝试检测 from_branch 可能基于哪个分支创建。
    /// 如果检测到基础分支，自动使用该基础分支作为比较基准。
    /// 这样只会获取 from_branch 相对于基础分支的提交（不包含基础分支的修改）。
    ///
    /// # 参数
    ///
    /// * `from_branch` - 源分支名称
    /// * `to_branch` - 目标分支名称
    ///
    /// # 返回
    ///
    /// 返回要 cherry-pick 的提交列表。
    fn get_new_commits(from_branch: &str, to_branch: &str) -> Result<Vec<String>> {
        // 1. 检测 from_branch 可能基于哪个分支创建
        let detected_base = detect_base_branch(from_branch, to_branch)?;

        // 2. 如果检测到基础分支，自动使用该基础分支
        let actual_base = if let Some(base_branch) = detected_base {
            log_success!(
                "Detected that '{}' is based on '{}', using it as base branch",
                from_branch,
                base_branch
            );
            log_info!(
                "This will only pick commits unique to '{}' (excluding '{}' changes)",
                from_branch,
                base_branch
            );
            base_branch
        } else {
            // 没有检测到基础分支，使用原始逻辑
            to_branch.to_string()
        };

        // 3. 获取提交列表
        GitBranch::get_commits_between(&actual_base, from_branch).wrap_err_with(|| {
            format!(
                "Failed to get commits between '{}' and '{}'",
                actual_base, from_branch
            )
        })
    }

    /// 检查工作区状态
    fn check_working_directory() -> Result<bool> {
        let has_changes = !GitCommit::status()?.trim().is_empty();
        if has_changes {
            log_warning!("Working directory has uncommitted changes");
            if ConfirmDialog::new("Stash changes?").with_default(true).prompt()? {
                GitStash::stash_push(Some("Stashed by workflow pr pick"))?;
                log_success!("Stashed changes");
                Ok(true)
            } else {
                color_eyre::eyre::bail!(
                    "Operation cancelled: working directory has uncommitted changes"
                );
            }
        } else {
            Ok(false)
        }
    }

    /// Cherry-pick 提交列表（不提交）
    fn cherry_pick_commits_no_commit(commits: &[String]) -> Result<()> {
        for (index, commit) in commits.iter().enumerate() {
            log_info!(
                "Cherry-picking commit {}/{}: {}",
                index + 1,
                commits.len(),
                &commit[..8]
            );
            GitCherryPick::cherry_pick_no_commit(commit)?;
        }
        Ok(())
    }

    /// 检查是否是 cherry-pick 冲突
    fn is_cherry_pick_conflict(error: &color_eyre::eyre::Report) -> bool {
        // 首先检查是否有 CHERRY_PICK_HEAD 文件（最可靠的方法）
        if GitCherryPick::is_cherry_pick_in_progress() {
            return true;
        }
        // 回退到错误消息检查
        let error_msg = error.to_string().to_lowercase();
        error_msg.contains("conflict") || error_msg.contains("could not apply")
    }

    /// 处理 cherry-pick 冲突
    ///
    /// 当 cherry-pick 遇到冲突时，提供详细的用户指引和选择：
    /// - 保持在 to_branch 上，让用户解决冲突
    /// - 询问用户是否要放弃并恢复原分支
    ///
    /// # 参数
    ///
    /// * `from_branch` - 源分支名称
    /// * `to_branch` - 目标分支名称（当前所在分支）
    /// * `current_branch` - 原始分支名称（需要恢复的分支）
    /// * `has_stashed` - 是否有 stash 需要恢复
    ///
    /// # 错误
    ///
    /// 如果用户选择放弃但恢复操作失败，返回相应的错误信息。
    fn handle_cherry_pick_conflict(
        from_branch: &str,
        to_branch: &str,
        current_branch: &str,
        has_stashed: bool,
    ) -> Result<()> {
        // 冲突时，保持在 to_branch 上，让用户解决冲突
        // 不能立即切换分支，否则会丢失 cherry-pick 的内容
        log_error!("Cherry-pick conflict detected!");
        log_info!(
            "You are currently on branch '{}' with cherry-pick conflicts.",
            to_branch
        );
        log_info!("Please resolve conflicts manually:");
        log_info!("  1. Edit conflicted files");
        log_info!("  2. git add <resolved-files>");
        log_info!("  3. git cherry-pick --continue");
        log_info!("\nTo abort and return to original branch:");
        log_info!("  git cherry-pick --abort");
        log_info!("  git checkout {}", current_branch);
        log_info!("\nAfter resolving conflicts, you can:");
        log_info!(
            "  - Continue with: workflow pr pick {} {} (will detect existing changes)",
            from_branch,
            to_branch
        );
        log_info!("  - Or manually create PR");

        // 询问用户是否要放弃并恢复
        let should_abort = ConfirmDialog::new("Abort cherry-pick and return to original branch?")
            .with_default(false)
            .prompt()?;

        if should_abort {
            // 用户选择放弃，先清理 cherry-pick 状态
            log_info!("Aborting cherry-pick...");
            if let Err(abort_err) = GitCherryPick::cherry_pick_abort() {
                log_warning!("Failed to abort cherry-pick: {}", abort_err);
                log_info!("You may need to manually run: git cherry-pick --abort");
            }
            // 恢复原分支
            GitBranch::checkout_branch(current_branch)?;
            if has_stashed {
                handle_stash_pop_result(GitStash::stash_pop(None));
            }
        } else {
            // 用户选择保留冲突状态，保持在 to_branch 上
            log_info!("Staying on branch '{}' for conflict resolution.", to_branch);
            log_info!("After resolving conflicts, you can continue with the PR creation.");

            // 询问用户是否要恢复 stash（在解决冲突时可能需要）
            if has_stashed {
                let restore_stash = ConfirmDialog::new(
                    "Restore stashed changes now? (You can restore later with 'git stash pop')",
                )
                .with_default(false)
                .prompt()?;
                if restore_stash {
                    handle_stash_pop_result(GitStash::stash_pop(None));
                }
            }
        }
        Ok(())
    }

    /// 获取源 PR 信息
    ///
    /// 临时切换到源分支获取 PR 信息，然后恢复原分支。
    /// 如果当前有未提交的更改（cherry-pick 的内容），会先 stash，切换回来后再恢复。
    /// 如果任何步骤失败，会尝试恢复原分支状态。
    fn get_source_pr_info(from_branch: &str) -> Result<Option<SourcePrInfo>> {
        // 临时切换到源分支获取 PR 信息
        let current_branch = GitBranch::current_branch()?;

        // 检查是否有未提交的更改（cherry-pick 的内容）
        let has_uncommitted = GitCommit::has_commit()
            .wrap_err("Failed to check uncommitted changes before switching branch")?;
        let needs_stash = has_uncommitted;

        // 如果有未提交的更改，先 stash
        if needs_stash {
            log_info!("Stashing cherry-picked changes before switching to source branch...");
            GitStash::stash_push(Some("Auto-stash before getting source PR info"))?;
        }

        // 切换到源分支，确保成功
        if let Err(e) = GitBranch::checkout_branch(from_branch)
            .wrap_err_with(|| format!("Failed to checkout source branch: {}", from_branch))
        {
            // 如果切换失败，尝试恢复 stash
            if needs_stash {
                handle_stash_pop_result(GitStash::stash_pop(None));
            }
            return Err(e);
        }

        // 获取 PR 信息
        let pr_id = match get_current_branch_pr_id() {
            Ok(id) => id,
            Err(e) => {
                // 获取 PR 信息失败，尝试恢复分支和 stash
                if let Err(restore_err) = GitBranch::checkout_branch(&current_branch) {
                    log_error!("Failed to restore branch after error: {}", restore_err);
                    log_error!(
                        "You may need to manually checkout: git checkout {}",
                        current_branch
                    );
                    // 如果之前有 stash，尝试恢复
                    if needs_stash {
                        handle_stash_pop_result(GitStash::stash_pop(None));
                    }
                    return Err(e)
                        .wrap_err("Failed to get PR ID from source branch")
                        .wrap_err(format!("Also failed to restore branch: {}", restore_err));
                }
                // 恢复 stash
                if needs_stash {
                    handle_stash_pop_result(GitStash::stash_pop(None));
                }
                return Err(e).wrap_err("Failed to get PR ID from source branch");
            }
        };

        // 恢复原分支（应该是 to_branch），确保成功
        GitBranch::checkout_branch(&current_branch)
            .wrap_err_with(|| format!("Failed to restore branch: {}", current_branch))?;

        // 恢复 stash（如果有）
        if needs_stash {
            log_info!("Restoring stashed cherry-picked changes...");
            match GitStash::stash_pop(None) {
                Ok(result) => {
                    if result.restored {
                        if let Some(ref msg) = result.message {
                            log_success!("{}", msg);
                        }
                    }
                    // 显示警告信息
                    for warning in &result.warnings {
                        log_warning!("{}", warning);
                    }
                    // 如果有警告，说明恢复可能有问题
                    if !result.restored && !result.warnings.is_empty() {
                        log_error!(
                            "Cherry-picked changes may have been lost. Please check: git stash list"
                        );
                    }
                }
                Err(e) => {
                    log_warning!("Failed to restore stash: {}", e);
                    log_error!(
                        "Cherry-picked changes may have been lost. Please check: git stash list"
                    );
                    color_eyre::eyre::bail!(
                        "Failed to restore stashed changes after getting PR info: {}",
                        e
                    );
                }
            }
        }

        if let Some(pr_id) = pr_id {
            let provider = create_provider_auto()?;
            let title = provider.get_pull_request_title(&pr_id).ok();
            let url = provider.get_pull_request_url(&pr_id).ok();
            let body = provider.get_pull_request_body(&pr_id).ok().flatten();
            Ok(Some(SourcePrInfo { title, url, body }))
        } else {
            Ok(None)
        }
    }

    /// 确定 LLM 的输入
    ///
    /// 如果能提取到 ticket，使用 ticket 从 Jira 获取 title（作为 LLM 的输入）
    /// 如果不能，使用源分支的标题（作为 LLM 的输入）
    fn determine_llm_input(
        extracted_info: &ExtractedPrInfo,
        source_pr_info: &Option<SourcePrInfo>,
    ) -> Result<String> {
        if let Some(ref ticket) = extracted_info.jira_ticket {
            // 如果能提取到 ticket，从 Jira 获取 title
            match Jira::get_ticket_info(ticket) {
                Ok(issue) => {
                    let jira_title = issue.fields.summary.trim().to_string();
                    if !jira_title.is_empty() {
                        log_success!("Using Jira ticket title for LLM: {}", jira_title);
                        return Ok(jira_title);
                    } else {
                        log_warning!("Jira ticket title is empty, falling back to source PR title");
                    }
                }
                Err(e) => {
                    log_warning!(
                        "Failed to get Jira ticket title: {}, falling back to source PR title",
                        e
                    );
                }
            }
        }

        // 不能提取到 ticket 或获取 Jira title 失败，使用源 PR 标题
        if let Some(info) = source_pr_info {
            if let Some(ref title) = info.title {
                log_success!("Using source PR title for LLM: {}", title);
                return Ok(title.clone());
            }
        }

        // 如果都没有，返回空字符串（LLM 会处理）
        log_warning!("No title found for LLM input, using empty string");
        Ok(String::new())
    }

    /// 交互式创建 PR（复用 create 的逻辑）
    ///
    /// 注意：此函数如果失败，调用者需要负责恢复原分支和 stash。
    /// 这是为了确保在错误发生时能够正确清理状态。
    fn create_pr_interactively(
        from_branch: &str,
        source_pr_info: &Option<SourcePrInfo>,
    ) -> Result<()> {
        // 复用 create 的交互式流程
        // 由于 create 的函数是私有的，我们需要重新实现或提取公共函数
        // 这里先实现核心逻辑

        // 1. 从源 PR 提取信息（Jira ticket、描述、变更类型）
        let extracted_info = extract_info_from_source_pr(source_pr_info);

        // 2. 确定分支类型（优先使用 repository prefix）
        // 需要在生成分支名之前确定类型，以便使用模板系统
        let branch_type = Self::resolve_branch_type()?;

        // 3. 确定 LLM 的输入（优先使用 Jira ticket 获取的 title，否则使用源 PR 标题）
        let llm_input = Self::determine_llm_input(&extracted_info, source_pr_info)?;

        // 4. 调用 LLM 生成分支名和 PR 标题（忽略 description）
        let (commit_title, branch_name, llm_generated_title) =
            Self::generate_commit_title_and_branch_name_for_pick(
                &extracted_info.jira_ticket,
                &llm_input,
                branch_type,
            )?;

        // 5. 确定最终的 ticket（如果能提取到，不需要输入；否则询问是否需要输入）
        let jira_ticket = if let Some(ref ticket) = extracted_info.jira_ticket {
            log_success!("Extracted Jira ticket from source PR: {}", ticket);
            let use_extracted =
                ConfirmDialog::new(format!("Use extracted Jira ticket: '{}'?", ticket))
                    .with_default(true)
                    .prompt()?;
            if use_extracted {
                Some(ticket.clone())
            } else {
                // 用户选择不使用，询问是否需要输入
                Self::resolve_jira_ticket(None)?
            }
        } else {
            // 不能提取到，询问是否需要输入
            Self::resolve_jira_ticket(None)?
        };

        // 6. 如果有 Jira ticket，检查并配置状态
        let created_pull_request_status = ensure_jira_status(&jira_ticket)?;

        // 7. 确定最终的 title（使用 LLM 生成的，询问用户是否使用）
        let title = {
            let use_llm_title = ConfirmDialog::new(format!(
                "Use LLM generated title: '{}'?",
                llm_generated_title
            ))
            .with_default(true)
            .prompt()?;
            if use_llm_title {
                llm_generated_title
            } else {
                // 用户选择不使用，使用源 PR 标题或让用户输入
                resolve_title(
                    source_pr_info.as_ref().and_then(|info| info.title.clone()),
                    &jira_ticket,
                    true, // confirm_if_provided = true for pick
                )?
            }
        };

        // 8. 获取描述（优先使用源 PR body 提取的，不使用 LLM 生成的）
        let short_description = if let Some(desc) = &extracted_info.description {
            log_success!("Extracted description from source PR body");
            // 询问用户是否使用提取的描述
            let use_extracted =
                ConfirmDialog::new(format!("Use description from source PR?\n{}", desc))
                    .with_default(true)
                    .prompt()?;
            if use_extracted {
                desc.clone()
            } else {
                // 用户选择不使用，让用户输入
                resolve_description(Some(desc.clone()))?
            }
        } else {
            // 没有提取到描述，让用户输入
            resolve_description(None)?
        };

        // 9. 选择变更类型（优先使用源 PR 的变更类型）
        let selected_types = if let Some(types) = &extracted_info.change_types {
            log_success!("Extracted change types from source PR");
            // 显示提取的变更类型，询问用户是否使用
            let mut types_str = String::new();
            for (i, change_type) in TYPES_OF_CHANGES.iter().enumerate() {
                if i < types.len() && types[i] {
                    types_str.push_str(&format!("  - [x] {}\n", change_type));
                } else {
                    types_str.push_str(&format!("  - [ ] {}\n", change_type));
                }
            }
            let use_extracted =
                ConfirmDialog::new(format!("Use change types from source PR?\n{}", types_str))
                    .with_default(true)
                    .prompt()?;
            if use_extracted {
                types.clone()
            } else {
                // 用户选择不使用，重新选择
                select_change_types()?
            }
        } else {
            select_change_types()?
        };

        // 10. 生成 PR body（添加 pick 说明）
        let mut pick_note = format!("\n#### Picked from\n\nBranch: `{}`\n", from_branch);
        if let Some(info) = source_pr_info {
            if let Some(ref url) = info.url {
                pick_note.push_str(&format!("Source PR: {}\n", url));
            }
        }

        // Get JIRA issue info (if exists) for template
        let jira_info = if let Some(ref ticket) = jira_ticket {
            Jira::get_ticket_info(ticket).ok()
        } else {
            None
        };

        let pull_request_body = generate_pull_request_body(
            &selected_types,
            Some(&short_description),
            jira_ticket.as_deref(),
            Some(&pick_note),
            jira_info.as_ref(),
        )?;

        // 11. 创建或更新分支
        // 如果失败，需要清理已创建的分支（如果有）
        let (actual_branch_name, default_branch) =
            Self::create_or_update_branch(&branch_name, &commit_title)
                .wrap_err_with(|| "Failed to create or update branch for PR")?;

        // 12. 创建或获取 PR
        let pull_request_url = create_or_get_pull_request(
            &actual_branch_name,
            &default_branch,
            &title,
            &pull_request_body,
        )
        .wrap_err_with(|| format!("Failed to create PR for branch: {}", actual_branch_name))?;

        // 13. 更新 Jira（如果有 ticket）
        // 即使失败也不影响 PR 创建，只记录警告
        if let Err(e) = update_jira_ticket(
            &jira_ticket,
            &created_pull_request_status,
            &pull_request_url,
            &actual_branch_name,
        ) {
            log_warning!("Failed to update Jira ticket: {}", e);
            log_info!("PR was created successfully, but Jira update failed");
        }

        // 14. 复制 PR URL 到剪贴板并打开浏览器
        // 即使失败也不影响 PR 创建，只记录警告
        if let Err(e) = copy_and_open_pull_request(&pull_request_url) {
            log_warning!("Failed to copy PR URL or open browser: {}", e);
            log_info!("PR URL: {}", pull_request_url);
        }

        log_success!("PR created successfully!");
        Ok(())
    }

    /// 确定分支类型（优先使用 repository prefix）
    ///
    /// 复用 branch create 和 pr create 的逻辑，优先使用 repository prefix 作为分支类型。
    fn resolve_branch_type() -> Result<BranchType> {
        BranchType::resolve_with_repo_prefix()
    }

    /// 获取或输入 Jira ticket（复用 create 的逻辑）
    ///
    /// 注意：在 pick 场景下，此函数只处理输入逻辑，确认逻辑已在调用处处理
    fn resolve_jira_ticket(jira_ticket: Option<String>) -> Result<Option<String>> {
        // 在 pick 场景下，此函数只会在确认不使用提取的 ticket 后调用，传入 None
        // 因此这里只需要处理输入逻辑
        let input = InputDialog::new("Jira ticket (optional)")
            .with_default(jira_ticket.as_deref().unwrap_or("").to_string())
            .allow_empty(true)
            .prompt()
            .wrap_err("Failed to get Jira ticket")?;

        let trimmed = input.trim().to_string();
        let ticket = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };

        if let Some(ref ticket) = ticket {
            validate_jira_ticket_format(ticket)?;
        }

        Ok(ticket)
    }

    /// 生成 commit title 和分支名（专门用于 pick，忽略 LLM 生成的 description）
    ///
    /// 使用与 branch create 和 pr create 相同的流程生成分支名：
    /// 1. 生成分支名 slug（使用 LLM 或回退方法）
    /// 2. 使用模板系统根据分支类型和 slug 生成分支名
    /// 3. 生成 commit title
    ///
    /// 返回 (commit_title, branch_name, llm_generated_title)
    fn generate_commit_title_and_branch_name_for_pick(
        jira_ticket: &Option<String>,
        llm_input: &str,
        branch_type: BranchType,
    ) -> Result<(String, String, String)> {
        let exists_branches = GitBranch::get_all_branches(true).ok();
        let git_diff = GitCommit::get_diff();

        // Step 1: 生成分支名 slug（使用 LLM 或回退方法）
        let (pr_title, branch_name_slug) =
            match CreateGenerator::generate(llm_input, exists_branches, git_diff) {
                Ok(content) => {
                    log_success!("Generated branch name using LLM: {}", content.branch_name);
                    // 提取 slug（移除可能的前缀）
                    let slug = BranchNaming::sanitize(&content.branch_name);
                    (content.pr_title, slug)
                }
                Err(e) => {
                    log_warning!(
                        "Failed to generate branch name using LLM: {}, using sanitized input",
                        e
                    );
                    // 回退：翻译并清理输入
                    let slug = BranchNaming::sanitize_and_translate_branch_name(llm_input)?;
                    (llm_input.to_string(), slug)
                }
            };

        // Step 2: 使用模板系统根据分支类型和 slug 生成分支名
        let branch_name = BranchNaming::from_type_and_slug(
            branch_type.as_str(),
            &branch_name_slug,
            jira_ticket.as_deref(),
        )?;

        // Step 3: 生成 commit title（使用模板系统）
        // 从分支类型映射到 Conventional Commits 提交类型
        let commit_type = branch_type.to_commit_type();

        let commit_title = generate_commit_title(
            jira_ticket.as_deref(),
            &pr_title,
            Some(commit_type), // commit_type - 从分支类型映射
            None,              // scope
            None,              // body
        )
        .unwrap_or_else(|_| {
            // Fallback to simple format if template fails
            match jira_ticket.as_deref() {
                Some(ticket) => format!("{}: {}", ticket, pr_title),
                None => format!("# {}", pr_title),
            }
        });

        Ok((commit_title, branch_name, pr_title))
    }

    /// 创建或更新分支（适配 pick 的特殊场景）
    fn create_or_update_branch(branch_name: &str, commit_title: &str) -> Result<(String, String)> {
        // 检查是否有未提交的修改（cherry-pick 的内容）
        let has_uncommitted =
            GitCommit::has_commit().wrap_err("Failed to check uncommitted changes")?;

        let current_branch = GitBranch::current_branch()?;
        let default_branch = GitBranch::get_default_branch()?;
        let is_default_branch = current_branch == default_branch;

        if is_default_branch {
            // 在默认分支上（TO_BRANCH 是默认分支）
            if has_uncommitted {
                // 有 cherry-pick 的修改 → 创建新分支并提交
                create_branch_from_default(branch_name, commit_title, &default_branch)
            } else {
                color_eyre::eyre::bail!(
                    "No changes on default branch '{}'. Cannot create PR without changes.",
                    default_branch
                );
            }
        } else {
            // 不在默认分支上（TO_BRANCH 不是默认分支）
            if has_uncommitted {
                // 有 cherry-pick 的修改 → 直接创建新分支
                // 因为我们已经切换到 TO_BRANCH 并 cherry-pick 了内容
                log_info!(
                    "You are on branch '{}' with cherry-picked changes.",
                    current_branch
                );
                log_info!(
                    "Will create new branch '{}' and commit changes...",
                    branch_name
                );

                // 检查目标分支是否存在
                let (exists_local, exists_remote) = GitBranch::is_branch_exists(branch_name)
                    .wrap_err("Failed to check if branch exists")?;

                if exists_local || exists_remote {
                    color_eyre::eyre::bail!(
                        "Branch '{}' already exists. Cannot create new branch with existing name.",
                        branch_name
                    );
                }

                // 创建新分支（Git 会自动把未提交的修改带到新分支）
                log_success!("Creating branch: {}", branch_name);
                GitBranch::checkout_branch(branch_name)
                    .wrap_err_with(|| {
                        format!(
                            "Failed to create branch '{}'. You are still on '{}' with uncommitted changes.",
                            branch_name, current_branch
                        )
                    })?;

                // 提交并推送
                Spinner::with("Committing changes...", || {
                    GitCommit::commit(commit_title, true) // no-verify
                })?;
                log_break!();
                log_info!("Pushing to remote...");
                log_break!();
                GitBranch::push(branch_name, true)?; // set-upstream

                Ok((branch_name.to_string(), default_branch))
            } else {
                color_eyre::eyre::bail!(
                    "No cherry-picked changes found. Cannot create PR without changes."
                );
            }
        }
    }

    /// 预览模式
    fn dry_run(from_branch: &str, to_branch: &str, commits: &[String]) -> Result<()> {
        log_info!("[DRY RUN] Would execute the following operations:");
        log_info!("  From branch: {}", from_branch);
        log_info!("  To branch: {}", to_branch);
        log_info!("  Commits to cherry-pick: {}", commits.len());
        for (index, commit) in commits.iter().enumerate() {
            log_info!("    {}. {}", index + 1, &commit[..8]);
        }
        log_info!("  Will switch to: {}", to_branch);
        log_info!("  Will cherry-pick (--no-commit) all commits");
        log_info!("  Will enter interactive PR creation flow");
        log_info!("  Will use LLM to generate branch name");
        Ok(())
    }
}

// SourcePrInfo 和 ExtractedPrInfo 已迁移到 lib/pr/body_parser.rs
