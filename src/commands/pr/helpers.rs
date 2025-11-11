//! PR 命令辅助函数
//!
//! 提供 PR 命令之间共享的辅助函数，减少代码重复。

use crate::{log_success, Codeup, GitHub, PlatformProvider, RepoType};
use anyhow::Result;

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
        RepoType::Codeup => <Codeup as PlatformProvider>::get_current_branch_pull_request()?,
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
