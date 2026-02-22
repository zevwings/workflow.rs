//! Pull 工具方法
//!
//! 提供带 SSH 保障和 stash 处理的 pull 操作。

use domain::GitError;
use prompt::{error, info};

use crate::bootstrap;
use crate::util::ensure_ssh_ready;

/// Pull 选项
#[derive(Default)]
pub struct PullOptions {
    /// 工作区有未提交更改时是否自动 stash（默认 true）
    pub auto_stash: bool,
    /// stash 时使用的消息
    pub stash_message: Option<&'static str>,
}

impl PullOptions {
    /// 默认选项：自动 stash，使用标准消息
    pub fn default_with_stash() -> Self {
        Self {
            auto_stash: true,
            stash_message: Some("Auto-stash before pull"),
        }
    }

    /// 不自动 stash（调用方已处理或确认工作区干净）
    pub fn no_stash() -> Self {
        Self {
            auto_stash: false,
            stash_message: None,
        }
    }
}

/// 执行带 SSH 保障的 pull 操作
///
/// 在执行 pull 前：
/// 1. 自动检查并确保 SSH 密钥就绪（仅当 origin 为 SSH 协议时）
/// 2. 若 `auto_stash` 为 true 且工作区有未提交更改，先 stash，pull 后恢复
///
/// 若 pull 因合并冲突失败，会输出解决冲突的指引并返回 `Err`。
///
/// # 参数
///
/// * `branch_name` - 要拉取的分支名
/// * `options` - pull 选项，可使用 `PullOptions::default_with_stash()` 或 `PullOptions::no_stash()`
///
/// # 错误
///
/// 返回 `Err` 当 SSH 保障失败、stash 失败、pull 失败（含合并冲突）或 stash 恢复失败时。
pub fn safe_pull(
    branch_name: &str,
    options: &PullOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    ensure_ssh_ready().map_err(|e| format!("{}", e))?;

    let git_repo = bootstrap::get_git_repository();

    let needs_stash = if options.auto_stash {
        let status = git_repo
            .get_working_tree_status()
            .map_err(|e| format!("Failed to check working tree status: {}", e))?;
        !status.is_clean()
    } else {
        false
    };

    if needs_stash {
        let msg = options.stash_message.unwrap_or("Auto-stash before pull");
        info!("Working tree has uncommitted changes, stashing...");
        git_repo
            .stash_push(Some(msg))
            .map_err(|e| format!("Failed to stash changes: {}", e))?;
    }

    let result = git_repo.pull(branch_name);

    if let Err(ref e) = result {
        if matches!(e, GitError::MergeConflict) {
            report_merge_conflict();
            return Err(format!("Pull failed: merge conflicts detected - {}", e).into());
        }
    }

    result
        .map_err(|e| -> Box<dyn std::error::Error> { format!("Failed to pull: {}", e).into() })?;

    if needs_stash {
        info!("Restoring stashed changes...");
        git_repo
            .stash_pop(0)
            .map_err(|e| format!("Failed to restore stashed changes: {}", e))?;
    }

    Ok(())
}

fn report_merge_conflict() {
    error!("Pull failed due to merge conflicts!");
    error!("Please resolve the conflicts manually:");
    info!("  1. Edit the conflicting files to resolve conflicts");
    info!("  2. Run 'git add <resolved-files>'");
    info!("  3. Run 'git commit' to complete the merge");
    info!("  Or run 'git merge --abort' to cancel the merge");
}
