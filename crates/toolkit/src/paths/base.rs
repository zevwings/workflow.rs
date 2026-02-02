//! 基础路径管理
//!
//! 提供基础路径获取功能，包括主目录、本地目录、配置基础目录和 iCloud 目录。

use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::paths::constants::WORKFLOW_DIR;
use crate::paths::PathError;
use crate::util::fs::DirectoryWalker;

/// 获取用户主目录
///
/// 使用 dirs crate 提供的跨平台主目录获取功能。
/// 这是一个统一的入口点，所有需要主目录的地方都应该调用此方法。
///
/// # 返回
///
/// 返回用户主目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法确定主目录，返回错误信息。
pub(crate) fn home_dir() -> Result<PathBuf, PathError> {
    dirs::home_dir().ok_or_else(|| PathError::Other("Cannot determine home directory".to_string()))
}

/// 尝试获取 iCloud 基础目录（仅 macOS）
///
/// 检查 iCloud Drive 是否可用，如果可用则返回 .workflow 目录路径。
///
/// # 返回
///
/// - `Some(PathBuf)` - iCloud Drive 可用且成功创建目录
/// - `None` - iCloud Drive 不可用或创建目录失败
///
/// # iCloud 路径
///
/// macOS: `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/`
#[cfg(target_os = "macos")]
pub(crate) fn try_icloud_base_dir() -> Option<PathBuf> {
    // 获取主目录
    let home = home_dir().ok()?;

    // 构建 iCloud Drive 基础路径
    // ~/Library/Mobile Documents/com~apple~CloudDocs
    let icloud_base = home
        .join("Library")
        .join("Mobile Documents")
        .join("com~apple~CloudDocs");

    // 检查 iCloud Drive 是否可用
    if !icloud_base.exists() || !icloud_base.is_dir() {
        return None;
    }

    // 尝试创建 .workflow 目录
    let workflow_dir = icloud_base.join(WORKFLOW_DIR);
    if DirectoryWalker::new(&workflow_dir).ensure_exists().is_err() {
        return None;
    }

    // 设置目录权限为 700（仅用户可访问）
    #[cfg(unix)]
    {
        let _ = fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700));
    }

    Some(workflow_dir)
}

/// 非 macOS 平台：总是返回 None
///
/// 注意：此函数在非 macOS 平台上不会被调用（调用处被 `#[cfg(target_os = "macos")]` 包裹），
/// 但为了保持 trait 实现的一致性，需要提供此实现。
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)] // 在非 macOS 平台上不会被调用，但需要提供实现以保持一致性
pub(crate) fn try_icloud_base_dir() -> Option<PathBuf> {
    None
}

/// 获取本地基础目录（总是可用）
///
/// 返回 `~/.workflow/` 目录（Unix）。
/// 此方法作为回退方案，确保在任何情况下都能获取到有效路径。
///
/// # 返回
///
/// 返回本地工作流目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub(crate) fn local_base_dir() -> Result<PathBuf, PathError> {
    let home = home_dir()?;
    let workflow_dir = home.join(WORKFLOW_DIR);

    // 确保目录存在
    _ = DirectoryWalker::new(&workflow_dir).ensure_exists();

    // 设置目录权限为 700（仅用户可访问）
    #[cfg(unix)]
    {
        fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
            PathError::Permission(format!(
                "Failed to set workflow directory permissions: {}",
                e
            ))
        })?;
    }

    Ok(workflow_dir)
}

/// 获取配置基础目录（支持 iCloud）
///
/// 决策逻辑：
/// 1. 检查环境变量 `WORKFLOW_DISABLE_ICLOUD`，如果设置则强制使用本地
/// 2. 在 macOS 上，如果 iCloud Drive 可用，优先使用 iCloud
/// 3. 如果 iCloud 不可用，回退到本地目录
/// 4. 在其他平台上，直接使用本地目录
///
/// **注意**：如果用户已有本地配置，需要手动迁移到 iCloud：
/// ```bash
/// cp -r ~/.workflow/config/* \
///    ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/
/// ```
///
/// # 返回
///
/// 返回配置基础目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub(crate) fn config_base_dir() -> Result<PathBuf, PathError> {
    // 检查用户是否明确禁用 iCloud
    if std::env::var("WORKFLOW_DISABLE_ICLOUD").is_ok() {
        return local_base_dir();
    }

    // macOS 上尝试 iCloud
    #[cfg(target_os = "macos")]
    {
        if let Some(icloud_dir) = try_icloud_base_dir() {
            return Ok(icloud_dir);
        }
    }

    // 回退到本地
    local_base_dir()
}
