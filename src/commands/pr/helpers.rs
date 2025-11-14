//! PR 命令辅助函数
//!
//! 提供 PR 命令之间共享的辅助函数，减少代码重复。

use crate::{log_info, log_success, Git};
use crate::{Codeup, GitHub, PlatformProvider, RepoType};
use anyhow::{Context, Error, Result};

/// 解析 PR ID（从参数或当前分支）
///
/// 如果提供了 `pull_request_id`，直接返回。
/// 否则，尝试从当前分支自动检测 PR ID。
///
/// # 参数
///
/// * `pull_request_id` - 可选的 PR ID（从命令行参数传入）
/// * `repo_type` - 仓库类型
///
/// # 返回
///
/// 返回解析后的 PR ID 字符串
///
/// # 错误
///
/// 如果无法自动检测 PR ID 且未提供参数，返回错误。
///
#[allow(dead_code)]
pub fn resolve_pull_request_id(
    pull_request_id: Option<String>,
    repo_type: &RepoType,
) -> Result<String> {
    if let Some(id) = pull_request_id {
        return Ok(id);
    }

    // 从当前分支获取 PR
    let pr_id = match repo_type {
        RepoType::GitHub => GitHub::get_current_branch_pull_request()?,
        RepoType::Codeup => Codeup::get_current_branch_pull_request()?,
        _ => {
            anyhow::bail!(
                "Auto-detection of PR ID is only supported for GitHub and Codeup repositories."
            );
        }
    };

    match pr_id {
        Some(id) => {
            log_success!("Found PR for current branch: #{}", id);
            Ok(id)
        }
        None => {
            let error_msg = match repo_type {
                RepoType::GitHub => "No PR found for current branch. Please specify PR ID.",
                RepoType::Codeup => {
                    "No PR found for current branch. Please specify PR ID or branch name."
                }
                _ => "No PR found for current branch.",
            };
            anyhow::bail!("{}", error_msg);
        }
    }
}

/// 从当前分支获取 PR ID
///
/// 这是 `resolve_pull_request_id(None, repo_type)` 的便捷方法。
///
/// # 参数
///
/// * `repo_type` - 仓库类型
///
/// # 返回
///
/// 返回当前分支对应的 PR ID
///
#[allow(dead_code)]
pub fn get_current_branch_pull_request(repo_type: &RepoType) -> Result<String> {
    resolve_pull_request_id(None, repo_type)
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
    Git::fetch()?;

    // 2. 检查并 stash 未提交的更改
    let has_stashed = Git::has_commit()?;
    if has_stashed {
        log_info!("Stashing local changes before switching branches...");
        Git::stash_push(Some(&format!(
            "Auto-stash before {} cleanup",
            operation_name
        )))?;
    }

    // 3. 切换到默认分支
    Git::checkout_branch(default_branch)
        .with_context(|| format!("Failed to checkout default branch: {}", default_branch))?;

    // 4. 更新本地默认分支
    Git::pull(default_branch)
        .with_context(|| format!("Failed to pull latest changes from {}", default_branch))?;

    // 5. 删除本地分支
    if Git::has_local_branch(current_branch)? {
        log_info!("Deleting local branch: {}", current_branch);
        Git::delete(current_branch, false)
            .or_else(|_| {
                log_info!("Branch may not be fully merged, trying force delete...");
                Git::delete(current_branch, true)
            })
            .context("Failed to delete local branch")?;
        log_success!("Local branch deleted: {}", current_branch);
    } else {
        log_info!("Local branch already deleted: {}", current_branch);
    }

    // 6. 恢复 stash
    if has_stashed {
        log_info!("Restoring stashed changes...");
        let _ = Git::stash_pop(); // 日志已在 stash_pop 中处理
    }

    // 7. 清理远程分支引用
    if let Err(e) = Git::prune_remote() {
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
