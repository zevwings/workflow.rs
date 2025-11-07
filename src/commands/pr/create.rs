use crate::commands::check::CheckCommand;
use crate::jira::status::JiraStatus;
use crate::{
    Browser, Clipboard, Codeup, Git, GitHub, Jira, PlatformProvider, PullRequestLLM, RepoType, Settings, TYPES_OF_CHANGES, extract_pull_request_id_from_url, generate_branch_name, generate_commit_title, generate_pull_request_body, log_error, log_info, log_success, log_warning, validate_jira_ticket_format
};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::io::{self, Write};

/// PR 创建命令
#[allow(dead_code)]
pub struct PullRequestCreateCommand;

/// Git 操作和 PR 创建的状态
struct GitOpsState {
    actual_branch_name: String,
    use_current_branch: bool,
    has_uncommitted: bool,
    use_stash: bool,
    is_default_branch: bool,
    committed_to_current_branch: bool,
    default_branch: String,
    existing_pr: Option<String>,
    skip_git_ops: bool,
}

impl PullRequestCreateCommand {
    /// 创建 PR（完整流程）
    #[allow(dead_code)]
    pub fn create(
        jira_ticket: Option<String>,
        title: Option<String>,
        description: Option<String>,
        dry_run: bool,
    ) -> Result<()> {
        // 1. 运行检查
        if !dry_run {
            CheckCommand::run_all()?;
        }

        // 2. 获取或输入 Jira ticket
        let jira_ticket = Self::get_or_input_jira_ticket(jira_ticket)?;

        // 3. 如果有 Jira ticket，检查并配置状态
        let created_pull_request_status = Self::configure_jira_status(&jira_ticket)?;

        // 4. 获取或生成 PR 标题
        let title = Self::get_or_generate_title(title, &jira_ticket)?;

        // 5. 生成 commit_title 和分支名
        let (commit_title, branch_name) = Self::generate_commit_title_and_branch_name(&jira_ticket, &title)?;

        // 6. 获取描述
        let description = Self::get_description(description)?;

        // 7. 选择变更类型
        let selected_types = Self::select_change_types()?;

        // 8. 生成 PR body
        let pull_request_body = generate_pull_request_body(
            &selected_types,
            Some(&description),
            jira_ticket.as_deref(),
            None, // dependency 暂时为空
        )?;

        if dry_run {
            log_info!("\n[DRY RUN] Would create branch: {}", branch_name);
            log_info!("[DRY RUN] Commit title: {}", commit_title);
            log_info!("[DRY RUN] PR body:\n{}", pull_request_body);
            return Ok(());
        }

        // 9. 检查状态并准备 Git 操作
        let git_ops_state = Self::prepare_git_operations(&branch_name, &commit_title)?;

        // 10. 执行 Git 操作（如果需要）
        Self::execute_git_operations(&git_ops_state, &commit_title)?;

        // 11. 创建 PR（根据仓库类型）
        let pull_request_url = Self::create_or_get_pull_request(
            &git_ops_state,
            &commit_title,
            &pull_request_body,
        )?;

        // 12. 更新 Jira（如果有 ticket）
        Self::update_jira_ticket(
            &jira_ticket,
            &created_pull_request_status,
            &pull_request_url,
            &description,
            &git_ops_state,
        )?;

        // 13. 复制 PR URL 到剪贴板并打开浏览器
        Self::copy_and_open_pr(&pull_request_url)?;

        log_success!("PR created successfully!");
        Ok(())
    }

