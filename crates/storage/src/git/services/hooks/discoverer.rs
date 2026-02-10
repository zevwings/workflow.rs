//! Hook 发现器
//!
//! 负责发现标准 Git hooks（`.git/hooks/` 或 `core.hooksPath` 指向的目录）。

use std::path::{Path, PathBuf};

use domain::GitError;
use git2::Repository;

/// Hook 发现器
///
/// 负责发现和验证 Git hooks 脚本。
/// 支持 `core.hooksPath` 配置，如果设置了则使用该路径，否则使用默认的 `.git/hooks/` 目录。
pub struct HookDiscoverer {
    hooks_dir: PathBuf,
}

impl HookDiscoverer {
    /// 创建 HookDiscoverer
    ///
    /// # 参数
    /// - `git_dir`: Git 目录路径（.git）
    /// - `repo_path`: 仓库根目录路径
    ///
    /// # 说明
    /// 会检查 Git 配置 `core.hooksPath`，如果设置了则使用该路径
    /// 否则使用默认的 `.git/hooks/` 目录
    pub fn new(git_dir: PathBuf, repo_path: PathBuf) -> Self {
        // 检查 core.hooksPath 配置（某些工具会设置此配置）
        // 使用 git2 读取配置，不依赖系统 git 命令
        let hooks_dir = if let Ok(hooks_path) = Self::get_core_hooks_path(&repo_path) {
            hooks_path
        } else {
            // 默认使用 .git/hooks/
            git_dir.join("hooks")
        };

        Self { hooks_dir }
    }

    /// 获取 core.hooksPath 配置值
    ///
    /// # 参数
    /// - `repo_path`: 仓库根目录路径
    ///
    /// # 返回
    /// - `Ok(PathBuf)`: 配置的 hooks 路径
    /// - `Err(GitError)`: 配置未设置或读取失败
    ///
    /// # 说明
    /// 使用 git2 库读取配置，不依赖系统安装的 git 命令。
    fn get_core_hooks_path(repo_path: &Path) -> Result<PathBuf, GitError> {
        // 使用 git2 打开仓库并读取配置
        let repo = Repository::open(repo_path)
            .map_err(|e| GitError::OperationFailed(format!("Failed to open repository: {}", e)))?;

        let config = repo
            .config()
            .map_err(|e| GitError::OperationFailed(format!("Failed to get config: {}", e)))?;

        // 使用 get_path 获取路径类型的配置值
        // git2 会自动处理相对路径的解析
        let hooks_path = config
            .get_path("core.hooksPath")
            .map_err(|_| GitError::OperationFailed("core.hooksPath not set".into()))?;

        // 如果是相对路径，相对于仓库根目录解析
        let hooks_path = if hooks_path.is_absolute() {
            hooks_path
        } else {
            repo_path.join(hooks_path)
        };

        Ok(hooks_path)
    }

    /// 发现指定名称的 hook
    ///
    /// # 参数
    /// - `hook_name`: Hook 名称（如 "pre-commit"）
    ///
    /// # 返回
    /// - `Ok(Some(PathBuf))`: 找到的 hook 脚本路径
    /// - `Ok(None)`: Hook 不存在或不可执行
    /// - `Err(GitError)`: 检查过程中出错
    pub fn find_hook(&self, hook_name: impl AsRef<str>) -> Result<Option<PathBuf>, GitError> {
        let hook_path = self.hooks_dir.join(hook_name.as_ref());

        // 检查文件是否存在
        if !hook_path.exists() {
            return Ok(None);
        }

        // 跳过 .sample 文件
        if hook_path.to_string_lossy().ends_with(".sample") {
            return Ok(None);
        }

        // 检查可执行权限
        if !self.is_executable(&hook_path)? {
            return Ok(None);
        }

        Ok(Some(hook_path))
    }

    /// 检查文件是否可执行
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// - `Ok(true)`: 文件可执行
    /// - `Ok(false)`: 文件不可执行
    /// - `Err(GitError)`: 检查过程中出错
    fn is_executable(&self, path: &Path) -> Result<bool, GitError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path).map_err(|e| {
                GitError::OperationFailed(format!("Failed to read file metadata: {}", e))
            })?;
            let mode = metadata.permissions().mode();
            Ok((mode & 0o111) != 0)
        }

        #[cfg(windows)]
        {
            // Windows 上通过扩展名判断
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                Ok(ext_str == "exe" || ext_str == "bat" || ext_str == "cmd" || ext_str == "ps1")
            } else {
                Ok(true) // 没有扩展名，假设可执行
            }
        }
    }
}
