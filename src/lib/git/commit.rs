//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）
//! - 修改最后一次提交（amend）
//! - 修改历史提交消息（reword）

use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

use super::commands::{GitCommitCommand, GitResetCommand};
use super::pre_commit::GitPreCommit;
use super::GitRepository;

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
    /// 使用 `git status --porcelain` 命令获取 Git 状态的简洁输出格式。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串（porcelain 格式）。
    pub fn status() -> Result<String> {
        let repo = GitRepository::open()?;
        GitCommitCommand::status(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get git status")
    }

    /// 检查工作区是否有未提交的更改
    ///
    /// 使用 `git status --porcelain` 命令检查工作区和暂存区是否有未提交的更改。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有未提交的更改（工作区或暂存区）
    /// - `Ok(false)` - 如果没有未提交的更改
    pub fn has_commit() -> Result<bool> {
        let repo = GitRepository::open()?;
        GitCommitCommand::has_changes(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to check repository status")
    }

    /// 检查指定路径的仓库是否有未提交的更改
    ///
    /// 使用 `git status --porcelain` 命令检查指定路径的 Git 仓库是否有未提交的更改
    /// （工作区或暂存区）。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库根目录路径
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 有未提交的更改
    /// - `Ok(false)` - 没有未提交的更改
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn has_commit_in(repo_path: impl AsRef<std::path::Path>) -> Result<bool> {
        let repo = GitRepository::open_at(repo_path)?;
        GitCommitCommand::has_changes(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to check repository status")
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
        let repo = GitRepository::open()?;
        // 使用 git diff --cached --quiet 检查暂存区
        // 如果命令成功（退出码为 0），说明没有暂存的文件
        // 如果命令失败（退出码非 0），说明有暂存的文件
        let has_staged = GitCommitCommand::has_staged(Some(repo.path()));
        Ok(has_staged)
    }

    /// 添加所有文件到暂存区
    ///
    /// 使用 `git add .` 命令将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        let repo = GitRepository::open()?;
        GitCommitCommand::add_all(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to stage all files")
    }

    /// 提交更改
    ///
    /// 使用 `git commit` 命令自动暂存所有已修改的文件，然后提交。
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
    /// 4. 执行提交操作（使用 git commit 命令）
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

        // 使用 git commit 命令创建提交
        let repo = GitRepository::open()?;
        GitCommitCommand::commit(message, no_verify, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to create commit")?;

        Ok(CommitResult {
            committed: true,
            message: None,
        })
    }

    /// 在指定路径创建提交
    ///
    /// 在指定路径的 Git 仓库中创建提交，不依赖当前工作目录。
    /// 如果 `auto_stage` 为 `true`，会自动暂存所有已修改的文件。
    ///
    /// # 参数
    ///
    /// * `repo_path` - 仓库路径
    /// * `message` - 提交消息
    /// * `auto_stage` - 是否自动暂存所有文件
    ///
    /// # 返回
    ///
    /// 返回 `CommitResult`，包含提交状态和消息。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::git::GitCommit;
    /// # use color_eyre::Result;
    /// # fn main() -> Result<()> {
    /// let result = GitCommit::commit_at("/path/to/repo", "Add new feature", true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn commit_at(
        repo_path: impl AsRef<Path>,
        message: &str,
        auto_stage: bool,
    ) -> Result<CommitResult> {
        let repo_path = repo_path.as_ref();

        // 打开仓库
        let repo = GitRepository::open_at(repo_path)?;

        // 检查是否有未提交的更改
        let has_changes = GitCommitCommand::has_changes(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to check repository status")?;

        if !has_changes {
            return Ok(CommitResult {
                committed: false,
                message: Some("Nothing to commit, working tree clean".to_string()),
            });
        }

        // 自动暂存（如果需要）
        if auto_stage {
            GitCommitCommand::add_all(Some(repo.path()))
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))
                .wrap_err("Failed to stage all files")?;
        }

        // 使用 git commit 命令创建提交
        GitCommitCommand::commit(message, false, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
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
        let repo = GitRepository::open().ok()?;
        let mut diff_parts = Vec::new();

        // 获取暂存区的修改（HEAD -> Index）
        if let Ok(staged_diff) = GitCommitCommand::get_diff(true, Some(repo.path())) {
            if !staged_diff.trim().is_empty() {
                diff_parts.push(format!("Staged changes:\n{}", staged_diff));
            }
        }

        // 获取工作区的修改（Index -> Workdir）
        if let Ok(worktree_diff) = GitCommitCommand::get_diff(false, Some(repo.path())) {
            if !worktree_diff.trim().is_empty() {
                diff_parts.push(format!("Working tree changes:\n{}", worktree_diff));
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
    /// 使用 `git reset --hard` 命令将工作区和暂存区重置到指定提交。
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
        let repo = GitRepository::open()?;
        let target_ref = target.unwrap_or("HEAD");

        GitResetCommand::reset_hard(Some(target_ref), Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to reset working directory to {}", target_ref))
    }

    /// 检查是否有最后一次 commit
    ///
    /// 使用 `git rev-parse HEAD` 命令检查是否有 commit 历史。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有 commit
    /// - `Ok(false)` - 如果没有 commit
    pub fn has_last_commit() -> Result<bool> {
        let repo = GitRepository::open()?;
        // 尝试获取 HEAD SHA，如果成功则说明有 commit
        match GitCommitCommand::get_head_sha(Some(repo.path())) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 检查是否有最后一次 commit（指定仓库路径）
    ///
    /// 使用 `git rev-parse HEAD` 命令检查指定仓库是否有 commit 历史。
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
        let repo = GitRepository::open_at(repo_path)?;
        // 尝试获取 HEAD SHA，如果成功则说明有 commit
        match GitCommitCommand::get_head_sha(Some(repo.path())) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 获取最后一次 commit 信息
    ///
    /// 使用 `git log` 命令获取最后一次 commit 的详细信息。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的详细信息。
    pub fn get_last_commit_info() -> Result<CommitInfo> {
        Self::get_commit_info("HEAD")
    }

    /// 获取最后一次 commit 的 SHA
    ///
    /// 使用 `git rev-parse HEAD` 命令获取最后一次 commit 的完整 SHA。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的完整 SHA。
    pub fn get_last_commit_sha() -> Result<String> {
        let repo = GitRepository::open()?;
        GitCommitCommand::get_head_sha(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get HEAD SHA")
    }

    /// 获取最后一次 commit 的消息
    ///
    /// 使用 `git log` 命令获取最后一次 commit 的提交消息。
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的提交消息。
    pub fn get_last_commit_message() -> Result<String> {
        let repo = GitRepository::open()?;
        GitCommitCommand::get_commit_message("HEAD", Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get last commit message")
    }

    /// 获取已修改但未暂存的文件列表
    ///
    /// 使用 `git status --porcelain` 命令获取已修改但未暂存的文件路径列表。
    ///
    /// # 返回
    ///
    /// 返回已修改但未暂存的文件路径列表。
    pub fn get_modified_files() -> Result<Vec<String>> {
        let repo = GitRepository::open()?;
        let status_output = GitCommitCommand::status(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get repository status")?;

        let mut files = Vec::new();
        for line in status_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // porcelain 格式：XY path
            // 如果 Y 是 M 或 D，说明工作区有修改
            if line.len() >= 2 {
                let worktree_status = line.chars().nth(1).unwrap_or(' ');
                if worktree_status == 'M' || worktree_status == 'D' {
                    // 提取文件路径（跳过状态字符和空格）
                    if let Some(path) = line.get(3..) {
                        files.push(path.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    /// 获取未跟踪的文件列表
    ///
    /// 使用 `git status --porcelain` 命令获取未跟踪的文件路径列表。
    ///
    /// # 返回
    ///
    /// 返回未跟踪的文件路径列表。
    pub fn get_untracked_files() -> Result<Vec<String>> {
        let repo = GitRepository::open()?;
        let status_output = GitCommitCommand::status(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get repository status")?;

        let mut files = Vec::new();
        for line in status_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // porcelain 格式：XY path
            // 如果 Y 是 ?，说明是未跟踪的文件
            if line.len() >= 2 {
                let worktree_status = line.chars().nth(1).unwrap_or(' ');
                if worktree_status == '?' {
                    // 提取文件路径（跳过状态字符和空格）
                    if let Some(path) = line.get(3..) {
                        files.push(path.to_string());
                    }
                }
            }
        }

        Ok(files)
    }

    /// 添加指定文件到暂存区
    ///
    /// 使用 `git add` 命令添加指定文件到暂存区。
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

        let repo = GitRepository::open()?;
        for file in files {
            GitCommitCommand::add(file, Some(repo.path()))
                .map_err(|e| color_eyre::eyre::eyre!("{}", e))
                .wrap_err_with(|| format!("Failed to add file to index: {}", file))?;
        }

        Ok(())
    }

    /// 执行 commit amend
    ///
    /// 使用 `git commit --amend` 命令修改最后一次提交。
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
    /// 2. 使用 `git commit --amend` 命令修改最后一次提交
    ///
    /// # 返回
    ///
    /// 返回新的 commit SHA。
    pub fn amend(message: Option<&str>, no_edit: bool, no_verify: bool) -> Result<String> {
        // 执行 pre-commit hooks（如果需要）
        if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
        }

        let repo = GitRepository::open()?;

        // 根据 no_edit 参数决定是否传递消息
        // 如果 no_edit 为 true，传递 None 以保留原消息
        // 如果 no_edit 为 false 且有 message，使用新消息
        // 如果 no_edit 为 false 且无 message，也传递 None 以保留原消息
        let amend_message = if no_edit { None } else { message };

        // 使用 git commit --amend 命令
        GitCommitCommand::amend(amend_message, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to amend commit")?;

        // 获取新的 commit SHA
        GitCommitCommand::get_head_sha(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get amended commit SHA")
    }

    /// 解析 commit 引用为完整的 SHA
    ///
    /// 使用 `git rev-parse` 命令解析 commit 引用为完整的 SHA。
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
        let repo = GitRepository::open()?;
        GitCommitCommand::rev_parse(reference, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to parse commit reference: {}", reference))
    }

    /// 获取指定 commit 的信息
    ///
    /// 使用 `git log` 命令获取指定 commit 的详细信息。
    ///
    /// # 参数
    ///
    /// * `commit_ref` - Commit 引用（如 "HEAD", "HEAD~2", SHA 等）
    ///
    /// # 返回
    ///
    /// 返回指定 commit 的详细信息。
    pub fn get_commit_info(commit_ref: &str) -> Result<CommitInfo> {
        let repo = GitRepository::open()?;

        // 获取 commit SHA
        let sha = GitCommitCommand::rev_parse(commit_ref, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to parse commit reference: {}", commit_ref))?;

        // 获取提交信息
        let (message, author, date) = GitCommitCommand::get_commit_info(&sha, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to get commit info: {}", commit_ref))?;

        Ok(CommitInfo {
            sha,
            message,
            author,
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
        let repo = GitRepository::open()?;
        // 使用 git merge-base --is-ancestor 检查
        // 如果命令成功（退出码为 0），说明 commit_sha 是 HEAD 的祖先
        let is_ancestor = GitCommitCommand::is_ancestor(commit_sha, "HEAD", Some(repo.path()));
        Ok(is_ancestor)
    }

    /// 获取当前分支的 commits 列表
    ///
    /// 使用 `git log` 命令获取当前分支最近的 commits。
    ///
    /// # 参数
    ///
    /// * `count` - 最大返回数量
    ///
    /// # 返回
    ///
    /// 返回 CommitInfo 列表，按时间顺序排列（从新到旧，HEAD 在第一个）。
    pub fn get_branch_commits(count: usize) -> Result<Vec<CommitInfo>> {
        let repo = GitRepository::open()?;

        // 使用 git log 获取提交列表
        // 格式：%H|%s|%an <%ae>|%ai
        let log_output = GitCommitCommand::log(
            Some(count),
            "%H|%s|%an <%ae>|%ai",
            None,
            false,
            Some(repo.path()),
        )
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
        .wrap_err("Failed to get branch commits")?;

        let mut commits = Vec::new();
        for line in log_output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                commits.push(CommitInfo {
                    sha: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                });
            }
        }

        Ok(commits)
    }

    /// 获取指定 commit 的父 commit SHA
    ///
    /// 使用 `git rev-parse` 命令获取指定 commit 的父 commit SHA。
    ///
    /// # 参数
    ///
    /// * `commit_sha` - 要获取父 commit 的 commit SHA
    ///
    /// # 返回
    ///
    /// 返回父 commit 的完整 SHA。如果 commit 没有父 commit（根 commit），返回错误。
    pub fn get_parent_commit(commit_sha: &str) -> Result<String> {
        let repo = GitRepository::open()?;
        GitCommitCommand::get_parent_sha(commit_sha, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Commit {} has no parent (root commit)", commit_sha))
    }

    /// 获取从指定 commit（不包括）到 HEAD 的所有 commits
    ///
    /// 使用 `git log` 命令获取从指定 commit 到 HEAD 的所有 commits，用于构建 rebase todo 文件。
    ///
    /// # 参数
    ///
    /// * `from_commit` - 起始 commit SHA（不包括此 commit）
    ///
    /// # 返回
    ///
    /// 返回 CommitInfo 列表，按时间顺序排列（从旧到新，第一个是最接近 from_commit 的 commit）。
    pub fn get_commits_from_to_head(from_commit: &str) -> Result<Vec<CommitInfo>> {
        let repo = GitRepository::open()?;

        // 检查 from_commit 是否是 HEAD
        let from_sha = GitCommitCommand::rev_parse(from_commit, Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to parse commit reference: {}", from_commit))?;

        let head_sha = GitCommitCommand::get_head_sha(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get HEAD SHA")?;

        // 如果 from_commit 就是 HEAD，返回空列表
        if from_sha == head_sha {
            return Ok(Vec::new());
        }

        // 使用 git log 获取从 from_commit 到 HEAD 的所有 commits
        // 格式：%H|%s|%an <%ae>|%ai
        // 使用 --reverse 使其从旧到新排列
        let log_output = GitCommitCommand::log(
            None,
            "%H|%s|%an <%ae>|%ai",
            Some(&format!("{}..HEAD", from_commit)),
            true,
            Some(repo.path()),
        )
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))
        .wrap_err("Failed to get commits from to head")?;

        let mut commits = Vec::new();
        for line in log_output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                commits.push(CommitInfo {
                    sha: parts[0].to_string(),
                    message: parts[1].to_string(),
                    author: parts[2].to_string(),
                    date: parts[3].to_string(),
                });
            }
        }

        Ok(commits)
    }

    /// 获取工作区状态统计
    ///
    /// 使用 `git status --porcelain` 命令统计已修改、已暂存和未跟踪的文件数量。
    ///
    /// # 返回
    ///
    /// 返回工作区状态统计信息。
    pub fn get_worktree_status() -> Result<WorktreeStatus> {
        let repo = GitRepository::open()?;
        let status_output = GitCommitCommand::status(Some(repo.path()))
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err("Failed to get repository status")?;

        let mut modified_count = 0;
        let mut staged_count = 0;
        let mut untracked_count = 0;

        for line in status_output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // porcelain 格式：XY path
            // X = 暂存区状态，Y = 工作区状态
            if line.len() >= 2 {
                let index_status = line.chars().nth(0).unwrap_or(' ');
                let worktree_status = line.chars().nth(1).unwrap_or(' ');

                // 统计工作区修改的文件（已修改但未暂存）
                if worktree_status == 'M' || worktree_status == 'D' {
                    modified_count += 1;
                }

                // 统计暂存区的文件（已暂存）
                if index_status == 'A' || index_status == 'M' || index_status == 'D' {
                    staged_count += 1;
                }

                // 统计未跟踪的文件
                if worktree_status == '?' {
                    untracked_count += 1;
                }
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
