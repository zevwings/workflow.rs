use anyhow::{Context, Result};
use duct::cmd;

/// Git 操作模块
pub struct Git;

impl Git {
    /// 检查 Git 状态
    pub fn status() -> Result<String> {
        cmd("git", &["status"])
            .read()
            .context("Failed to run git status")
    }

    /// 获取当前分支名
    pub fn current_branch() -> Result<String> {
        let output = cmd("git", &["branch", "--show-current"])
            .read()
            .context("Failed to get current branch")?;

        Ok(output.trim().to_string())
    }

    /// 添加所有文件到暂存区
    pub fn add_all() -> Result<()> {
        cmd("git", &["add", "--all"])
            .run()
            .context("Failed to add all files")?;
        Ok(())
    }

    /// 检查分支是否存在（本地或远程）
    /// 返回 (本地存在, 远程存在)
    fn branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        // 检查本地分支
        let local_branches = cmd("git", &["branch", "--list"])
            .read()
            .context("Failed to list local branches")?;
        let exists_local = local_branches
            .lines()
            .any(|line| line.trim().trim_start_matches('*').trim() == branch_name);

        // 检查远程分支
        let remote_branches = cmd("git", &["branch", "-r", "--list"])
            .read()
            .context("Failed to list remote branches")?;
        let exists_remote = remote_branches
            .lines()
            .any(|line| {
                line.trim()
                    .strip_prefix("origin/")
                    .map(|b| b == branch_name)
                    .unwrap_or(false)
            });

        Ok((exists_local, exists_remote))
    }

    /// 创建或切换到分支
    ///
    /// 如果分支已存在且是当前分支，则跳过
    /// 如果分支已存在但不是当前分支，则切换到该分支
    /// 如果分支只存在于远程，则创建本地分支并跟踪远程分支
    /// 如果分支不存在，则创建新分支
    pub fn checkout_branch(branch_name: &str) -> Result<()> {
        // 检查是否是当前分支
        let current_branch = Self::current_branch()?;
        if current_branch == branch_name {
            // 已经是当前分支，无需操作
            return Ok(());
        }

        // 检查分支是否存在
        let (exists_local, exists_remote) = Self::branch_exists(branch_name)?;

        if exists_local {
            // 分支已存在于本地，切换到它
            cmd("git", &["checkout", branch_name])
                .run()
                .context(format!("Failed to checkout branch: {}", branch_name))?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            cmd("git", &["checkout", "-b", branch_name, &format!("origin/{}", branch_name)])
                .run()
                .context(format!("Failed to checkout remote branch: {}", branch_name))?;
        } else {
            // 分支不存在，创建新分支
            cmd("git", &["checkout", "-b", branch_name])
                .run()
                .context(format!("Failed to create branch: {}", branch_name))?;
        }
        Ok(())
    }

    /// 提交更改
    pub fn commit(message: &str, no_verify: bool) -> Result<()> {
        let mut args = vec!["commit", "-m", message];
        if no_verify {
            args.push("--no-verify");
        }

        cmd("git", &args).run().context("Failed to commit")?;
        Ok(())
    }

    /// 推送到远程仓库
    pub fn push(branch_name: &str, set_upstream: bool) -> Result<()> {
        let mut args = vec!["push"];
        if set_upstream {
            args.push("-u");
        }
        args.push("origin");
        args.push(branch_name);

        cmd("git", &args)
            .run()
            .context(format!("Failed to push branch: {}", branch_name))?;
        Ok(())
    }

    /// 快速更新代码
    ///
    /// # Arguments
    /// * `commit_message` - 提交消息。如果为 None，使用默认消息 "update"
    ///
    /// # 执行的操作
    /// 1. 执行 `git add --all`
    /// 2. 执行 `git commit`（使用指定的提交消息）
    /// 3. 执行 `git push`
    ///
    /// # Note
    /// 此方法只负责 Git 操作，不关心提交消息的来源（可以是 PR 标题、自定义消息等）
    pub fn update(commit_message: Option<String>) -> Result<()> {
        use crate::{log_success, log_warning};
        use anyhow::Context;

        // 1. 确定提交消息
        let message = commit_message.unwrap_or_else(|| {
            log_warning!("No commit message provided, using default message");
            "update".to_string()
        });

        log_success!("Using commit message: {}", message);

        // 2. 执行 git add --all
        log_success!("Staging all changes...");
        Self::add_all().context("Failed to stage changes")?;

        // 3. 执行 git commit
        log_success!("Committing changes...");
        Self::commit(&message, false)?; // 不使用 --no-verify

        // 4. 执行 git push
        let current_branch = Self::current_branch()?;
        log_success!("Pushing to remote...");
        Self::push(&current_branch, false)?; // 不使用 -u（分支应该已经存在）

        log_success!("Update completed successfully!");
        Ok(())
    }
}
