//! Diff 业务逻辑服务
//!
//! 提供 diff 相关的业务逻辑实现。

use std::collections::HashMap;

use domain::{CommitChangeType, CommitFileChange, GitError};
use git2::{Delta, Diff, DiffDelta, DiffFormat, DiffHunk, DiffLine, DiffOptions};
use toolkit::log_warn;

use crate::git::services::GitContext;

/// Diff 服务接口
pub trait DiffService: Send + Sync {
    /// 获取工作区相对于指定分支的完整 diff
    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError>;

    /// 获取指定 commit 的 diff 内容（相对其第一父提交的 patch 字符串）
    fn get_commit_diff(&self, ref_or_sha: &str) -> Result<Option<String>, GitError>;

    /// 获取暂存区的完整 diff
    ///
    /// 返回暂存区相对于 HEAD 的完整 diff 内容，等价于 `git diff --cached`。
    fn get_staged_diff(&self) -> Result<Option<String>, GitError>;

    /// 获取将源分支合并到目标分支时会引入的 diff（仅已提交部分）
    ///
    /// 即 `merge_base(target_branch, branch)..branch` 的 tree diff，
    /// 与执行 merge 时"本次合并会引入的改动"一致。
    fn get_merge_diff(&self, branch: &str, target_branch: &str)
        -> Result<Option<String>, GitError>;

    /// 获取将源分支合并到目标分支时会变更的文件列表（与 get_merge_diff 范围一致）
    fn get_merge_changed_files(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Vec<CommitFileChange>, GitError>;

    /// 检测文件是否为纯格式化变更
    ///
    /// 对比正常 diff 和忽略空白的 diff：
    /// - 如果正常 diff 有变更，但忽略空白后无变更 → 纯格式化
    /// - 否则为实质性变更
    ///
    /// # 参数
    /// - `base_ref`: 基准引用（分支名或 commit SHA）
    /// - `target_ref`: 目标引用
    /// - `file_path`: 要检测的文件路径
    ///
    /// # 返回
    /// - `Ok(true)`: 纯格式化变更
    /// - `Ok(false)`: 包含实质性变更
    fn is_formatting_only_change(
        &self,
        base_ref: &str,
        target_ref: &str,
        file_path: &str,
    ) -> Result<bool, GitError>;
}

/// Diff 服务实现
pub struct DiffServiceImpl {
    ctx: GitContext,
}

impl DiffServiceImpl {
    /// 创建新的 Diff 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }

    /// 配置 diff 选项
    fn configure_diff_options(&self, opts: &mut DiffOptions, include_untracked: bool) {
        opts.include_untracked(include_untracked);
        opts.include_ignored(false);
        opts.include_typechange(true);
        opts.include_typechange_trees(true);
        // 限制文件大小（跳过大于 1MB 的文件）
        opts.max_size(1024 * 1024);
    }
}

impl DiffService for DiffServiceImpl {
    fn get_commit_diff(&self, ref_or_sha: &str) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();
        let commit = repo
            .revparse_single(ref_or_sha)
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?
            .peel_to_commit()
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?;

