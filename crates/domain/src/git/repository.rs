//! Git 仓储接口
//!
//! 提供 Git 仓库的完整操作接口定义。

use crate::{
    BlameLineInfo, CommitFileChange, CommitInfo, GitError, MergeStrategy, RemoteDirection,
    RepoInfo, StashApplyResult, StashEntry, StashPopResult, TagCreateInfo, TagCreateScope,
    TagDeleteInfo, TagDeleteScope, WorkingTreeStatus,
};

/// 仅提供仓库信息的仓储接口（兼容 GitHub 等依赖）
pub trait GitRepoRepository: Send + Sync {
    fn get_repo_info(&self) -> RepoInfo;
}

impl<T: GitRepository + Send + Sync> GitRepoRepository for T {
    fn get_repo_info(&self) -> RepoInfo {
        self.get_repo_info()
    }
}

/// Git 仓储接口
///
/// 提供 Git 仓库的完整操作，包括仓库管理、分支、提交、合并、变基、远程、标签和追溯等功能。
pub trait GitRepository: Send + Sync {
    // ========== Repo 操作 ==========

    /// 获取仓库信息
    ///
    /// 一次性获取仓库的所有基本信息，包括：
    /// - 是否为 Git 仓库
    /// - 仓库类型（GitHub、Codeup、Unknown）
    /// - origin 远程仓库 URL
    /// - Git 目录路径
    /// - 仓库名称（owner/repo 格式）
    fn get_repo_info(&self) -> RepoInfo;

    /// 获取 .gitignore 中的目录模式
    ///
    /// 解析 .gitignore 文件，提取其中的目录模式（以 / 结尾或常见的构建/缓存目录）。
    /// 这些模式可用于在 git 操作中提前过滤，避免扫描大型目录。
    ///
    /// # 返回
    /// 返回目录模式列表，例如 ["target", "node_modules", "dist"]
    ///
    /// # 注意
    /// - 如果 .gitignore 不存在，返回空列表
    /// - 只提取目录模式，不包括文件模式
    /// - 自动添加一些常见的大型目录（如果 .gitignore 中没有）
    fn get_ignore_directory_patterns(&self) -> Vec<String>;

    /// 获取工作区相对于指定分支的完整 diff
    ///
    /// 获取当前工作区、暂存区和已提交更改相对于指定分支的完整 diff。
    /// 这包括：
    /// 1. 已提交的更改（当前分支相对于基础分支）
    /// 2. 暂存区的更改
    /// 3. 工作区的未暂存更改
    ///
    /// # 性能优化（内部自动处理）
    /// - 自动从 .gitignore 读取并排除大型目录
    /// - 跳过大文件（> 1MB）以避免内存问题
    /// - 限制总 diff 大小以提高 LLM 处理速度
    ///
    /// # 参数
    /// - `base_branch`: 基础分支名称，例如 "main" 或 "master"
    ///
    /// # 返回
    /// - `Ok(Some(String))`: 返回 diff 内容
    /// - `Ok(None)`: 没有更改
    /// - `Err`: 操作失败
    fn get_working_tree_diff(&self, base_branch: &str) -> Result<Option<String>, GitError>;

    /// 获取将源分支合并到目标分支时会引入的 diff（仅已提交部分）
    ///
    /// 等价于 `git diff $(git merge-base target_branch branch)..branch`，
    /// 即合并时“本次会引入的改动”，与目标分支在合并后的尖端无关。
    ///
    /// # 参数
    /// - `branch`: 源分支（如当前分支 `feature/path`）
    /// - `target_branch`: 目标分支（如 `master`）
    fn get_merge_diff(&self, branch: &str, target_branch: &str)
        -> Result<Option<String>, GitError>;

    /// 获取将源分支合并到目标分支时会变更的文件列表
    ///
    /// 即 `merge_base(target_branch, branch)..branch` 的变更文件，与 `get_merge_diff` 范围一致。
    fn get_merge_changed_files(
        &self,
        branch: &str,
        target_branch: &str,
    ) -> Result<Vec<CommitFileChange>, GitError>;

