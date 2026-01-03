//! Git 分支操作命令封装
//!
//! 提供分支相关的所有 Git 命令操作，包括：
//! - 分支查询（当前分支、分支列表、分支存在性检查）
//! - 分支操作（创建、切换、删除）
//! - 分支合并（merge、rebase）
//! - 分支推送（push、pull）

use crate::git::commands::command::{git_options, git_refs, GitCommand};
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 分支命令操作
pub struct GitBranchCommand;

impl GitBranchCommand {
    /// 获取当前分支名
    ///
    /// 使用 `git branch --show-current` 命令
    pub fn current_branch(cwd: Option<&Path>) -> Result<String> {
        let output = GitCommand::run(&["branch", git_options::SHOW_CURRENT], cwd)
            .map_err(GitCommand::to_eyre_error)?;

        let branch = output.trim();
        if branch.is_empty() {
            return Err(color_eyre::eyre::eyre!(
                "Not on any branch (detached HEAD state)"
            ));
        }

        Ok(branch.to_string())
    }

    /// 检查分支是否存在（本地）
    ///
    /// 使用 `git show-ref --verify` 命令
    pub fn branch_exists_local(branch_name: &str, cwd: Option<&Path>) -> Result<bool> {
        Ok(GitCommand::check(
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", branch_name),
            ],
            cwd,
        ))
    }

    /// 检查分支是否存在（远程）
    ///
    /// 使用 `git show-ref --verify` 命令
    pub fn branch_exists_remote(
        branch_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<bool> {
        let remote = remote.unwrap_or("origin");
        Ok(GitCommand::check(
            &[
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/remotes/{}/{}", remote, branch_name),
            ],
            cwd,
        ))
    }

    /// 检查分支是否存在（本地和远程）
    ///
    /// 返回 `(本地存在, 远程存在)`
    pub fn branch_exists(
        branch_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<(bool, bool)> {
        let exists_local = Self::branch_exists_local(branch_name, cwd)?;
        let exists_remote = Self::branch_exists_remote(branch_name, remote, cwd)?;
        Ok((exists_local, exists_remote))
    }

    /// 创建分支
    ///
    /// 使用 `git branch <name>` 命令
    pub fn create_branch(branch_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["branch", branch_name], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to create branch: {}", branch_name))
    }

    /// 创建或切换分支
    ///
    /// 优先使用 `git switch`（Git 2.23+），失败时回退到 `git checkout`
    pub fn checkout_branch(branch_name: &str, create: bool, cwd: Option<&Path>) -> Result<()> {
        if create {
            // 优先使用 git switch -c
            if GitCommand::execute(&["switch", "-c", branch_name], cwd).is_ok() {
                return Ok(());
            }
            // 回退到 git checkout -b
            GitCommand::execute(&["checkout", "-b", branch_name], cwd)
                .map_err(GitCommand::to_eyre_error)
        } else {
            // 优先使用 git switch
            if GitCommand::execute(&["switch", branch_name], cwd).is_ok() {
                return Ok(());
            }
            // 回退到 git checkout
            GitCommand::execute(&["checkout", branch_name], cwd).map_err(GitCommand::to_eyre_error)
        }
        .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))
    }

    /// 删除分支
    ///
    /// 使用 `git branch -d` 或 `git branch -D` 命令
    pub fn delete_branch(branch_name: &str, force: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["branch"];
        if force {
            args.push("-D");
        } else {
            args.push("-d");
        }
        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to delete branch: {}", branch_name))
    }

    /// 获取所有本地分支
    ///
    /// 使用 `git branch` 命令
    pub fn list_branches(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output = GitCommand::run(&["branch"], cwd).map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines_with(&output, |s| {
            s.trim_start_matches('*').trim().to_string()
        }))
    }

    /// 检查分支是否已合并
    ///
    /// 使用 `git branch --merged` 命令
    pub fn is_merged(branch_name: &str, target: Option<&str>, cwd: Option<&Path>) -> Result<bool> {
        let target = target.unwrap_or(git_refs::HEAD);
        let output = GitCommand::run(&["branch", "--merged", target], cwd)
            .map_err(GitCommand::to_eyre_error)?;

        // 检查输出中是否包含该分支名
        Ok(output.lines().any(|line| {
            let line = line.trim().trim_start_matches('*').trim();
            line == branch_name || line.ends_with(&format!("/{}", branch_name))
        }))
    }

    /// 合并分支
    ///
    /// 使用 `git merge` 命令
    pub fn merge_branch(
        branch_name: &str,
        strategy: Option<&str>,
        no_ff: bool,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let mut args = vec!["merge"];

        if let Some(strategy) = strategy {
            args.push("--strategy");
            args.push(strategy);
        }

        if no_ff {
            args.push(git_options::NO_FF);
        }

        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::handle_merge_error)
            .wrap_err_with(|| format!("Failed to merge branch: {}", branch_name))
    }

    /// 推送分支
    ///
    /// 使用 `git push` 命令
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    /// * `force` - 是否使用 `--force-with-lease` 强制推送
    /// * `remote` - 远程仓库名称（默认为 "origin"）
    /// * `cwd` - 工作目录（可选）
    pub fn push(
        branch_name: &str,
        force: bool,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        let mut args = vec!["push"];

        if force {
            args.push(git_options::FORCE_WITH_LEASE);
        }

        args.push(remote);
        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::handle_auth_error)
            .wrap_err_with(|| format!("Failed to push branch {} to {}", branch_name, remote))
    }

    /// 推送分支并设置上游分支
    ///
    /// 使用 `git push -u` 命令推送分支并设置上游跟踪
    pub fn push_with_upstream(
        branch_name: &str,
        force: bool,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        let mut args = vec!["push", "-u"];

        if force {
            args.push(git_options::FORCE_WITH_LEASE);
        }

        args.push(remote);
        args.push(branch_name);

        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::handle_auth_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to push branch {} to {} with upstream",
                    branch_name, remote
                )
            })
    }

    /// 拉取分支
    ///
    /// 使用 `git pull` 命令
    pub fn pull(branch_name: &str, remote: Option<&str>, cwd: Option<&Path>) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        GitCommand::execute(&["pull", remote, branch_name], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to pull branch {} from {}", branch_name, remote))
    }

    /// 切换分支
    ///
    /// 使用 `git checkout <branch>` 命令
    pub fn checkout(branch_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["checkout", branch_name], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to checkout branch: {}", branch_name))
    }

    /// 创建并切换分支
    ///
    /// 使用 `git checkout -b <branch> [<start-point>]` 命令
    pub fn checkout_create(
        branch_name: &str,
        start_point: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let mut args = vec!["checkout", "-b", branch_name];
        if let Some(start) = start_point {
            args.push(start);
        }
        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to create and checkout branch: {}", branch_name))
    }

    /// 合并分支（快进模式）
    ///
    /// 使用 `git merge --ff-only <branch>` 命令
    pub fn merge_ff_only(branch_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["merge", git_options::FF_ONLY, branch_name], cwd)
            .map_err(GitCommand::handle_merge_error)
            .wrap_err_with(|| format!("Failed to fast-forward merge branch: {}", branch_name))
    }

    /// 合并分支（压缩模式）
    ///
    /// 使用 `git merge --squash <branch>` 命令
    pub fn merge_squash(branch_name: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["merge", git_options::SQUASH, branch_name], cwd)
            .map_err(GitCommand::handle_merge_error)
            .wrap_err_with(|| format!("Failed to squash merge branch: {}", branch_name))
    }

    /// 变基分支
    ///
    /// 使用 `git rebase <branch>` 命令
    pub fn rebase(target_branch: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["rebase", target_branch], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to rebase onto branch: {}", target_branch))
    }

    /// 变基分支（指定新基和上游）
    ///
    /// 使用 `git rebase --onto <newbase> <upstream> <branch>` 命令
    pub fn rebase_onto(
        newbase: &str,
        upstream: &str,
        branch: &str,
        cwd: Option<&Path>,
    ) -> Result<()> {
        GitCommand::execute(&["rebase", "--onto", newbase, upstream, branch], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to rebase {} onto {} from {}",
                    branch, newbase, upstream
                )
            })
    }

    /// 推送分支到远程
    ///
    /// 使用 `git push <remote> <branch>` 命令
    ///
    /// **注意**：此方法与 `push()` 功能相同，只是参数顺序不同。
    /// 建议使用 `push()` 方法以保持一致性。
    #[deprecated(note = "使用 push() 方法代替，参数顺序：push(branch_name, force, remote, cwd)")]
    pub fn push_branch(
        branch_name: &str,
        remote: Option<&str>,
        force: bool,
        cwd: Option<&Path>,
    ) -> Result<()> {
        Self::push(branch_name, force, remote, cwd)
    }

    /// 删除远程分支
    ///
    /// 使用 `git push <remote> --delete <branch>` 命令
    pub fn delete_remote_branch(
        branch_name: &str,
        remote: Option<&str>,
        cwd: Option<&Path>,
    ) -> Result<()> {
        let remote = remote.unwrap_or(GitCommand::DEFAULT_REMOTE);
        GitCommand::execute(&["push", remote, "--delete", branch_name], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| {
                format!(
                    "Failed to delete remote branch {} from {}",
                    branch_name, remote
                )
            })
    }

    /// 获取远程分支列表
    ///
    /// 使用 `git branch -r --format=%(refname:short)` 命令
    pub fn list_remote_branches(remote: Option<&str>, cwd: Option<&Path>) -> Result<Vec<String>> {
        let mut args: Vec<&str> = vec!["branch", "-r", "--format=%(refname:short)"];
        let pattern;
        if let Some(remote_name) = remote {
            pattern = format!("refs/remotes/{}", remote_name);
            args.push(&pattern);
        }
        let output = GitCommand::run(&args, cwd).map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 获取本地分支列表（格式化）
    ///
    /// 使用 `git branch --format=%(refname:short)` 命令
    pub fn list_local_branches_formatted(cwd: Option<&Path>) -> Result<Vec<String>> {
        let output = GitCommand::run(&["branch", "--format=%(refname:short)"], cwd)
            .map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 获取符号引用
    ///
    /// 使用 `git symbolic-ref <ref>` 命令
    pub fn symbolic_ref(reference: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["symbolic-ref", reference], cwd)
            .map_err(GitCommand::to_eyre_error)
            .map(|s| s.trim().to_string())
            .wrap_err_with(|| format!("Failed to get symbolic ref: {}", reference))
    }

    /// 列出远程引用
    ///
    /// 使用 `git ls-remote --symref <remote> <ref>` 命令
    pub fn ls_remote_symref(remote: &str, reference: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["ls-remote", "--symref", remote, reference], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to list remote refs: {} {}", remote, reference))
    }

    /// 获取分支的提交信息
    ///
    /// 使用 `git log --format` 命令
    pub fn get_branch_commits(branch: &str, format: &str, cwd: Option<&Path>) -> Result<String> {
        GitCommand::run(&["log", "--format", format, branch], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to get commits for branch: {}", branch))
    }

    /// 检查提交是否在远程分支中
    ///
    /// 使用 `git branch -r --contains <commit>` 命令
    pub fn remote_branch_contains(commit_sha: &str, cwd: Option<&Path>) -> Result<Vec<String>> {
        let output = GitCommand::run(&["branch", "-r", "--contains", commit_sha], cwd)
            .map_err(GitCommand::to_eyre_error)?;

        Ok(GitCommand::parse_lines(&output))
    }

    /// 设置分支的上游分支
    ///
    /// 使用 `git branch --set-upstream-to <upstream> <branch>` 命令
    pub fn set_upstream_to(upstream: &str, branch: &str, cwd: Option<&Path>) -> Result<()> {
        GitCommand::execute(&["branch", "--set-upstream-to", upstream, branch], cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to set upstream {} for branch {}", upstream, branch))
    }

    /// 重命名分支
    ///
    /// 使用 `git branch -m [<old>] <new>` 命令
    pub fn rename_branch(old_name: Option<&str>, new_name: &str, cwd: Option<&Path>) -> Result<()> {
        let mut args = vec!["branch", "-m"];
        if let Some(old) = old_name {
            args.push(old);
        }
        args.push(new_name);
        GitCommand::execute(&args, cwd)
            .map_err(GitCommand::to_eyre_error)
            .wrap_err_with(|| format!("Failed to rename branch to: {}", new_name))
    }
}
