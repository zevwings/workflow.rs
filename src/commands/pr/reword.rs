//! PR Reword 命令
//!
//! 基于 PR diff 自动生成并更新 PR 标题和描述。

use anyhow::{Context, Result};

use crate::base::dialog::ConfirmDialog;
use crate::base::indicator::Spinner;
use crate::git::GitRepo;
use crate::jira::helpers::extract_jira_ticket_id;
use crate::jira::Jira;
use crate::log_break;
use crate::log_debug;
use crate::log_info;
use crate::log_success;
use crate::log_warning;
use crate::pr::body_parser::{extract_jira_ticket_from_body, parse_change_types_from_body};
use crate::pr::helpers::{generate_pull_request_body, resolve_pull_request_id};
use crate::pr::llm::RewordGenerator;
use crate::pr::platform::create_provider_auto;

/// PR Reword 命令
pub struct PullRequestRewordCommand;

impl PullRequestRewordCommand {
    /// 执行 PR reword 命令
    ///
    /// 基于 PR diff 自动生成新的标题和描述，并更新到 PR。
    ///
    /// # 参数
    ///
    /// * `pull_request_id` - PR ID（可选，如果不提供则自动检测当前分支的 PR）
    /// * `title` - 仅更新标题
    /// * `description` - 仅更新描述
    /// * `dry_run` - 仅预览，不实际更新
    pub fn reword(
        pull_request_id: Option<String>,
        title: bool,
        description: bool,
        dry_run: bool,
    ) -> Result<()> {
        // 检查是否在 Git 仓库中
        if !GitRepo::is_git_repo() {
            anyhow::bail!(
                "Not in a Git repository. Please run this command in a Git repository directory."
            );
        }

        // 获取 PR ID（从参数或当前分支）
        let pr_id = resolve_pull_request_id(pull_request_id)?;

        // 创建平台提供者
        let provider = create_provider_auto()?;

        // 获取当前 PR 标题和描述
        let current_title = Spinner::with(format!("Fetching PR #{} information...", pr_id), || {
            provider.get_pull_request_title(&pr_id)
        })
        .context("Failed to get PR title")?;

        let current_body = Spinner::with("Fetching PR description...", || {
            provider.get_pull_request_body(&pr_id)
        })
        .context("Failed to get PR body")?;

        log_info!("Current PR #{}:", pr_id);
        log_info!("  Title:       {}", current_title);
        if let Some(ref body) = current_body {
            log_info!("  Description: {}", body);
            log_break!();
        } else {
            log_info!("  Description: (empty)");
            log_break!();
        }

        // 获取 PR diff
        let pr_diff = Spinner::with("Fetching PR diff...", || {
            provider.get_pull_request_diff(&pr_id)
        })
        .context("Failed to get PR diff")?;

        if pr_diff.trim().is_empty() {
            log_warning!("PR diff is empty. Cannot generate new title and description.");
            anyhow::bail!("PR diff is empty. Please ensure the PR has changes.");
        }

        // 显示 PR diff 信息（用于调试）
        let diff_length = pr_diff.chars().count();
        let diff_lines: Vec<&str> = pr_diff.lines().collect();
        let diff_line_count = diff_lines.len();
        log_debug!(
            "PR diff: {} characters, {} lines",
            diff_length,
            diff_line_count
        );

        // 显示 diff 的前几行，帮助用户确认获取的 diff 是否正确
        if diff_line_count > 0 {
            let preview_lines: String =
                diff_lines.iter().take(10).copied().collect::<Vec<_>>().join("\n");
            log_debug!("PR diff preview (first 10 lines):");
            log_debug!("{}", preview_lines);
            if diff_line_count > 10 {
                log_debug!("... ({} more lines)", diff_line_count - 10);
            }
            log_break!();
        }

        // 从当前 PR body 中提取信息
        let current_change_types = current_body.as_deref().and_then(parse_change_types_from_body);

        // 如果成功解析了 change_types，显示日志
        if let Some(ref types) = current_change_types {
            let selected_count = types.iter().filter(|&&t| t).count();
            if selected_count > 0 {
                log_success!(
                    "Found {} selected change type(s) in current PR",
                    selected_count
                );
                log_debug!("Change types: {:?}", types);
            } else {
                log_info!("No change types selected in current PR");
            }
        } else {
            log_warning!("Could not parse change types from current PR body, will use default (none selected)");
            if let Some(ref body) = current_body {
                log_debug!(
                    "Current PR body preview (first 500 chars): {}",
                    &body.chars().take(500).collect::<String>()
                );
            }
        }

        let jira_ticket = current_body
            .as_deref()
            .and_then(extract_jira_ticket_from_body)
            .or_else(|| extract_jira_ticket_id(&current_title));

        // 使用 LLM 生成新的标题和描述
        let reword_result = Spinner::with("Generating title and description with LLM...", || {
            RewordGenerator::reword_from_diff(&pr_diff, Some(&current_title))
        })
        .context("Failed to generate PR title and description")?;

        log_info!("Generated from PR diff:");
        log_info!("  Title:       {}", reword_result.pr_title);
        if let Some(ref desc) = reword_result.description {
            log_info!("  Description: {}", desc);
        } else {
            log_info!("  Description: (empty)");
        }

        // 预览模式：只显示结果，不更新
        if dry_run {
            log_success!("Dry run mode: PR will not be updated.");
            log_info!("Remove --dry-run flag to actually update the PR.");
            return Ok(());
        }

        // 确定要更新的内容
        // 逻辑：如果指定了标志，则更新对应内容；如果都不指定，则两者都更新（默认行为）
        // - 只指定 --title：只更新标题
        // - 只指定 --description：只更新描述
        // - 同时指定 --title --description：两者都更新
        // - 都不指定：两者都更新（默认行为）
        let update_title = !description || title;
        let update_body = !title || description;

        // 显示对比
        if update_title {
            log_info!("Title:");
            log_info!("  Current:  {}", current_title);
            log_info!("  New:      {}", reword_result.pr_title);
        }

        if update_body {
            log_info!("Description:");
            let current_preview = current_body.as_deref().unwrap_or("(empty)");
            // 生成新的完整 PR body（用于预览）
            let new_body_preview = Self::generate_new_pr_body(
                &reword_result,
                current_change_types.as_deref(),
                jira_ticket.as_deref(),
            )
            .unwrap_or_else(|e| {
                log_warning!("Failed to generate PR body preview: {}", e);
                reword_result.description.as_deref().unwrap_or("(empty)").to_string()
            });
            log_info!("  Current:  {}", current_preview);
            log_info!("  New:      {}", new_body_preview);
        }

        log_break!();

        // 确认更新
        let confirm_message = if update_title && update_body {
            format!("Update PR #{} with generated title and description?", pr_id)
        } else if update_title {
            format!(
                "Update PR #{} title to '{}'?",
                pr_id, reword_result.pr_title
            )
        } else {
            format!("Update PR #{} description?", pr_id)
        };

        let confirmed = ConfirmDialog::new(&confirm_message).with_default(true).prompt()?;

        if !confirmed {
            log_info!("Update cancelled.");
            return Ok(());
        }

        // 执行更新
        let new_title = if update_title {
            Some(reword_result.pr_title.as_str())
        } else {
            None
        };

        // 生成新的完整 PR body（如果需要更新）
        let new_body_string = if update_body {
            Some(Self::generate_new_pr_body(
                &reword_result,
                current_change_types.as_deref(),
                jira_ticket.as_deref(),
            )?)
        } else {
            None
        };

        let new_body = new_body_string.as_deref();

        Spinner::with("Updating PR...", || {
            provider.update_pull_request(&pr_id, new_title, new_body)
        })
        .context("Failed to update PR")?;

        log_break!();
        log_success!("PR #{} updated successfully!", pr_id);
        if update_title {
            log_success!("  Title:       {}", reword_result.pr_title);
        }
        if update_body {
            if let Some(ref desc) = reword_result.description {
                log_success!("  Description: {}", desc);
            }
        }

        // 显示 PR URL
        let pr_url = provider.get_pull_request_url(&pr_id)?;
        log_info!("  URL:         {}", pr_url);

        Ok(())
    }

