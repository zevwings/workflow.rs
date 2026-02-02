//! Commit amend 命令实现
//!
//! Amend 操作说明：
//! 1. 获取当前 HEAD commit 的信息（包括原提交消息）
//! 2. 从索引（暂存区）创建新的 tree（包含所有暂存的文件）
//! 3. 创建新的 commit，使用新的 tree 和原 commit 的 parent
//! 4. 更新分支引用指向新的 commit
//!
//! 这意味着它会将当前暂存区的所有更改添加到上次提交中。

use prompt::{error, info, input};

use crate::registry;

/// Commit Amend 命令
pub struct CommitAmendCommand {
    message: Option<String>,
    no_edit: bool,
    verify: bool,
}

impl CommitAmendCommand {
    /// 创建新的 CommitAmendCommand
    ///
    /// # 参数
    ///
    /// * `message` - 新的提交消息
    /// * `no_edit` - 是否不编辑提交消息
    /// * `verify` - 是否启用 pre-commit hooks（默认 false，即跳过）
    pub fn new(message: Option<String>, no_edit: bool, verify: bool) -> Self {
        Self {
            message,
            no_edit,
            verify,
        }
    }

    /// 运行 `workflow commit amend` 命令
    ///
    /// 默认跳过 pre-commit hooks，除非用户显式指定 `--verify`。
    /// 如果没有提供 `-m` 且没有 `--no-edit`，会交互式输入新消息（默认值为原消息）。
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let git_repo = registry::get_git_repository();

        // 确定是否跳过 pre-commit
        // - 如果指定了 --verify，则不跳过（no_verify = false）
        // - 如果未指定 --verify，则跳过（no_verify = true，默认行为）
        let no_verify = !self.verify;

        // 确定提交消息
        let message = if self.no_edit {
            // --no-edit: 保留原消息，不交互式输入
            None
        } else if let Some(ref msg) = self.message {
            // -m 参数: 使用提供的消息
            Some(msg.clone())
        } else {
            // 没有 -m 且没有 --no-edit: 交互式输入，默认值为原消息
            // 获取 HEAD commit 的信息（使用符号引用 "HEAD"）
            let commit_info = git_repo
                .get_commit_info("HEAD")
                .map_err(|e| format!("Failed to get commit info: {}", e))?;

            // 交互式输入新消息，默认值为原消息
            let new_message = input!("Enter commit message:")
                .default(&commit_info.message)
                .prompt()
                .map_err(|e| format!("Failed to get commit message: {}", e))?;

            Some(new_message)
        };

        match git_repo.amend_commit(message.as_deref(), self.no_edit, no_verify) {
            Ok(sha) => {
                info!("Commit amended successfully: {}", sha);
                Ok(())
            }
            Err(e) => {
                error!("Failed to amend commit: {}", e);
                Err(format!("Failed to amend commit: {}", e).into())
            }
        }
    }
}
