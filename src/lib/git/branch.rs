use anyhow::{Context, Result};
use duct::cmd;

use super::commit::Git;

/// 合并策略枚举
///
/// 定义不同的 Git 合并策略。
#[derive(Debug, Clone, Copy)]
pub enum MergeStrategy {
    /// 普通合并（创建合并提交）
    Merge,
    /// Squash 合并（将分支的所有提交压缩为一个提交）
    Squash,
    /// 只允许 fast-forward 合并（如果无法 fast-forward 则失败）
    FastForwardOnly,
}

impl Git {
    /// 获取当前分支名
    ///
    /// 使用 `git branch --show-current` 获取当前分支的名称。
    ///
    /// # 返回
    ///
    /// 返回当前分支的名称（去除首尾空白）。
    ///
    /// # 错误
    ///
    /// 如果不在 Git 仓库中或命令执行失败，返回相应的错误信息。
    pub fn current_branch() -> Result<String> {
        let output = cmd("git", &["branch", "--show-current"])
            .read()
            .context("Failed to get current branch")?;

        Ok(output.trim().to_string())
    }

    /// 检查分支是否存在（本地或远程）
    ///
    /// 使用 `git rev-parse --verify` 检查指定分支在本地和远程是否存在。
    /// 该方法比使用 `git branch` 更高效。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 返回元组 `(本地存在, 远程存在)`：
    /// - `(true, true)` - 分支在本地和远程都存在
    /// - `(true, false)` - 分支只在本地存在
    /// - `(false, true)` - 分支只在远程存在
    /// - `(false, false)` - 分支不存在
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn is_branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        // 使用 git rev-parse --verify 检查本地分支（更高效）
        let exists_local = cmd(
            "git",
            &[
                "rev-parse",
                "--verify",
                &format!("refs/heads/{}", branch_name),
            ],
        )
        .stdout_null()
        .stderr_null()
        .run()
        .is_ok();

        // 使用 git rev-parse --verify 检查远程分支（更高效）
        let exists_remote = cmd(
            "git",
            &[
                "rev-parse",
                "--verify",
                &format!("refs/remotes/origin/{}", branch_name),
            ],
        )
        .stdout_null()
        .stderr_null()
        .run()
        .is_ok();

