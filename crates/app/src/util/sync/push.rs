//! Push 工具方法
//!
//! 提供带 SSH 保障的 push 操作。

use prompt::{spinner, success};

use crate::bootstrap;
use crate::util::ensure_ssh_ready;

/// 执行带 SSH 保障的 push 操作
///
/// 在执行 push 前自动检查并确保 SSH 密钥就绪（仅当 origin 为 SSH 协议时），
/// 并在 push 过程中显示 loading 提示。调用方无需再包裹 spinner。
///
/// # 参数
///
/// * `branch_name` - 要推送的分支名
/// * `set_upstream` - 是否设置上游跟踪分支
///
/// # 错误
///
/// 返回 `Err` 当 SSH 保障失败或 push 失败时。
pub fn safe_push(
    branch_name: Option<String>,
    set_upstream: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_ssh_ready().map_err(|e| format!("{}", e))?;

    let git_repo = bootstrap::get_git_repository();

    let branch_to_push = if let Some(name) = branch_name {
        name
    } else {
        git_repo.get_current_branch()?
    };

    spinner!("Pushing branch '{}' to remote...", branch_to_push)
        .with(|| git_repo.push(&branch_to_push, set_upstream))
        .map_err(|e| format!("Failed to push branch: {}", e))?;

    success!("Pushed branch '{}' to remote", branch_to_push);

    Ok(())
}