        let parent = match commit.parent(0) {
            Ok(p) => p,
            Err(_) => return Ok(None),
        };
        let parent_tree = parent.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit_tree = commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut patch = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(s) = std::str::from_utf8(line.content()) {
                patch.push_str(s);
            }
            true
        })
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if patch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(patch))
        }
    }

    fn get_staged_diff(&self) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();

        // Get HEAD tree
        let head = repo
            .head()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get HEAD: {}", e)))?;
        let head_tree = head
            .peel_to_tree()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get HEAD tree: {}", e)))?;

        // Get staged (index) tree
        let index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        // Create diff between HEAD tree and index
        let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&index), None).map_err(|e| {
            GitError::OperationFailed(format!("Failed to create staged diff: {}", e))
        })?;

        // Generate patch
        let mut patch = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(s) = std::str::from_utf8(line.content()) {
                patch.push_str(s);
            }
            true
        })
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if patch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(patch))
        }
    }

    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();

        // 获取 base branch 的 tree
        let base_commit = repo
            .revparse_single(base_branch)
            .and_then(|obj| obj.peel_to_commit())
            .map_err(|e| GitError::BranchNotFound(format!("{}: {}", base_branch, e)))?;
        let base_tree = base_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 获取索引（暂存区）的 tree
        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;
        let index_tree_id = index.write_tree().map_err(|e| GitError::IndexError(e.to_string()))?;
        let index_tree = repo
            .find_tree(index_tree_id)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut diff_str = String::new();
        const MAX_DIFF_SIZE: usize = 10 * 1024 * 1024;
        let mut total_size = 0usize;
        let mut size_limit_reached = false;

        // 获取忽略模式
        let ignore_patterns = self.ctx.get_ignore_directory_patterns();

        // 辅助闭包：处理单个 diff
        let mut process_diff = |diff: &Diff| -> Result<(), GitError> {
            // 尝试先使用 print，如果失败则手动构建
            let print_result = diff.print(DiffFormat::Patch, |delta, _hunk, line| {
                if size_limit_reached {
                    return true;
                }

                // 检查文件是否应该被过滤（基于忽略模式）
                if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                    if let Some(path_str) = path.to_str() {
                        for pattern in &ignore_patterns {
                            if path_str.starts_with(pattern.as_str()) {
                                return true; // 跳过匹配忽略模式的文件
                            }
                        }
                    }
                }

                if let Ok(content) = std::str::from_utf8(line.content()) {
                    total_size += content.len();
                    if total_size > MAX_DIFF_SIZE {
                        size_limit_reached = true;
                        diff_str.push_str("\n... [diff truncated: size limit reached] ...\n");
                        return true;
                    }
                    diff_str.push_str(content);
                }
                true
            });

            // 如果 print 没有产生任何输出但有 deltas，手动处理未追踪文件
            if diff_str.is_empty() && diff.deltas().len() > 0 {
                for delta in diff.deltas() {
                    // 检查是否应该被过滤
                    let should_skip = if let Some(path) =
                        delta.new_file().path().or_else(|| delta.old_file().path())
                    {
                        if let Some(path_str) = path.to_str() {
                            ignore_patterns
                                .iter()
                                .any(|pattern| path_str.starts_with(pattern.as_str()))
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_skip {
                        continue;
                    }

                    // 对于新文件，读取内容并生成简单的 diff 格式
                    if delta.status() == Delta::Untracked {
                        if let Some(path) = delta.new_file().path() {
                            // 获取工作目录，如果是 bare repository 则跳过
                            let Some(workdir) = repo.workdir() else {
                                log_warn!(
                                    "Cannot process untracked file in bare repository: {}",
                                    path.display()
                                );
                                continue;
                            };
                            let file_path = workdir.join(path);
                            if let Ok(content) = std::fs::read_to_string(&file_path) {
                                let diff_header = format!(
                                    "diff --git a/{} b/{}\nnew file\n--- /dev/null\n+++ b/{}\n",
                                    path.display(),
                                    path.display(),
                                    path.display()
                                );
                                diff_str.push_str(&diff_header);

                                for line in content.lines() {
                                    total_size += line.len() + 2; // +2 for "+ " prefix
                                    if total_size > MAX_DIFF_SIZE {
                                        size_limit_reached = true;
                                        diff_str.push_str(
                                            "\n... [diff truncated: size limit reached] ...\n",
                                        );
                                        break;
                                    }
                                    diff_str.push_str(&format!("+{}\n", line));
                                }

                                if size_limit_reached {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            print_result.map_err(|e| GitError::OperationFailed(e.to_string()))
        };

        // 预先获取 HEAD tree（如果存在），避免重复获取
        let head_tree = repo.head().and_then(|r| r.peel_to_commit()).and_then(|c| c.tree()).ok();

        // 1. 已提交更改（base -> HEAD）
        if let Some(ref ht) = head_tree {
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo
                .diff_tree_to_tree(Some(&base_tree), Some(ht), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            process_diff(&diff)?;
        }

        // 2. 暂存区更改（HEAD -> index）
        if let Some(ref ht) = head_tree {
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo
                .diff_tree_to_tree(Some(ht), Some(&index_tree), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            process_diff(&diff)?;
        } else {
            // 没有 HEAD（新分支）
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo
                .diff_tree_to_tree(Some(&base_tree), Some(&index_tree), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
            process_diff(&diff)?;
        }

        // 3. 工作区未暂存更改（index -> working tree）
        let mut diff_options = DiffOptions::new();
        self.configure_diff_options(&mut diff_options, true);
        let diff = repo
            .diff_index_to_workdir(Some(&index), Some(&mut diff_options))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        process_diff(&diff)?;

        if size_limit_reached {
            log_warn!(
                "Diff size limit reached ({}MB), content truncated",
                MAX_DIFF_SIZE / 1024 / 1024
            );
        }

        if diff_str.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(diff_str))
        }
    }

    fn get_merge_diff(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();

        let branch_commit = repo
            .revparse_single(branch)
            .map_err(|_| GitError::BranchNotFound(branch.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let target_commit = repo
            .revparse_single(target_branch)
            .map_err(|_| GitError::BranchNotFound(target_branch.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let merge_base_oid = repo
            .merge_base(branch_commit.id(), target_commit.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let merge_base_commit = repo
            .find_commit(merge_base_oid)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let base_tree =
            merge_base_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let branch_tree =
            branch_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut diff_options = DiffOptions::new();
        self.configure_diff_options(&mut diff_options, false);
        let diff = repo
            .diff_tree_to_tree(
                Some(&base_tree),
                Some(&branch_tree),
                Some(&mut diff_options),
            )
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut patch = String::new();
        diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
            if let Ok(s) = std::str::from_utf8(line.content()) {
                patch.push_str(s);
            }
            true
        })
        .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        if patch.is_empty() {
            Ok(None)
        } else {
            Ok(Some(patch))
        }
    }

    fn get_merge_changed_files(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Vec<CommitFileChange>, GitError> {
        let repo = self.ctx.repository();

        let branch_commit = repo
            .revparse_single(branch)
            .map_err(|_| GitError::BranchNotFound(branch.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let target_commit = repo
            .revparse_single(target_branch)
            .map_err(|_| GitError::BranchNotFound(target_branch.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let merge_base_oid = repo
            .merge_base(branch_commit.id(), target_commit.id())
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let merge_base_commit = repo
            .find_commit(merge_base_oid)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let base_tree =
            merge_base_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let branch_tree =
            branch_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut diff_options = DiffOptions::new();
        self.configure_diff_options(&mut diff_options, false);
        let diff = repo
            .diff_tree_to_tree(
                Some(&base_tree),
                Some(&branch_tree),
                Some(&mut diff_options),
            )
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let mut path_stats: HashMap<String, (u32, u32)> =
            HashMap::with_capacity(diff.deltas().len());
        let mut line_cb =
            |delta: DiffDelta<'_>, _hunk: Option<DiffHunk<'_>>, line: DiffLine<'_>| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .and_then(|p: &std::path::Path| p.to_str().map(String::from));
                if let Some(path) = path {
                    let entry = path_stats.entry(path).or_insert((0, 0));
                    match line.origin() {
                        '+' => entry.0 = entry.0.saturating_add(1),
                        '-' => entry.1 = entry.1.saturating_add(1),
                        _ => {}
                    }
                }
                true
            };
        if let Err(e) = diff.foreach(
            &mut |_delta, _progress| true,
            None,
            None,
            Some(&mut line_cb),
        ) {
            log_warn!("Failed to iterate diff lines: {}", e);
        }

        let files: Vec<CommitFileChange> = diff
            .deltas()
            .map(|delta| {
                let change_type = delta_status_to_commit_change_type(delta.status());
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .and_then(|p| p.to_str().map(String::from))
                    .unwrap_or_default();
                let old_path = delta.old_file().path().and_then(|p| p.to_str().map(String::from));
                let (additions, deletions) = path_stats.get(&path).copied().unwrap_or((0, 0));
                CommitFileChange {
                    path,
                    change_type,
                    old_path,
                    additions: Some(additions),
                    deletions: Some(deletions),
                }
            })
            .collect();

        Ok(files)
    }

    fn is_formatting_only_change(
        &self,
        base_ref: &str,
        target_ref: &str,
        file_path: &str,
    ) -> Result<bool, GitError> {
        let repo = self.ctx.repository();

        // 解析 base 和 target 引用
        let base_commit = repo
            .revparse_single(base_ref)
            .map_err(|_| GitError::CommitNotFound(base_ref.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let target_commit = repo
            .revparse_single(target_ref)
            .map_err(|_| GitError::CommitNotFound(target_ref.to_string()))?
            .peel_to_commit()
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let base_tree = base_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let target_tree =
            target_commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 1. 正常 diff（包含空白变更）
        let normal_diff = repo
            .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 2. 忽略空白的 diff
        let mut opts = DiffOptions::new();
        opts.ignore_whitespace(true);
        let ignore_ws_diff = repo
            .diff_tree_to_tree(Some(&base_tree), Some(&target_tree), Some(&mut opts))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 3. 统计指定文件的变更量
        let normal_changes = count_file_changes(&normal_diff, file_path);
        let ignore_ws_changes = count_file_changes(&ignore_ws_diff, file_path);

        // 4. 判断：正常有变更 && 忽略空白后无变更 = 纯格式化
        Ok(normal_changes > 0 && ignore_ws_changes == 0)
    }
}

/// 统计指定文件在 diff 中的变更行数
///
/// 仅统计 '+' 和 '-' 行（不包含上下文行）
fn count_file_changes(diff: &Diff, target_path: &str) -> u32 {
    let mut changes = 0u32;
    let _ = diff.foreach(
        &mut |delta, _progress| {
            // 只处理匹配目标路径的文件
            let matches = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .map(|p| p == target_path)
                .unwrap_or(false);
            matches
        },
        None,
        None,
        Some(&mut |_delta, _hunk, line| {
            // 统计新增和删除行
            if matches!(line.origin(), '+' | '-') {
                changes = changes.saturating_add(1);
            }
            true
        }),
    );
    changes
}

fn delta_status_to_commit_change_type(status: Delta) -> CommitChangeType {
    match status {
        Delta::Added => CommitChangeType::Added,
        Delta::Modified | Delta::Unmodified => CommitChangeType::Modified,
        Delta::Deleted => CommitChangeType::Deleted,
        Delta::Renamed => CommitChangeType::Renamed,
        Delta::Copied => CommitChangeType::Copied,
        Delta::Typechange => CommitChangeType::TypeChanged,
        _ => CommitChangeType::Modified,
    }
}
