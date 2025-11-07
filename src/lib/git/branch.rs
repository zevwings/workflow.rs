use anyhow::{Context, Result};
use duct::cmd;

use super::commit::Git;

impl Git {
    /// 获取当前分支名
    pub fn current_branch() -> Result<String> {
        let output = cmd("git", &["branch", "--show-current"])
            .read()
            .context("Failed to get current branch")?;

        Ok(output.trim().to_string())
    }

    /// 检查分支是否存在（本地或远程）
    /// 返回 (本地存在, 远程存在)
    pub fn is_branch_exists(branch_name: &str) -> Result<(bool, bool)> {
        // 使用 git rev-parse --verify 检查本地分支（更高效）
        let exists_local = cmd("git", &["rev-parse", "--verify", &format!("refs/heads/{}", branch_name)])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok();

        // 使用 git rev-parse --verify 检查远程分支（更高效）
        let exists_remote = cmd("git", &["rev-parse", "--verify", &format!("refs/remotes/origin/{}", branch_name)])
            .stdout_null()
            .stderr_null()
            .run()
            .is_ok();

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
            let result = cmd("git", &["switch", "--track", &format!("origin/{}", branch_name)])
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
    fn find_default_branch_from_remote() -> Result<String> {
        let remote_branches = cmd("git", &["branch", "-r"]).read()?;
        let common_defaults = ["main", "master", "develop", "dev"];

        for default_name in &common_defaults {
            let branch_ref = format!("origin/{}", default_name);
            if remote_branches.lines().any(|line| line.trim() == branch_ref) {
                return Ok(default_name.to_string());
            }
        }

        anyhow::bail!("Could not determine default branch")
    }

    /// 检查分支是否领先于指定分支（是否有新提交）
    ///
    /// 返回 true 如果分支有新的提交，false 如果分支为空或与指定分支相同
    pub fn is_branch_ahead(branch_name: &str, base_branch: &str) -> Result<bool> {
        // 检查分支是否领先于指定分支
        // 使用 git rev-list 来检查是否有新提交
        let output = cmd("git", &["rev-list", "--count", &format!("{}..{}", base_branch, branch_name)])
            .read()
            .context("Failed to check branch commits")?;

        let count: u32 = output.trim().parse()
            .context("Failed to parse commit count")?;

        Ok(count > 0)
    }
}