    /// 获取或输入 Jira ticket
    ///
    /// 如果提供了 ticket，验证其格式；如果没有提供，提示用户输入并验证。
    fn get_or_input_jira_ticket(jira_ticket: Option<String>) -> Result<Option<String>> {
        if let Some(ticket) = jira_ticket {
            // 验证 ticket 格式
            if !ticket.trim().is_empty() {
                validate_jira_ticket_format(&ticket)?;
            }
            Ok(Some(ticket))
        } else {
            // 提示用户输入
            print!("Jira ticket (optional): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let ticket = input.trim().to_string();

            if ticket.is_empty() {
                Ok(None)
            } else {
                // 验证输入的 ticket 格式
                validate_jira_ticket_format(&ticket)?;
                Ok(Some(ticket))
            }
        }
    }

    /// 提示用户输入 PR 标题
    ///
    /// 使用 Input 组件提示用户输入 PR 标题，并验证输入不能为空。
    fn input_pr_title() -> Result<String> {
        let title: String = Input::new()
            .with_prompt("PR title (required)")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("PR title is required and cannot be empty")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .context("Failed to get PR title")?;
        Ok(title)
    }

    /// 获取或生成 PR 标题
    ///
    /// 如果提供了标题，直接使用；如果有 Jira ticket，尝试从 Jira 获取标题；
    /// 否则提示用户输入标题。
    fn get_or_generate_title(title: Option<String>, jira_ticket: &Option<String>) -> Result<String> {
        let title = if let Some(t) = title {
            t
        } else if let Some(ref ticket) = jira_ticket {
            // 如果有 Jira ticket，尝试从 Jira 获取标题
            log_success!("Getting PR title from Jira ticket...");

            match Jira::get_ticket_info(ticket) {
                Ok(issue) => {
                    let summary = issue.fields.summary.trim().to_string();
                    if summary.is_empty() {
                        log_warning!("Jira ticket summary is empty, falling back to manual input");
                        // 回退到手动输入
                        Self::input_pr_title()?
                    } else {
                        log_success!("Using Jira ticket summary: {}", summary);
                        summary
                    }
                }
                Err(e) => {
                    log_error!("Failed to get ticket info: {}", e);
                    log_warning!("Falling back to manual input");
                    // 回退到手动输入
                    Self::input_pr_title()?
                }
            }
        } else {
            Self::input_pr_title()?
        };

        Ok(title)
    }

    /// 获取 PR 描述
    ///
    /// 如果提供了描述，直接使用；否则提示用户输入描述（可选）。
    fn get_description(description: Option<String>) -> Result<String> {
        if let Some(desc) = description {
            Ok(desc)
        } else {
            let desc: String = Input::new()
                .with_prompt("Short description (optional)")
                .allow_empty(true)
                .interact_text()
                .context("Failed to get description")?;
            Ok(desc)
        }
    }

    /// 准备 Git 操作
    ///
    /// 检查 Git 状态、分支存在情况、PR 存在情况，并根据状态决定处理方式。
    /// 返回包含所有状态信息的 `GitOpsState` 结构体。
    fn prepare_git_operations(branch_name: &str, commit_title: &str) -> Result<GitOpsState> {
        // 检查状态：未提交修改、分支存在、PR 存在、当前分支是否是默认分支
        let has_uncommitted = Git::has_commit()
            .context("Failed to check uncommitted changes")?;
        let current_branch = Git::current_branch()
            .context("Failed to get current branch")?;
        let default_branch = Git::get_default_branch()
            .context("Failed to get default branch")?;
        let is_default_branch = current_branch == default_branch;

        // 检查当前分支是否有提交（相对于默认分支）
        // 如果用户在非默认分支上，且工作树干净，且当前分支有提交，询问是否在当前分支上创建 PR
        let (actual_branch_name, use_current_branch) = if !has_uncommitted && !is_default_branch {
            let current_is_branch_ahead = Git::is_branch_ahead(&current_branch, &default_branch)
                .context("Failed to check if current branch has commits")?;

            if current_is_branch_ahead {
                log_info!("\nYou are on branch '{}' with commits.", current_branch);
                let should_use_current = Confirm::new()
                    .with_prompt(format!("Create PR for current branch '{}'? (otherwise will create new branch '{}')", current_branch, branch_name))
                    .default(true)
                    .interact()
                    .context("Failed to get confirmation")?;

                if should_use_current {
                    // 使用当前分支名作为 PR 分支名
                    (current_branch.clone(), true)
                } else {
                    (branch_name.to_string(), false)
                }
            } else {
                (branch_name.to_string(), false)
            }
        } else {
            (branch_name.to_string(), false)
        };

        let (exists_local, exists_remote) = Git::is_branch_exists(&actual_branch_name)
            .context("Failed to check branch existence")?;
        let repo_type = Git::detect_repo_type()?;

        // 处理未提交修改的情况
        let mut use_stash = false; // 是否使用 stash 带到新分支
        let mut committed_to_current_branch = false; // 是否已经提交到当前分支
        if has_uncommitted {
            if !is_default_branch {
                // 在非默认分支上有未提交修改
                log_warning!(
                    "\n⚠️  You are on branch '{}' (not default branch '{}')",
                    current_branch,
                    default_branch
                );
                log_info!("You have uncommitted changes.");

                let options = vec![
                    "Commit changes to current branch first (recommended)",
                    "Bring changes to new branch (using stash)",
                    "Cancel operation",
                ];

                let selection = Select::new()
                    .with_prompt("How would you like to handle uncommitted changes?")
                    .items(&options)
                    .default(0)
                    .interact()
                    .context("Failed to get user selection")?;

                match selection {
                    0 => {
                        // 选项 A：提交到当前分支
                        log_info!("Will commit changes to current branch first...");
                        // 提示输入提交消息
                        let commit_msg: String = Input::new()
                            .with_prompt("Commit message for current branch")
                            .default(format!("WIP: {}", commit_title))
                            .interact_text()
                            .context("Failed to get commit message")?;

                        log_success!("Committing changes to current branch: {}", current_branch);

                        // 检查提交前是否有未提交的更改
                        let has_changes_before = Git::has_commit()
                            .unwrap_or(false);

                        Git::commit(&commit_msg, true)?; // no-verify

                        // 检查提交后是否还有未提交的更改
                        let has_changes_after = Git::has_commit()
                            .unwrap_or(false);

                        // 如果提交前有更改，但提交后仍然有更改，说明提交可能失败了
                        if has_changes_before && has_changes_after {
                            log_warning!("Commit completed but uncommitted changes still exist.");
                            log_warning!("This might indicate that changes could not be committed.");
                            log_warning!("Will use stash to bring changes to new branch instead.");
                            use_stash = true;
                            committed_to_current_branch = false;
                        } else {
                            log_success!("Pushing changes to current branch...");
                            Git::push(&current_branch, false)?; // 不使用 -u
                            committed_to_current_branch = true;
                        }
                        // 更新状态：现在没有未提交的修改了
                        // 注意：这里不直接修改 has_uncommitted，而是通过 committed_to_current_branch 标记
                    }
                    1 => {
                        // 选项 B：使用 stash 带到新分支
                        log_info!("Will stash changes and bring them to new branch...");
                        use_stash = true;
                    }
                    2 => {
                        // 选项 C：取消操作
                        anyhow::bail!("Operation cancelled.");
                    }
                    _ => {
                        anyhow::bail!("Invalid selection.");
                    }
                }
            } else {
                // 在默认分支上有未提交修改，直接带到新分支（不需要 stash，因为 Git 会自动带过去）
                // 但是为了确保更改被正确带到新分支，我们仍然使用 stash 来保证一致性
                log_info!("You have uncommitted changes on default branch '{}'.", current_branch);
                log_info!("Will bring changes to new branch...");
                use_stash = true;
            }
        }

        // 检查分支是否已有 PR（只在分支已存在时检查）
        // 注意：如果已经提交到当前分支或使用了 stash，这里可以安全地切换分支检查 PR
        let existing_pr = if exists_local || exists_remote {
            // 尝试切换到分支检查 PR（如果切换失败，假设没有 PR）
            let current_branch_for_check = Git::current_branch().ok();
            if let Ok(()) = Git::checkout_branch(&actual_branch_name) {
                // 成功切换到分支，检查 PR
                let pr = match repo_type {
                    RepoType::GitHub => {
                        <GitHub as PlatformProvider>::get_current_branch_pull_request()
                            .ok()
                            .flatten()
                    }
                    RepoType::Codeup => {
                        <Codeup as PlatformProvider>::get_current_branch_pull_request()
                            .ok()
                            .flatten()
                    }
                    RepoType::Unknown => None,
                };
                // 恢复原分支
                if let Some(ref orig) = current_branch_for_check {
                    let _ = Git::checkout_branch(orig);
                }
                pr
            } else {
                // 切换分支失败（可能有未提交修改），无法检查 PR
                // 恢复原分支
                if let Some(ref orig) = current_branch_for_check {
                    let _ = Git::checkout_branch(orig);
                }
                None
            }
        } else {
            None
        };

        // 根据状态组合决定处理方式
        let mut skip_git_ops = false; // 是否跳过 Git 操作（分支、提交、推送）

        if exists_remote && existing_pr.is_none() {
            // 场景：分支已推送但无 PR（可能是上次创建失败）
            log_warning!("\n⚠️  Branch '{}' exists on remote but no PR found.", actual_branch_name);
            log_info!("This might indicate a previous failed PR creation.");

            if has_uncommitted {
                log_info!("You have uncommitted changes.");
                let should_continue = Confirm::new()
                    .with_prompt("Continue with PR creation? (will commit and push changes)")
                    .default(true)
                    .interact()
                    .context("Failed to get confirmation")?;

                if !should_continue {
                    anyhow::bail!("PR creation cancelled.");
                }
                // 继续正常流程：切换到分支、提交、推送、创建 PR
            } else {
                // 无未提交修改，可以直接创建 PR
                log_info!("No uncommitted changes. Skipping Git operations...");
                let should_create_pr = Confirm::new()
                    .with_prompt("Create PR for existing branch?")
                    .default(true)
                    .interact()
                    .context("Failed to get confirmation")?;

                if !should_create_pr {
                    anyhow::bail!("PR creation cancelled.");
                }

                // 跳过 Git 操作，直接创建 PR
                skip_git_ops = true;
            }
        } else if exists_local || exists_remote {
            // 场景：分支已存在（但可能有 PR 也可能没有）
            if let Some(ref pr_id) = existing_pr {
                log_warning!("\n⚠️  Branch '{}' already exists and has PR #{}", actual_branch_name, pr_id);

                if has_uncommitted {
                    log_info!("You have uncommitted changes.");
                    let should_continue = Confirm::new()
                        .with_prompt("Continue to commit changes and update existing PR?")
                        .default(true)
                        .interact()
                        .context("Failed to get confirmation")?;

                    if !should_continue {
                        anyhow::bail!("PR creation cancelled.");
                    }
                    // 继续正常流程：切换到分支、提交、推送（PR 已存在，无需创建）
                } else {
                    log_info!("No uncommitted changes. PR #{} already exists.", pr_id);
                    // 无未提交修改，只需要获取 PR URL 即可
                    skip_git_ops = true;
                }
            } else {
                log_warning!("\n⚠️  Branch '{}' already exists.", actual_branch_name);
                if has_uncommitted {
                    log_info!("You have uncommitted changes.");
                    let should_continue = Confirm::new()
                        .with_prompt("Continue to commit changes and create PR?")
                        .default(true)
                        .interact()
                        .context("Failed to get confirmation")?;

                    if !should_continue {
                        anyhow::bail!("PR creation cancelled.");
                    }
                    // 继续正常流程
                } else {
                    log_info!("No uncommitted changes.");
                    let should_continue = Confirm::new()
                        .with_prompt("Continue to create PR?")
                        .default(true)
                        .interact()
                        .context("Failed to get confirmation")?;

                    if !should_continue {
                        anyhow::bail!("PR creation cancelled.");
                    }
                    // 继续正常流程（虽然可能不需要提交，但会检查）
                }
            }
        } else if has_uncommitted {
            // 场景：分支不存在，但有未提交修改（正常流程）
            log_info!("\nYou have uncommitted changes. Will create branch and commit.");
        }

        Ok(GitOpsState {
            actual_branch_name,
            use_current_branch,
            has_uncommitted,
            use_stash,
            is_default_branch,
            committed_to_current_branch,
            default_branch,
            existing_pr,
            skip_git_ops,
        })
    }

    /// 选择变更类型
    ///
    /// 提示用户选择变更类型，返回一个布尔向量，表示每个类型是否被选中。
    fn select_change_types() -> Result<Vec<bool>> {
        log_info!("\nTypes of changes:");
        let selections = MultiSelect::new()
            .with_prompt("Select change types (use space to select, enter to confirm)")
            .items(TYPES_OF_CHANGES)
            .interact()
            .context("Failed to select change types")?;

        let selected_types: Vec<bool> = (0..TYPES_OF_CHANGES.len())
            .map(|i| selections.contains(&i))
            .collect();

        Ok(selected_types)
    }

    /// 生成 commit title 和分支名
    ///
    /// 首先生成 commit title，然后尝试使用 LLM 生成分支名。
    /// 如果 LLM 生成失败或结果无效，回退到默认方法。
    /// 返回 (commit_title, branch_name) 元组。
    fn generate_commit_title_and_branch_name(
        jira_ticket: &Option<String>,
        title: &str,
    ) -> Result<(String, String)> {
        let commit_title = generate_commit_title(jira_ticket.as_deref(), title);

        // 尝试使用 LLM 根据 commit_title 生成分支名
        let branch_name = match PullRequestLLM::generate(&commit_title) {
            Ok(content) => {
                log_success!("Generated branch name with LLM: {}", content.branch_name);
                let mut branch_name: String = content.branch_name;

                // 如果有 Jira ticket，添加到分支名前缀
                if let Some(ticket) = jira_ticket.as_deref() {
                    branch_name = format!("{}-{}", ticket, branch_name);
                }

                // 如果有 GITHUB_BRANCH_PREFIX，添加前缀
                let settings = Settings::get();
                if let Some(prefix) = &settings.github_branch_prefix {
                    if !prefix.trim().is_empty() {
                        branch_name = format!("{}/{}", prefix.trim(), branch_name);
                    }
                }

                branch_name
            }
            Err(_) => {
                // 回退到原来的方法
                generate_branch_name(jira_ticket.as_deref(), title)?
            }
        };

        Ok((commit_title, branch_name))
    }

    /// 创建或获取 PR
    ///
    /// 如果分支已有 PR，获取 PR URL；否则创建新 PR。
    /// 在创建 PR 前会检查分支是否有提交。
    fn create_or_get_pull_request(
        git_ops_state: &GitOpsState,
        commit_title: &str,
        pull_request_body: &str,
    ) -> Result<String> {
        // 如果分支已有 PR，跳过创建步骤，直接使用现有 PR
        if let Some(ref pr_id) = git_ops_state.existing_pr {
            log_info!("PR #{} already exists for branch '{}'", pr_id, git_ops_state.actual_branch_name);
            let repo_type = Git::detect_repo_type()?;
            match repo_type {
                RepoType::GitHub => {
                    <GitHub as PlatformProvider>::get_pull_request_url(pr_id)
                }
                RepoType::Codeup => {
                    <Codeup as PlatformProvider>::get_pull_request_url(pr_id)
                }
                RepoType::Unknown => {
                    anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
                }
            }
        } else {
            // 分支无 PR，创建新 PR
            // 先检查分支是否有提交（相对于默认分支）
            let has_commits = Git::is_branch_ahead(&git_ops_state.actual_branch_name, &git_ops_state.default_branch)
                .context("Failed to check branch commits")?;

            if !has_commits {
                log_warning!("Branch '{}' has no commits compared to '{}'.", git_ops_state.actual_branch_name, git_ops_state.default_branch);
                log_warning!("GitHub does not allow creating PRs for empty branches.");
                log_warning!("Please make some changes and commit them before creating a PR.");
                anyhow::bail!(
                    "Cannot create PR: branch '{}' has no commits. Please commit changes first.",
                    git_ops_state.actual_branch_name
                );
            }

            let repo_type = Git::detect_repo_type()?;
            let pull_request_url = match repo_type {
                RepoType::GitHub => {
                    log_success!("Creating PR via GitHub...");
                    <GitHub as PlatformProvider>::create_pull_request(
                        commit_title,
                        pull_request_body,
                        &git_ops_state.actual_branch_name,
                        None,
                    )?
                }
                RepoType::Codeup => {
                    log_success!("Creating PR via Codeup...");
                    <Codeup as PlatformProvider>::create_pull_request(
                        commit_title,
                        pull_request_body,
                        &git_ops_state.actual_branch_name,
                        None,
                    )?
                }
                RepoType::Unknown => {
                    anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
                }
            };

            log_success!("PR created: {}", pull_request_url);
            Ok(pull_request_url)
        }
    }

    /// 执行 Git 操作
    ///
    /// 根据 `git_ops_state` 的状态执行相应的 Git 操作：
    /// - 如果跳过 Git 操作，只确保在正确的分支上
    /// - 如果使用当前分支且工作树干净，只需要推送
    /// - 否则：创建分支、stash/恢复修改、提交、推送
    fn execute_git_operations(git_ops_state: &GitOpsState, commit_title: &str) -> Result<()> {
        if !git_ops_state.skip_git_ops {
            // 如果选择使用当前分支，且工作树干净，只需要推送（如果还没有推送）
            if git_ops_state.use_current_branch && !git_ops_state.has_uncommitted {
                log_info!("Using current branch '{}' for PR.", git_ops_state.actual_branch_name);
                // 检查分支是否已推送
                let (_, exists_remote_current) = Git::is_branch_exists(&git_ops_state.actual_branch_name)
                    .context("Failed to check if current branch exists on remote")?;
                if !exists_remote_current {
                    log_success!("Pushing current branch to remote...");
                    Git::push(&git_ops_state.actual_branch_name, true)?; // set-upstream
                } else {
                    log_info!("Branch '{}' already exists on remote.", git_ops_state.actual_branch_name);
                }
            } else {
                // 如果使用 stash，先保存修改
                if git_ops_state.use_stash {
                    log_success!("Stashing uncommitted changes...");
                    Git::stash_push(Some(&format!("WIP: {}", commit_title)))?;
                }

                // 如果不在默认分支上，需要先切换到默认分支再创建新分支
                if !git_ops_state.is_default_branch && (git_ops_state.committed_to_current_branch || git_ops_state.use_stash) {
                    log_info!("Switching to default branch '{}' to create new branch...", git_ops_state.default_branch);
                    Git::checkout_branch(&git_ops_state.default_branch)?;
                }

                log_success!("\nCreating branch: {}", git_ops_state.actual_branch_name);
                Git::checkout_branch(&git_ops_state.actual_branch_name)?;

                // 如果使用 stash，恢复修改
                if git_ops_state.use_stash {
                    log_success!("Restoring stashed changes...");
                    if let Err(e) = Git::stash_pop() {
                        // 如果 stash_pop 失败（可能是冲突），检查是否有冲突
                        if Git::has_unmerged().unwrap_or(false) {
                            log_warning!("Merge conflicts detected. Please resolve conflicts manually.");
                            log_warning!("After resolving conflicts:");
                            log_warning!("  1. Stage resolved files: git add <file>");
                            log_warning!("  2. Commit changes: git commit -m \"{}\"", commit_title);
                            log_warning!("  3. Push to remote: git push -u origin {}", git_ops_state.actual_branch_name);
                            log_warning!("  4. Create PR manually or run this command again");
                            anyhow::bail!("Failed to restore stashed changes due to merge conflicts. Please resolve conflicts manually and continue.");
                        } else {
                            // 没有冲突但失败了，返回错误
                            return Err(e);
                        }
                    }
                }

                log_success!("Committing changes...");
                Git::commit(commit_title, true)?; // no-verify
                log_success!("Pushing to remote...");
                Git::push(&git_ops_state.actual_branch_name, true)?; // set-upstream
            }
        } else {
            // 跳过 Git 操作，但需要确保在正确的分支上
            let current_branch = Git::current_branch()?;
            if current_branch != git_ops_state.actual_branch_name {
                log_info!("Switching to branch: {}", git_ops_state.actual_branch_name);
                Git::checkout_branch(&git_ops_state.actual_branch_name)?;
            }
        }

        Ok(())
    }

    /// 更新 Jira ticket
    ///
    /// 如果有 Jira ticket 和状态配置，更新 ticket：
    /// - 分配任务
    /// - 更新状态
    /// - 添加评论（PR URL 和描述）
    /// - 写入历史记录
    fn update_jira_ticket(
        jira_ticket: &Option<String>,
        created_pull_request_status: &Option<String>,
        pull_request_url: &str,
        description: &str,
        git_ops_state: &GitOpsState,
    ) -> Result<()> {
        if let Some(ref ticket) = jira_ticket {
            if let Some(ref status) = created_pull_request_status {
                log_success!("Updating Jira ticket...");
                // 分配任务
                Jira::assign_ticket(ticket, None)?;
                // 更新状态
                Jira::move_ticket(ticket, status)?;
                // 添加评论（PR URL）
                Jira::add_comment(ticket, pull_request_url)?;
                // 添加描述（如果有）
                if !description.trim().is_empty() {
                    Jira::add_comment(ticket, description)?;
                }

                // 写入历史记录
                let pull_request_id = extract_pull_request_id_from_url(pull_request_url)?;
                let repository = Git::get_remote_url().ok();
                JiraStatus::write_work_history(
                    ticket,
                    &pull_request_id,
                    Some(pull_request_url),
                    repository.as_deref(),
                    Some(&git_ops_state.actual_branch_name),
                )?;
            }
        }
        Ok(())
    }

    /// 配置 Jira ticket 状态
    ///
    /// 如果有 Jira ticket，检查并配置状态。如果已配置则读取，否则进行交互式配置。
    fn configure_jira_status(jira_ticket: &Option<String>) -> Result<Option<String>> {
        if let Some(ref ticket) = jira_ticket {
            // 读取状态配置
            if let Ok(Some(status)) = JiraStatus::read_pull_request_created_status(ticket) {
                Ok(Some(status))
            } else {
                // 如果没有配置，提示配置
                log_info!(
                    "No status configuration found for {}, configuring...",
                    ticket
                );
                JiraStatus::configure_interactive(ticket)
                    .with_context(|| {
                        format!(
                            "Failed to configure Jira ticket '{}'. Please ensure it's a valid ticket ID (e.g., PROJECT-123). If you don't need Jira integration, leave this field empty.",
                            ticket
                        )
                    })?;
                let status = JiraStatus::read_pull_request_created_status(ticket)
                    .context("Failed to read status after configuration")?;
                Ok(status)
            }
        } else {
            Ok(None)
        }
    }

    /// 复制 PR URL 到剪贴板并在浏览器中打开
    fn copy_and_open_pr(pull_request_url: &str) -> Result<()> {
        // 复制 PR URL 到剪贴板
        Clipboard::copy(pull_request_url)?;
        log_success!("Copied {} to clipboard", pull_request_url);

        // 打开浏览器
        std::thread::sleep(std::time::Duration::from_secs(1));
        Browser::open(pull_request_url)?;

        Ok(())
    }
}
