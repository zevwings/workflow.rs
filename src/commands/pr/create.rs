use crate::commands::check::CheckCommand;
use crate::jira::status::JiraStatus;
use crate::{
    extract_pull_request_id_from_url, generate_branch_name, generate_commit_title,
    generate_pull_request_body, log_error, log_info, log_success, log_warning, Browser, Clipboard,
    Codeup, Git, GitHub, Jira, PlatformProvider, RepoType, LLM, TYPES_OF_CHANGES,
};
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use std::io::{self, Write};

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
        let jira_ticket = if let Some(ticket) = jira_ticket {
            // 验证 ticket 格式
            if !ticket.trim().is_empty() {
                // 简单的格式验证：Jira ticket 应该是 PROJECT-123 格式，或纯项目名
                use crate::jira::helpers::extract_jira_project;
                let is_valid_format = if let Some(project) = extract_jira_project(&ticket) {
                    // 如果是 ticket 格式（PROJ-123），检查项目名是否有效
                    project
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                } else {
                    // 如果是项目名格式，检查是否只包含有效字符
                    ticket
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                };

                if !is_valid_format {
                    anyhow::bail!(
                        "Invalid Jira ticket format: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name).\n  - Ticket names should contain only letters, numbers, and hyphens\n  - Project names should contain only letters, numbers, and underscores\n  - Do not use branch names or paths (e.g., 'zw/修改打包脚本问题')",
                        ticket
                    );
                }
            }
            Some(ticket)
        } else {
            print!("Jira ticket (optional): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let ticket = input.trim().to_string();
            if ticket.is_empty() {
                None
            } else {
                // 验证输入的 ticket 格式
                use crate::jira::helpers::extract_jira_project;
                let is_valid_format = if let Some(project) = extract_jira_project(&ticket) {
                    project
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                } else {
                    ticket
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                };

                if !is_valid_format {
                    anyhow::bail!(
                        "Invalid Jira ticket format: '{}'. Expected format: 'PROJ-123' (ticket) or 'PROJ' (project name).\n  - Ticket names should contain only letters, numbers, and hyphens\n  - Project names should contain only letters, numbers, and underscores\n  - Do not use branch names or paths (e.g., 'zw/修改打包脚本问题')",
                        ticket
                    );
                }
                Some(ticket)
            }
        };

        // 3. 如果有 Jira ticket，检查并配置状态
        let mut created_pull_request_status = None;
        if let Some(ref ticket) = jira_ticket {
            // 读取状态配置
            if let Ok(Some(status)) = JiraStatus::read_pull_request_created_status(ticket) {
                created_pull_request_status = Some(status);
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
                created_pull_request_status = JiraStatus::read_pull_request_created_status(ticket)
                    .context("Failed to read status after configuration")?;
            }
        }

        // 4. 获取或生成 PR 标题
        let title = if let Some(t) = title {
            t
        } else if let Some(ref ticket) = jira_ticket {
            // 如果有 Jira ticket，尝试使用 LLM 生成标题
            log_success!("Generating PR title from Jira ticket...");

            match LLM::get_issue_desc(ticket) {
                Ok(desc) => {
                    if let Some(ref translated) = desc.translated_desc {
                        log_success!("Original: {}", desc.issue_desc);
                        log_success!("Translated: {}", translated);
                        translated.clone()
                    } else {
                        log_success!("Using original description: {}", desc.issue_desc);
                        desc.issue_desc.clone()
                    }
                }
                Err(e) => {
                    log_error!("AI generation failed: {}", e);
                    log_warning!("Falling back to manual input");
                    // 回退到手动输入
                    print!("PR title (required): ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    let input_title = input.trim().to_string();

                    if input_title.is_empty() {
                        anyhow::bail!("PR title is required");
                    }
                    input_title
                }
            }
        } else {
            let title: String = Input::new()
                .with_prompt("PR title (required)")
                .interact_text()
                .context("Failed to get PR title")?;
            title
        };

        if title.trim().is_empty() {
            anyhow::bail!("PR title is required");
        }

        // 5. 获取描述
        let description = if let Some(desc) = description {
            desc
        } else {
            let desc: String = Input::new()
                .with_prompt("Short description (optional)")
                .allow_empty(true)
                .interact_text()
                .context("Failed to get description")?;
            desc
        };

        // 6. 选择变更类型
        log_info!("\nTypes of changes:");
        let selections = MultiSelect::new()
            .with_prompt("Select change types (use space to select, enter to confirm)")
            .items(TYPES_OF_CHANGES)
            .interact()
            .context("Failed to select change types")?;

        let selected_types: Vec<bool> = (0..TYPES_OF_CHANGES.len())
            .map(|i| selections.contains(&i))
            .collect();

        // 7. 生成 commit_title 和分支名
        let commit_title = generate_commit_title(jira_ticket.as_deref(), &title);

        // 尝试使用 LLM 根据 commit_title 生成分支名
        let branch_name = match LLM::generate_branch_name(&commit_title) {
            Ok(llm_branch_name) => {
                // 验证分支名是否包含非 ASCII 字符（如中文）
                let is_valid_ascii = llm_branch_name.chars().all(|c| c.is_ascii() || c == '-');
                let cleaned_branch_name = if !is_valid_ascii {
                    // 如果包含非 ASCII 字符，使用 transform_to_branch_name 再次清理
                    let cleaned = crate::pr::helpers::transform_to_branch_name(&llm_branch_name);
                    // 如果清理后为空或只包含连字符，回退到原来的方法
                    if cleaned.is_empty() || cleaned.chars().all(|c| c == '-') {
                        log_info!("LLM generated branch name contains non-ASCII characters and cleaned result is invalid, falling back to default method");
                        generate_branch_name(jira_ticket.as_deref(), &title)?
                    } else {
                        cleaned
                    }
                } else {
                    llm_branch_name
                };

                log_success!("Generated branch name with LLM: {}", cleaned_branch_name);
                let mut branch_name = cleaned_branch_name;

                // 如果有 Jira ticket，添加到分支名前缀
                if let Some(ticket) = jira_ticket.as_deref() {
                    branch_name = format!("{}--{}", ticket, branch_name);
                }

                // 如果有 GITHUB_BRANCH_PREFIX，添加前缀
                let settings = crate::settings::Settings::get();
                if let Some(prefix) = &settings.github_branch_prefix {
                    if !prefix.trim().is_empty() {
                        branch_name = format!("{}/{}", prefix.trim(), branch_name);
                    }
                }

                branch_name
            }
            Err(_) => {
                // 回退到原来的方法
                generate_branch_name(jira_ticket.as_deref(), &title)?
            }
        };

        // 8. 生成 PR body
        let pull_request_body = generate_pull_request_body(
            &selected_types,
            Some(&description),
            jira_ticket.as_deref(),
        )?;

        if dry_run {
            log_info!("\n[DRY RUN] Would create branch: {}", branch_name);
            log_info!("[DRY RUN] Commit title: {}", commit_title);
            log_info!("[DRY RUN] PR body:\n{}", pull_request_body);
            return Ok(());
        }

        // 9. 检查状态：未提交修改、分支存在、PR 存在、当前分支是否是默认分支
        let has_uncommitted = Git::has_uncommitted_changes()
            .context("Failed to check uncommitted changes")?;
        let current_branch = Git::current_branch()
            .context("Failed to get current branch")?;
        let default_branch = Git::get_default_branch()
            .context("Failed to get default branch")?;
        let is_default_branch = current_branch == default_branch;

        let (exists_local, exists_remote) = Git::branch_exists(&branch_name)
            .context("Failed to check branch existence")?;
        let repo_type = Git::detect_repo_type()?;

        // 处理非默认分支上有未提交修改的情况
        let mut use_stash = false; // 是否使用 stash 带到新分支
        let mut committed_to_current_branch = false; // 是否已经提交到当前分支
        if has_uncommitted && !is_default_branch {
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
                        .default(format!("WIP: {}", commit_title.clone()))
                        .interact_text()
                        .context("Failed to get commit message")?;

                    log_success!("Committing changes to current branch: {}", current_branch);
                    Git::commit(&commit_msg, true)?; // no-verify

                    log_success!("Pushing changes to current branch...");
                    Git::push(&current_branch, false)?; // 不使用 -u

                    committed_to_current_branch = true;
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
        }

        // 检查分支是否已有 PR（只在分支已存在时检查）
        // 注意：如果已经提交到当前分支或使用了 stash，这里可以安全地切换分支检查 PR
        let existing_pr = if exists_local || exists_remote {
            // 尝试切换到分支检查 PR（如果切换失败，假设没有 PR）
            let current_branch_for_check = Git::current_branch().ok();
            if let Ok(()) = Git::checkout_branch(&branch_name) {
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
            log_warning!("\n⚠️  Branch '{}' exists on remote but no PR found.", branch_name);
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
                log_warning!("\n⚠️  Branch '{}' already exists and has PR #{}", branch_name, pr_id);

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
                log_warning!("\n⚠️  Branch '{}' already exists.", branch_name);
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

        // 10. 执行 Git 操作（如果需要）
        if !skip_git_ops {
            // 如果使用 stash，先保存修改
            if use_stash {
                log_success!("Stashing uncommitted changes...");
                Git::stash(Some(&format!("WIP: {}", commit_title)))?;
            }

            // 如果不在默认分支上，需要先切换到默认分支再创建新分支
            if !is_default_branch && (committed_to_current_branch || use_stash) {
                log_info!("Switching to default branch '{}' to create new branch...", default_branch);
                Git::checkout_branch(&default_branch)?;
            }

            log_success!("\nCreating branch: {}", branch_name);
            Git::checkout_branch(&branch_name)?;

            // 如果使用 stash，恢复修改
            if use_stash {
                log_success!("Restoring stashed changes...");
                Git::stash_pop()?;
            }

            log_success!("Committing changes...");
            Git::commit(&commit_title, true)?; // no-verify

            log_success!("Pushing to remote...");
            Git::push(&branch_name, true)?; // set-upstream
        } else {
            // 跳过 Git 操作，但需要确保在正确的分支上
            let current_branch = Git::current_branch()?;
            if current_branch != branch_name {
                log_info!("Switching to branch: {}", branch_name);
                Git::checkout_branch(&branch_name)?;
            }
        }

        // 10. 创建 PR（根据仓库类型）
        // 如果分支已有 PR，跳过创建步骤，直接使用现有 PR
        let pull_request_url = if let Some(ref pr_id) = existing_pr {
            log_info!("PR #{} already exists for branch '{}'", pr_id, branch_name);
            let repo_type = Git::detect_repo_type()?;
            match repo_type {
                RepoType::GitHub => {
                    <GitHub as PlatformProvider>::get_pull_request_url(pr_id)?
                }
                RepoType::Codeup => {
                    <Codeup as PlatformProvider>::get_pull_request_url(pr_id)?
                }
                RepoType::Unknown => {
                    anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
                }
            }
        } else {
            // 分支无 PR，创建新 PR
            let repo_type = Git::detect_repo_type()?;
            match repo_type {
                RepoType::GitHub => {
                    log_success!("Creating PR via GitHub...");
                    <GitHub as PlatformProvider>::create_pull_request(
                        &commit_title,
                        &pull_request_body,
                        &branch_name,
                        None,
                    )?
                }
                RepoType::Codeup => {
                    log_success!("Creating PR via Codeup...");
                    <Codeup as PlatformProvider>::create_pull_request(
                        &commit_title,
                        &pull_request_body,
                        &branch_name,
                        None,
                    )?
                }
                RepoType::Unknown => {
                    anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
                }
            }
        };

        log_success!("PR created: {}", pull_request_url);

        // 11. 更新 Jira（如果有 ticket）
        if let Some(ref ticket) = jira_ticket {
            if let Some(ref status) = created_pull_request_status {
                log_success!("Updating Jira ticket...");
                // 分配任务
                Jira::assign_ticket(ticket, None)?;
                // 更新状态
                Jira::move_ticket(ticket, status)?;
                // 添加评论（PR URL）
                Jira::add_comment(ticket, &pull_request_url)?;
                // 添加描述（如果有）
                if !description.trim().is_empty() {
                    Jira::add_comment(ticket, &description)?;
                }

                // 写入历史记录
                let pull_request_id = extract_pull_request_id_from_url(&pull_request_url)?;
                let repository = Git::get_remote_url().ok();
                JiraStatus::write_work_history(
                    ticket,
                    &pull_request_id,
                    Some(&pull_request_url),
                    repository.as_deref(),
                    Some(&branch_name),
                )?;
            }
        }

        // 12. 复制 PR URL 到剪贴板
        Clipboard::copy(&pull_request_url)?;
        log_success!("Copied {} to clipboard", pull_request_url);

        // 13. 打开浏览器
        std::thread::sleep(std::time::Duration::from_secs(1));
        Browser::open(&pull_request_url)?;

        log_success!("PR created successfully!");
        Ok(())
    }
}