        Ok((exists_local, exists_remote))
    }

    /// 创建或切换到分支
    ///
    /// 根据分支的存在情况执行相应的操作：
    /// - 如果分支已存在且是当前分支，则跳过
    /// - 如果分支已存在但不是当前分支，则切换到该分支
    /// - 如果分支只存在于远程，则创建本地分支并跟踪远程分支
    /// - 如果分支不存在，则创建新分支
    ///
    /// 优先使用 `git switch`（Git 2.23+），如果失败则回退到 `git checkout`。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要创建或切换的分支名称
    ///
    /// # 错误
    ///
    /// 如果分支操作失败，返回相应的错误信息。
    pub fn checkout_branch(branch_name: &str) -> Result<()> {
        // 检查是否是当前分支
        let current_branch = Self::current_branch()?;
        if current_branch == branch_name {
            // 已经是当前分支，无需操作
            return Ok(());
        }

        // 检查分支是否存在
        let (exists_local, exists_remote) = Self::is_branch_exists(branch_name)?;

        if exists_local {
            // 分支已存在于本地，切换到它
            // 优先使用 git switch（Git 2.23+），如果失败则回退到 git checkout
            let result = cmd("git", &["switch", branch_name])
                .stdout_null()
                .stderr_null()
                .run();

            if result.is_err() {
                // 回退到 git checkout
                cmd("git", &["checkout", branch_name])
                    .run()
                    .context(format!("Failed to checkout branch: {}", branch_name))?;
            }
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            // 优先使用 git switch --track（Git 2.23+），如果失败则回退到 git checkout
            let result = cmd(
                "git",
                &["switch", "--track", &format!("origin/{}", branch_name)],
            )
            .stdout_null()
            .stderr_null()
            .run();

            if result.is_err() {
                // 回退到 git checkout -b
                cmd(
                    "git",
                    &[
                        "checkout",
                        "-b",
                        branch_name,
                        &format!("origin/{}", branch_name),
                    ],
                )
                .run()
                .context(format!("Failed to checkout remote branch: {}", branch_name))?;
            }
        } else {
            // 分支不存在，创建新分支
            // 优先使用 git switch -c（Git 2.23+），如果失败则回退到 git checkout -b
            let result = cmd("git", &["switch", "-c", branch_name])
                .stdout_null()
                .stderr_null()
                .run();

            if result.is_err() {
                // 回退到 git checkout -b
                cmd("git", &["checkout", "-b", branch_name])
                    .run()
                    .context(format!("Failed to create branch: {}", branch_name))?;
            }
        }
        Ok(())
    }

    /// 获取默认分支
    ///
    /// 统一使用 Git 命令从远程获取默认分支，适用于所有 Git 仓库类型
    /// （包括 GitHub、Codeup、GitLab 等）
    ///
    /// 尝试通过以下方式获取默认分支：
    /// 1. 使用 `git ls-remote --symref origin HEAD` 直接从远程获取符号引用
    /// 2. 如果失败，使用 `git remote show origin` 获取
    /// 3. 如果都失败，从远程分支列表中查找常见的默认分支名（main, master, develop, dev）
    pub fn get_default_branch() -> Result<String> {
        // 首先尝试使用 ls-remote --symref 直接从远程获取符号引用
        // 这是最可靠的方法，因为它直接从远程仓库查询，不依赖本地设置
        let output = cmd("git", &["ls-remote", "--symref", "origin", "HEAD"])
            .read()
            .or_else(|_| {
                // 如果 --symref 失败，尝试从 remote show origin 获取
                let remote_info = cmd("git", &["remote", "show", "origin"]).read()?;
                for line in remote_info.lines() {
                    if line.contains("HEAD branch:") {
                        if let Some(branch) = line.split("HEAD branch:").nth(1) {
                            return Ok(branch.trim().to_string());
                        }
                    }
                }
                anyhow::bail!("Could not determine default branch")
            })?;

        // 解析 ls-remote --symref 的输出
        // 输出格式通常是: "ref: refs/heads/main\tHEAD"
        if let Some(branch) = Self::parse_symref_output(&output) {
            return Ok(branch);
        }

        // 如果没有找到符号引用，尝试从远程分支列表中查找常见的默认分支名
        Self::find_default_branch_from_remote()
    }

    /// 解析 ls-remote --symref 的输出
    ///
    /// 从 `git ls-remote --symref` 的输出中提取默认分支名称。
    /// 输出格式通常是：`ref: refs/heads/main\tHEAD`
    ///
    /// # 参数
    ///
    /// * `output` - `git ls-remote --symref` 命令的输出字符串
    ///
    /// # 返回
    ///
    /// 返回默认分支名称（如果找到），否则返回 `None`。
    fn parse_symref_output(output: &str) -> Option<String> {
        for line in output.lines() {
            if line.starts_with("ref:") {
                // 提取 refs/heads/<branch> 中的分支名
                if let Some(parts) = line.split_whitespace().nth(1) {
                    if let Some(branch) = parts.strip_prefix("refs/heads/") {
                        return Some(branch.to_string());
                    }
                }
            }
        }
        None
    }

    /// 从远程分支列表中查找常见的默认分支名
    ///
    /// 当无法通过其他方式获取默认分支时，从远程分支列表中查找常见的默认分支名。
    /// 按顺序查找：`main`、`master`、`develop`、`dev`。
    ///
    /// # 返回
    ///
    /// 返回找到的默认分支名称。
    ///
    /// # 错误
    ///
    /// 如果没有找到任何常见的默认分支，返回相应的错误信息。
    fn find_default_branch_from_remote() -> Result<String> {
        let remote_branches = cmd("git", &["branch", "-r"]).read()?;
        let common_defaults = ["main", "master", "develop", "dev"];

        for default_name in &common_defaults {
            let branch_ref = format!("origin/{}", default_name);
            if remote_branches
                .lines()
                .any(|line| line.trim() == branch_ref)
            {
                return Ok(default_name.to_string());
            }
        }

        anyhow::bail!("Could not determine default branch")
    }

    /// 检查分支是否领先于指定分支（是否有新提交）
    ///
    /// 使用 `git rev-list --count` 检查指定分支相对于基础分支是否有新的提交。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    /// * `base_branch` - 基础分支名称（用于比较）
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果分支有新的提交
    /// - `Ok(false)` - 如果分支为空或与指定分支相同
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn is_branch_ahead(branch_name: &str, base_branch: &str) -> Result<bool> {
        // 检查分支是否领先于指定分支
        // 使用 git rev-list 来检查是否有新提交
        let output = cmd(
            "git",
            &[
                "rev-list",
                "--count",
                &format!("{}..{}", base_branch, branch_name),
            ],
        )
        .read()
        .context("Failed to check branch commits")?;

        let count: u32 = output
            .trim()
            .parse()
            .context("Failed to parse commit count")?;

        Ok(count > 0)
    }

    /// 从远程拉取指定分支的最新更改
    ///
    /// 使用 `git pull origin <branch>` 从远程仓库拉取指定分支的最新更改。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要拉取的分支名称
    ///
    /// # 错误
    ///
    /// 如果拉取失败，返回相应的错误信息。
    pub fn pull(branch_name: &str) -> Result<()> {
        cmd("git", &["pull", "origin", branch_name])
            .run()
            .context(format!(
                "Failed to pull latest changes from {}",
                branch_name
            ))?;
        Ok(())
    }

    /// 推送到远程仓库
    ///
    /// 将指定分支推送到远程仓库的 `origin`。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    /// * `set_upstream` - 是否设置上游分支（使用 `-u` 选项）
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
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

    /// 删除本地分支
    ///
    /// 删除指定的本地分支。如果分支未完全合并，可以使用 `force` 参数强制删除。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的分支名称
    /// * `force` - 是否强制删除（使用 `-D` 而不是 `-d`）
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete(branch_name: &str, force: bool) -> Result<()> {
        let flag = if force { "-D" } else { "-d" };
        cmd("git", &["branch", flag, branch_name])
            .run()
            .with_context(|| format!("Failed to delete local branch: {}", branch_name))?;
        Ok(())
    }

    /// 删除远程分支
    ///
    /// 使用 `git push origin --delete <branch_name>` 删除远程分支。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要删除的远程分支名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(branch_name: &str) -> Result<()> {
        cmd("git", &["push", "origin", "--delete", branch_name])
            .run()
            .with_context(|| format!("Failed to delete remote branch: {}", branch_name))?;
        Ok(())
    }

    /// 合并指定分支到当前分支
    ///
    /// 根据指定的合并策略将源分支合并到当前分支。
    ///
    /// # 参数
    ///
    /// * `source_branch` - 要合并的源分支名称
    /// * `strategy` - 合并策略
    ///
    /// # 错误
    ///
    /// 如果合并失败（包括冲突），返回相应的错误信息。
    pub fn merge_branch(source_branch: &str, strategy: MergeStrategy) -> Result<()> {
        let mut args = vec!["merge"];

        match strategy {
            MergeStrategy::Merge => {
                // 普通合并，不需要额外参数
            }
            MergeStrategy::Squash => {
                args.push("--squash");
            }
            MergeStrategy::FastForwardOnly => {
                args.push("--ff-only");
            }
        }

        args.push(source_branch);

        cmd("git", &args).run().with_context(|| {
            format!(
                "Failed to merge branch '{}' into current branch",
                source_branch
            )
        })?;

        Ok(())
    }

    /// 检查是否有合并冲突
    ///
    /// 使用 `git diff --check` 检查是否有合并冲突标记。
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果检测到合并冲突
    /// - `Ok(false)` - 如果没有冲突
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn has_merge_conflicts() -> Result<bool> {
        // 检查是否有未合并的文件
        let has_unmerged = cmd("git", &["diff", "--check"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_err();

        if has_unmerged {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        let merge_head_exists = cmd("git", &["rev-parse", "--verify", "MERGE_HEAD"])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok();

        if merge_head_exists {
            // 检查是否有未解决的冲突文件
            let unmerged_files = cmd("git", &["diff", "--name-only", "--diff-filter=U"])
                .read()
                .ok();

            if let Some(files) = unmerged_files {
                return Ok(!files.trim().is_empty());
            }
        }

        Ok(false)
    }
}
