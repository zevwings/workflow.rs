use crate::commands::check::CheckCommand;
use crate::jira::status::JiraStatus;
use crate::{
    Browser, Clipboard, Codeup, Git, GitHub, Jira, PlatformProvider, PullRequestLLM, RepoType, Settings, TYPES_OF_CHANGES, extract_pull_request_id_from_url, generate_branch_name, generate_commit_title, generate_pull_request_body, log_info, log_success, log_warning, validate_jira_ticket_format
};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect};

/// PR 创建命令
#[allow(dead_code)]
pub struct PullRequestCreateCommand;


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
        let jira_ticket = Self::resolve_jira_ticket(jira_ticket)?;

        // 3. 如果有 Jira ticket，检查并配置状态
        let created_pull_request_status = Self::ensure_jira_status(&jira_ticket)?;

        // 4. 获取或生成 PR 标题
        let title = Self::resolve_title(title, &jira_ticket)?;

        // 5. 生成 commit_title 和分支名
        let (commit_title, branch_name) = Self::generate_commit_title_and_branch_name(&jira_ticket, &title)?;

        // 6. 获取描述
        let description = Self::resolve_description(description)?;

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
            log_info!("[DRY RUN] Would create branch: {}", branch_name);
            log_info!("[DRY RUN] Commit title: {}", commit_title);
            log_info!("[DRY RUN] PR body:\n{}", pull_request_body);
            return Ok(());
        }

        // 9. 创建或更新分支
        let (actual_branch_name, default_branch) = Self::create_or_update_branch(
            &branch_name,
            &commit_title,
        )?;

        // 11. 创建或获取 PR
        let pull_request_url = Self::create_or_get_pull_request(
            &actual_branch_name,
            &default_branch,
            &commit_title,
            &pull_request_body,
        )?;

        // 12. 更新 Jira（如果有 ticket）
        Self::update_jira_ticket(
            &jira_ticket,
            &created_pull_request_status,
            &pull_request_url,
            &description,
            &actual_branch_name,
        )?;

        // 13. 复制 PR URL 到剪贴板并打开浏览器
        Self::copy_and_open_pull_request(&pull_request_url)?;

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
            // 使用 dialoguer::Input 保持一致性
            let input: String = Input::new()
                .with_prompt("Jira ticket (optional)")
                .allow_empty(true)
                .interact_text()
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

    /// 配置 Jira ticket 状态
    ///
    /// 步骤 3：如果有 Jira ticket，检查并配置状态。如果已配置则读取，否则进行交互式配置。
    fn ensure_jira_status(jira_ticket: &Option<String>) -> Result<Option<String>> {
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

    /// 提示用户输入 PR 标题
    ///
    /// 步骤 4（辅助函数）：使用 Input 组件提示用户输入 PR 标题，并验证输入不能为空。
    fn input_pull_request_title() -> Result<String> {
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
    /// 步骤 4：如果提供了标题，直接使用；如果有 Jira ticket，尝试从 Jira 获取标题；
    /// 否则提示用户输入标题。
    fn resolve_title(title: Option<String>, jira_ticket: &Option<String>) -> Result<String> {
        if let Some(t) = title {
            return Ok(t);
        }

            // 如果有 Jira ticket，尝试从 Jira 获取标题
        if let Some(ref ticket) = jira_ticket {
            log_success!("Getting PR title from Jira ticket...");

            if let Ok(issue) = Jira::get_ticket_info(ticket) {
                    let summary = issue.fields.summary.trim().to_string();
                if !summary.is_empty() {
                    log_success!("Using Jira ticket summary: {}", summary);
                    return Ok(summary);
                }
                        log_warning!("Jira ticket summary is empty, falling back to manual input");
                    } else {
                log_warning!("Failed to get ticket info, falling back to manual input");
            }
        }

        // 回退到手动输入（统一处理）
        Self::input_pull_request_title()
    }

    /// 应用分支名前缀（Jira ticket 和 github_branch_prefix）
    ///
    /// 步骤 5（辅助函数）：统一处理分支名前缀逻辑，避免代码重复。
    fn apply_branch_name_prefixes(
        mut branch_name: String,
        jira_ticket: Option<&str>,
    ) -> Result<String> {
        // 如果有 Jira ticket，添加到分支名前缀
        if let Some(ticket) = jira_ticket {
            branch_name = format!("{}-{}", ticket, branch_name);
        }

        // 如果有 GITHUB_BRANCH_PREFIX，添加前缀
        let settings = Settings::get();
        if let Some(prefix) = &settings.github_branch_prefix {
            let trimmed = prefix.trim();
            if !trimmed.is_empty() {
                branch_name = format!("{}/{}", trimmed, branch_name);
            }
        }

        Ok(branch_name)
    }

    /// 生成 commit title 和分支名
    ///
    /// 步骤 5：首先生成 commit title，然后尝试使用 LLM 生成分支名。
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
                // 使用辅助函数统一处理前缀逻辑，避免重复代码
                Self::apply_branch_name_prefixes(content.branch_name, jira_ticket.as_deref())?
            }
            Err(_) => {
                // 回退到原来的方法（已包含前缀逻辑）
                generate_branch_name(jira_ticket.as_deref(), title)?
            }
        };

        Ok((commit_title, branch_name))
    }

    /// 获取 PR 描述
    ///
    /// 步骤 6：如果提供了描述，直接使用；否则提示用户输入描述（可选）。
    fn resolve_description(description: Option<String>) -> Result<String> {
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

    /// 选择变更类型
    ///
    /// 步骤 7：提示用户选择变更类型，返回一个布尔向量，表示每个类型是否被选中。
    fn select_change_types() -> Result<Vec<bool>> {
        log_info!("Types of changes:");
        let selections = MultiSelect::new()
            .with_prompt("Select change types (use space to select, enter to confirm)")
            .items(TYPES_OF_CHANGES)
            .interact()
            .context("Failed to select change types")?;

        // 简化转换逻辑：直接使用 HashSet 的 contains 方法
        let selected_types: Vec<bool> = (0..TYPES_OF_CHANGES.len())
            .map(|i| selections.contains(&i))
            .collect();

        Ok(selected_types)
    }

    /// 在默认分支上创建新分支并提交
    ///
    /// 步骤 9 的辅助方法：当用户在默认分支上有未提交修改时，直接创建新分支
    /// （Git 会自动把未提交的修改带到新分支），然后提交并推送。
    ///
    /// # 流程
    /// 1. 创建新分支（未提交的修改会自动带到新分支）
    /// 2. 提交更改
    /// 3. 推送到远程
    fn create_branch_from_default(
        branch_name: &str,
        commit_title: &str,
        default_branch: &str,
    ) -> Result<(String, String)> {
        log_info!("You are on default branch '{}' with uncommitted changes.", default_branch);
        log_info!("Will create new branch and commit changes...");

        // 直接创建新分支（Git 会自动把未提交的修改带到新分支）
        log_success!("Creating branch: {}", branch_name);
        Git::checkout_branch(branch_name)?;

        // 提交并推送
        log_success!("Committing changes...");
        Git::commit(commit_title, true)?; // no-verify
        log_success!("Pushing to remote...");
        Git::push(branch_name, true)?; // set-upstream

        Ok((branch_name.to_string(), default_branch.to_string()))
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
        log_info!("Will commit and create PR on current branch '{}'...", current_branch);

        // 检查是否已推送
        let (_, exists_remote) = Git::is_branch_exists(current_branch)
            .context("Failed to check if branch exists on remote")?;

        // 提交
        log_success!("Committing changes...");
        Git::commit(commit_title, true)?; // no-verify

        // 推送（如需要）
        if !exists_remote {
            log_success!("Pushing to remote...");
            Git::push(current_branch, true)?; // set-upstream
        } else {
            log_info!("Branch '{}' already exists on remote.", current_branch);
            log_success!("Pushing latest changes...");
            Git::push(current_branch, false)?; // 不使用 -u，因为已经设置过
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
        log_info!("Will create new branch '{}' and commit changes...", branch_name);

        // 使用 stash 暂存修改
        log_success!("Stashing uncommitted changes...");
        Git::stash_push(Some(&format!("WIP: {}", commit_title)))?;

        // 切换到默认分支
        log_info!("Switching to default branch '{}'...", default_branch);
        Git::checkout_branch(default_branch)?;

        // 拉取最新的代码
        log_success!("Pulling latest changes from '{}'...", default_branch);
        Git::pull(default_branch)?;

        // 检查目标分支是否存在，如果存在则报错（此方法应该创建新分支）
        let (exists_local, exists_remote) = Git::is_branch_exists(branch_name)
            .context("Failed to check if branch exists")?;

        if exists_local || exists_remote {
            anyhow::bail!("Branch '{}' already exists. Cannot create new branch with existing name.", branch_name);
        }

        // 创建新分支
        log_success!("Creating branch: {}", branch_name);
        Git::checkout_branch(branch_name)?;

        // 恢复 stash
        log_success!("Restoring stashed changes...");
        if let Err(e) = Git::stash_pop() {
            if Git::has_unmerged().unwrap_or(false) {
                log_warning!("Merge conflicts detected. Please resolve conflicts manually.");
                anyhow::bail!("Failed to restore stashed changes due to merge conflicts. Please resolve conflicts manually and continue.");
            } else {
                return Err(e);
            }
        }

        // 提交并推送
        log_success!("Committing changes...");
        Git::commit(commit_title, true)?; // no-verify
        log_success!("Pushing to remote...");
        Git::push(branch_name, true)?; // set-upstream

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
        let should_use_current = Confirm::new()
            .with_prompt(format!("Create PR for current branch '{}'?", current_branch))
            .default(true)
            .interact()
            .context("Failed to get confirmation")?;

        if !should_use_current {
            anyhow::bail!("Operation cancelled.");
        }

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
        log_info!("Branch '{}' has commits but not pushed to remote.", current_branch);
        let should_use_current = Confirm::new()
            .with_prompt(format!("Push and create PR for current branch '{}'?", current_branch))
            .default(true)
            .interact()
            .context("Failed to get confirmation")?;

        if !should_use_current {
            anyhow::bail!("Operation cancelled.");
        }

        // 推送
        log_success!("Pushing to remote...");
        Git::push(current_branch, true)?; // set-upstream

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
    fn create_or_update_branch(
        branch_name: &str,
        commit_title: &str,
    ) -> Result<(String, String)> {
        // 1. 检查是否有未提交的修改
        let has_uncommitted = Git::has_commit()
            .context("Failed to check uncommitted changes")?;

        // 2. 获取当前分支和默认分支
        let current_branch = Git::current_branch()
            .context("Failed to get current branch")?;
        let default_branch = Git::get_default_branch()
            .context("Failed to get default branch")?;
        let is_default_branch = current_branch == default_branch;

        // 3. 判断当前是否是默认分支
        if is_default_branch {
            // 在默认分支上
            if has_uncommitted {
                // 有未提交修改 → 创建新分支并提交
                Self::create_branch_from_default(branch_name, commit_title, &default_branch)
            } else {
                // 无未提交修改 → 提示：默认分支上没有更改，无法创建 PR
                anyhow::bail!("No changes on default branch '{}'. Cannot create PR without changes.", default_branch);
            }
        } else {
            // 不在默认分支上
            if has_uncommitted {
                // 有未提交的代码 → 询问用户是否需要在当前分支创建 PR
                log_info!("You are on branch '{}' with uncommitted changes.", current_branch);
                let should_use_current = Confirm::new()
                    .with_prompt(format!("Create PR for current branch '{}'? (otherwise will create new branch '{}')", current_branch, branch_name))
                    .default(true)
                    .interact()
                    .context("Failed to get confirmation")?;

                if should_use_current {
                    // 用户期望在当前分支提交并创建 PR
                    Self::commit_and_push_current_branch(&current_branch, commit_title, &default_branch)
                } else {
                    // 用户不期望在当前分支提交并创建 → 使用 stash 创建新分支
                    Self::create_new_branch_with_stash(&current_branch, branch_name, commit_title, &default_branch)
                }
            } else {
                // 无未提交的代码 → 判断当前分支是否在远程分支上
                let (_, exists_remote) = Git::is_branch_exists(&current_branch)
                    .context("Failed to check if branch exists on remote")?;

                // 检查当前分支是否有提交（相对于默认分支）
                let has_commits = Git::is_branch_ahead(&current_branch, &default_branch)
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

    /// 创建或获取 PR
    ///
    /// 步骤 11：检查分支是否已有 PR，如果有则获取 PR URL，否则创建新 PR。
    /// 在创建 PR 前会检查分支是否有提交。
    ///
    /// 返回 PR URL。
    fn create_or_get_pull_request(
        branch_name: &str,
        default_branch: &str,
        commit_title: &str,
        pull_request_body: &str,
    ) -> Result<String> {
        // 检查分支是否已有 PR
        let repo_type = Git::detect_repo_type()?;
        let existing_pr = match repo_type {
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

        if let Some(pr_id) = existing_pr {
            log_info!("PR #{} already exists for branch '{}'", pr_id, branch_name);
            match repo_type {
                RepoType::GitHub => {
                    <GitHub as PlatformProvider>::get_pull_request_url(&pr_id)
                }
                RepoType::Codeup => {
                    <Codeup as PlatformProvider>::get_pull_request_url(&pr_id)
                }
                RepoType::Unknown => {
                    anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
                }
            }
        } else {
            // 分支无 PR，创建新 PR
            // 先检查分支是否有提交（相对于默认分支）
            let has_commits = Git::is_branch_ahead(branch_name, default_branch)
                .context("Failed to check branch commits")?;

            if !has_commits {
                log_warning!("Branch '{}' has no commits compared to '{}'.", branch_name, default_branch);
                log_warning!("GitHub does not allow creating PRs for empty branches.");
                log_warning!("Please make some changes and commit them before creating a PR.");
                anyhow::bail!(
                    "Cannot create PR: branch '{}' has no commits. Please commit changes first.",
                    branch_name
                );
            }

            let pull_request_url = match repo_type {
                RepoType::GitHub => {
                    log_success!("Creating PR via GitHub...");
                    <GitHub as PlatformProvider>::create_pull_request(
                        commit_title,
                        pull_request_body,
                        branch_name,
                        None,
                    )?
                }
                RepoType::Codeup => {
                    log_success!("Creating PR via Codeup...");
                    <Codeup as PlatformProvider>::create_pull_request(
                        commit_title,
                        pull_request_body,
                        branch_name,
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

    /// 更新 Jira ticket
    ///
    /// 步骤 12：如果有 Jira ticket 和状态配置，更新 ticket：
    /// - 分配任务
    /// - 更新状态
    /// - 添加评论（PR URL 和描述）
    /// - 写入历史记录
    fn update_jira_ticket(
        jira_ticket: &Option<String>,
        created_pull_request_status: &Option<String>,
        pull_request_url: &str,
        description: &str,
        branch_name: &str,
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
                    Some(branch_name),
                )?;
            }
        }
        Ok(())
    }

    /// 复制 PR URL 到剪贴板并在浏览器中打开
    ///
    /// 步骤 13：复制 PR URL 到剪贴板并在浏览器中打开。
    fn copy_and_open_pull_request(pull_request_url: &str) -> Result<()> {
        // 复制 PR URL 到剪贴板
        Clipboard::copy(pull_request_url)?;
        log_success!("Copied {} to clipboard", pull_request_url);

        // 打开浏览器
        std::thread::sleep(std::time::Duration::from_secs(1));
        Browser::open(pull_request_url)?;

        Ok(())
    }
}
