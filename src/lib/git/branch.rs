use anyhow::{Context, Result};
use std::collections::HashSet;

use super::helpers::{
    check_ref_exists, check_success, cmd_read, cmd_run, remove_branch_prefix, switch_or_checkout,
    COMMON_DEFAULT_BRANCHES,
};

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

/// Git 分支管理
///
/// 提供分支相关的操作功能，包括：
/// - 获取当前分支名
/// - 检查分支是否存在
/// - 创建或切换分支
/// - 获取默认分支
/// - 合并分支
/// - 推送和删除分支
pub struct GitBranch;

impl GitBranch {
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
        cmd_read(&["branch", "--show-current"]).context("Failed to get current branch")
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
        let exists_local = check_ref_exists(&format!("refs/heads/{}", branch_name));

        // 使用 git rev-parse --verify 检查远程分支（更高效）
        let exists_remote = check_ref_exists(&format!("refs/remotes/origin/{}", branch_name));

        Ok((exists_local, exists_remote))
    }

    /// 检查分支是否在本地存在
    ///
    /// 这是 `is_branch_exists` 的便捷方法，只检查本地分支是否存在。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 如果分支在本地存在，返回 `true`，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn has_local_branch(branch_name: &str) -> Result<bool> {
        let (exists_local, _) = Self::is_branch_exists(branch_name)?;
        Ok(exists_local)
    }

    /// 检查分支是否在远程存在
    ///
    /// 这是 `is_branch_exists` 的便捷方法，只检查远程分支是否存在。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要检查的分支名称
    ///
    /// # 返回
    ///
    /// 如果分支在远程存在，返回 `true`，否则返回 `false`。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn has_remote_branch(branch_name: &str) -> Result<bool> {
        let (_, exists_remote) = Self::is_branch_exists(branch_name)?;
        Ok(exists_remote)
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
            switch_or_checkout(
                &["switch", branch_name],
                &["checkout", branch_name],
                format!("Failed to checkout branch: {}", branch_name),
            )?;
        } else if exists_remote {
            // 分支只存在于远程，创建本地分支并跟踪远程分支
            let remote_ref = format!("origin/{}", branch_name);
            switch_or_checkout(
                &["switch", "--track", &remote_ref],
                &["checkout", "-b", branch_name, &remote_ref],
                format!("Failed to checkout remote branch: {}", branch_name),
            )?;
        } else {
            // 分支不存在，创建新分支
            switch_or_checkout(
                &["switch", "-c", branch_name],
                &["checkout", "-b", branch_name],
                format!("Failed to create branch: {}", branch_name),
            )?;
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
        // 尝试方法1：使用 ls-remote --symref 直接从远程获取符号引用
        if let Ok(output) = cmd_read(&["ls-remote", "--symref", "origin", "HEAD"]) {
            if let Some(branch) = Self::parse_symref_output(&output) {
                return Ok(branch);
            }
        }

        // 尝试方法2：从 remote show origin 获取
        if let Ok(branch) = Self::get_default_branch_from_remote_show() {
            return Ok(branch);
        }

        // 尝试方法3：从远程分支列表中查找常见的默认分支名
        Self::find_default_branch_from_remote().context("Failed to get default branch")
    }

    /// 从 `git remote show origin` 获取默认分支
    fn get_default_branch_from_remote_show() -> Result<String> {
        let remote_info = cmd_read(&["remote", "show", "origin"])?;
        for line in remote_info.lines() {
            if line.contains("HEAD branch:") {
                if let Some(branch) = line.split("HEAD branch:").nth(1) {
                    return Ok(branch.trim().to_string());
                }
            }
        }
        anyhow::bail!("Could not determine default branch from remote show")
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
        let remote_branches =
            cmd_read(&["branch", "-r"]).context("Failed to get default branch")?;

        for default_name in COMMON_DEFAULT_BRANCHES {
            let branch_ref = format!("origin/{}", default_name);
            if remote_branches.lines().any(|line| line.trim() == branch_ref) {
                return Ok(default_name.to_string());
            }
        }

        anyhow::bail!("Could not determine default branch")
    }

    /// 获取所有分支（本地和远程），并排除重复
    ///
    /// 获取所有本地分支和远程分支，去除重复的分支名称，返回去重后的分支列表。
    /// 远程分支的 `origin/` 前缀会被移除，只保留分支名称。
    ///
    /// # 参数
    ///
    /// * `remove_prefix` - 是否移除分支名称的前缀（如 `prefix/branch-name` -> `branch-name`）
    ///   如果为 `true`，会移除 `prefix/` 和 `ticket--` 格式的前缀
    ///
    /// # 返回
    ///
    /// 返回去重后的分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use workflow::GitBranch;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 获取完整分支名（包含前缀）
    /// let branches = GitBranch::get_all_branches(false)?;
    /// // 返回: ["main", "zw/code-optimization", "develop", ...]
    ///
    /// // 获取基础分支名（去掉前缀）
    /// let base_branches = GitBranch::get_all_branches(true)?;
    /// // 返回: ["main", "code-optimization", "develop", ...]
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_all_branches(remove_prefix: bool) -> Result<Vec<String>> {
        // 获取本地分支列表
        let local_output = cmd_read(&["branch"]).context("Failed to get local branches")?;

        // 获取远程分支列表
        let remote_output = cmd_read(&["branch", "-r"]).context("Failed to get remote branches")?;

        // 使用 HashSet 去重
        let mut branch_set = HashSet::new();

        // 解析本地分支
        Self::parse_local_branches(&local_output, &mut branch_set);

        // 解析远程分支
        Self::parse_remote_branches(&remote_output, &mut branch_set);

        // 转换为排序后的 Vec
        let mut branches: Vec<String> = branch_set.into_iter().collect();
        branches.sort();

        // 如果需要移除前缀，提取基础名称
        if remove_prefix {
            Ok(Self::extract_base_branch_names(branches))
        } else {
            Ok(branches)
        }
    }

    /// 获取所有本地分支
    ///
    /// 只获取本地分支列表，不包括远程分支。
    ///
    /// # 返回
    ///
    /// 返回本地分支名称列表（按字母顺序排序）
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn get_local_branches() -> Result<Vec<String>> {
        // 获取本地分支列表
        let local_output = cmd_read(&["branch"]).context("Failed to get local branches")?;

        // 使用 HashSet 去重
        let mut branch_set = HashSet::new();

        // 解析本地分支
        Self::parse_local_branches(&local_output, &mut branch_set);

        // 转换为排序后的 Vec
        let mut branches: Vec<String> = branch_set.into_iter().collect();
        branches.sort();

        Ok(branches)
    }

    /// 解析本地分支列表
    fn parse_local_branches(output: &str, branch_set: &mut HashSet<String>) {
        for line in output.lines() {
            let line = line.trim();
            // 跳过空行和 HEAD 指向的行（如 "  remotes/origin/HEAD -> origin/main"）
            if line.is_empty() || line.contains("->") {
                continue;
            }
            // 移除 `*` 标记（当前分支）和首尾空白
            let branch_name = line.trim_start_matches('*').trim();
            if !branch_name.is_empty() {
                branch_set.insert(branch_name.to_string());
            }
        }
    }

    /// 解析远程分支列表
    fn parse_remote_branches(output: &str, branch_set: &mut HashSet<String>) {
        for line in output.lines() {
            let line = line.trim();
            // 跳过空行和 HEAD 指向的行
            if line.is_empty() || line.contains("->") {
                continue;
            }
            // 移除 `origin/` 前缀
            let branch_name = if let Some(name) = line.strip_prefix("origin/") {
                name.trim()
            } else {
                // 如果不是 origin/ 开头，尝试移除其他远程名称
                line.split('/').nth(1).unwrap_or(line).trim()
            };
            if !branch_name.is_empty() {
                branch_set.insert(branch_name.to_string());
            }
        }
    }

    /// 提取分支的基础名称（去掉前缀）
    ///
    /// 从完整的分支名中提取基础名称，支持两种格式：
    /// - `prefix/branch-name`（使用 `/` 分割）
    /// - `ticket--branch-name`（使用 `--` 分割）
    ///
    /// # 参数
    ///
    /// * `branches` - 分支名称列表
    ///
    /// # 返回
    ///
    /// 返回去重后的基础分支名称列表（按字母顺序排序）
    ///
    /// # 示例
    ///
    /// ```
    /// use workflow::GitBranch;
    ///
    /// let branches = vec!["zw/code-optimization".to_string(), "master".to_string()];
    /// let base_names = GitBranch::extract_base_branch_names(branches);
    /// // 返回: ["code-optimization", "master"]
    /// ```
    pub fn extract_base_branch_names(branches: Vec<String>) -> Vec<String> {
        let mut base_names: Vec<String> = branches
            .iter()
            .map(|branch| remove_branch_prefix(branch).to_string())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        base_names.sort();
        base_names
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
        let output = cmd_read(&[
            "rev-list",
            "--count",
            &format!("{}..{}", base_branch, branch_name),
        ])
        .context("Failed to check branch commits")?;

        let count: u32 = output.parse().context("Failed to parse commit count")?;

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
        cmd_run(&["pull", "origin", branch_name])
            .with_context(|| format!("Failed to pull latest changes from {}", branch_name))
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

        cmd_run(&args).with_context(|| format!("Failed to push branch: {}", branch_name))
    }

    /// 使用 force-with-lease 强制推送到远程仓库
    ///
    /// 使用 `--force-with-lease` 选项安全地强制推送分支到远程仓库。
    /// 这比 `--force` 更安全，因为它会检查远程分支是否有新的提交。
    ///
    /// # 参数
    ///
    /// * `branch_name` - 要推送的分支名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push_force_with_lease(branch_name: &str) -> Result<()> {
        cmd_run(&["push", "--force-with-lease", "origin", branch_name])
            .with_context(|| format!("Failed to force push branch: {}", branch_name))
    }

    /// 将当前分支 rebase 到目标分支
    ///
    /// 使用 `git rebase` 将当前分支的提交重新应用到目标分支之上。
    ///
    /// # 参数
    ///
    /// * `target_branch` - 目标分支引用（本地分支名或 origin/branch-name）
    ///
    /// # 错误
    ///
    /// 如果 rebase 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，rebase 会暂停，需要用户手动解决冲突后继续。
    pub fn rebase_onto(target_branch: &str) -> Result<()> {
        cmd_run(&["rebase", target_branch])
            .with_context(|| format!("Failed to rebase onto branch: {}", target_branch))
    }

    /// 将指定范围的提交 rebase 到目标分支
    ///
    /// 使用 `git rebase --onto` 将 `<upstream>..<branch>` 范围内的提交
    /// rebase 到 `<newbase>` 之上。这样可以只 rebase 分支独有的提交，
    /// 排除上游分支的提交。
    ///
    /// # 参数
    ///
    /// * `newbase` - 新的基础分支（要 rebase 到的分支）
    /// * `upstream` - 上游分支（rebase 范围的起点，排除此分支的提交）
    /// * `branch` - 要 rebase 的分支（rebase 范围的终点）
    ///
    /// # 错误
    ///
    /// 如果 rebase 失败（包括冲突），返回相应的错误信息。
    ///
    /// # 注意
    ///
    /// 如果遇到冲突，rebase 会暂停，需要用户手动解决冲突后继续。
    ///
    /// # 示例
    ///
    /// 如果 `test-rebase` 基于 `develop-` 创建，但想 rebase 到 `master`：
    /// ```no_run
    /// use workflow::GitBranch;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 只 rebase test-rebase 独有的提交（排除 develop- 的提交）到 master
    /// GitBranch::rebase_onto_with_upstream("master", "develop-", "test-rebase")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rebase_onto_with_upstream(newbase: &str, upstream: &str, branch: &str) -> Result<()> {
        cmd_run(&["rebase", "--onto", newbase, upstream, branch]).with_context(|| {
            format!(
                "Failed to rebase '{}' onto '{}' (excluding '{}' commits)",
                branch, newbase, upstream
            )
        })
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
        cmd_run(&["branch", flag, branch_name])
            .with_context(|| format!("Failed to delete local branch: {}", branch_name))
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
        cmd_run(&["push", "origin", "--delete", branch_name])
            .with_context(|| format!("Failed to delete remote branch: {}", branch_name))
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
    ///
    /// # 注意
    ///
    /// 即使 git merge 命令返回错误（如引用更新失败），如果合并实际上已经完成
    /// （通过检查 MERGE_HEAD 或合并 commit 的存在），也会认为操作成功。
    pub fn merge_branch(source_branch: &str, strategy: MergeStrategy) -> Result<()> {
        // 保存合并前的 HEAD，用于验证合并是否完成
        let head_before = cmd_read(&["rev-parse", "HEAD"]).ok();

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

        let merge_result = cmd_run(&args);

        // 检查合并是否实际上已经完成
        // 即使命令返回错误，如果合并已经完成，我们也认为操作成功
        if merge_result.is_err() {
            if Self::verify_merge_completed(head_before) {
                // 合并实际上已经完成，忽略引用更新错误
                return Ok(());
            }

            // 否则返回原始错误
            return merge_result.with_context(|| {
                format!(
                    "Failed to merge branch '{}' into current branch",
                    source_branch
                )
            });
        }

        Ok(())
    }

    /// 验证合并是否已经完成
    ///
    /// 通过检查 MERGE_HEAD 是否存在或 HEAD 是否改变来判断合并是否完成。
    fn verify_merge_completed(head_before: Option<String>) -> bool {
        // 检查是否有 MERGE_HEAD（表示合并正在进行或刚完成）
        let has_merge_head = check_ref_exists("MERGE_HEAD");

        // 检查 HEAD 是否已经改变（表示合并可能已经完成）
        let head_after = cmd_read(&["rev-parse", "HEAD"]).ok();
        let head_changed = if let (Some(before), Some(after)) = (head_before, head_after) {
            before.trim() != after.trim()
        } else {
            false
        };

        head_changed || has_merge_head
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
        if !check_success(&["diff", "--check"]) {
            return Ok(true);
        }

        // 检查 MERGE_HEAD 是否存在（表示正在进行合并）
        if check_ref_exists("MERGE_HEAD") {
            // 检查是否有未解决的冲突文件
            if let Ok(files) = cmd_read(&["diff", "--name-only", "--diff-filter=U"]) {
                return Ok(!files.trim().is_empty());
            }
        }

        Ok(false)
    }

    /// 检查分支是否已合并到指定分支
    ///
    /// 使用 `git branch --merged` 检查指定分支是否已合并到基础分支。
    ///
    /// # 参数
    ///
    /// * `branch` - 要检查的分支名称
    /// * `base_branch` - 基础分支名称（用于检查合并状态）
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 如果分支已合并到基础分支
    /// - `Ok(false)` - 如果分支未合并
    ///
    /// # 错误
    ///
    /// 如果命令执行失败，返回相应的错误信息。
    pub fn is_branch_merged(branch: &str, base_branch: &str) -> Result<bool> {
        let output = cmd_read(&["branch", "--merged", base_branch])
            .context("Failed to check merged branches")?;

        // 检查分支是否在输出中
        Ok(output.lines().any(|line| {
            let line = line.trim().trim_start_matches('*').trim();
            line == branch
        }))
    }

    /// 获取两个分支之间的提交列表
    ///
    /// 使用 `git rev-list` 获取 from_branch 相对于 to_branch 的所有新提交。
    ///
    /// # 参数
    ///
    /// * `base_branch` - 基础分支名称（to_branch）
    /// * `head_branch` - 头部分支名称（from_branch）
    ///
    /// # 返回
    ///
    /// 返回提交哈希列表，按时间顺序排列（从旧到新）。
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn get_commits_between(base_branch: &str, head_branch: &str) -> Result<Vec<String>> {
        let output = cmd_read(&["rev-list", &format!("{}..{}", base_branch, head_branch)])
            .context("Failed to get commits between branches")?;

        let commits: Vec<String> = output
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        Ok(commits)
    }

    /// 获取两个分支的共同祖先（merge base）
    ///
    /// 使用 `git merge-base` 获取两个分支的共同祖先提交。
    ///
    /// # 参数
    ///
    /// * `branch1` - 第一个分支名称
    /// * `branch2` - 第二个分支名称
    ///
    /// # 返回
    ///
    /// 返回共同祖先的提交哈希。如果两个分支没有共同祖先，返回错误。
    ///
    /// # 错误
    ///
    /// 如果分支不存在或命令执行失败，返回相应的错误信息。
    pub fn merge_base(branch1: &str, branch2: &str) -> Result<String> {
        let output = cmd_read(&["merge-base", branch1, branch2]).with_context(|| {
            format!(
                "Failed to get merge base between '{}' and '{}'",
                branch1, branch2
            )
        })?;

        let merge_base = output.trim().to_string();
        if merge_base.is_empty() {
            anyhow::bail!(
                "No common ancestor found between '{}' and '{}'",
                branch1,
                branch2
            );
        }

        Ok(merge_base)
    }

    /// 检查一个分支是否直接基于另一个分支创建
    ///
    /// 通过比较 merge-base 和候选分支的 HEAD 来判断 from_branch 是否直接基于 candidate_branch 创建。
    ///
    /// # 参数
    ///
    /// * `from_branch` - 要检查的分支
    /// * `candidate_branch` - 候选的基础分支
    ///
    /// # 返回
    ///
    /// 如果 from_branch 直接基于 candidate_branch 创建，返回 `true`，否则返回 `false`。
    ///
    /// # 说明
    ///
    /// 判断逻辑：
    /// - 如果 `merge-base(from_branch, candidate_branch) == candidate_branch` 的 HEAD，
    ///   说明 from_branch 可能是直接基于 candidate_branch 创建的
    pub fn is_branch_based_on(from_branch: &str, candidate_branch: &str) -> Result<bool> {
        // 如果两个分支相同，返回 false
        if from_branch == candidate_branch {
            return Ok(false);
        }

        // 获取 merge-base
        let merge_base_commit = match Self::merge_base(from_branch, candidate_branch) {
            Ok(commit) => commit,
            Err(_) => return Ok(false),
        };

        // 获取 candidate_branch 的 HEAD
        let candidate_head = cmd_read(&["rev-parse", candidate_branch])
            .with_context(|| format!("Failed to get HEAD of branch '{}'", candidate_branch))?;
        let candidate_head = candidate_head.trim();

        // 如果 merge-base 等于 candidate_branch 的 HEAD，说明 from_branch 直接基于 candidate_branch
        Ok(merge_base_commit == candidate_head)
    }
}
