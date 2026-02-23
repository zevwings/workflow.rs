//! Hook 上下文信息
//!
//! 提供执行 Git hooks 时所需的上下文信息。

use std::path::PathBuf;

/// Hook 执行结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookResult {
    /// Hook 执行成功
    Success,
    /// Hook 执行失败，包含错误消息
    Failure(String),
    /// Hook 修改了文件，需要重新暂存
    Modified,
}

/// Hook 上下文信息
///
/// 包含执行 hook 时所需的所有上下文信息，根据不同的 hook 类型填充相应的字段。
#[derive(Debug, Clone)]
pub struct HookContext {
    /// 仓库路径
    pub repo_path: PathBuf,
    /// Git 目录路径
    pub git_dir: PathBuf,

    // Commit 相关
    /// 暂存区文件列表
    pub staged_files: Vec<String>,
    /// 提交消息（prepare-commit-msg, commit-msg）
    pub commit_message: Option<String>,
    /// 提交 SHA（commit-msg, post-commit）
    pub commit_sha: Option<String>,

    // Push 相关
    /// 分支名称
    pub branch_name: Option<String>,

    /// 要推送的提交列表
    pub commits_to_push: Vec<String>,
}

impl HookContext {
    /// 创建新的 Hook 上下文
    pub fn new(repo_path: PathBuf, git_dir: PathBuf) -> Self {
        Self {
            repo_path,
            git_dir,
            staged_files: Vec::new(),
            commit_message: None,
            commit_sha: None,
            branch_name: None,
            commits_to_push: Vec::new(),
        }
    }

    /// 创建用于 pre-commit hook 的上下文
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    /// - `git_dir`: Git 目录路径（.git）
    /// - `staged_files`: 暂存区文件列表
    pub fn for_pre_commit(repo_path: PathBuf, git_dir: PathBuf, staged_files: Vec<String>) -> Self {
        Self {
            repo_path,
            git_dir,
            staged_files,
            commit_message: None,
            commit_sha: None,
            branch_name: None,
            commits_to_push: Vec::new(),
        }
    }

    /// 创建用于 commit-msg hook 的上下文
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    /// - `git_dir`: Git 目录路径（.git）
    /// - `commit_message`: 提交消息
    /// - `commit_sha`: 提交 SHA（可选）
    pub fn for_commit_msg(
        repo_path: PathBuf,
        git_dir: PathBuf,
        commit_message: String,
        commit_sha: Option<String>,
    ) -> Self {
        Self {
            repo_path,
            git_dir,
            staged_files: Vec::new(),
            commit_message: Some(commit_message),
            commit_sha,
            branch_name: None,
            commits_to_push: Vec::new(),
        }
    }

    /// 创建用于 pre-push hook 的上下文
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    /// - `git_dir`: Git 目录路径（.git）
    /// - `branch_name`: 分支名称
    /// - `commits_to_push`: 要推送的提交列表
    pub fn for_pre_push(
        repo_path: PathBuf,
        git_dir: PathBuf,
        branch_name: String,
        commits_to_push: Vec<String>,
    ) -> Self {
        Self {
            repo_path,
            git_dir,
            staged_files: Vec::new(),
            commit_message: None,
            commit_sha: None,
            branch_name: Some(branch_name),
            commits_to_push,
        }
    }

    /// 创建用于 post-merge hook 的上下文
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    /// - `git_dir`: Git 目录路径（.git）
    #[allow(dead_code)]
    pub fn for_post_merge(repo_path: PathBuf, git_dir: PathBuf) -> Self {
        Self {
            repo_path,
            git_dir,
            staged_files: Vec::new(),
            commit_message: None,
            commit_sha: None,
            branch_name: None,
            commits_to_push: Vec::new(),
        }
    }

    // ========== Builder 方法 ==========

    /// 设置暂存区文件列表
    #[allow(dead_code)]
    pub fn with_staged_files(mut self, files: Vec<String>) -> Self {
        self.staged_files = files;
        self
    }

    /// 设置提交消息
    #[allow(dead_code)]
    pub fn with_commit_message(mut self, message: impl Into<String>) -> Self {
        self.commit_message = Some(message.into());
        self
    }

    /// 设置提交 SHA
    pub fn with_commit_sha(mut self, sha: impl Into<String>) -> Self {
        self.commit_sha = Some(sha.into());
        self
    }

    /// 设置分支名称
    #[allow(dead_code)]
    pub fn with_branch_name(mut self, name: impl Into<String>) -> Self {
        self.branch_name = Some(name.into());
        self
    }
}
