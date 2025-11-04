use crate::commands::{check::CheckCommand, jira::status::JiraStatus};
use crate::{
    extract_pr_id_from_url, generate_branch_name, generate_commit_title, generate_pr_body,
    log_error, log_info, log_success, log_warning, Browser, Clipboard, Codeup, Git, GitHub, Jira, Platform, RepoType, LLM,
    TYPES_OF_CHANGES,
};
use anyhow::{Context, Result};
use dialoguer::{Input, MultiSelect};
use std::io::{self, Write};

/// PR 创建命令
pub struct PRCreateCommand;

impl PRCreateCommand {
    /// 创建 PR（完整流程）
    pub fn create(
        jira_ticket: Option<String>,
        title: Option<String>,
        description: Option<String>,
        dry_run: bool,
    ) -> Result<()> {
        // 1. 运行检查
        if !dry_run {
            CheckCommand::run_all()?;
            CheckCommand::check_pre_commit()?;
        }

        // 2. 获取或输入 Jira ticket
        let jira_ticket = if let Some(ticket) = jira_ticket {
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
                Some(ticket)
            }
        };

        // 3. 如果有 Jira ticket，检查并配置状态
        let mut created_pr_status = None;
        if let Some(ref ticket) = jira_ticket {
            // 读取状态配置
            if let Ok(Some(status)) = JiraStatus::read_pr_created_status(ticket) {
                created_pr_status = Some(status);
            } else {
                // 如果没有配置，提示配置
                log_info!(
                    "No status configuration found for {}, configuring...",
                    ticket
                );
                JiraStatus::configure_interactive(ticket)?;
                created_pr_status = JiraStatus::read_pr_created_status(ticket)
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

        // 7. 生成分支名
        let branch_name = generate_branch_name(jira_ticket.as_deref(), &title)?;
        let commit_title = generate_commit_title(jira_ticket.as_deref(), &title);

        // 8. 生成 PR body
        let pr_body =
            generate_pr_body(&selected_types, Some(&description), jira_ticket.as_deref())?;

        if dry_run {
            log_info!("\n[DRY RUN] Would create branch: {}", branch_name);
            log_info!("[DRY RUN] Commit title: {}", commit_title);
            log_info!("[DRY RUN] PR body:\n{}", pr_body);
            return Ok(());
        }

        // 9. 创建分支、提交、推送
        log_success!("\nCreating branch: {}", branch_name);
        Git::checkout_branch(&branch_name)?;

        log_success!("Committing changes...");
        Git::commit(&commit_title, true)?; // no-verify

        log_success!("Pushing to remote...");
        Git::push(&branch_name, true)?; // set-upstream

        // 10. 创建 PR（根据仓库类型）
        let repo_type = Git::detect_repo_type()?;
        let pr_url = match repo_type {
            RepoType::GitHub => {
                log_success!("Creating PR via GitHub...");
                <GitHub as Platform>::create_pr(&commit_title, &pr_body, &branch_name, None)?
            }
            RepoType::Codeup => {
                log_success!("Creating PR via Codeup...");
                <Codeup as Platform>::create_pr(&commit_title, &pr_body, &branch_name, None)?
            }
            RepoType::Unknown => {
                anyhow::bail!("Unknown repository type. Only GitHub and Codeup are supported.");
            }
        };

        log_success!("PR created: {}", pr_url);

        // 11. 更新 Jira（如果有 ticket）
        if let Some(ref ticket) = jira_ticket {
            if let Some(ref status) = created_pr_status {
                log_success!("Updating Jira ticket...");
                // 分配任务
                Jira::assign_ticket(ticket, None)?;
                // 更新状态
                Jira::move_ticket(ticket, status)?;
                // 添加评论（PR URL）
                Jira::add_comment(ticket, &pr_url)?;
                // 添加描述（如果有）
                if !description.trim().is_empty() {
                    Jira::add_comment(ticket, &description)?;
                }

                // 写入历史记录
                let pr_id = extract_pr_id_from_url(&pr_url)?;
                let repository = Git::get_remote_url().ok();
                crate::jira::status::write_work_history(
                    ticket,
                    &pr_id,
                    Some(&pr_url),
                    repository.as_deref(),
                    Some(&branch_name),
                )?;
            }
        }

        // 12. 复制 PR URL 到剪贴板
        Clipboard::copy(&pr_url)?;
        log_success!("Copied {} to clipboard", pr_url);

        // 13. 打开浏览器
        std::thread::sleep(std::time::Duration::from_secs(1));
        Browser::open(&pr_url)?;

        log_success!("PR created successfully!");
        Ok(())
    }
}
