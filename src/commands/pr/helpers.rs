//! PR 命令辅助函数
//!
//! 提供 PR 命令之间共享的辅助函数，减少代码重复。

use crate::base::dialog::{ConfirmDialog, InputDialog, MultiSelectDialog};
use crate::base::indicator::Spinner;
use crate::base::util::{Browser, Clipboard};
use crate::git::{GitBranch, GitCommit, GitRepo, GitStash};
use crate::jira::status::JiraStatus;
use crate::jira::Jira;
use crate::jira::JiraWorkHistory;
use crate::pr::helpers::{extract_pull_request_id_from_url, get_current_branch_pr_id};
use crate::pr::{create_provider, TYPES_OF_CHANGES};
use crate::{log_break, log_info, log_success, log_warning};
use anyhow::{Context, Error, Result};

/// 处理 stash_pop 的结果
///
/// 统一处理 `GitStash::stash_pop()` 的返回结果，显示消息和警告。
///
/// # 参数
///
/// * `result` - `stash_pop()` 的返回结果
pub fn handle_stash_pop_result(result: Result<crate::git::StashPopResult>) {
    match result {
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

/// 检查错误是否表示 PR 已合并
///
/// 这是一个备用检查，用于处理以下情况：
/// 1. 状态检查失败（网络问题等）
/// 2. 竞态条件：在状态检查和实际合并之间，PR 被其他进程合并了
///
/// # 参数
///
/// * `error` - 要检查的错误
///
/// # 返回
///
/// 如果错误表示 PR 已合并，返回 `true`，否则返回 `false`
#[allow(dead_code)]
pub fn is_pr_already_merged_error(error: &Error) -> bool {
    let error_msg = error.to_string().to_lowercase();

    // 优先检查明确的错误消息
    if error_msg.contains("already been merged")
        || error_msg.contains("pull request has already been merged")
        || error_msg.contains("not mergeable")
    {
        return true;
    }

    // 检查 HTTP 状态码（需要结合错误消息，避免误判）
    // 405 (Method Not Allowed) - 某些 API 在 PR 已合并时返回此状态码
    // 422 (Unprocessable Entity) - GitHub API 在 PR 已合并时可能返回此状态码
    // 但需要确保错误消息中包含 merge 相关的内容，避免误判其他错误
    if error_msg.contains("405") && error_msg.contains("merge") {
        return true;
    }
    if error_msg.contains("422") && error_msg.contains("merge") {
        return true;
    }

    false
}

/// 检查错误是否表示 PR 已关闭
///
/// 这是一个备用检查，用于处理以下情况：
/// 1. 状态检查失败（网络问题等）
/// 2. 竞态条件：在状态检查和实际关闭之间，PR 被其他进程关闭了
///
/// # 参数
///
/// * `error` - 要检查的错误
///
/// # 返回
///
/// 如果错误表示 PR 已关闭，返回 `true`，否则返回 `false`
#[allow(dead_code)]
pub fn is_pr_already_closed_error(error: &Error) -> bool {
    let error_msg = error.to_string().to_lowercase();

    // 优先检查明确的错误消息
    if error_msg.contains("already been closed")
        || error_msg.contains("pull request has already been closed")
        || error_msg.contains("is already closed")
        || error_msg.contains("state is closed")
    {
        return true;
    }

    // 检查 HTTP 状态码（需要结合错误消息，避免误判）
    // 422 (Unprocessable Entity) - GitHub API 在 PR 已关闭时可能返回此状态码
    // 但需要确保错误消息中包含 close 相关的内容，避免误判其他错误
    if error_msg.contains("422") && (error_msg.contains("close") || error_msg.contains("closed")) {
        return true;
    }

    false
}

/// 通用的分支清理逻辑
///
/// 在 PR 操作（合并、关闭等）后，切换到默认分支并删除当前分支。
///
/// # 参数
///
/// * `current_branch` - 当前分支名称
/// * `default_branch` - 默认分支名称（通常是 main 或 master）
/// * `operation_name` - 操作名称（用于日志消息，如 "PR merge" 或 "PR close"）
///
/// # 流程
///
/// 1. 检查是否在默认分支（如果是，直接返回）
/// 2. 更新远程分支信息 (fetch)
/// 3. 检查并 stash 未提交的更改
/// 4. 切换到默认分支
/// 5. 拉取最新代码
/// 6. 删除本地分支
/// 7. 恢复 stash
/// 8. 清理远程分支引用 (prune)
///
/// # 错误
///
/// 如果任何步骤失败，返回相应的错误信息。
pub fn cleanup_branch(
    current_branch: &str,
    default_branch: &str,
    operation_name: &str,
) -> Result<()> {
    // 如果当前分支已经是默认分支，不需要清理
    if current_branch == default_branch {
        log_info!("Already on default branch: {}", default_branch);
        return Ok(());
    }

    log_info!("Switching to default branch: {}", default_branch);

    // 1. 更新远程分支信息
    GitRepo::fetch()?;

    // 2. 检查并 stash 未提交的更改
    let has_stashed = GitCommit::has_commit()?;
    if has_stashed {
        log_info!("Stashing local changes before switching branches...");
        GitStash::stash_push(Some(&format!(
            "Auto-stash before {} cleanup",
            operation_name
        )))?;
    }

    // 3. 切换到默认分支
    GitBranch::checkout_branch(default_branch)
        .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

    // 4. 更新本地默认分支
    GitBranch::pull(default_branch)
        .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

    // 5. 删除本地分支
    if GitBranch::has_local_branch(current_branch)? {
        log_info!("Deleting local branch: {}", current_branch);
        GitBranch::delete(current_branch, false)
            .or_else(|_| {
                log_info!("Branch may not be fully merged, trying force delete...");
                GitBranch::delete(current_branch, true)
            })
            .context("Failed to delete local branch")?;
        log_success!("Local branch deleted: {}", current_branch);
    } else {
        log_info!("Local branch already deleted: {}", current_branch);
    }

    // 6. 恢复 stash
    if has_stashed {
        log_info!("Restoring stashed changes...");
        handle_stash_pop_result(GitStash::stash_pop());
    }

    // 7. 清理远程分支引用
    if let Err(e) = GitRepo::prune_remote() {
        log_info!("Warning: Failed to prune remote references: {}", e);
        log_info!("This is a non-critical cleanup operation. Local cleanup is complete.");
    }

    log_success!(
        "Cleanup completed: switched to {} and deleted local branch {}",
        default_branch,
        current_branch
    );

    Ok(())
}

// apply_branch_name_prefixes has been moved to lib/branch module
// Use branch::BranchPrefix::apply() instead

/// Detect which branch a given branch might be based on
///
/// By checking all branches, find the branch that the given branch might be directly based on.
/// If a base branch is detected, return its name.
///
/// # Arguments
///
/// * `branch` - The branch name to detect
/// * `exclude_branch` - The branch to exclude from detection (usually the target branch)
///
/// # Returns
///
/// Returns `Some(base_branch_name)` if a base branch is detected, otherwise returns `None`.
///
/// # Examples
///
/// ```no_run
/// use workflow::commands::pr::helpers::detect_base_branch;
///
/// // Detect which branch test-rebase is based on (excluding master)
/// let base = detect_base_branch("test-rebase", "master")?;
/// // May return: Some("develop-")
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn detect_base_branch(branch: &str, exclude_branch: &str) -> Result<Option<String>> {
    log_info!("Detecting base branch for '{}'...", branch);

    // Get all branches (excluding branch and exclude_branch)
    let all_branches = GitBranch::get_all_branches(false)
        .context("Failed to get all branches for base branch detection")?;

    // Sort by priority: check common base branches first
    let mut candidate_branches: Vec<String> = all_branches
        .into_iter()
        .filter(|b| b != branch && b != exclude_branch)
        .collect();

    // Prioritize checking common base branch names (develop, dev, staging, etc.)
    let common_base_branches = ["develop", "dev", "staging", "test"];
    candidate_branches.sort_by(|a, b| {
        let a_priority = common_base_branches
            .iter()
            .position(|&name| a == name || a.ends_with(&format!("/{}", name)))
            .unwrap_or(usize::MAX);
        let b_priority = common_base_branches
            .iter()
            .position(|&name| b == name || b.ends_with(&format!("/{}", name)))
            .unwrap_or(usize::MAX);
        a_priority.cmp(&b_priority)
    });

    // Check each candidate branch
    for candidate in &candidate_branches {
        match GitBranch::is_branch_based_on(branch, candidate) {
            Ok(true) => {
                log_success!(
                    "Detected that '{}' is likely based on '{}'",
                    branch,
                    candidate
                );
                return Ok(Some(candidate.clone()));
            }
            Ok(false) => {
                // Continue checking next branch
            }
            Err(e) => {
                // Check failed, log warning but continue
                log_warning!(
                    "Failed to check if '{}' is based on '{}': {}",
                    branch,
                    candidate,
                    e
                );
            }
        }
    }

    log_info!("No base branch detected for '{}'", branch);
    Ok(None)
}

/// 配置 Jira ticket 状态
///
/// 如果有 Jira ticket，检查并配置状态。如果已配置则读取，否则进行交互式配置。
///
/// # 参数
///
/// * `jira_ticket` - Jira ticket ID（可选）
///
/// # 返回
///
/// 返回配置的 PR 创建状态（如果有），否则返回 `None`。
pub fn ensure_jira_status(jira_ticket: &Option<String>) -> Result<Option<String>> {
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
            let config_result = JiraStatus::configure_interactive(ticket)
                .with_context(|| {
                    format!(
                        "Failed to configure Jira ticket '{}'. Please ensure it's a valid ticket ID (e.g., PROJECT-123). If you don't need Jira integration, leave this field empty.",
                        ticket
                    )
                })?;
            log_success!("Jira status configuration saved");
            log_info!(
                "  PR created status: {}",
                config_result.created_pull_request_status
            );
            log_info!(
                "  PR merged status: {}",
                config_result.merged_pull_request_status
            );
            Ok(Some(config_result.created_pull_request_status))
        }
    } else {
        Ok(None)
    }
}