    /// 生成新的完整 PR body
    ///
    /// 使用模板系统生成包含标题、change_types 和描述的完整 PR body。
    fn generate_new_pr_body(
        reword_result: &crate::pr::llm::PullRequestReword,
        current_change_types: Option<&[bool]>,
        jira_ticket: Option<&str>,
    ) -> Result<String> {
        use crate::pr::platform::TYPES_OF_CHANGES;

        // 使用当前 change_types，如果没有则默认都不选中
        let selected_types: Vec<bool> = if let Some(types) = current_change_types {
            log_debug!("Using parsed change_types: {:?}", types);
            types.to_vec()
        } else {
            log_debug!("No change_types found, using default (all false)");
            vec![false; TYPES_OF_CHANGES.len()]
        };

        // 获取 Jira 信息（如果存在）
        let jira_info = if let Some(ticket) = jira_ticket {
            Jira::get_ticket_info(ticket).ok()
        } else {
            None
        };

        // 使用 LLM 生成的描述作为 short_description
        let short_description = reword_result.description.as_deref();

        // 生成完整的 PR body
        generate_pull_request_body(
            &selected_types,
            short_description,
            jira_ticket,
            None, // dependency 暂时为空
            jira_info.as_ref(),
        )
        .context("Failed to generate PR body")
    }
}
