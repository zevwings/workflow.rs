use anyhow::{Context, Result};

use super::helpers::{cmd_read, cmd_run};
use crate::trace_warn;

/// Stash 恢复结果
#[derive(Debug, Clone)]
pub struct StashPopResult {
    /// 是否成功恢复
    pub restored: bool,
    /// 消息
    pub message: Option<String>,
    /// 警告消息列表
    pub warnings: Vec<String>,
}

/// Git Stash 管理
///
/// 提供 stash 相关的操作功能，包括：
/// - 保存未提交的修改到 stash
/// - 恢复 stash 中的修改
/// - 检查是否有未合并的文件（冲突）
pub struct GitStash;

impl GitStash {
    /// 保存未提交的修改到 stash
    ///
    /// 使用 `git stash push` 将当前工作区和暂存区的未提交修改保存到 stash。
    /// 如果提供了消息，则使用 `-m` 选项添加 stash 消息。
    ///
    /// # 参数
    ///
    /// * `message` - 可选的 stash 消息，用于标识这次 stash 的内容
    ///
    /// # 错误
    ///
    /// 如果 stash 操作失败，返回相应的错误信息。
    pub fn stash_push(message: Option<&str>) -> Result<()> {
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }
        cmd_run(&args).context("Failed to stash changes")
    }

    /// 恢复 stash 中的修改
    ///
    /// 使用 `git stash pop` 恢复最近一次 stash 中的修改。
    /// 如果遇到合并冲突或其他错误，会返回包含警告信息的结果，而不是抛出错误。
    ///
    /// # 行为
    ///
    /// 1. 尝试执行 `git stash pop` 恢复修改
    /// 2. 如果成功，返回成功结果
    /// 3. 如果失败，检查是否有未合并的文件（冲突）
    /// 4. 如果有冲突，返回包含警告信息的结果，保留 stash entry
    /// 5. 如果没有冲突但失败，返回包含警告信息的结果
    ///
    /// # 返回
    ///
    /// 返回 `StashPopResult`，包含恢复状态、消息和警告信息。
    /// 即使遇到错误，也会返回结果而不是抛出错误，调用者需要检查 `restored` 字段和 `warnings` 字段。
    pub fn stash_pop() -> Result<StashPopResult> {
        // 尝试执行 git stash pop
        let result = cmd_run(&["stash", "pop"]);

        match result {
            Ok(_) => Ok(StashPopResult {
                restored: true,
                message: Some("Stashed changes restored".to_string()),
                warnings: vec![],
            }),
            Err(e) => {
                // 检查是否有未合并的路径（冲突文件）
                if Self::has_unmerged().unwrap_or(false) {
                    let warnings = vec![
                        "Merge conflicts detected when restoring stashed changes.".to_string(),
                        "The stash entry is kept in case you need it again.".to_string(),
                        "Please resolve the conflicts manually and then:".to_string(),
                        "  1. Resolve conflicts in the affected files".to_string(),
                        "  2. Stage the resolved files with: git add <file>".to_string(),
                        "  3. Continue with your workflow".to_string(),
                    ];
                    // 记录到 tracing（用于调试）
                    for warning in &warnings {
                        trace_warn!("{}", warning);
                    }
                    // 返回包含警告的结果，而不是抛出错误
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                } else {
                    // 没有冲突但失败了，返回包含警告的结果
                    let warnings = vec![
                        format!("Failed to restore stashed changes: {}", e),
                        "You can manually restore them with: git stash pop".to_string(),
                    ];
                    // 记录到 tracing（用于调试）
                    for warning in &warnings {
                        trace_warn!("{}", warning);
                    }
                    // 返回包含警告的结果，而不是抛出错误
                    Ok(StashPopResult {
                        restored: false,
                        message: None,
                        warnings,
                    })
                }
            }
        }
    }

    /// 检查是否有未合并的文件（冲突文件）
    ///
    /// 使用 `git ls-files -u` 检查是否有未合并的路径
    /// 返回 true 如果有冲突文件，false 如果没有
    pub fn has_unmerged() -> Result<bool> {
        // 使用 git ls-files -u 检查是否有未合并的路径
        // -u 选项：显示未合并的文件
        let output = cmd_read(&["ls-files", "-u"]).context("Failed to check unmerged files")?;

        Ok(!output.trim().is_empty())
    }
}