    /// 检测文件是否为纯格式化变更
    ///
    /// 对比正常 diff 和忽略空白的 diff。如果正常 diff 有变更但忽略空白后无变更，
    /// 则认为是纯格式化变更（仅包含空格、缩进、换行符等调整）。
    ///
    /// # 参数
    /// - `base_ref`: 基准引用（分支名或 commit SHA）
    /// - `target_ref`: 目标引用
    /// - `file_path`: 要检测的文件路径
    ///
    /// # 返回
    /// - `Ok(true)`: 纯格式化变更
    /// - `Ok(false)`: 包含实质性变更
    ///
    /// # 使用场景
    /// - Summary 服务在采样时排除格式化文件
    /// - 识别批量格式化提交
    fn is_formatting_only_change(
        &self,
        base_ref: &str,
        target_ref: &str,
        file_path: &str,
    ) -> Result<bool, GitError>;

    // ========== Branch 操作 ==========

    /// 创建新分支
    fn create_branch(&self, name: &str) -> Result<(), GitError>;

    /// 删除本地分支
    fn delete_local_branch(&self, name: &str, force: bool) -> Result<(), GitError>;

    /// 删除远程分支
    ///
    /// # 参数
    /// - `name`: 分支名称（不包含 origin/ 前缀）
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功
    /// - `Err(GitError::BranchNotFound)`: 远程分支不存在
    /// - `Err`: 其他错误
    fn delete_remote_branch(&self, name: &str) -> Result<(), GitError>;

    /// 重命名分支
    fn rename_branch(&self, old_name: Option<&str>, new_name: &str) -> Result<(), GitError>;

    /// 列出分支
    ///
    /// # 参数
    /// - `remove_prefix`: 是否移除前缀
    /// - `all`: true 返回本地和远程分支，false 只返回本地分支
    ///
    /// # 返回
    /// 返回分支信息列表，包含原始名称、显示名称和是否为远程分支
    fn list_branches(
        &self,
        remove_prefix: bool,
        all: bool,
    ) -> Result<Vec<crate::BranchInfo>, GitError>;

    /// 切换或创建分支
    fn checkout_branch(&self, name: &str) -> Result<(), GitError>;

    /// 获取当前分支名
    fn get_current_branch(&self) -> Result<String, GitError>;

    /// 检查分支是否存在
    ///
    /// 返回元组 `(本地存在, 远程存在)`
    fn has_branch(&self, name: &str) -> Result<(bool, bool), GitError>;

    /// 获取默认分支
    fn get_default_branch(&self) -> Result<String, GitError>;

    /// 推断当前分支的目标合并分支
    ///
    /// 使用组合策略推断当前分支应该合并到哪个分支：
    /// 1. 优先从 reflog 查找分支创建来源（最准确但可能不存在）
    /// 2. 使用 merge base 分析找到最近的候选分支
    /// 3. 如果都失败，返回 None
    ///
    /// # 参数
    /// - `current_branch`: 当前分支名称
    ///
    /// # 返回
    /// - `Ok(Some(branch_name))`: 推断出的目标分支
    /// - `Ok(None)`: 无法推断
    /// - `Err`: 操作失败
    fn infer_target_branch(&self, current_branch: &str) -> Result<Option<String>, GitError>;

    // ========== Commit 操作 ==========

    /// 获取提交信息
    ///
    /// 参数支持多种格式：
    /// - 完整 SHA (40 字符): `"a1b2c3d4..."`
    /// - 短 SHA (至少 7 字符): `"a1b2c3d"`
    /// - 符号引用: `"HEAD"`, `"main"`, `"origin/main"`
    /// - 相对引用: `"HEAD~1"`, `"main^"`
    fn get_commit_info(&self, ref_or_sha: &str) -> Result<CommitInfo, GitError>;