/// 提示用户输入 PR 标题
///
/// 使用 Input 组件提示用户输入 PR 标题，并验证输入不能为空。
///
/// # 返回
///
/// 返回用户输入的 PR 标题。
pub fn input_pull_request_title() -> Result<String> {
    let title = InputDialog::new("PR title (required)")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Err("PR title is required and cannot be empty".to_string())
            } else {
                Ok(())
            }
        })
        .prompt()
        .context("Failed to get PR title")?;
    Ok(title)
}

/// 获取 PR 描述
///
/// 如果提供了描述，直接使用；否则提示用户输入描述（可选）。
///
/// # 参数
///
/// * `description` - 可选的描述字符串
///
/// # 返回
///
/// 返回描述字符串（可能为空）。
pub fn resolve_description(description: Option<String>) -> Result<String> {
    if let Some(desc) = description {
        Ok(desc)
    } else {
        let desc = InputDialog::new("Short description (optional)")
            .allow_empty(true)
            .prompt()
            .context("Failed to get description")?;
        Ok(desc)
    }
}

/// 选择变更类型（手动选择）
///
/// 提示用户手动选择变更类型，返回一个布尔向量，表示每个类型是否被选中。
///
/// # 返回
///
/// 返回布尔向量，表示每个 PR 变更类型是否被选中。
pub fn select_change_types() -> Result<Vec<bool>> {
    log_info!("Types of changes:");
    let options: Vec<&str> = TYPES_OF_CHANGES.to_vec();
    let selected_items = MultiSelectDialog::new(
        "Select change types (use space to select, enter to confirm)",
        options,
    )
    .prompt()
    .context("Failed to select change types")?;

    // 转换选中的项为布尔向量
    let selected_types: Vec<bool> =
        TYPES_OF_CHANGES.iter().map(|&item| selected_items.contains(&item)).collect();

    Ok(selected_types)
}

