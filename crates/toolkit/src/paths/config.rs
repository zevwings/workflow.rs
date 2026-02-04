//! 配置路径管理
//!
//! 提供所有配置相关的路径获取功能。

use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::paths::base::{config_base_dir, local_base_dir};
use crate::paths::constants::{
    CONFIG_DIR, JIRA_CONFIG_FILE, LLM_CONFIG_FILE, PROJECT_CONFIG_FILE, USER_CONFIG_FILE,
    WORKFLOW_CONFIG_FILE, WORKFLOW_DIR,
};
use crate::paths::PathError;
use crate::util::fs::directory;

/// 创建目录并设置权限（700，仅用户可访问）
///
/// 这是一个辅助函数，用于统一处理目录创建和权限设置。
fn create_dir_with_permissions(dir: &Path, name: &str) -> Result<(), PathError> {
    // 确保目录存在
    directory::ensure_exists(dir)?;

    // 设置目录权限为 700（仅用户可访问，仅 Unix）
    #[cfg(unix)]
    {
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            PathError::Permission(format!(
                "Failed to set {} directory permissions: {}",
                name, e
            ))
        })?;
    }

    // Windows 不需要显式设置权限
    #[cfg(not(unix))]
    let _ = name;

    Ok(())
}

/// 获取配置目录路径（支持 iCloud 同步）
///
/// 返回配置文件存储目录。在 macOS 上，如果 iCloud Drive 可用，
/// 配置将保存到 iCloud 并自动同步到其他设备。
///
/// # 路径示例
///
/// - macOS + iCloud：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
/// - macOS 无 iCloud / 其他系统：`~/.workflow/config/`
///
/// # 返回
///
/// 返回配置目录的 `PathBuf`。
///
/// # 错误
///
/// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
pub fn config_dir() -> Result<PathBuf, PathError> {
    // 使用支持 iCloud 的配置基础目录
    let config_dir = config_base_dir()?.join(CONFIG_DIR);
    create_dir_with_permissions(&config_dir, "config")?;
    Ok(config_dir)
}

/// 获取主配置文件路径
///
/// 返回 `~/.workflow/config/workflow.toml` 的路径。
pub fn workflow_config_path() -> Result<PathBuf, PathError> {
    Ok(config_dir()?.join(WORKFLOW_CONFIG_FILE))
}

/// 获取 LLM 配置文件路径
///
/// 返回 `~/.workflow/config/llm.toml` 的路径。
pub fn llm_config_path() -> Result<PathBuf, PathError> {
    Ok(config_dir()?.join(LLM_CONFIG_FILE))
}

/// 获取 Jira 配置文件路径
///
/// 返回 `~/.workflow/config/jira.toml` 的路径。
/// 这是合并后的 Jira 配置文件，包含用户和状态配置。
pub fn jira_config_path() -> Result<PathBuf, PathError> {
    Ok(config_dir()?.join(JIRA_CONFIG_FILE))
}

/// 获取常用命令配置文件路径
///
/// 返回 `~/.workflow/config/commands.toml` 的路径。
pub fn commands_config_path() -> Result<PathBuf, PathError> {
    Ok(config_dir()?.join("commands.toml"))
}

/// 获取仓库根目录路径
///
/// 返回当前工作目录（仓库根目录）的路径。
/// 这是仓库的根目录，用于存储仓库特定的配置（如 `.workflow/config.toml`）。
///
/// # 路径示例
///
/// - 当前工作目录：`/path/to/repo`
///
/// # 返回
///
/// 返回仓库根目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取当前工作目录，返回相应的错误信息。
pub fn repo_dir() -> Result<PathBuf, PathError> {
    std::env::current_dir().map_err(PathError::Io)
}

/// 获取项目级配置文件路径（当前工作目录）
///
/// 返回当前工作目录下的 `.workflow/config.toml` 路径。
/// 这是项目级别的配置文件，用于存储仓库特定的配置（如分支前缀、提交模板等）。
///
/// # 路径示例
///
/// - 相对于当前工作目录：`.workflow/config.toml`
///
/// # 返回
///
/// 返回项目级配置文件的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取当前工作目录，返回相应的错误信息。
pub fn project_config_path() -> Result<PathBuf, PathError> {
    Ok(project_config_file(&repo_dir()?))
}

