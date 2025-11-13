//! PR 命令辅助函数
//!
//! 提供 PR 命令之间共享的辅助函数，减少代码重复。

use crate::{log_success, Codeup, GitHub, PlatformProvider, RepoType};
use anyhow::{Error, Result};

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
        RepoType::GitHub => <GitHub as PlatformProvider>::get_current_branch_pull_request()?,
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