/// 在默认分支上创建新分支并提交
///
/// 当用户在默认分支上有未提交修改时，直接创建新分支
/// （Git 会自动把未提交的修改带到新分支），然后提交并推送。
///
/// # 参数
///
/// * `branch_name` - 要创建的新分支名称
/// * `commit_title` - 提交标题
/// * `default_branch` - 默认分支名称
///
/// # 返回
///
/// 返回实际使用的分支名和默认分支名的元组。
///
/// # 流程
///
/// 1. 创建新分支（未提交的修改会自动带到新分支）
/// 2. 提交更改
/// 3. 推送到远程
pub fn create_branch_from_default(
    branch_name: &str,
    commit_title: &str,
    default_branch: &str,
) -> Result<(String, String)> {
    log_info!(
        "You are on default branch '{}' with uncommitted changes.",
        default_branch
    );
    log_info!("Will create new branch and commit changes...");

    // 直接创建新分支（Git 会自动把未提交的修改带到新分支）
    log_success!("Creating branch: {}", branch_name);
    GitBranch::checkout_branch(branch_name)?;

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

/// 复制 PR URL 到剪贴板并在浏览器中打开
///
/// 复制 PR URL 到剪贴板并在浏览器中打开。
///
/// # 参数
///
/// * `pull_request_url` - PR URL
pub fn copy_and_open_pull_request(pull_request_url: &str) -> Result<()> {
    // 复制 PR URL 到剪贴板
    Clipboard::copy(pull_request_url)?;
    log_success!("Copied {} to clipboard", pull_request_url);

    // 打开浏览器
    std::thread::sleep(std::time::Duration::from_secs(1));
    Browser::open(pull_request_url)?;

    Ok(())
}

/// 获取或生成 PR 标题
///
/// 如果提供了标题，根据 `confirm_if_provided` 参数决定是否询问用户确认使用；
/// 如果有 Jira ticket，尝试从 Jira 获取标题；否则提示用户手动输入。
///
/// # 参数
///
/// * `title` - 可选的标题字符串
/// * `jira_ticket` - 可选的 Jira ticket ID
/// * `confirm_if_provided` - 如果提供了标题，是否询问用户确认使用
///
/// # 返回
///
/// 返回 PR 标题字符串。
pub fn resolve_title(
    title: Option<String>,
    jira_ticket: &Option<String>,
    confirm_if_provided: bool,
) -> Result<String> {
    // 如果有提供的标题，根据参数决定是否询问确认
    if let Some(t) = title {
        if confirm_if_provided {
            log_success!("Using source PR title: {}", t);
            let use_source = ConfirmDialog::new(format!("Use source PR title: '{}'?", t))
                .with_default(true)
                .prompt()?;
            if use_source {
                return Ok(t);
            }
            // 用户选择不使用，继续后续逻辑
        } else {
            // 不需要确认，直接使用
            return Ok(t);
        }
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

    // 回退到手动输入
    input_pull_request_title()
}

/// 创建或获取 PR
///
/// 检查分支是否已有 PR，如果有则获取 PR URL，否则创建新 PR。
/// 在创建 PR 前会检查分支是否有提交。
///
/// # 参数
///
/// * `branch_name` - 分支名称
/// * `default_branch` - 默认分支名称
/// * `pr_title` - PR 标题
/// * `pull_request_body` - PR body
///
/// # 返回
///
/// 返回 PR URL。
pub fn create_or_get_pull_request(
    branch_name: &str,
    default_branch: &str,
    pr_title: &str,
    pull_request_body: &str,
) -> Result<String> {
    // 检查分支是否已有 PR
    let existing_pr = get_current_branch_pr_id()?;

    if let Some(pr_id) = existing_pr {
        log_info!("PR #{} already exists for branch '{}'", pr_id, branch_name);
        let provider = create_provider()?;
        provider.get_pull_request_url(&pr_id)
    } else {
        // 分支无 PR，创建新 PR
        // 先检查分支是否有提交（相对于默认分支）
        let has_commits = GitBranch::is_branch_ahead(branch_name, default_branch)
            .context("Failed to check branch commits")?;

        if !has_commits {
            log_warning!(
                "Branch '{}' has no commits compared to '{}'.",
                branch_name,
                default_branch
            );
            log_warning!("GitHub does not allow creating PRs for empty branches.");
            log_warning!("Please make some changes and commit them before creating a PR.");
            anyhow::bail!(
                "Cannot create PR: branch '{}' has no commits. Please commit changes first.",
                branch_name
            );
        }

        let provider = create_provider()?;
        let pull_request_url = Spinner::with("Creating PR...", || {
            provider.create_pull_request(pr_title, pull_request_body, branch_name, None)
        })?;

        log_success!("PR created: {}", pull_request_url);
        Ok(pull_request_url)
    }
}

/// 更新 Jira ticket
///
/// 如果有 Jira ticket 和状态配置，更新 ticket：
/// - 分配任务
/// - 更新状态
/// - 添加评论（PR URL）
/// - 写入历史记录
///
/// # 参数
///
/// * `jira_ticket` - 可选的 Jira ticket ID
/// * `created_pull_request_status` - 可选的 PR 创建状态
/// * `pull_request_url` - PR URL
/// * `branch_name` - 分支名称
pub fn update_jira_ticket(
    jira_ticket: &Option<String>,
    created_pull_request_status: &Option<String>,
    pull_request_url: &str,
    branch_name: &str,
) -> Result<()> {
    if let Some(ref ticket) = jira_ticket {
        if let Some(ref status) = created_pull_request_status {
            Spinner::with("Updating Jira ticket...", || -> Result<()> {
                // 分配任务
                Jira::assign_ticket(ticket, None)?;
                // 更新状态
                Jira::move_ticket(ticket, status)?;
                // 添加评论（PR URL）
                Jira::add_comment(ticket, pull_request_url)?;
                Ok(())
            })?;

            // 写入历史记录
            let pull_request_id = extract_pull_request_id_from_url(pull_request_url)?;
            let repository = GitRepo::get_remote_url().ok();
            JiraWorkHistory::write_work_history(
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
