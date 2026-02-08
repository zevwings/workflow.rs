//! 提交业务逻辑服务
//!
//! 提供提交相关的业务逻辑实现。

use std::collections::HashMap;
use std::sync::Arc;

use crate::git::services::context::GitContext;
use crate::git::services::hooks::{git_hooks, HookContext, HookResult, HookService};
use domain::git::{
    CommitChangeType, CommitFileChange, CommitInfo, FileStatusInfo, FileStatusType, GitError,
    WorkingTreeStatus,
};

/// 提交服务接口
pub trait CommitService: Send + Sync {
    /// 获取提交信息
    ///
    /// 参数支持多种格式：
    /// - 完整 SHA (40 字符): `"a1b2c3d4..."`
    /// - 短 SHA (至少 7 字符): `"a1b2c3d"`
    /// - 符号引用: `"HEAD"`, `"main"`, `"origin/main"`
    /// - 相对引用: `"HEAD~1"`, `"main^"`
    fn get_commit_info(&self, ref_or_sha: &str) -> Result<CommitInfo, GitError>;

    /// 获取指定 commit 变更的文件列表
    fn get_commit_changed_files(&self, ref_or_sha: &str)
        -> Result<Vec<CommitFileChange>, GitError>;

    /// 获取工作树状态
    fn get_working_tree_status(&self) -> Result<WorkingTreeStatus, GitError>;

    /// 创建提交
    ///
    /// # 参数
    /// - `message`: 提交消息
    /// - `all`: 是否添加所有更改（包括未跟踪的文件）
    ///
    /// # 返回
    /// 返回创建的提交的 SHA
    fn commit(&self, message: &str, all: bool) -> Result<String, GitError>;
}

/// 提交服务实现
pub struct CommitServiceImpl {
    ctx: GitContext,
    hook_service: Arc<dyn HookService>,
}

impl CommitServiceImpl {
    /// 创建新的提交服务实例
    pub fn new(ctx: GitContext, hook_service: Arc<dyn HookService>) -> Self {
        Self { ctx, hook_service }
    }
}

impl CommitService for CommitServiceImpl {
    fn get_commit_info(&self, ref_or_sha: &str) -> Result<CommitInfo, GitError> {
        let repo = self.ctx.repository();

        // revparse_single 支持 SHA、符号引用和相对引用
        let obj = repo
            .revparse_single(ref_or_sha)
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?;

        let commit = obj
            .peel_to_commit()
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?;

        let author = commit.author();
        let committer = commit.committer();

        Ok(CommitInfo {
            sha: commit.id().to_string(),
            message: commit.message().unwrap_or("(no message)").to_string(),
            summary: commit.summary().unwrap_or("(no message)").to_string(),
            author_name: author.name().unwrap_or("Unknown").to_string(),
            author_email: author.email().unwrap_or("unknown").to_string(),
            author_time: author.when().seconds(),
            committer_name: committer.name().unwrap_or("Unknown").to_string(),
            committer_email: committer.email().unwrap_or("unknown").to_string(),
            committer_time: committer.when().seconds(),
            parents: commit.parent_ids().map(|id| id.to_string()).collect(),
        })
    }

