use anyhow::{Context, Result};
use duct::cmd;

use super::commit::Git;

impl Git {
    /// 保存未提交的修改到 stash
    pub fn stash_push(message: Option<&str>) -> Result<()> {
        let mut args = vec!["stash", "push"];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }
        cmd("git", &args)
            .run()
            .context("Failed to stash changes")?;
        Ok(())
    }

    /// 恢复 stash 中的修改
    ///
    /// 如果遇到合并冲突，会保留 stash entry 并返回错误，提示用户手动解决冲突
    pub fn stash_pop() -> Result<()> {
        use crate::log_warning;

        // 尝试执行 git stash pop
        let result = cmd("git", &["stash", "pop"]).run();

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // 检查是否有未合并的路径（冲突文件）
                if Self::has_unmerged()? {
                    log_warning!("Merge conflicts detected when restoring stashed changes.");
                    log_warning!("The stash entry is kept in case you need it again.");
                    log_warning!("Please resolve the conflicts manually and then:");
                    log_warning!("  1. Resolve conflicts in the affected files");
                    log_warning!("  2. Stage the resolved files with: git add <file>");
                    log_warning!("  3. Continue with your workflow");
                    anyhow::bail!(
                        "Failed to pop stash due to merge conflicts. Please resolve conflicts manually."
                    );
                } else {
                    // 没有冲突但失败了，返回原始错误
                    Err(e).context("Failed to pop stash")
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
        let output = cmd("git", &["ls-files", "-u"])
            .read()
            .context("Failed to check unmerged files")?;

        Ok(!output.trim().is_empty())
    }
}
