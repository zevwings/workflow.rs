//! Diff 业务逻辑服务
//!
//! 提供 diff 相关的业务逻辑实现。

use super::GitContext;
use domain::git::GitError;
use git2::DiffOptions;

/// Diff 服务接口
pub trait DiffService: Send + Sync {
    /// 获取工作区相对于指定分支的完整 diff
    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError>;
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
    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError> {
        let repo = self.ctx.repository();

        // 获取 base branch 的 tree
        let base_commit = repo
            .revparse_single(base_branch)
            .and_then(|obj| obj.peel_to_commit())
            .map_err(|e| GitError::OperationFailed(format!("Failed to find base branch '{}': {}", base_branch, e)))?;
        let base_tree = base_commit.tree().map_err(|e| GitError::OperationFailed(format!("Failed to get base tree: {}", e)))?;

        // 获取索引（暂存区）的 tree
        let mut index = repo.index().map_err(|e| GitError::OperationFailed(format!("Failed to get index: {}", e)))?;
        let index_tree_id = index.write_tree().map_err(|e| GitError::OperationFailed(format!("Failed to write index tree: {}", e)))?;
        let index_tree = repo.find_tree(index_tree_id).map_err(|e| GitError::OperationFailed(format!("Failed to find index tree: {}", e)))?;

        let mut diff_str = String::new();
        const MAX_DIFF_SIZE: usize = 10 * 1024 * 1024;
        let mut total_size = 0usize;
        let mut size_limit_reached = false;

        // 获取忽略模式
        let ignore_patterns = self.ctx.get_ignore_directory_patterns();

        // 辅助闭包：处理单个 diff
        let mut process_diff = |diff: &git2::Diff| -> Result<(), GitError> {
            // 尝试先使用 print，如果失败则手动构建
            let print_result = diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
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
                    let should_skip = if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                        if let Some(path_str) = path.to_str() {
                            ignore_patterns.iter().any(|pattern| path_str.starts_with(pattern.as_str()))
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
                    if delta.status() == git2::Delta::Untracked {
                        if let Some(path) = delta.new_file().path() {
                            let file_path = repo.workdir().unwrap().join(path);
                            if let Ok(content) = std::fs::read_to_string(&file_path) {
                                let diff_header = format!("diff --git a/{} b/{}\nnew file\n--- /dev/null\n+++ b/{}\n",
                                    path.display(), path.display(), path.display());
                                diff_str.push_str(&diff_header);

                                for line in content.lines() {
                                    total_size += line.len() + 2; // +2 for "+ " prefix
                                    if total_size > MAX_DIFF_SIZE {
                                        size_limit_reached = true;
                                        diff_str.push_str("\n... [diff truncated: size limit reached] ...\n");
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

            print_result.map_err(|e| GitError::OperationFailed(format!("Failed to print diff: {}", e)))
        };

        // 1. 已提交更改（base -> HEAD）
        if let Ok(head_commit) = repo.head().and_then(|r| r.peel_to_commit()) {
            let head_tree = head_commit.tree().map_err(|e| GitError::OperationFailed(format!("Failed to get head tree: {}", e)))?;
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(format!("Failed to create committed diff: {}", e)))?;
            process_diff(&diff)?;
        }

        // 2. 暂存区更改（HEAD -> index）
        if let Ok(head_commit) = repo.head().and_then(|r| r.peel_to_commit()) {
            let head_tree = head_commit.tree().map_err(|e| GitError::OperationFailed(format!("Failed to get head tree: {}", e)))?;
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(format!("Failed to create staged diff: {}", e)))?;
            process_diff(&diff)?;
        } else {
            // 没有 HEAD（新分支）
            let mut diff_options = DiffOptions::new();
            self.configure_diff_options(&mut diff_options, false);
            let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&index_tree), Some(&mut diff_options))
                .map_err(|e| GitError::OperationFailed(format!("Failed to create staged diff: {}", e)))?;
            process_diff(&diff)?;
        }

        // 3. 工作区未暂存更改（index -> working tree）
        let mut diff_options = DiffOptions::new();
        self.configure_diff_options(&mut diff_options, true);
        let diff = repo.diff_index_to_workdir(Some(&index), Some(&mut diff_options))
            .map_err(|e| GitError::OperationFailed(format!("Failed to create working tree diff: {}", e)))?;
        process_diff(&diff)?;

        if size_limit_reached {
            eprintln!("Warning: Diff size limit reached ({}MB), content truncated", MAX_DIFF_SIZE / 1024 / 1024);
        }

        if diff_str.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(diff_str))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo_with_file;

    #[test]
    fn test_get_working_tree_diff_empty() {
        let (_tmp, ctx) = setup_repo_with_file();
        let service = DiffServiceImpl::new(ctx);
        let diff = service.get_working_tree_diff("HEAD").unwrap();
        assert_eq!(diff, None);
    }

    #[test]
    fn test_get_working_tree_diff_with_changes() {
        let (tmp, ctx) = setup_repo_with_file();

        // 创建一个新文件
        let new_file_path = tmp.path().join("new_test_file.txt");
        std::fs::write(&new_file_path, "new file content").unwrap();
        assert!(new_file_path.exists());

        let service = DiffServiceImpl::new(ctx);
        let diff = service.get_working_tree_diff("HEAD").unwrap();

        assert!(diff.is_some(), "Diff should not be empty when there are new files");
        let diff_content = diff.unwrap();
        assert!(diff_content.contains("new_test_file.txt"));
        assert!(diff_content.contains("new file content"));
    }
}