/// 获取项目配置文件路径（指定仓库路径）
///
/// 返回指定仓库目录下的 `.workflow/config.toml` 路径。
///
/// # 参数
///
/// * `repo_path` - 仓库根目录路径
///
/// # 返回
///
/// 返回项目配置文件的 `PathBuf`。
pub fn project_config_file(repo_path: &Path) -> PathBuf {
    repo_path.join(WORKFLOW_DIR).join(PROJECT_CONFIG_FILE)
}

/// 获取用户配置文件路径（指定仓库路径）
///
/// 返回指定仓库目录下的 `.workflow/user.toml` 路径。
///
/// # 参数
///
/// * `repo_path` - 仓库根目录路径
///
/// # 返回
///
/// 返回用户配置文件的 `PathBuf`。
pub fn user_config_file(repo_path: &Path) -> PathBuf {
    repo_path.join(WORKFLOW_DIR).join(USER_CONFIG_FILE)
}

/// 获取仓库工作流目录路径（指定仓库路径）
///
/// 返回指定仓库目录下的 `.workflow/` 目录路径。
///
/// # 参数
///
/// * `repo_path` - 仓库根目录路径
///
/// # 返回
///
/// 返回仓库工作流目录的 `PathBuf`。
pub fn repo_workflow_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(WORKFLOW_DIR)
}

/// 获取个人偏好配置文件路径
///
/// 返回 `~/.workflow/config/repository.toml` 的路径。
/// 支持 iCloud 同步（在 macOS 上，如果 iCloud 可用）。
///
/// # 路径示例
///
/// - macOS + iCloud：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/repository.toml`
/// - macOS 无 iCloud / 其他系统：`~/.workflow/config/repository.toml`
///
/// # 返回
///
/// 返回个人偏好配置文件的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建配置目录，返回相应的错误信息。
pub fn repository_config_path() -> Result<PathBuf, PathError> {
    Ok(config_dir()?.join("repository.toml"))
}

/// 获取工作流目录路径（支持 iCloud）
///
/// 返回工作流基础目录。如果配置在 iCloud，此方法返回 iCloud 路径。
///
/// # 返回
///
/// 返回工作流目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub fn workflow_dir() -> Result<PathBuf, PathError> {
    // 直接返回配置基础目录
    config_base_dir()
}

/// 获取工作历史目录路径（强制本地，不同步）
///
/// 返回 `~/.workflow/work-history/`（总是本地路径）。
///
/// **重要**：工作历史是设备本地的，不应该跨设备同步，因为：
/// - 每个设备的工作历史是独立的
/// - 避免多设备冲突（不同设备可能处理不同的 PR）
/// - 防止历史记录混乱（PR ID 可能在不同仓库中重复）
/// - 性能考虑（本地读写更快，不需要等待 iCloud 同步）
///
/// # 路径示例
///
/// - 所有平台：`~/.workflow/work-history/`
///
/// # 返回
///
/// 返回工作历史目录的 `PathBuf`。
///
/// # 错误
///
/// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
pub fn work_history_dir() -> Result<PathBuf, PathError> {
    // 强制使用本地路径，不使用 iCloud
    let history_dir = local_base_dir()?.join("work-history");
    create_dir_with_permissions(&history_dir, "work-history")?;
    Ok(history_dir)
}

/// 获取日志目录路径（强制本地，不同步）
///
/// 返回 `~/.workflow/logs/`（总是本地路径）。
///
/// **重要**：日志文件是设备本地的，不应该跨设备同步，因为：
/// - 每个设备的日志是独立的
/// - 避免 iCloud 同步延迟影响性能
/// - 日志文件可能较大，不适合同步
///
/// # 路径示例
///
/// - 所有平台：`~/.workflow/logs/`
///
/// # 返回
///
/// 返回日志目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub fn logs_dir() -> Result<PathBuf, PathError> {
    // 强制使用本地路径，不使用 iCloud
    let logs_dir = local_base_dir()?.join("logs");
    create_dir_with_permissions(&logs_dir, "logs")?;
    Ok(logs_dir)
}
