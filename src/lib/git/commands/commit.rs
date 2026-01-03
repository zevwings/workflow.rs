//! Git 提交操作命令封装
//!
//! 提供提交相关的所有 Git 命令操作，包括：
//! - 状态检查（status、has_changes）
//! - 暂存操作（add、add_all）
//! - 提交操作（commit、amend）
//! - 提交信息（get_commit_info、get_diff）

use crate::git::commands::command::{git_options, GitCommand};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 提交命令操作
pub struct GitCommitCommand;

impl GitCommitCommand {
    /// 检查 Git 状态
    ///
    /// 使用 `git status --porcelain` 命令
    pub fn status(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["status", git_options::PORCELAIN], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to get git status")
    }

    /// 检查是否有未提交的更改
    ///
    /// 使用 `git status --porcelain` 命令
    pub fn has_changes(cwd: Option<&Path>) -> Result<bool> {
        let output = Self::status(cwd)?;
        Ok(!output.trim().is_empty())
    }

    /// 暂存文件
    ///
    /// 使用 `git add <file>` 命令
    pub fn add(file: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["add", file], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to stage file: {}", file))
    }

    /// 暂存所有文件
    ///
    /// 使用 `git add .` 命令
    pub fn add_all(cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["add", "."], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to stage all files")
    }

    /// 创建提交
    ///
    /// 使用 `git commit -m <message>` 命令
    pub fn commit(message: &str, no_verify: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["commit", "-m", message];

        if no_verify {
            args.push(git_options::NO_VERIFY);
        }

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to commit: {}", message))
    }

