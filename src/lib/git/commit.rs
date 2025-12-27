//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）
//! - 修改最后一次提交（amend）
//! - 修改历史提交消息（reword）

use chrono::{FixedOffset, TimeZone};
use color_eyre::{eyre::WrapErr, Result};

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
    /// 使用 git2 库获取 Git 状态的简洁输出格式（类似 `git status --porcelain`）。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串（porcelain 格式）。
    pub fn status() -> Result<String> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        let mut lines = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            if let Some(path) = entry.path() {
                // 格式化状态行（porcelain 格式）
                // 格式：XY path
                // X = 暂存区状态，Y = 工作区状态
                let mut status_str = String::new();

                // 暂存区状态
                if status.is_index_new() {
                    status_str.push('A');
                } else if status.is_index_modified() {
                    status_str.push('M');
                } else if status.is_index_deleted() {
                    status_str.push('D');
                } else if status.is_index_renamed() {
                    status_str.push('R');
                } else if status.is_index_typechange() {
                    status_str.push('T');
                } else {
                    status_str.push(' ');
                }

                // 工作区状态
                if status.is_wt_new() {
                    status_str.push('?');
                } else if status.is_wt_modified() {
                    status_str.push('M');
                } else if status.is_wt_deleted() {
                    status_str.push('D');
                } else if status.is_wt_typechange() {
                    status_str.push('T');
                } else if status.is_wt_renamed() {
                    status_str.push('R');
                } else {
                    status_str.push(' ');
                }

                lines.push(format!("{} {}", status_str, path));
            }
        }

        Ok(lines.join("\n"))
    }

    /// 检查工作区是否有未提交的更改
    ///
    /// 使用 git2 库检查工作区和暂存区是否有未提交的更改。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有未提交的更改（工作区或暂存区）
    /// - `Ok(false)` - 如果没有未提交的更改
    pub fn has_commit() -> Result<bool> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        // 检查是否有任何更改（工作区或暂存区）
        for entry in statuses.iter() {
            let status = entry.status();
            // 如果有任何非忽略的状态，说明有未提交的更改
            if !status.is_ignored() && status != git2::Status::CURRENT {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// 检查是否有暂存的文件
    ///
    /// 使用 `git diff --cached --quiet` 检查暂存区是否有文件。
    /// `--quiet` 选项会在有差异时返回非零退出码，无差异时返回 0。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有暂存的文件
    /// - `Ok(false)` - 如果没有暂存的文件
    pub(crate) fn has_staged() -> Result<bool> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(false);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        // 检查是否有暂存的文件（索引区有更改）
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
    /// 使用 git2 库将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        let repo = open_repo()?;
        let mut index = repo.index().wrap_err("Failed to open repository index")?;

        // 添加所有文件（包括修改、新增和删除的文件）
        index
            .add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)
            .wrap_err("Failed to add all files to index")?;

        index.write().wrap_err("Failed to write index")?;
        Ok(())
    }

    /// 提交更改
    ///
    /// 使用 git2 库自动暂存所有已修改的文件，然后提交。
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
    /// 4. 执行提交操作（使用 git2）
    ///
    /// # 返回
    ///
    /// 返回 `CommitResult`，包含提交状态和消息。
    pub fn commit(message: &str, no_verify: bool) -> Result<CommitResult> {
        let has_changes = Self::has_commit()?;

        if !has_changes {
            return Ok(CommitResult {
                committed: false,
                message: Some("Nothing to commit, working tree clean".to_string()),
            });
        }

        // 注意：即使文件已经在暂存区，执行 add_all() 也是安全的，不会造成问题
        // 这样可以确保所有更改都被暂存，包括未暂存和已暂存的更改
        Self::add_all().wrap_err("Failed to stage changes")?;

        // 执行 pre-commit hooks（如果需要）
        if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
        }

        // 使用 git2 创建提交
        let repo = open_repo()?;
        let mut index = repo.index().wrap_err("Failed to open repository index")?;

        // 将索引写入树
        let tree_id = index.write_tree().wrap_err("Failed to write index to tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        // 获取签名（作者和提交者）
        let signature = repo.signature().wrap_err("Failed to get repository signature")?;

        // 获取父提交（如果有）
        let parent_commit = repo
            .head()
            .ok()
            .and_then(|head| head.target().and_then(|oid| repo.find_commit(oid).ok()));
        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        // 创建提交
        let _commit_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )
            .wrap_err("Failed to create commit")?;

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
        let repo = open_repo().ok()?;
        let mut diff_parts = Vec::new();

        // 获取 HEAD 树和索引
        let head_tree = repo.head().ok()?.peel_to_tree().ok();
        let index = repo.index().ok()?;

        // 获取暂存区的修改（HEAD -> Index）
        if let Some(ref tree) = head_tree {
            if let Ok(diff) = repo.diff_tree_to_index(Some(tree), Some(&index), None) {
                let mut staged_output = Vec::new();
                if diff
                    .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                        staged_output.extend_from_slice(line.content());
                        true
                    })
                    .is_ok()
                    && !staged_output.is_empty()
                {
                    if let Ok(staged_diff) = String::from_utf8(staged_output) {
                        if !staged_diff.trim().is_empty() {
                            diff_parts.push(format!("Staged changes:\n{}", staged_diff));
                        }
                    }
                }
            }
        } else {
            // 如果没有 HEAD（新仓库），获取索引中的所有文件
            if let Ok(diff) = repo.diff_tree_to_index(None, Some(&index), None) {
                let mut staged_output = Vec::new();
                if diff
                    .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                        staged_output.extend_from_slice(line.content());
                        true
                    })
                    .is_ok()
                    && !staged_output.is_empty()
                {
                    if let Ok(staged_diff) = String::from_utf8(staged_output) {
                        if !staged_diff.trim().is_empty() {
                            diff_parts.push(format!("Staged changes:\n{}", staged_diff));
                        }
                    }
                }
            }
        }

        // 获取工作区的修改（Index -> Workdir）
        if let Ok(diff) = repo.diff_index_to_workdir(Some(&index), None) {
            let mut worktree_output = Vec::new();
            if diff
                .print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                    worktree_output.extend_from_slice(line.content());
                    true
                })
                .is_ok()
                && !worktree_output.is_empty()
            {
                if let Ok(worktree_diff) = String::from_utf8(worktree_output) {
                    if !worktree_diff.trim().is_empty() {
                        diff_parts.push(format!("Working tree changes:\n{}", worktree_diff));
                    }
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
    /// 使用 git2 库将工作区和暂存区重置到指定提交。
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
        let target_ref = target.unwrap_or("HEAD");

        // 解析目标引用
        let obj = repo
            .revparse_single(target_ref)
            .wrap_err_with(|| format!("Failed to parse target reference: {}", target_ref))?;

        // 执行硬重置（重置索引和工作区）
        repo.reset(
            &obj,
            git2::ResetType::Hard,
            Some(
                git2::build::CheckoutBuilder::default()
                    .force()
                    .remove_ignored(false)
                    .remove_untracked(false),
            ),
        )
        .wrap_err_with(|| format!("Failed to reset working directory to {}", target_ref))?;

        Ok(())
    }

    /// 检查是否有最后一次 commit
    ///
    /// 使用 git2 库检查是否有 commit 历史。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有 commit
    /// - `Ok(false)` - 如果没有 commit
    pub fn has_last_commit() -> Result<bool> {
        let repo = open_repo()?;
        let head_result = repo.head();
        match head_result {
            Ok(head) => {
                let commit_result = head.peel_to_commit();
                Ok(commit_result.is_ok())
            }
            Err(_) => Ok(false),
        }
    }

    /// 检查是否有最后一次 commit（指定仓库路径）
    ///
    /// 使用 git2 库检查指定仓库是否有 commit 历史。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有 commit
    /// - `Ok(false)` - 如果没有 commit
    pub fn has_last_commit_in(repo_path: impl AsRef<std::path::Path>) -> Result<bool> {
        let repo: git2::Repository =
            git2::Repository::open(repo_path.as_ref()).wrap_err("Failed to open repository")?;
        let head_result = repo.head();
        match head_result {
            Ok(head) => {
                let commit_result = head.peel_to_commit();
                Ok(commit_result.is_ok())
            }
            Err(_) => Ok(false),
        }
    }

    /// 获取最后一次 commit 信息
    ///
    /// 使用 git2 库获取最后一次 commit 的详细信息。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的详细信息。
    pub fn get_last_commit_info() -> Result<CommitInfo> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;

        let sha = commit.id().to_string();
        let message = commit
            .message()
            .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown");
        let author_email = author.email().unwrap_or("unknown@example.com");
        let author_str = format!("{} <{}>", author_name, author_email);

        let time = commit.time();
        // 格式化日期时间：YYYY-MM-DD HH:MM:SS +HHMM
        // 使用 chrono 库格式化时间
        let offset = FixedOffset::east_opt(time.offset_minutes() * 60)
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
        let datetime = offset
            .timestamp_opt(time.seconds(), 0)
            .single()
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timestamp"))?;
        let date = datetime.format("%Y-%m-%d %H:%M:%S %z").to_string();

        Ok(CommitInfo {
            sha,
            message,
            author: author_str,
            date,
        })
    }

    /// 获取最后一次 commit 的 SHA
    ///
    /// 使用 git2 库获取最后一次 commit 的完整 SHA。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的完整 SHA。
    pub fn get_last_commit_sha() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let oid = head
            .target()
            .ok_or_else(|| color_eyre::eyre::eyre!("HEAD does not point to a valid commit"))?;
        Ok(oid.to_string())
    }

    /// 获取最后一次 commit 的消息
    ///
    /// 使用 git2 库获取最后一次 commit 的提交消息。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的提交消息。
    pub fn get_last_commit_message() -> Result<String> {
        let repo = open_repo()?;
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;
        Ok(commit
            .message()
            .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
            .to_string())
    }

    /// 获取已修改但未暂存的文件列表
    ///
    /// 使用 git2 库获取已修改但未暂存的文件路径列表。
    ///
    /// # 返回
    ///
    /// 返回已修改但未暂存的文件路径列表。
    pub fn get_modified_files() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(false);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        let mut files = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            // 只包含已修改但未暂存的文件（工作区修改）
            if status.is_wt_modified() || status.is_wt_deleted() {
                if let Some(path) = entry.path() {
                    files.push(path.to_string());
                }
            }
        }

        Ok(files)
    }

    /// 获取未跟踪的文件列表
    ///
    /// 使用 git2 库获取未跟踪的文件路径列表。
    ///
    /// # 返回
    ///
    /// 返回未跟踪的文件路径列表。
    pub fn get_untracked_files() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        let mut files = Vec::new();
        for entry in statuses.iter() {
            let status = entry.status();
            // 只包含未跟踪的文件
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
    /// 使用 git2 库添加指定文件到暂存区。
    ///
    /// # 参数
    ///
    /// * `files` - 要添加的文件路径列表
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn add_files(files: &[String]) -> Result<()> {
        if files.is_empty() {
            return Ok(());
        }

        let repo = open_repo()?;
        let mut index = repo.index().wrap_err("Failed to open repository index")?;

        for file in files {
            index
                .add_path(file.as_ref())
                .wrap_err_with(|| format!("Failed to add file to index: {}", file))?;
        }

        index.write().wrap_err("Failed to write index")?;
        Ok(())
    }

    /// 执行 commit amend
    ///
    /// 使用 git2 库修改最后一次提交。
    ///
    /// # 参数
    ///
    /// * `message` - 新的提交消息（如果为 `None` 且 `no_edit` 为 `false`，保留原消息）
    /// * `no_edit` - 是否不编辑消息（保留原消息）
    /// * `no_verify` - 是否跳过 pre-commit hooks 验证
    ///
    /// # 行为
    ///
    /// 1. 如果 `no_verify` 为 `false` 且存在 pre-commit hooks，则执行 hooks
    /// 2. 使用 git2 修改最后一次提交
    ///
    /// # 返回
    ///
    /// 返回新的 commit SHA。
    pub fn amend(message: Option<&str>, no_edit: bool, no_verify: bool) -> Result<String> {
        // 执行 pre-commit hooks（如果需要）
        if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
        }

        let repo = open_repo()?;

        // 获取当前 HEAD 提交
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let parent_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;

        // 获取索引并写入树
        let mut index = repo.index().wrap_err("Failed to open repository index")?;
        let tree_id = index.write_tree().wrap_err("Failed to write index to tree")?;
        let tree = repo.find_tree(tree_id).wrap_err("Failed to find tree")?;

        // 获取签名
        let signature = repo.signature().wrap_err("Failed to get repository signature")?;

        // 确定提交消息
        let commit_message = if no_edit {
            // 保留原消息
            parent_commit
                .message()
                .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
                .to_string()
        } else if let Some(msg) = message {
            msg.to_string()
        } else {
            // 如果没有提供消息且 no_edit 为 false，保留原消息
            parent_commit
                .message()
                .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
                .to_string()
        };

        // 获取父提交（amend 的父提交是原提交的父提交）
        // 需要先获取父提交，然后构建向量
        let parent_commit_ref = parent_commit.parent(0).ok();
        let parents: Vec<&git2::Commit> = if let Some(ref parent) = parent_commit_ref {
            vec![parent]
        } else {
            Vec::new()
        };

        // 创建新的提交（amend）
        let commit_oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &commit_message,
                &tree,
                &parents,
            )
            .wrap_err("Failed to amend commit")?;

        Ok(commit_oid.to_string())
    }

    /// 解析 commit 引用为完整的 SHA
    ///
    /// 使用 git2 库解析 commit 引用为完整的 SHA。
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

        // 如果是 commit，直接返回 SHA
        if let Some(commit) = obj.as_commit() {
            return Ok(commit.id().to_string());
        }

        // 如果是 tag，获取其目标
        if let Some(tag) = obj.as_tag() {
            return Ok(tag.target_id().to_string());
        }

        // 如果是其他对象，尝试 peel 到 commit
        let commit = obj
            .peel_to_commit()
            .wrap_err_with(|| format!("Reference {} does not point to a commit", reference))?;

        Ok(commit.id().to_string())
    }

    /// 获取指定 commit 的信息
    ///
    /// 使用 git2 库获取指定 commit 的详细信息。
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
            .wrap_err_with(|| format!("Failed to get commit from reference: {}", commit_ref))?;

        let sha = commit.id().to_string();
        let message = commit
            .message()
            .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
            .to_string();

        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown");
        let author_email = author.email().unwrap_or("unknown@example.com");
        let author_str = format!("{} <{}>", author_name, author_email);

        let time = commit.time();
        // 格式化日期时间：YYYY-MM-DD HH:MM:SS +HHMM
        // 使用 chrono 库格式化时间
        let offset = FixedOffset::east_opt(time.offset_minutes() * 60)
            .unwrap_or_else(|| FixedOffset::east_opt(0).unwrap());
        let datetime = offset
            .timestamp_opt(time.seconds(), 0)
            .single()
            .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timestamp"))?;
        let date = datetime.format("%Y-%m-%d %H:%M:%S %z").to_string();

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
    /// 使用 `git merge-base --is-ancestor` 检查指定的 commit
    /// 是否是当前分支（HEAD）的祖先。
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
        // 使用 git merge-base --is-ancestor <commit_sha> HEAD
        // 如果 commit_sha 是 HEAD 的祖先，返回 true
        // 注意：如果 commit_sha == HEAD，也返回 true
        let repo = open_repo()?;
        let commit_oid = git2::Oid::from_str(commit_sha)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit_sha))?;
        let head = repo.head()?.peel_to_commit()?;
        let merge_base = repo.merge_base(commit_oid, head.id())?;
        Ok(merge_base == head.id())
    }

    /// 获取当前分支的 commits 列表
    ///
    /// 使用 git2 库获取当前分支最近的 commits。
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

        // 获取 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;

        // 使用 revwalk 遍历提交历史
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(head_commit.id()).wrap_err("Failed to push HEAD to revwalk")?;

        let mut commits = Vec::new();
        for (index, oid) in revwalk.enumerate() {
            if index >= count {
                break;
            }

            let oid = oid.wrap_err("Failed to get commit OID from revwalk")?;
            let commit = repo
                .find_commit(oid)
                .wrap_err_with(|| format!("Failed to find commit: {}", oid))?;

            // 获取提交信息
            let sha = commit.id().to_string();
            let message = commit
                .message()
                .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
                .to_string();

            // 获取作者信息
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown");
            let author_email = author.email().unwrap_or("unknown@example.com");
            let author_str = format!("{} <{}>", author_name, author_email);

            // 获取日期并格式化
            let time = commit.time();
            let offset = FixedOffset::east_opt(time.offset_minutes() * 60)
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timezone offset"))?;
            let datetime = offset
                .timestamp_opt(time.seconds(), 0)
                .single()
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timestamp"))?;
            let date = datetime.format("%Y-%m-%d %H:%M:%S %z").to_string();

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
    /// 使用 git2 库获取指定 commit 的父 commit SHA。
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
        let commit_oid = git2::Oid::from_str(commit_sha)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", commit_sha))?;
        let commit = repo
            .find_commit(commit_oid)
            .wrap_err_with(|| format!("Failed to find commit: {}", commit_sha))?;

        let parent = commit
            .parent(0)
            .wrap_err_with(|| format!("Commit {} has no parent (root commit)", commit_sha))?;

        Ok(parent.id().to_string())
    }

    /// 获取从指定 commit（不包括）到 HEAD 的所有 commits
    ///
    /// 使用 git2 库获取从指定 commit 到 HEAD 的所有 commits，用于构建 rebase todo 文件。
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

        // 解析 from_commit SHA
        let from_oid = git2::Oid::from_str(from_commit)
            .wrap_err_with(|| format!("Invalid commit SHA: {}", from_commit))?;
        let from_commit_obj = repo
            .find_commit(from_oid)
            .wrap_err_with(|| format!("Failed to find commit: {}", from_commit))?;

        // 获取 HEAD commit
        let head = repo.head().wrap_err("Failed to get HEAD reference")?;
        let head_commit = head.peel_to_commit().wrap_err("Failed to get commit from HEAD")?;

        // 如果 from_commit 就是 HEAD，返回空列表
        if from_commit_obj.id() == head_commit.id() {
            return Ok(Vec::new());
        }

        // 使用 revwalk 遍历从 from_commit 到 HEAD 的所有 commits
        let mut revwalk = repo.revwalk().wrap_err("Failed to create revwalk")?;
        revwalk.push(head_commit.id()).wrap_err("Failed to push HEAD to revwalk")?;
        revwalk.hide(from_oid).wrap_err("Failed to hide from_commit from revwalk")?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid.wrap_err("Failed to get commit OID from revwalk")?;
            let commit = repo
                .find_commit(oid)
                .wrap_err_with(|| format!("Failed to find commit: {}", oid))?;

            // 获取提交信息
            let sha = commit.id().to_string();
            let message = commit
                .message()
                .ok_or_else(|| color_eyre::eyre::eyre!("Commit message is not valid UTF-8"))?
                .to_string();

            // 获取作者信息
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown");
            let author_email = author.email().unwrap_or("unknown@example.com");
            let author_str = format!("{} <{}>", author_name, author_email);

            // 获取日期并格式化
            let time = commit.time();
            let offset = FixedOffset::east_opt(time.offset_minutes() * 60)
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timezone offset"))?;
            let datetime = offset
                .timestamp_opt(time.seconds(), 0)
                .single()
                .ok_or_else(|| color_eyre::eyre::eyre!("Invalid timestamp"))?;
            let date = datetime.format("%Y-%m-%d %H:%M:%S %z").to_string();

            commits.push(CommitInfo {
                sha,
                message,
                author: author_str,
                date,
            });
        }

        // 反转列表，使其从旧到新排列（--reverse 的效果）
        commits.reverse();

        Ok(commits)
    }

    /// 获取工作区状态统计
    ///
    /// 使用 git2 库统计已修改、已暂存和未跟踪的文件数量。
    ///
    /// # 返回
    ///
    /// 返回工作区状态统计信息。
    pub fn get_worktree_status() -> Result<WorktreeStatus> {
        let repo = open_repo()?;
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = repo
            .statuses(Some(&mut status_options))
            .wrap_err("Failed to get repository statuses")?;

        let mut modified_count = 0;
        let mut staged_count = 0;
        let mut untracked_count = 0;

        for entry in statuses.iter() {
            let status = entry.status();

            // 统计工作区修改的文件（已修改但未暂存）
            if status.is_wt_modified() || status.is_wt_deleted() {
                modified_count += 1;
            }

            // 统计暂存区的文件（已暂存）
            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
                staged_count += 1;
            }

            // 统计未跟踪的文件
            if status.is_wt_new() {
                untracked_count += 1;
            }
        }

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
