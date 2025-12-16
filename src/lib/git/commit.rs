//! Git 提交相关操作
//!
//! 本模块提供了 Git 提交相关的核心功能，包括：
//! - 检查 Git 状态和工作区更改
//! - 暂存文件（add）
//! - 提交更改（commit）
//! - 修改最后一次提交（amend）
//! - 修改历史提交消息（reword）

use color_eyre::{eyre::WrapErr, Result};

use super::helpers::{check_success, cmd_read, cmd_run, cmd_run_with_env};
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
    /// 使用 `--porcelain` 选项获取简洁、稳定的输出格式。
    /// 该格式适合程序解析，不包含颜色和装饰性输出。
    ///
    /// # 返回
    ///
    /// 返回 Git 状态的简洁输出字符串。
    pub fn status() -> Result<String> {
        cmd_read(&["status", "--porcelain"]).wrap_err("Failed to run git status")
    }

    /// 检查工作区是否有未提交的更改
    ///
    /// 检查工作区和暂存区是否有未提交的更改。
    /// 使用 `git diff --quiet` 检查工作区，使用 `git diff --cached --quiet` 检查暂存区。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有未提交的更改（工作区或暂存区）
    /// - `Ok(false)` - 如果没有未提交的更改
    #[allow(dead_code)]
    pub fn has_commit() -> Result<bool> {
        // 检查工作区是否有未提交的更改
        // 如果有差异，返回非零退出码（is_err() 返回 true）
        let has_worktree_changes = !check_success(&["diff", "--quiet"]);

        // 检查暂存区是否有未提交的更改
        let has_staged_changes = Self::has_staged()?;

        Ok(has_worktree_changes || has_staged_changes)
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
        // 使用 git diff --cached --quiet 检查是否有暂存的文件
        // --quiet 选项：如果有差异，返回非零退出码；如果没有差异，返回 0
        // 所以如果命令成功（返回 true），说明没有暂存的文件；如果失败（返回 false），说明有暂存的文件
        Ok(!check_success(&["diff", "--cached", "--quiet"]))
    }

    /// 添加所有文件到暂存区
    ///
    /// 使用 `git add --all` 将所有已修改、新增和删除的文件添加到暂存区。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn add_all() -> Result<()> {
        cmd_run(&["add", "--all"]).wrap_err("Failed to add all files")
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
        // 1. 使用 git diff --quiet 检查是否有更改（更高效）
        let has_changes = Self::has_commit()?;

        // 2. 如果没有更改，直接返回
        if !has_changes {
            return Ok(CommitResult {
                committed: false,
                message: Some("Nothing to commit, working tree clean".to_string()),
            });
        }

        // 3. 暂存所有已修改的文件
        // 注意：即使文件已经在暂存区，执行 add_all() 也是安全的，不会造成问题
        // 这样可以确保所有更改都被暂存，包括未暂存和已暂存的更改
        Self::add_all().wrap_err("Failed to stage changes")?;

        // 6. 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        let should_skip_hook = if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
            true // 已经通过 Rust 代码执行了检查，hook 脚本应该跳过
        } else {
            false
        };

        // 7. 执行提交
        let mut args = vec!["commit", "-m", message];
        if no_verify {
            args.push("--no-verify");
        }

        // 如果已经通过 Rust 代码执行了检查，设置环境变量告诉 hook 脚本跳过执行
        if should_skip_hook {
            cmd_run_with_env(&args, &[("WORKFLOW_PRE_COMMIT_SKIP", "1")])?;
        } else {
            cmd_run(&args).wrap_err("Failed to commit")?;
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
        let mut diff_parts = Vec::new();

        // 获取暂存区的修改
        if let Ok(staged_diff) = cmd_read(&["diff", "--cached"]) {
            if !staged_diff.trim().is_empty() {
                diff_parts.push(format!("Staged changes:\n{}", staged_diff));
            }
        }

        // 获取工作区的修改
        if let Ok(worktree_diff) = cmd_read(&["diff"]) {
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
    /// 使用 `git reset --hard <target>` 将工作区和暂存区重置到指定提交。
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
        let target = target.unwrap_or("HEAD");
        cmd_run(&["reset", "--hard", target])
            .wrap_err_with(|| format!("Failed to reset working directory to {}", target))
    }

    /// 检查是否有最后一次 commit
    ///
    /// 使用 `git log -1` 检查是否有 commit 历史。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果有 commit
    /// - `Ok(false)` - 如果没有 commit
    pub fn has_last_commit() -> Result<bool> {
        Ok(check_success(&["log", "-1", "--oneline"]))
    }

    /// 获取最后一次 commit 信息
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的详细信息。
    pub fn get_last_commit_info() -> Result<CommitInfo> {
        let sha = cmd_read(&["log", "-1", "--format=%H"])?;
        let message = cmd_read(&["log", "-1", "--format=%s"])?;
        let author = cmd_read(&["log", "-1", "--format=%an <%ae>"])?;
        let date = cmd_read(&["log", "-1", "--format=%ai"])?;

        Ok(CommitInfo {
            sha,
            message,
            author,
            date,
        })
    }

    /// 获取最后一次 commit 的 SHA
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的完整 SHA。
    pub fn get_last_commit_sha() -> Result<String> {
        cmd_read(&["log", "-1", "--format=%H"]).wrap_err("Failed to get last commit SHA")
    }

    /// 获取最后一次 commit 的消息
    ///
    /// # 返回
    ///
    /// 返回最后一次 commit 的提交消息。
    pub fn get_last_commit_message() -> Result<String> {
        cmd_read(&["log", "-1", "--format=%s"]).wrap_err("Failed to get last commit message")
    }

    /// 获取已修改但未暂存的文件列表
    ///
    /// # 返回
    ///
    /// 返回已修改但未暂存的文件路径列表。
    pub fn get_modified_files() -> Result<Vec<String>> {
        let output = cmd_read(&["diff", "--name-only"])?;
        if output.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(output.lines().map(|s| s.to_string()).collect())
        }
    }

    /// 获取未跟踪的文件列表
    ///
    /// # 返回
    ///
    /// 返回未跟踪的文件路径列表。
    pub fn get_untracked_files() -> Result<Vec<String>> {
        let output = cmd_read(&["ls-files", "--others", "--exclude-standard"])?;
        if output.trim().is_empty() {
            Ok(Vec::new())
        } else {
            Ok(output.lines().map(|s| s.to_string()).collect())
        }
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

        let mut args = vec!["add"];
        for file in files {
            args.push(file);
        }

        cmd_run(&args).wrap_err("Failed to add files")
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
        // 如果不需要跳过验证，且存在 pre-commit，则先执行 pre-commit
        let should_skip_hook = if !no_verify && GitPreCommit::has_pre_commit() {
            GitPreCommit::run_pre_commit()?;
            true // 已经通过 Rust 代码执行了检查，hook 脚本应该跳过
        } else {
            false
        };

        // 构建命令参数
        let mut args = vec!["commit", "--amend"];

        if no_edit {
            args.push("--no-edit");
        } else if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }

        if no_verify {
            args.push("--no-verify");
        }

        // 如果已经通过 Rust 代码执行了检查，设置环境变量告诉 hook 脚本跳过执行
        if should_skip_hook {
            cmd_run_with_env(&args, &[("WORKFLOW_PRE_COMMIT_SKIP", "1")])?;
        } else {
            cmd_run(&args).wrap_err("Failed to amend commit")?;
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
        let sha = cmd_read(&["rev-parse", reference])
            .wrap_err(format!("Failed to parse commit reference: {}", reference))?;
        Ok(sha.trim().to_string())
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
        let sha = cmd_read(&["log", "-1", "--format=%H", commit_ref])
            .wrap_err(format!("Failed to get commit info for: {}", commit_ref))?;
        let message = cmd_read(&["log", "-1", "--format=%s", commit_ref])
            .wrap_err(format!("Failed to get commit message for: {}", commit_ref))?;
        let author = cmd_read(&["log", "-1", "--format=%an <%ae>", commit_ref])
            .wrap_err(format!("Failed to get commit author for: {}", commit_ref))?;
        let date = cmd_read(&["log", "-1", "--format=%ai", commit_ref])
            .wrap_err(format!("Failed to get commit date for: {}", commit_ref))?;

        Ok(CommitInfo {
            sha: sha.trim().to_string(),
            message: message.trim().to_string(),
            author: author.trim().to_string(),
            date: date.trim().to_string(),
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
        Ok(check_success(&["merge-base", "--is-ancestor", commit_sha, "HEAD"]))
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
        // 使用 git log 获取当前分支的 commits
        // git log -n <count> --format="%H|%s|%an <%ae>|%ai"
        let output = cmd_read(&[
            "log",
            &format!("-{}", count),
            "--format=%H|%s|%an <%ae>|%ai",
        ])?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut commits = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                commits.push(CommitInfo {
                    sha: parts[0].trim().to_string(),
                    message: parts[1].trim().to_string(),
                    author: parts[2].trim().to_string(),
                    date: parts[3].trim().to_string(),
                });
            }
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
        let parent_sha = cmd_read(&["rev-parse", &format!("{}^", commit_sha)])
            .wrap_err_with(|| format!("Failed to get parent commit of: {}", commit_sha))?;
        Ok(parent_sha.trim().to_string())
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
        // 使用 git log 获取从 from_commit 到 HEAD 的所有 commits
        // git log from_commit..HEAD --format="%H|%s|%an <%ae>|%ai" --reverse
        // --reverse 表示从旧到新排列
        let output = cmd_read(&[
            "log",
            &format!("{}..HEAD", from_commit),
            "--format=%H|%s|%an <%ae>|%ai",
            "--reverse",
        ])?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut commits = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 4 {
                commits.push(CommitInfo {
                    sha: parts[0].trim().to_string(),
                    message: parts[1].trim().to_string(),
                    author: parts[2].trim().to_string(),
                    date: parts[3].trim().to_string(),
                });
            }
        }

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
