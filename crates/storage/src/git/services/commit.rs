//! 提交业务逻辑服务
//!
//! 提供提交相关的业务逻辑实现。

use std::collections::HashMap;
use std::sync::Arc;

use toolkit::{log_debug, log_info, log_warn};

use crate::git::services::context::GitContext;
use crate::git::services::hooks::{git_hooks, HookContext, HookResult, HookService};
use domain::{
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

    /// 获取暂存区文件列表
    ///
    /// 返回当前暂存区（staging area）中的文件变更列表。
    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError>;

    /// 添加所有更改到暂存区
    ///
    /// 等价于 `git add -A`。
    fn add_all(&self) -> Result<(), GitError>;

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

    /// 全树扫描并添加（等价于 `git add -A`）。
    ///
    /// 使用 git2 的 statuses() 获取变更文件，然后逐个添加/删除。
    /// 比 add_all(["."]) 快，因为 StatusOptions 会自动排除被忽略的文件。
    fn add_all_full_tree(&self) -> Result<(), GitError> {
        log_debug!("commit: add_all_full_tree start (using git2 statuses)");

        let repo = self.ctx.repository();

        // 使用 StatusOptions 获取变更（自动排除 .gitignore 中的文件）
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false) // 关键：排除被忽略的文件
            .exclude_submodules(true);

        log_debug!("commit: fetching file statuses");
        let statuses = repo
            .statuses(Some(&mut opts))
            .map_err(|e| GitError::OperationFailed(format!("Failed to get statuses: {}", e)))?;

        log_debug!("commit: found {} files with changes", statuses.len());

        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 遍历每个文件，根据状态添加或删除
        for entry in statuses.iter() {
            let path = entry
                .path()
                .ok_or_else(|| GitError::OperationFailed("Invalid file path".into()))?;
            let status = entry.status();

            // 跳过冲突文件（需要手动解决）
            if status.is_conflicted() {
                continue;
            }

            // 处理删除的文件
            if status.is_wt_deleted() {
                index
                    .remove_path(std::path::Path::new(path))
                    .map_err(|e| GitError::IndexError(e.to_string()))?;
                continue;
            }

            // 添加新文件或修改的文件
            if status.is_wt_new() || status.is_wt_modified() || status.is_wt_typechange() {
                index
                    .add_path(std::path::Path::new(path))
                    .map_err(|e| GitError::IndexError(e.to_string()))?;
            }
        }

        index.write().map_err(|e| GitError::IndexError(e.to_string()))?;

        log_debug!("commit: add_all_full_tree done");
        Ok(())
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

    fn add_all(&self) -> Result<(), GitError> {
        // 直接使用 add_all_full_tree()（已移除性能问题的回调）。
        // 不使用 add_changed_paths()：它依赖的 get_working_tree_status() 在大仓库中可能卡住。
        log_debug!("commit: add_all start");
        self.add_all_full_tree()
    }

    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError> {
        let repo = self.ctx.repository();
        let head = repo
            .head()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get HEAD: {}", e)))?;
        let head_tree = head
            .peel_to_tree()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get HEAD tree: {}", e)))?;

        let index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        let diff = repo
            .diff_tree_to_index(Some(&head_tree), Some(&index), None)
            .map_err(|e| GitError::OperationFailed(format!("Failed to get staged diff: {}", e)))?;

        // 使用 HashMap 来收集文件信息和统计
        let mut file_map: HashMap<String, CommitFileChange> = HashMap::new();

        // 第一次遍历：收集文件基本信息（路径、变更类型）
        diff.foreach(
            &mut |delta, _progress| {
                let change_type = match delta.status() {
                    git2::Delta::Added => CommitChangeType::Added,
                    git2::Delta::Deleted => CommitChangeType::Deleted,
                    git2::Delta::Modified => CommitChangeType::Modified,
                    git2::Delta::Renamed => CommitChangeType::Renamed,
                    git2::Delta::Copied => CommitChangeType::Copied,
                    git2::Delta::Typechange => CommitChangeType::TypeChanged,
                    _ => return true, // Skip other types
                };

                let path =
                    delta.new_file().path().and_then(|p| p.to_str()).unwrap_or("").to_string();

                if path.is_empty() {
                    return true;
                }

                let old_path = if matches!(change_type, CommitChangeType::Renamed) {
                    delta.old_file().path().and_then(|p| p.to_str()).map(String::from)
                } else {
                    None
                };

                file_map.insert(
                    path.clone(),
                    CommitFileChange {
                        path,
                        old_path,
                        change_type,
                        additions: Some(0), // 初始化为 0，稍后在 print 回调中计算
                        deletions: Some(0), // 初始化为 0，稍后在 print 回调中计算
                    },
                );

                true
            },
            None,
            None,
            None,
        )
        .map_err(|e| GitError::OperationFailed(format!("Failed to iterate diff: {}", e)))?;

        // 第二次遍历：使用 print 回调计算每个文件的行数统计
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let path = delta.new_file().path().and_then(|p| p.to_str()).unwrap_or("");

            if path.is_empty() {
                return true;
            }

            if let Some(file) = file_map.get_mut(path) {
                // 统计新增和删除的行数
                match line.origin() {
                    '+' => {
                        // 新增行（不包括 +++ 文件头）
                        if let Some(adds) = file.additions.as_mut() {
                            *adds += 1;
                        }
                    }
                    '-' => {
                        // 删除行（不包括 --- 文件头）
                        if let Some(dels) = file.deletions.as_mut() {
                            *dels += 1;
                        }
                    }
                    _ => {
                        // 其他行（上下文、文件头等）不计数
                    }
                }
            }

            true
        })
        .map_err(|e| GitError::OperationFailed(format!("Failed to calculate file stats: {}", e)))?;

        // 转换为 Vec 并返回
        Ok(file_map.into_values().collect())
    }

    fn commit(&self, message: &str, all: bool) -> Result<String, GitError> {
        log_debug!("commit: start (all={})", all);

        // 获取签名（必须在获取 repo 锁之前，避免死锁）
        let signature = self.ctx.get_signature()?;
        log_debug!("commit: got signature");

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

        // 添加更改到暂存区（add_all 内部优先按变更路径添加，失败时回退全树扫描）
        if all {
            log_debug!("commit: staging changes (add_all)");
            self.add_all()?;
            log_debug!("commit: staging done");
        }

        // 在 add_all() 之后获取 repo 锁，避免死锁
        let repo = self.ctx.repository();
        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;
        if !all {
            index.write().map_err(|e| GitError::IndexError(e.to_string()))?;
        }

        // 获取暂存文件列表（用于 hook 上下文）
        let staged_files = Self::get_staged_files(&index);

        // 释放 repo 锁以执行 hook
        drop(repo);

        // [1] pre-commit hook
        log_debug!("commit: pre-commit hook start");
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
                log_info!("Files modified by hook have been re-staged");
            }
            HookResult::Success => {}
        }
        log_debug!("commit: pre-commit hook done");

        // 重新获取 repo 锁执行提交
        log_debug!("commit: writing tree and creating commit");
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
        log_debug!("commit: created oid {}", oid);

        // [2] commit-msg hook
        log_debug!("commit: commit-msg hook start");
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
        log_debug!("commit: commit-msg hook done");

        // [3] post-commit hook（失败不影响提交结果，仅记录警告）
        log_debug!("commit: post-commit hook start");
        let post_commit_context =
            HookContext::new(repo_path, git_dir).with_commit_sha(oid.to_string());
        if let Err(e) = self.hook_service.execute_hook(git_hooks::POST_COMMIT, &post_commit_context)
        {
            log_warn!("post-commit hook failed (commit already succeeded): {}", e);
        }
        log_debug!("commit: post-commit hook done, commit complete");

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