    /// 修改最后一次提交
    ///
    /// 使用 `git commit --amend` 命令
    pub fn amend(message: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["commit", "--amend"];

        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        } else {
            args.push("--no-edit");
        }

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to amend commit")
    }

    /// 获取当前 HEAD 的 SHA
    ///
    /// 使用 `git rev-parse HEAD` 命令
    pub fn get_head_sha(cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "HEAD"], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err("Failed to get HEAD SHA")
    }

    /// 获取提交信息
    ///
    /// 使用 `git log --format` 命令，通过一次调用获取所有信息
    ///
    /// # 返回
    ///
    /// 返回 `(消息, 作者, 日期)` 元组
    pub fn get_commit_info(
        commit_sha: &str,
        cwd: Option<&Path>,
    ) -> Result<(String, String, String)> {
        // 使用一次 git log 调用获取所有信息
        // 格式：%s%n%an <%ae>%n%ai
        // 第一行：提交消息
        // 第二行：作者
        // 第三行：日期
        let output = GitCommand::run(
            &["log", "-1", "--format=%s%n%an <%ae>%n%ai", commit_sha],
            cwd,
        )
        .map_err(GitCommand::to_eyre_error)?;

        let lines: Vec<&str> = output.lines().collect();
        if lines.len() < 3 {
            return Err(color_eyre::eyre::eyre!(
                "Unexpected output format from git log for commit: {}",
                commit_sha
            ));
        }

        Ok((
            lines[0].trim().to_string(),
            lines[1].trim().to_string(),
            lines[2].trim().to_string(),
        ))
    }

    /// 获取差异内容
    ///
    /// 使用 `git diff` 命令
    pub fn get_diff(staged: bool, cwd: Option<&Path>) -> Result<String> {
        let mut args = vec!["diff"];

        if staged {
            args.push("--cached");
        }

        GitCommand::run(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to get diff")
    }

    /// 检查是否有暂存的文件
    ///
    /// 使用 `git diff --cached --quiet` 命令
    pub fn has_staged(cwd: Option<&Path>) -> bool {
        !GitCommand::check(&["diff", "--cached", "--quiet"], cwd)
    }

    /// 检查工作区是否有冲突
    ///
    /// 使用 `git diff --check` 命令
    pub fn check_conflicts(cwd: Option<&Path>) -> Result<bool> {
        let output = GitCommand::run(&["diff", "--check"], cwd).ok();
        Ok(output.is_some() && !output.unwrap_or_default().trim().is_empty())
    }

    /// 获取提交的 SHA
    ///
    /// 使用 `git rev-parse <ref>` 命令
    pub fn rev_parse(reference: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", reference], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to parse reference: {}", reference))
    }

    /// 验证引用是否存在
    ///
    /// 使用 `git rev-parse --verify <ref>` 命令
    pub fn verify_ref(reference: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["rev-parse", "--verify", reference], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to verify reference: {}", reference))
    }

    /// 获取父提交的 SHA
    ///
    /// 使用 `git rev-parse <ref>^` 命令
    pub fn get_parent_sha(commit_sha: &str, cwd: Option<&Path>) -> Result<String> {
        Self::rev_parse(&format!("{}^", commit_sha), cwd)
    }

    /// 获取提交日志（单行格式）
    ///
    /// 使用 `git log --oneline <from>..<to>` 或 `git log --oneline <ref>` 命令
    pub fn log_oneline(
        from: Option<&str>,
        to: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<Vec<String>> {
        let mut args: Vec<&str> = vec!["log", "--oneline"];
        let range;

        if let (Some(from_ref), Some(to_ref)) = (from, to) {
            range = format!("{}..{}", from_ref, to_ref);
            args.push(&range);
        } else if let Some(ref_ref) = from.or(to) {
            args.push(ref_ref);
        }

        let output = GitCommand::run(&args, cwd).map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 获取提交消息
    ///
    /// 使用 `git log -1 --format=%s <ref>` 命令
    pub fn get_commit_message(commit_sha: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["log", "-1", "--format=%s", commit_sha], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to get commit message for: {}", commit_sha))
    }

    /// 检查提交是否是另一个提交的祖先
    ///
    /// 使用 `git merge-base --is-ancestor <ancestor> <descendant>` 命令
    pub fn is_ancestor(ancestor: &str, descendant: &str, cwd: Option<&Path>) -> bool {
        GitCommand::check(&["merge-base", "--is-ancestor", ancestor, descendant], cwd)
    }

    /// 查找两个提交的共同祖先
    ///
    /// 使用 `git merge-base <commit1> <commit2>` 命令
    pub fn merge_base(commit1: &str, commit2: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["merge-base", commit1, commit2], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| {
                format!(
                    "Failed to find merge base between {} and {}",
                    commit1, commit2
                )
            })
    }

    /// 列出提交列表
    ///
    /// 使用 `git rev-list <range>` 命令
    pub fn rev_list(range: &str, cwd: Option<&Path>) -> Result<Vec<String>> {
        let output =
            GitCommand::run(&["rev-list", range], cwd).map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 统计提交数量
    ///
    /// 使用 `git rev-list --count <range>` 命令
    pub fn rev_list_count(range: &str, cwd: Option<&Path>) -> Result<u32> {
        let output = GitCommand::run(&["rev-list", "--count", range], cwd)
            .map_err(GitCommand::to_eyre_error)?;

        output
            .trim()
            .parse()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to parse rev-list count: {}", e))
    }

    /// 获取提交日志（自定义格式）
    ///
    /// 使用 `git log` 命令，支持自定义格式和范围
    pub fn log(
        count: Option<usize>,
        format: &str,
        range: Option<&str>,
        reverse: bool,
        cwd: Option<&Path>,
    ) -> Result<String> {
        let mut args: Vec<String> = vec!["log".to_string()];
        let format_str = format.to_string();

        if reverse {
            args.push("--reverse".to_string());
        }

        let count_str;
        if let Some(n) = count {
            count_str = format!("-{}", n);
            args.push(count_str);
        }

        args.push("--format".to_string());
        args.push(format_str);

        if let Some(r) = range {
            args.push(r.to_string());
        }

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        GitCommand::run(&args_refs, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err("Failed to get log")
    }

    /// 显示对象内容
    ///
    /// 使用 `git show` 命令显示对象（提交、标签等）的内容
    pub fn show(
        reference: &str,
        stat: bool,
        format: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<String> {
        let mut args: Vec<String> = vec!["show".to_string()];
        let format_str;

        if stat {
            args.push("--stat".to_string());
        }

        if let Some(fmt) = format {
            format_str = format!("--format={}", fmt);
            args.push(format_str);
        }

        args.push(reference.to_string());

        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        GitCommand::run(&args_refs, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to show {}", reference))
    }
}
