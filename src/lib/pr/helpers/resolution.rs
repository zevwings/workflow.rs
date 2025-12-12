//! PR ID 解析相关辅助函数
//!
//! 提供解析和获取 PR ID 的函数。

use crate::git::{GitRepo, RepoType};
use crate::pr::platform::create_provider_auto;
use color_eyre::Result;

/// 获取当前分支的 PR ID
///
/// 这是一个便捷函数，专门用于获取当前分支的 PR ID。
///
/// # 返回
///
/// 返回当前分支的 PR ID（如果存在），否则返回 None
pub fn get_current_branch_pr_id() -> Result<Option<String>> {
    let provider = create_provider_auto()?;
    provider.get_current_branch_pull_request()
}

/// 解析 PR ID（从参数或当前分支）
///
/// 如果提供了 `pull_request_id`，直接返回。
/// 否则，尝试从当前分支自动检测 PR ID。
///
/// # 参数
///
/// * `pull_request_id` - 可选的 PR ID（从命令行参数传入）
///
/// # 返回
///
/// 返回解析后的 PR ID 字符串
///
/// # 错误
///
/// 如果无法自动检测 PR ID 且未提供参数，返回错误。
pub fn resolve_pull_request_id(pull_request_id: Option<String>) -> Result<String> {
    if let Some(id) = pull_request_id {
        return Ok(id);
    }

    let provider = create_provider_auto()?;
    match provider.get_current_branch_pull_request()? {
        Some(id) => Ok(id),
        None => {
            let repo_type = GitRepo::detect_repo_type()?;
            let error_msg = match repo_type {
                RepoType::GitHub => "No PR found for current branch. Please specify PR ID.",
                RepoType::Codeup | RepoType::Unknown => {
                    "Unsupported repository type. Only GitHub is currently supported."
                }
            };
            color_eyre::eyre::bail!("{}", error_msg);
        }
    }
}