    /// 获取指定 commit 变更的文件列表
    ///
    /// 与 `get_commit_info` 相同的 ref 格式。根 commit 或无变更时返回空列表；合并 commit 与第一父提交做 diff。
    fn get_commit_changed_files(&self, ref_or_sha: &str)
        -> Result<Vec<CommitFileChange>, GitError>;

    /// 获取指定 commit 的 diff 内容（patch 字符串）
    ///
    /// 与 `get_commit_info` 相同的 ref 格式。根 commit 返回 `None`；有父提交时返回相对第一父的 patch。
    fn get_commit_diff(&self, ref_or_sha: &str) -> Result<Option<String>, GitError>;

    /// 获取工作树状态
    fn get_working_tree_status(&self) -> Result<WorkingTreeStatus, GitError>;

    /// 获取暂存区文件列表
    ///
    /// 返回当前暂存区（staging area）中的文件变更列表，包括：
    /// - 新增的文件（Added）
    /// - 修改的文件（Modified）
    /// - 删除的文件（Deleted）
    /// - 重命名的文件（Renamed）
    ///
    /// # 返回
    /// - `Ok(Vec<CommitFileChange>)`: 暂存区文件列表
    /// - `Err`: 操作失败
    ///
    /// # 示例
    /// ```rust,ignore
    /// let staged_files = repo.get_staged_files()?;
    /// for file in staged_files {
    ///     println!("{}: +{} -{}", file.path, file.additions, file.deletions);
    /// }
    /// ```
    fn get_staged_files(&self) -> Result<Vec<CommitFileChange>, GitError>;

    /// 获取暂存区的完整 diff
    ///
    /// 返回暂存区相对于 HEAD 的完整 diff 内容，等价于 `git diff --cached`。
    /// 这是即将被提交的变更内容。
    ///
    /// # 返回
    /// - `Ok(Some(String))`: 返回 diff 内容
    /// - `Ok(None)`: 暂存区为空
    /// - `Err`: 操作失败
    ///
    /// # 示例
    /// ```rust,ignore
    /// if let Some(diff) = repo.get_staged_diff()? {
    ///     println!("Staged changes:\n{}", diff);
    /// } else {
    ///     println!("No staged changes");
    /// }
    /// ```
    fn get_staged_diff(&self) -> Result<Option<String>, GitError>;

    /// 添加所有更改到暂存区
    ///
    /// 等价于 `git add -A`，将所有工作区的更改（包括新文件、修改和删除）添加到暂存区。
    /// 此方法只添加文件到暂存区，不创建提交。
    ///
    /// # 性能优化（内部自动处理）
    /// - 自动从 .gitignore 读取并排除大型目录
    /// - 跳过匹配 ignore patterns 的路径以提高性能
    ///
    /// # 返回
    /// - `Ok(())`: 添加成功
    /// - `Err`: 操作失败
    ///
    /// # 示例
    /// ```rust,ignore
    /// repo.add_all()?;
    /// let staged_files = repo.get_staged_files()?;
    /// println!("Staged {} files", staged_files.len());
    /// ```
    fn add_all(&self) -> Result<(), GitError>;

    /// 创建提交
    ///
    /// # 参数
    /// - `message`: 提交消息
    /// - `all`: 是否添加所有更改（包括未跟踪的文件）
    ///
    /// # 返回
    /// 返回创建的提交的 SHA
    fn commit(&self, message: &str, all: bool) -> Result<String, GitError>;

    // ========== Merge 操作 ==========

    /// 合并指定分支到当前分支
    fn merge_branch(&self, source_branch: &str, strategy: MergeStrategy) -> Result<(), GitError>;

    /// 检查是否有合并冲突
    fn has_merge_conflicts(&self) -> Result<bool, GitError>;

    /// 检查分支是否已合并到指定分支
    fn is_branch_merged(&self, branch: &str, base_branch: &str) -> Result<bool, GitError>;

    /// 获取两个分支的共同祖先（merge base）
    fn merge_base(&self, branch1: &str, branch2: &str) -> Result<String, GitError>;

