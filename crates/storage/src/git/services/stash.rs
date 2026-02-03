//! Stash 业务逻辑服务
//!
//! 提供 Git stash 相关的业务逻辑实现。

use chrono::{DateTime, Local, TimeZone};

use super::GitContext;
use domain::git::{GitError, StashApplyResult, StashEntry, StashPopResult, StashStat};

/// Stash 服务接口
pub trait StashService: Send + Sync {
    /// 创建 stash
    ///
    /// # 参数
    /// - `message`: 可选的 stash 消息
    ///
    /// # 返回
    /// 返回创建的 stash 的索引（0 表示最新的 stash）
    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError>;

    /// 应用并删除 stash
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 返回 `StashPopResult`，包含恢复状态、消息和警告信息
    fn stash_pop(&self, index: usize) -> Result<StashPopResult, GitError>;

    /// 应用 stash（不删除）
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 返回 `StashApplyResult`，包含应用状态、冲突信息和警告
    fn stash_apply(&self, index: usize) -> Result<StashApplyResult, GitError>;

    /// 列出所有 stash 条目
    ///
    /// # 返回
    /// 返回所有 stash 条目的列表，按索引从新到旧排列（stash@{0} 在第一个）
    fn stash_list(&self) -> Result<Vec<StashEntry>, GitError>;

    /// 删除指定的 stash
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 成功返回 Ok(())
    fn stash_drop(&self, index: usize) -> Result<(), GitError>;

    /// 检查是否有未合并的文件（冲突文件）
    fn has_unmerged(&self) -> Result<bool, GitError>;
}

/// Stash 服务实现
pub struct StashServiceImpl {
    ctx: GitContext,
}

impl StashServiceImpl {
    /// 创建新的 Stash 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }

    /// 从 stash 完整消息中提取分支名和消息
    ///
    /// stash 消息格式：
    /// - `WIP on <branch>: <message>`
    /// - `On <branch>: <message>`
    fn extract_branch_and_message(full_message: &str) -> (String, String) {
        // 尝试匹配 "WIP on <branch>: " 或 "On <branch>: "
        if let Some(pos) = full_message.find("WIP on ") {
            let after_wip = &full_message[pos + 7..]; // "WIP on " 的长度是 7
            if let Some(colon_pos) = after_wip.find(": ") {
                let branch = after_wip[..colon_pos].to_string();
                let message = after_wip[colon_pos + 2..].to_string();
                return (branch, message);
            }
        } else if let Some(pos) = full_message.find("On ") {
            let after_on = &full_message[pos + 3..]; // "On " 的长度是 3
            if let Some(colon_pos) = after_on.find(": ") {
                let branch = after_on[..colon_pos].to_string();
                let message = after_on[colon_pos + 2..].to_string();
                return (branch, message);
            }
        }

        // 如果无法提取，返回整个消息作为消息，分支为 unknown
        ("unknown".to_string(), full_message.to_string())
    }

    /// 获取 stash 的统计信息
    #[allow(dead_code)]
    fn get_stash_stat(&self, stash_commit: &git2::Commit) -> Option<StashStat> {
        let repo = self.ctx.repository();

        // 获取 stash commit 的父 commit（工作目录状态的基准）
        let parent = stash_commit.parent(0).ok()?;
        let parent_tree = parent.tree().ok()?;
        let stash_tree = stash_commit.tree().ok()?;

        // 计算 diff
        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&stash_tree), None)
            .ok()?;

        let stats = diff.stats().ok()?;

        Some(StashStat {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        })
    }
}

impl StashService for StashServiceImpl {
    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError> {
        let mut repo = self.ctx.repository_mut();
        let signature = repo.signature().map_err(|e| {
            GitError::OperationFailed(format!("无法获取 Git 签名: {}", e))
        })?;

        let stash_message = message.unwrap_or("Stashed changes");
        let flags = git2::StashFlags::INCLUDE_UNTRACKED;

        repo.stash_save(&signature, stash_message, Some(flags))
            .map_err(|e| GitError::OperationFailed(format!("创建 stash 失败: {}", e)))?;

        // 返回 0，表示最新的 stash
        Ok(0)
    }

    fn stash_pop(&self, index: usize) -> Result<StashPopResult, GitError> {
        // 先应用 stash
        let apply_result = self.stash_apply(index)?;

        if apply_result.applied && !apply_result.has_conflicts {
            // 应用成功且没有冲突，删除 stash
            self.stash_drop(index)?;

            Ok(StashPopResult {
                restored: true,
                message: Some(format!("Stash stash@{{{}}} applied and removed", index)),
                warnings: vec![],
            })
        } else if apply_result.has_conflicts {
            // 有冲突，保留 stash
            Ok(StashPopResult {
                restored: false,
                message: None,
                warnings: vec![
                    format!("Merge conflicts detected when applying stash stash@{{{}}}", index),
                    "The stash entry is kept in case you need it again.".to_string(),
                    "Please resolve the conflicts manually and then:".to_string(),
                    "  1. Resolve conflicts in the affected files".to_string(),
                    "  2. Stage the resolved files with: git add <file>".to_string(),
                    "  3. Continue with your workflow".to_string(),
                ],
            })
        } else {
            // 应用失败
            Ok(StashPopResult {
                restored: false,
                message: None,
                warnings: apply_result.warnings,
            })
        }
    }

