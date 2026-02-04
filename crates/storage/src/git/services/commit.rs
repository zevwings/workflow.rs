//! 提交业务逻辑服务
//!
//! 提供提交相关的业务逻辑实现。

use super::GitContext;
use domain::git::{CommitInfo, FileStatusInfo, FileStatusType, GitError, WorkingTreeStatus};

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
}

impl CommitServiceImpl {
    /// 创建新的提交服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
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

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
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

        let repo = self.ctx.repository();

        let mut index = repo.index().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 添加更改到暂存区
        if all {
            // 使用 git2 的批量更新方法，比逐个文件添加更快
            // update_all 会自动处理修改、删除、新增的文件，并尊重 .gitignore
            index
                .update_all(["*"].iter(), None)
                .map_err(|e| GitError::IndexError(e.to_string()))?;

            // 添加新文件（update_all 不会添加新文件）
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .map_err(|e| GitError::IndexError(e.to_string()))?;
        }
        // 如果 all=false，直接使用已暂存的文件进行提交

        // 写入索引到磁盘
        index.write().map_err(|e| GitError::IndexError(e.to_string()))?;

        // 检查是否有更改需要提交
        let tree_id = index.write_tree().map_err(|e| GitError::IndexError(e.to_string()))?;
        let tree = repo.find_tree(tree_id).map_err(|e| GitError::OperationFailed(e.to_string()))?;

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
        let oid = if let Some(parent) = parent_commit {
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
        };

        Ok(oid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::testing::setup_repo_with_file;

    #[test]
    #[ignore]
    fn test_get_commit_info() {
        let (_tmp, ctx) = setup_repo_with_file();
        let sha = {
            let repo = ctx.repository();
            let commit_id = {
                let head = repo.head().unwrap();
                head.peel_to_commit().unwrap().id()
            };
            drop(repo);
            commit_id.to_string()
        };

        let service = CommitServiceImpl::new(ctx);
        let info = service.get_commit_info(&sha).unwrap();
        assert_eq!(info.summary, "Initial commit");
    }

    #[test]
    #[ignore]
    fn test_get_working_tree_status() {
        let (tmp, ctx) = setup_repo_with_file();

        // 创建一个未跟踪的文件
        std::fs::write(tmp.path().join("new.txt"), "new file").unwrap();

        let service = CommitServiceImpl::new(ctx);
        let status = service.get_working_tree_status().unwrap();

        assert!(!status.untracked.is_empty());
    }
}