    /// 获取将源分支合并到目标分支时会引入的 commit 列表
    ///
    /// 即源分支上有而目标分支上没有的 commit（从源分支尖端到两分支 merge base 之间）。
    fn commits_to_merge(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<String>, GitError>;

    // ========== Rebase 操作 ==========

    /// 将当前分支 rebase 到目标分支
    fn rebase_onto(&self, target_branch: &str) -> Result<(), GitError>;

    /// 将指定范围的提交 rebase 到目标分支
    fn rebase_onto_with_upstream(
        &self,
        newbase: &str,
        upstream: &str,
        branch: &str,
    ) -> Result<(), GitError>;

    // ========== Remote 操作 ==========

    /// 推送到远程仓库
    fn push(&self, branch_name: &str, set_upstream: bool) -> Result<(), GitError>;

    /// 从远程拉取指定分支的最新更改
    fn pull(&self, branch_name: &str) -> Result<(), GitError>;

    /// 检查 commit 是否在远程分支中
    fn is_commit_in_remote_branch(&self, branch: &str, commit_sha: &str) -> Result<bool, GitError>;

    /// 检查远程分支是可以推送/拉取
    fn is_remote_available(&self) -> Result<Vec<RemoteDirection>, GitError>;
    // ========== Stash 操作 ==========

    /// 创建 stash
    ///
    /// # 参数
    /// - `message`: 可选的 stash 消息
    ///
    /// # 返回
    /// 返回创建的 stash 的索引（0 表示最新的 stash）
    fn stash_push(&self, message: Option<&str>) -> Result<usize, GitError>;

    /// 应用并删除 stash
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 成功返回 Ok(())
    fn stash_pop(&self, index: usize) -> Result<StashPopResult, GitError>;

    /// 应用 stash（不删除）
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 返回 `StashApplyResult`，包含应用状态、冲突信息和警告
    fn stash_apply(&self, index: usize) -> Result<StashApplyResult, GitError>;

    /// 列出所有 stash 条目
    ///
    /// # 返回
    /// 返回所有 stash 条目的列表，按索引从新到旧排列（stash@{0} 在第一个）
    fn stash_list(&self) -> Result<Vec<StashEntry>, GitError>;

    /// 删除指定的 stash
    ///
    /// # 参数
    /// - `index`: stash 索引（0 表示最新的 stash）
    ///
    /// # 返回
    /// 成功返回 Ok(())
    fn stash_drop(&self, index: usize) -> Result<(), GitError>;

    // ========== Tag 操作 ==========

    /// 创建 Tag
    fn create_tag(
        &self,
        name: &str,
        target: Option<&str>,
        message: Option<&str>,
        scope: TagCreateScope,
        force: bool,
    ) -> Result<TagCreateInfo, GitError>;

    /// 删除 Tag
    fn delete_tag(
        &self,
        name: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<TagDeleteInfo, GitError>;

    /// 按模式删除 Tag
    fn delete_tags_by_pattern(
        &self,
        pattern: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<Vec<TagDeleteInfo>, GitError>;

    /// 列出所有 Tag
    fn list_tags(&self, include_remote: bool) -> Result<Vec<String>, GitError>;

    /// 检查 Tag 是否存在
    ///
    /// 返回元组 `(本地存在, 远程存在)`
    fn has_tag(&self, name: &str) -> Result<(bool, bool), GitError>;

    /// 预览删除操作（dry-run）
    fn preview_delete(
        &self,
        name: Option<&str>,
        pattern: Option<&str>,
        scope: TagDeleteScope,
    ) -> Result<Vec<TagDeleteInfo>, GitError>;

    // ========== Blame 操作 ==========

    /// 获取文件的 blame 信息
    fn get_file_blame(
        &self,
        file_path: &str,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError>;

    /// 获取文件指定行范围的 blame 信息
    fn get_file_blame_range(
        &self,
        file_path: &str,
        start_line: usize,
        end_line: usize,
        revision: Option<&str>,
    ) -> Result<Vec<BlameLineInfo>, GitError>;
}