    fn stash_apply(&self, index: usize) -> Result<StashApplyResult, GitError> {
        let mut repo = self.ctx.repository_mut();
        let mut options = git2::StashApplyOptions::default();

        let result = repo.stash_apply(index, Some(&mut options));

        match result {
            Ok(_) => {
                // 检查是否有冲突
                drop(repo); // 释放 mutable borrow
                let has_conflicts = self.has_unmerged().unwrap_or(false);

                Ok(StashApplyResult {
                    applied: true,
                    has_conflicts,
                    message: Some(format!("Stash stash@{{{}}} applied successfully", index)),
                    warnings: if has_conflicts {
                        vec!["Merge conflicts detected. Please resolve them manually.".to_string()]
                    } else {
                        vec![]
                    },
                    stat: None, // TODO: 添加统计信息
                })
            }
            Err(e) => {
                // 检查是否有冲突
                drop(repo); // 释放 mutable borrow
                let has_conflicts = self.has_unmerged().unwrap_or(false);

                Ok(StashApplyResult {
                    applied: false,
                    has_conflicts,
                    message: None,
                    warnings: vec![
                        format!("Failed to apply stash stash@{{{}}}: {}", index, e),
                        if has_conflicts {
                            "Merge conflicts detected. Please resolve them manually.".to_string()
                        } else {
                            "The stash entry is kept. You can try again later.".to_string()
                        },
                    ],
                    stat: None,
                })
            }
        }
    }

    fn stash_list(&self) -> Result<Vec<StashEntry>, GitError> {
        let mut entries = Vec::new();
        let mut repo = self.ctx.repository_mut();

        // 使用 stash_foreach 遍历所有 stash
        let mut error: Option<GitError> = None;

        repo.stash_foreach(|index, message, oid| {
            // 获取 commit 信息
            let repo_ref = self.ctx.repository();
            let commit = match repo_ref.find_commit(*oid) {
                Ok(c) => c,
                Err(e) => {
                    error = Some(GitError::OperationFailed(format!(
                        "无法获取 stash commit: {}",
                        e
                    )));
                    return false; // 停止遍历
                }
            };

            // 获取时间戳
            let time = commit.time();
            let timestamp = Local
                .timestamp_opt(time.seconds(), 0)
                .single()
                .map(|dt: DateTime<Local>| dt);

            // 从消息中提取分支名和消息
            let (branch, msg) = Self::extract_branch_and_message(message);

            entries.push(StashEntry {
                index,
                branch,
                message: msg,
                commit_hash: oid.to_string(),
                timestamp,
            });

            true // 继续遍历
        })
        .map_err(|e| GitError::OperationFailed(format!("列出 stash 失败: {}", e)))?;

        if let Some(e) = error {
            return Err(e);
        }

        // 按索引排序（从新到旧）
        entries.sort_by_key(|e| e.index);

        Ok(entries)
    }

    fn stash_drop(&self, index: usize) -> Result<(), GitError> {
        let mut repo = self.ctx.repository_mut();

        repo.stash_drop(index)
            .map_err(|e| GitError::OperationFailed(format!("删除 stash 失败: {}", e)))?;

        Ok(())
    }

    fn has_unmerged(&self) -> Result<bool, GitError> {
        let repo = self.ctx.repository();

        // 检查索引中是否有冲突
        let index = repo
            .index()
            .map_err(|e| GitError::OperationFailed(format!("无法获取索引: {}", e)))?;

        Ok(index.has_conflicts())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo_with_file;
    use std::fs;

    #[test]
    fn test_stash_push_and_list() {
        let (tmp, ctx) = setup_repo_with_file();

        // 创建一些未提交的更改
        let file_path = tmp.path().join("test.txt");
        fs::write(&file_path, "modified content").unwrap();

        let service = StashServiceImpl::new(ctx);

        // 创建 stash
        let result = service.stash_push(Some("Test stash message"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // 列出 stash
        let entries = service.stash_list().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].index, 0);
    }

    #[test]
    fn test_stash_list_empty() {
        let (_tmp, ctx) = setup_repo_with_file();
        let service = StashServiceImpl::new(ctx);

        let entries = service.stash_list().unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_extract_branch_and_message() {
        // 测试 "WIP on" 格式
        let (branch, message) =
            StashServiceImpl::extract_branch_and_message("WIP on main: test message");
        assert_eq!(branch, "main");
        assert_eq!(message, "test message");

        // 测试 "On" 格式
        let (branch, message) =
            StashServiceImpl::extract_branch_and_message("On feature/test: another message");
        assert_eq!(branch, "feature/test");
        assert_eq!(message, "another message");

        // 测试无法解析的格式
        let (branch, message) =
            StashServiceImpl::extract_branch_and_message("some random message");
        assert_eq!(branch, "unknown");
        assert_eq!(message, "some random message");
    }
}
