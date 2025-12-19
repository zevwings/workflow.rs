//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）
//! - 修改最后一次提交（amend）
//! - 修改历史提交消息（reword）

use color_eyre::{eyre::eyre, eyre::WrapErr, Result};
use git2::{DiffFormat, IndexAddOption, Signature, StatusOptions};
use std::env;

use super::helpers::open_repo;
use super::pre_commit::GitPreCommit;

/// Git 提交结果
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// 是否已提交
    pub committed: bool,
    /// 消息（如果工作区干净）
    pub message: Option<String>,
}

/// Commit 信息
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit SHA
    pub sha: String,
    /// 提交消息
    pub message: String,
    /// 作者
    pub author: String,
    /// 日期
    pub date: String,
}

/// 工作区状态
#[derive(Debug, Clone)]
pub struct WorktreeStatus {
    /// 已修改文件数量
    pub modified_count: usize,
    /// 已暂存文件数量
    pub staged_count: usize,
    /// 未跟踪文件数量
    pub untracked_count: usize,
}

/// Git 提交管理
///
/// 提供提交相关的操作功能，包括：
/// - 检查 Git 状态和工作区更改
/// - 暂存文件（add）
/// - 提交更改（commit）
pub struct GitCommit;

impl GitCommit {
    /// 检查 Git 状态
    ///
    /// 使用 git2 获取状态并转换为 porcelain 格式。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串。
    pub fn status() -> Result<String> {
        let repo = open_repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts)).wrap_err("Failed to get status")?;

        let mut output = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("");

            // 转换为 porcelain 格式
            let status_str = if status.is_index_new() {
                "A "
            } else if status.is_index_modified() {
                "M "
            } else if status.is_index_deleted() {
                "D "
            } else if status.is_wt_modified() {
                " M"
            } else if status.is_wt_deleted() {
                " D"
            } else if status.is_wt_new() {
                "??"
            } else {
                "  "
            };

            output.push(format!("{}{}", status_str, path));
        }

        Ok(output.join("\n"))
    }

    /// 检查工作区是否有未提交的更改
    ///
    /// 检查工作区和暂存区是否有未提交的更改。
    /// 使用 git2 检查状态。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有未提交的更改（工作区或暂存区）
    /// - `Ok(false)` - 如果没有未提交的更改
    #[allow(dead_code)]
    pub fn has_commit() -> Result<bool> {
        let repo = open_repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts)).wrap_err("Failed to get status")?;

        // 检查是否有任何更改
        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_wt_modified()
                || status.is_wt_deleted()
                || status.is_wt_new()
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 检查是否有暂存的文件
    ///
    /// 使用 git2 检查暂存区是否有文件。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有暂存的文件
    /// - `Ok(false)` - 如果没有暂存的文件
    pub(crate) fn has_staged() -> Result<bool> {
        let repo = open_repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(false);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts)).wrap_err("Failed to get status")?;

        // 检查索引区是否有更改
        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 添加所有文件到暂存区
    ///
    /// 使用 git2 将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        let repo = open_repo()?;
        let mut index = repo.index().wrap_err("Failed to open index")?;
        index
            .add_all(["*"], IndexAddOption::DEFAULT, None)
            .wrap_err("Failed to add all files")?;
        index.write().wrap_err("Failed to write index")
    }

    /// 提交更改
    ///
    /// 自动暂存所有已修改的文件，然后提交。
    /// 如果存在 pre-commit hooks，会在提交前执行（除非 `no_verify` 为 `true`）。
    ///
    /// # 参数
    ///
    /// * `message` - 提交消息
    /// * `no_verify` - 是否跳过 pre-commit hooks 验证
    ///
    /// # 行为
    ///
    /// 1. 检查是否有未提交的更改，如果没有则直接返回
    /// 2. 暂存所有已修改的文件
    /// 3. 如果 `no_verify` 为 `false` 且存在 pre-commit hooks，则执行 hooks
    /// 4. 执行提交操作
    ///
    /// # 返回
    ///
    /// 返回 `CommitResult`，包含提交状态和消息。
    pub fn commit(message: &str, no_verify: bool) -> Result<CommitResult> {
        let repo = open_repo()?;

        // 1. 检查是否有未提交的更改
        let has_changes = Self::has_commit()?;

        // 2. 如果没有更改，直接返回
        if !has_changes {
            return Ok(CommitResult {
                committed: false,
                message: Some("Nothing to commit, working tree clean".to_string()),
            });
        }

        // 3. 暂存所有已修改的文件
        Self::add_all().wrap_err("Failed to stage changes")?;

        // 4. 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        let should_skip_hook = if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
            // 设置环境变量告诉 hook 脚本跳过执行（如果后续通过 git 命令执行）
            env::set_var("WORKFLOW_PRE_COMMIT_SKIP", "1");
            true
        } else {
            false
        };

        // 5. 使用 git2 创建提交
        let mut index = repo.index().wrap_err("Failed to open index")?;
        let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        // 获取签名
        let config = repo.config().ok();
        let name = config
            .as_ref()
            .and_then(|c| c.get_string("user.name").ok())
            .unwrap_or_else(|| "Unknown".to_string());
        let email = config
            .as_ref()
            .and_then(|c| c.get_string("user.email").ok())
            .unwrap_or_else(|| "unknown@example.com".to_string());
        let signature = Signature::now(&name, &email).wrap_err("Failed to create signature")?;

        // 获取 HEAD commit（如果有）
        let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

        if let Some(parent) = parent_commit {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &[&parent],
            )
            .wrap_err("Failed to create commit")?;
        } else {
            // 初始提交，没有父提交
            repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
                .wrap_err("Failed to create initial commit")?;
        }

        // 清理环境变量
        if should_skip_hook {
            env::remove_var("WORKFLOW_PRE_COMMIT_SKIP");
        }

        Ok(CommitResult {
            committed: true,
            message: None,
        })
    }

    /// 获取 Git 修改内容（工作区和暂存区）
    ///
    /// 获取工作区和暂存区的所有修改内容，用于传递给 LLM 生成分支名和 PR 标题。
    ///
    /// # 返回
    ///
    /// 返回 `Option<String>`：
    /// - `Some(String)` - 如果有修改内容，包含暂存区和工作区的修改
    /// - `None` - 如果没有修改内容
    ///
    /// # 格式
    ///
    /// 返回的字符串格式：
    /// ```text
    /// Staged changes:
    /// {staged diff content}
    ///
    /// Working tree changes:
    /// {worktree diff content}
    /// ```
    pub fn get_diff() -> Option<String> {
        let repo = match open_repo() {
            Ok(r) => r,
            Err(_) => return None,
        };

        let mut diff_parts = Vec::new();

        // 获取暂存区的修改（index vs HEAD）
        if let Ok(head) = repo.head() {
            if let Ok(head_commit) = head.peel_to_commit() {
                if let Ok(head_tree) = head_commit.tree() {
                    if let Ok(index) = repo.index() {
                        if let Ok(diff) =
                            repo.diff_tree_to_index(Some(&head_tree), Some(&index), None)
                        {
                            let mut diff_str = Vec::new();
                            if diff
                                .print(DiffFormat::Patch, |_delta, _hunk, line| {
                                    let content = line.content();
                                    diff_str.push(content.to_vec());
                                    true
                                })
                                .is_ok()
                            {
                                let diff_bytes = diff_str.concat();
                                let staged_diff = String::from_utf8_lossy(&diff_bytes);
                                if !staged_diff.trim().is_empty() {
                                    diff_parts.push(format!("Staged changes:\n{}", staged_diff));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 获取工作区的修改（index vs workdir）
        if let Ok(diff) = repo.diff_index_to_workdir(None, None) {
            let mut diff_str = Vec::new();
            if diff
                .print(DiffFormat::Patch, |_delta, _hunk, line| {
                    let content = line.content();
                    diff_str.push(content.to_vec());
                    true
                })
                .is_ok()
            {
                let diff_bytes = diff_str.concat();
                let worktree_diff = String::from_utf8_lossy(&diff_bytes);
                if !worktree_diff.trim().is_empty() {
                    diff_parts.push(format!("Working tree changes:\n{}", worktree_diff));
                }
            }
        }

        if diff_parts.is_empty() {
            None
        } else {
            Some(diff_parts.join("\n\n"))
        }
    }

    /// 重置工作区到指定提交
    ///
    /// 使用 git2 将工作区和暂存区重置到指定提交。
    /// 这会丢弃所有未提交的更改。
    ///
    /// # 参数
    ///
    /// * `target` - 目标提交引用（如 "HEAD", "HEAD~1", 分支名等）
    ///   如果为 `None`，则重置到当前 HEAD
    ///
    /// # 警告
    ///
    /// 此操作会**永久丢弃**工作区和暂存区的所有未提交更改，请谨慎使用。
    ///
    /// # 错误
    ///
    /// 如果重置失败，返回相应的错误信息。
    pub fn reset_hard(target: Option<&str>) -> Result<()> {
        let repo = open_repo()?;
        let target = target.unwrap_or("HEAD");

        // 解析目标引用
        let target_obj = repo
            .revparse_single(target)
            .wrap_err_with(|| format!("Failed to parse target: {}", target))?;

        // 执行 hard reset
        repo.reset(&target_obj, git2::ResetType::Hard, None)
            .wrap_err_with(|| format!("Failed to reset working directory to {}", target))
    }

    /// 检查是否有最后一次 commit
    ///
    /// 使用 git2 检查是否有 commit 历史。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有 commit
    /// - `Ok(false)` - 如果没有 commit
    pub fn has_last_commit() -> Result<bool> {
        let repo = open_repo()?;
        let head_result = repo.head();
        Ok(head_result.is_ok())
    }

    /// 获取最后一次 commit 信息
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的详细信息。
    pub fn get_last_commit_info() -> Result<CommitInfo> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

        let sha = commit.id().to_string();
        let message = commit.message().ok_or_else(|| eyre!("Commit has no message"))?.to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown");
        let author_email = author.email().unwrap_or("unknown@example.com");
        let author_str = format!("{} <{}>", author_name, author_email);

        let time = commit.time();
        let offset_seconds = time.offset_minutes() as i32 * 60;
        let tz = chrono::FixedOffset::east_opt(offset_seconds)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
        let date_time = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .map(|dt| dt.with_timezone(&tz))
            .unwrap_or_else(|| chrono::Utc::now().with_timezone(&tz));
        let date = date_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

        Ok(CommitInfo {
            sha,
            message,
            author: author_str,
            date,
        })
    }

    /// 获取最后一次 commit 的 SHA
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的完整 SHA。
    pub fn get_last_commit_sha() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        Ok(commit.id().to_string())
    }

    /// 获取最后一次 commit 的消息
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的提交消息。
    pub fn get_last_commit_message() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        commit
            .message()
            .ok_or_else(|| eyre!("Commit has no message"))
            .map(|s| s.to_string())
    }

    /// 获取已修改但未暂存的文件列表
    ///
    /// # 返回
    ///
    /// 返回已修改但未暂存的文件路径列表。
    pub fn get_modified_files() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let diff = repo.diff_index_to_workdir(None, None)?;
        let mut files = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            None,
        )
        .wrap_err("Failed to iterate diff")?;

        Ok(files)
    }

    /// 获取未跟踪的文件列表
    ///
    /// # 返回
    ///
    /// 返回未跟踪的文件路径列表。
    pub fn get_untracked_files() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        opts.include_ignored(false);

        let statuses = repo.statuses(Some(&mut opts)).wrap_err("Failed to get status")?;

        let mut files = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_wt_new() {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// 添加指定文件到暂存区
    ///
    /// # 参数
    ///
    /// * `files` - 要添加的文件路径列表
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn add_files(files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let repo = open_repo()?;
        let mut index = repo.index().wrap_err("Failed to open index")?;
        for file in files {
            index
                .add_path(std::path::Path::new(file))
                .wrap_err_with(|| format!("Failed to add file: {}", file))?;
        }
        index.write().wrap_err("Failed to write index")
    }

    /// 执行 commit amend
    ///
    /// # 参数
    ///
    /// * `message` - 新的提交消息（如果为 `None`，使用 `--no-edit`）
    /// * `no_edit` - 是否不编辑消息（使用 `--no-edit`）
    /// * `no_verify` - 是否跳过 pre-commit hooks 验证
    ///
    /// # 行为
    ///
    /// 1. 如果 `no_verify` 为 `false` 且存在 pre-commit hooks，则执行 hooks
    /// 2. 执行 `git commit --amend`
    ///
    /// # 返回
    ///
    /// 返回新的 commit SHA。
    pub fn amend(message: Option<&str>, no_edit: bool, no_verify: bool) -> Result<String> {
        let repo = open_repo()?;

        // 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        let should_skip_hook = if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
            env::set_var("WORKFLOW_PRE_COMMIT_SKIP", "1");
            true
        } else {
            false
        };

        // 获取当前的 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let old_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

        // 获取新的提交消息
        let new_message = if no_edit {
            // 使用原来的消息
            old_commit.message().ok_or_else(|| eyre!("Commit has no message"))?.to_string()
        } else if let Some(msg) = message {
            msg.to_string()
        } else {
            // 如果没有提供消息且 no_edit 为 false，使用原来的消息
            old_commit.message().ok_or_else(|| eyre!("Commit has no message"))?.to_string()
        };

        // 获取当前的索引并写入 tree
        let mut index = repo.index().wrap_err("Failed to open index")?;
        let tree_id = index.write_tree_to(&repo).wrap_err("Failed to write tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        // 获取签名
        let config = repo.config().ok();
        let name = config
            .as_ref()
            .and_then(|c| c.get_string("user.name").ok())
            .unwrap_or_else(|| old_commit.author().name().unwrap_or("Unknown").to_string());
        let email =
            config
                .as_ref()
                .and_then(|c| c.get_string("user.email").ok())
                .unwrap_or_else(|| {
                    old_commit.author().email().unwrap_or("unknown@example.com").to_string()
                });
        let signature = Signature::now(&name, &email).wrap_err("Failed to create signature")?;

        // 获取父提交
        let mut parents = Vec::new();
        for i in 0..old_commit.parent_count() {
            if let Ok(parent) = old_commit.parent(i) {
                parents.push(parent);
            }
        }

        // 创建新的 commit，替换旧的
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &new_message,
            &tree,
            &parent_refs,
        )
        .wrap_err("Failed to amend commit")?;

        // 清理环境变量
        if should_skip_hook {
            env::remove_var("WORKFLOW_PRE_COMMIT_SKIP");
        }

        // 返回新的 commit SHA
        Self::get_last_commit_sha()
    }

    /// 解析 commit 引用为完整的 SHA
    ///
    /// 支持格式：HEAD, HEAD~n, SHA, 分支名等
    ///
    /// # 参数
    ///
    /// * `reference` - Commit 引用（如 "HEAD", "HEAD~2", "abc1234" 等）
    ///
    /// # 返回
    ///
    /// 返回完整的 commit SHA（40 个字符）。
    pub fn parse_commit_ref(reference: &str) -> Result<String> {
        let repo = open_repo()?;
        let obj = repo
            .revparse_single(reference)
            .wrap_err_with(|| format!("Failed to parse commit reference: {}", reference))?;
        Ok(obj.id().to_string())
    }

    /// 获取指定 commit 的信息
    ///
    /// # 参数
    ///
    /// * `commit_ref` - Commit 引用（如 "HEAD", "HEAD~2", SHA 等）
    ///
    /// # 返回
    ///
    /// 返回指定 commit 的详细信息。
    pub fn get_commit_info(commit_ref: &str) -> Result<CommitInfo> {
        let repo = open_repo()?;
        let obj = repo
            .revparse_single(commit_ref)
            .wrap_err_with(|| format!("Failed to parse commit reference: {}", commit_ref))?;
        let commit = obj
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit: {}", commit_ref))?;

        let sha = commit.id().to_string();
        let message = commit.message().ok_or_else(|| eyre!("Commit has no message"))?.to_string();
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown");
        let author_email = author.email().unwrap_or("unknown@example.com");
        let author_str = format!("{} <{}>", author_name, author_email);

        let time = commit.time();
        let offset_seconds = time.offset_minutes() as i32 * 60;
        let tz = chrono::FixedOffset::east_opt(offset_seconds)
            .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
        let date_time = chrono::DateTime::from_timestamp(time.seconds(), 0)
            .map(|dt| dt.with_timezone(&tz))
            .unwrap_or_else(|| chrono::Utc::now().with_timezone(&tz));
        let date = date_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

        Ok(CommitInfo {
            sha,
            message,
            author: author_str,
            date,
        })
    }

    /// 检查 commit 是否是 HEAD
    ///
    /// # 参数
    ///
    /// * `commit_sha` - 要检查的 commit SHA
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 是 HEAD
    /// - `Ok(false)` - 不是 HEAD
    pub fn is_head_commit(commit_sha: &str) -> Result<bool> {
        let head_sha = Self::get_last_commit_sha()?;
        Ok(commit_sha.trim() == head_sha.trim())
    }

    /// 检查 commit 是否在当前分支的历史中
    ///
    /// 使用 git2 检查指定的 commit 是否是当前分支（HEAD）的祖先。
    ///
    /// # 参数
    ///
    /// * `commit_sha` - 要检查的 commit SHA
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - commit 在当前分支历史中
    /// - `Ok(false)` - commit 不在当前分支历史中
    pub fn is_commit_in_current_branch(commit_sha: &str) -> Result<bool> {
        let repo = open_repo()?;

        // 解析 commit SHA
        let commit_oid = repo
            .revparse_single(commit_sha)
            .wrap_err_with(|| format!("Failed to parse commit: {}", commit_sha))?
            .id();

        // 获取 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;
        let head_oid = head_commit.id();

        // 如果 commit_sha == HEAD，返回 true
        if commit_oid == head_oid {
            return Ok(true);
        }

        // 使用 merge_base 检查：如果 merge_base(commit_sha, HEAD) == commit_sha，说明 commit_sha 是 HEAD 的祖先
        let merge_base_oid =
            repo.merge_base(commit_oid, head_oid).wrap_err("Failed to find merge base")?;

        Ok(merge_base_oid == commit_oid)
    }

    /// 获取当前分支的 commits 列表
    ///
    /// 获取当前分支最近的 commits。
    ///
    /// # 参数
    ///
    /// * `count` - 最大返回数量
    ///
    /// # 返回
    ///
    /// 返回 CommitInfo 列表，按时间顺序排列（从新到旧，HEAD 在第一个）。
    pub fn get_branch_commits(count: usize) -> Result<Vec<CommitInfo>> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(head_commit.id()).wrap_err("Failed to push commit")?;

        let mut commits = Vec::new();
        for (i, oid_result) in revwalk.enumerate() {
            if i >= count {
                break;
            }
            let oid = oid_result.wrap_err("Failed to get commit OID")?;
            let commit = repo.find_commit(oid).wrap_err("Failed to find commit")?;

            let sha = commit.id().to_string();
            let message = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown");
            let author_email = author.email().unwrap_or("unknown@example.com");
            let author_str = format!("{} <{}>", author_name, author_email);

            let time = commit.time();
            let offset_seconds = time.offset_minutes() as i32 * 60;
            let tz = chrono::FixedOffset::east_opt(offset_seconds)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
            let date_time = chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.with_timezone(&tz))
                .unwrap_or_else(|| chrono::Utc::now().with_timezone(&tz));
            let date = date_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

            commits.push(CommitInfo {
                sha,
                message,
                author: author_str,
                date,
            });
        }

        Ok(commits)
    }

    /// 获取指定 commit 的父 commit SHA
    ///
    /// # 参数
    ///
    /// * `commit_sha` - 要获取父 commit 的 commit SHA
    ///
    /// # 返回
    ///
    /// 返回父 commit 的完整 SHA。如果 commit 没有父 commit（根 commit），返回错误。
    pub fn get_parent_commit(commit_sha: &str) -> Result<String> {
        let repo = open_repo()?;
        let obj = repo
            .revparse_single(commit_sha)
            .wrap_err_with(|| format!("Failed to parse commit: {}", commit_sha))?;
        let commit = obj
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit: {}", commit_sha))?;

        let parent = commit
            .parent(0)
            .wrap_err_with(|| format!("Commit {} has no parent", commit_sha))?;
        Ok(parent.id().to_string())
    }

    /// 获取从指定 commit（不包括）到 HEAD 的所有 commits
    ///
    /// 用于构建 rebase todo 文件，获取需要 rebase 的 commits 列表。
    ///
    /// # 参数
    ///
    /// * `from_commit` - 起始 commit SHA（不包括此 commit）
    ///
    /// # 返回
    ///
    /// 返回 CommitInfo 列表，按时间顺序排列（从旧到新，第一个是最接近 from_commit 的 commit）。
    pub fn get_commits_from_to_head(from_commit: &str) -> Result<Vec<CommitInfo>> {
        let repo = open_repo()?;

        // 解析 from_commit
        let from_obj = repo
            .revparse_single(from_commit)
            .wrap_err_with(|| format!("Failed to parse commit: {}", from_commit))?;
        let from_commit_obj = from_obj
            .peel_to_commit()
            .wrap_err_with(|| format!("Failed to get commit: {}", from_commit))?;

        // 获取 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get HEAD commit")?;

        // 创建 revwalk，从 HEAD 到 from_commit（不包括）
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(head_commit.id()).wrap_err("Failed to push HEAD commit")?;
        revwalk.hide(from_commit_obj.id()).wrap_err("Failed to hide from commit")?;

        let mut commits = Vec::new();
        for oid_result in revwalk {
            let oid = oid_result.wrap_err("Failed to get commit OID")?;
            let commit = repo.find_commit(oid).wrap_err("Failed to find commit")?;

            let sha = commit.id().to_string();
            let message = commit.message().unwrap_or("").lines().next().unwrap_or("").to_string();
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown");
            let author_email = author.email().unwrap_or("unknown@example.com");
            let author_str = format!("{} <{}>", author_name, author_email);

            let time = commit.time();
            let offset_seconds = time.offset_minutes() as i32 * 60;
            let tz = chrono::FixedOffset::east_opt(offset_seconds)
                .unwrap_or_else(|| chrono::FixedOffset::east_opt(0).unwrap());
            let date_time = chrono::DateTime::from_timestamp(time.seconds(), 0)
                .map(|dt| dt.with_timezone(&tz))
                .unwrap_or_else(|| chrono::Utc::now().with_timezone(&tz));
            let date = date_time.format("%Y-%m-%d %H:%M:%S %z").to_string();

            commits.push(CommitInfo {
                sha,
                message,
                author: author_str,
                date,
            });
        }

        // 反转列表，使其从旧到新排列
        commits.reverse();
        Ok(commits)
    }

    /// 获取工作区状态统计
    ///
    /// 解析 Git status 输出，统计已修改、已暂存和未跟踪的文件数量。
    ///
    /// # 返回
    ///
    /// 返回工作区状态统计信息。
    pub fn get_worktree_status() -> Result<WorktreeStatus> {
        let status = Self::status()?;

        let modified_count =
            status.lines().filter(|l| l.starts_with(" M") || l.starts_with("M ")).count();
        let staged_count = status
            .lines()
            .filter(|l| l.starts_with("M ") || l.starts_with("A ") || l.starts_with("D "))
            .count();
        let untracked_count = status.lines().filter(|l| l.starts_with("??")).count();

        Ok(WorktreeStatus {
            modified_count,
            staged_count,
            untracked_count,
        })
    }

    /// 格式化工作区状态为字符串
    ///
    /// # 参数
    ///
    /// * `status` - 工作区状态
    ///
    /// # 返回
    ///
    /// 返回格式化的字符串。
    pub fn format_worktree_status(status: &WorktreeStatus) -> String {
        format!(
            "  工作区状态:\n    - 已修改文件:  {} 个\n    - 已暂存文件:  {} 个\n    - 未跟踪文件:  {} 个",
            status.modified_count, status.staged_count, status.untracked_count
        )
    }
}