    fn get_commit_changed_files(
        &self,
        ref_or_sha: &str,
    ) -> Result<Vec<CommitFileChange>, GitError> {
        let repo = self.ctx.repository();
        let commit = repo
            .revparse_single(ref_or_sha)
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?
            .peel_to_commit()
            .map_err(|_| GitError::CommitNotFound(ref_or_sha.to_string()))?;

        let parent = match commit.parent(0) {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };
        let parent_tree = parent.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
        let commit_tree = commit.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let diff = repo
            .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 按文件统计增删行数（仅统计文本 diff，二进制等无统计）
        let mut path_stats: HashMap<String, (u32, u32)> =
            HashMap::with_capacity(diff.deltas().len());
        let mut line_cb = |delta: git2::DiffDelta<'_>,
                           _hunk: Option<git2::DiffHunk<'_>>,
                           line: git2::DiffLine<'_>| {
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
            toolkit::log_warn!("Failed to iterate diff lines: {}", e);
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

    fn get_working_tree_status(&self) -> Result<WorkingTreeStatus, GitError> {
        let repo = self.ctx.repository();

        // 配置 status 选项以提高性能
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false) // 排除被忽略的文件
            .exclude_submodules(true); // 排除子模块

        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let status_count = statuses.len();
        let mut staged = Vec::with_capacity(status_count);
        let mut unstaged = Vec::with_capacity(status_count);
        let mut untracked = Vec::with_capacity(status_count);
        let mut conflicted = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry
                .path()
                .ok_or_else(|| GitError::OperationFailed("Invalid file path".into()))?
                .to_string();

            // 检查冲突
            if status.is_conflicted() {
                conflicted.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::Conflicted,
                    old_path: None,
                });
                continue;
            }

            // 检查索引（暂存区）的变更
            if status.is_index_new() {
                staged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::NewStaged,
                    old_path: None,
                });
            } else if status.is_index_modified() {
                staged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::ModifiedStaged,
                    old_path: None,
                });
            } else if status.is_index_deleted() {
                staged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::DeletedStaged,
                    old_path: None,
                });
            } else if status.is_index_renamed() {
                staged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::Renamed,
                    old_path: entry.head_to_index().and_then(|diff| {
                        diff.old_file().path().and_then(|p| p.to_str()).map(String::from)
                    }),
                });
            } else if status.is_index_typechange() {
                staged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::TypeChanged,
                    old_path: None,
                });
            }

            // 检查工作树的变更
            if status.is_wt_new() {
                untracked.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::NewUntracked,
                    old_path: None,
                });
            } else if status.is_wt_modified() {
                unstaged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::ModifiedUnstaged,
                    old_path: None,
                });
            } else if status.is_wt_deleted() {
                unstaged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::DeletedUnstaged,
                    old_path: None,
                });
            } else if status.is_wt_typechange() {
                unstaged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::TypeChanged,
                    old_path: None,
                });
            } else if status.is_wt_renamed() {
                unstaged.push(FileStatusInfo {
                    path: path.clone(),
                    status_type: FileStatusType::Renamed,
                    old_path: None,
                });
            }
        }

        Ok(WorkingTreeStatus {
            staged,
            unstaged,
            untracked,
            conflicted,
        })
    }

    fn commit(&self, message: &str, all: bool) -> Result<String, GitError> {
        // 获取签名（必须在获取 repo 锁之前，避免死锁）
        let signature = self.ctx.get_signature()?;

        // 获取路径信息（用于 hook 上下文）
        let (repo_path, git_dir) = {
            let repo = self.ctx.repository();
            let git_dir = repo.path().to_path_buf();
            let repo_path = repo
                .workdir()
                .ok_or_else(|| GitError::OperationFailed("Not a work tree".into()))?
                .to_path_buf();
            (repo_path, git_dir)
        };

        let repo = self.ctx.repository();
        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 添加更改到暂存区
        if all {
            // 获取需要跳过的目录模式（从 .gitignore 和常见大型目录）
            // 即使这些目录在 .gitignore 中，扫描它们仍然很慢，
            // 所以使用回调函数提前跳过以提高性能
            let ignore_patterns = self.ctx.get_ignore_directory_patterns();

            index
                .add_all(
                    ["."].iter(),
                    git2::IndexAddOption::DEFAULT,
                    Some(&mut |path, _| {
                        // 跳过大型目录以提高性能
                        if let Some(path_str) = path.to_str() {
                            if ignore_patterns.iter().any(|pattern| path_str.starts_with(pattern)) {
                                return 1; // Skip this path
                            }
                        }
                        0 // Add this path (git2 会自动处理 .gitignore)
                    }),
                )
                .map_err(|e| GitError::IndexError(e.to_string()))?;
        }
        // 如果 all=false，直接使用已暂存的文件进行提交

        // 写入索引到磁盘
        index.write().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 获取暂存文件列表（用于 hook 上下文）
        let staged_files = Self::get_staged_files(&index);

        // 释放 repo 锁以执行 hook
        drop(repo);

        // [1] pre-commit hook
        let hook_context =
            HookContext::for_pre_commit(repo_path.clone(), git_dir.clone(), staged_files);

        match self.hook_service.execute_hook(git_hooks::PRE_COMMIT, &hook_context)? {
            HookResult::Failure(msg) => {
                return Err(GitError::HookFailed(format!(
                    "{} hook failed: {}",
                    git_hooks::PRE_COMMIT,
                    msg
                )));
            }
            HookResult::Modified => {
                // Hook 修改了文件（如 cargo fmt），需要重新添加到暂存区
                let repo = self.ctx.repository();
                let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

                // 获取需要跳过的目录模式
                let ignore_patterns = self.ctx.get_ignore_directory_patterns();

                // 重新添加所有修改的文件到暂存区
                index
                    .add_all(
                        ["."].iter(),
                        git2::IndexAddOption::DEFAULT | git2::IndexAddOption::CHECK_PATHSPEC,
                        Some(&mut |path, _| {
                            if let Some(path_str) = path.to_str() {
                                if ignore_patterns
                                    .iter()
                                    .any(|pattern| path_str.starts_with(pattern))
                                {
                                    return 1; // Skip
                                }
                            }
                            0 // Add
                        }),
                    )
                    .map_err(|e| GitError::IndexError(e.to_string()))?;

                index.write().map_err(|e| GitError::IndexError(e.to_string()))?;
                toolkit::log_info!("Files modified by hook have been re-staged");
            }
            HookResult::Success => {}
        }

        // 重新获取 repo 锁执行提交
        let oid = {
            let repo = self.ctx.repository();
            let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

            // 检查是否有更改需要提交
            let tree_id = index.write_tree().map_err(|e| GitError::IndexError(e.to_string()))?;
            let tree =
                repo.find_tree(tree_id).map_err(|e| GitError::OperationFailed(e.to_string()))?;

            // 检查是否有实际更改（比较 tree 和 HEAD）
            let parent_commit = repo.head().and_then(|head| head.peel_to_commit()).ok();

            // 如果有父提交，检查 tree 是否与父提交的 tree 相同
            if let Some(ref parent) = parent_commit {
                let parent_tree =
                    parent.tree().map_err(|e| GitError::OperationFailed(e.to_string()))?;
                if tree_id == parent_tree.id() {
                    return Err(GitError::OperationFailed("Nothing to commit".into()));
                }
            }

            // 创建提交
            if let Some(parent) = parent_commit {
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent],
                )
                .map_err(|e| GitError::OperationFailed(e.to_string()))?
            } else {
                // 首次提交，没有父提交
                repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
                    .map_err(|e| GitError::OperationFailed(e.to_string()))?
            }
            // repo 锁在这里自动释放
        };

        // [2] commit-msg hook
        let commit_msg_context = HookContext::for_commit_msg(
            repo_path.clone(),
            git_dir.clone(),
            message.to_string(),
            Some(oid.to_string()),
        );

        if let HookResult::Failure(msg) =
            self.hook_service.execute_hook(git_hooks::COMMIT_MSG, &commit_msg_context)?
        {
            return Err(GitError::HookFailed(format!(
                "{} hook failed: {}",
                git_hooks::COMMIT_MSG,
                msg
            )));
        }

        // [3] post-commit hook（失败不影响提交结果，仅记录警告）
        let post_commit_context =
            HookContext::new(repo_path, git_dir).with_commit_sha(oid.to_string());
        if let Err(e) = self.hook_service.execute_hook(git_hooks::POST_COMMIT, &post_commit_context)
        {
            toolkit::log_warn!("post-commit hook failed (commit already succeeded): {}", e);
        }

        Ok(oid.to_string())
    }
}

fn delta_status_to_commit_change_type(status: git2::Delta) -> CommitChangeType {
    use git2::Delta;
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

impl CommitServiceImpl {
    /// 获取暂存区文件列表
    fn get_staged_files(index: &git2::Index) -> Vec<String> {
        index
            .iter()
            .filter_map(|entry| std::str::from_utf8(&entry.path).ok().map(String::from))
            .collect()
    }
}
