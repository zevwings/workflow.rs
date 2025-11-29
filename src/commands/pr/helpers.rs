//! PR 命令辅助函数
//!
//! 提供 PR 命令之间共享的辅助函数，减少代码重复。

use crate::base::settings::settings::Settings;
use crate::git::{GitBranch, GitCommit, GitRepo, GitStash};
use crate::{log_info, log_success, log_warning};
use anyhow::{Context, Error, Result};

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
        let _ = GitStash::stash_pop(); // 日志已在 stash_pop 中处理
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

/// 应用分支名前缀（Jira ticket 和 github_branch_prefix）
///
/// 统一处理分支名前缀逻辑，避免代码重复。
/// 先添加 Jira ticket 前缀（如果提供），然后添加 GitHub 分支前缀（如果配置）。
///
/// # 参数
///
/// * `branch_name` - 基础分支名
/// * `jira_ticket` - Jira ticket ID（可选）
///
/// # 返回
///
/// 应用前缀后的完整分支名
///
/// # 示例
///
/// ```
/// use crate::commands::pr::helpers::apply_branch_name_prefixes;
///
/// // 只有基础分支名
/// let name = apply_branch_name_prefixes("feature-branch".to_string(), None)?;
/// // 返回: "feature-branch"
///
/// // 带 Jira ticket
/// let name = apply_branch_name_prefixes("feature-branch".to_string(), Some("PROJ-123"))?;
/// // 返回: "PROJ-123-feature-branch"
///
/// // 带 Jira ticket 和 GitHub 前缀
/// // 假设配置了 github_branch_prefix = "user"
/// let name = apply_branch_name_prefixes("feature-branch".to_string(), Some("PROJ-123"))?;
/// // 返回: "user/PROJ-123-feature-branch"
/// ```
pub fn apply_branch_name_prefixes(
    mut branch_name: String,
    jira_ticket: Option<&str>,
) -> Result<String> {
    // 如果有 Jira ticket，添加到分支名前缀
    if let Some(ticket) = jira_ticket {
        branch_name = format!("{}-{}", ticket, branch_name);
    }

    // 如果有 GITHUB_BRANCH_PREFIX，添加前缀
    let settings = Settings::get();
    if let Some(prefix) = settings.github.get_current_branch_prefix() {
        let trimmed = prefix.trim();
        if !trimmed.is_empty() {
            branch_name = format!("{}/{}", trimmed, branch_name);
        }
    }

    Ok(branch_name)
}

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
/// use crate::commands::pr::helpers::detect_base_branch;
///
/// // Detect which branch test-rebase is based on (excluding master)
/// let base = detect_base_branch("test-rebase", "master")?;
/// // May return: Some("develop-")
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
