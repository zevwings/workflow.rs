//! 存储信息查询 API
//!
//! 提供配置存储位置和信息查询功能。

use crate::paths::config::{config_dir, work_history_dir};
use crate::paths::PathError;

/// 检查配置是否存储在 iCloud
///
/// # 返回
///
/// - `true` - 配置当前存储在 iCloud Drive
/// - `false` - 配置存储在本地
pub fn is_config_in_icloud() -> bool {
    #[cfg(target_os = "macos")]
    {
        use crate::paths::base::try_icloud_base_dir;
        try_icloud_base_dir().is_some()
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// 获取存储位置的用户友好描述
///
/// # 返回
///
/// - "iCloud Drive (synced across devices)" - 使用 iCloud
/// - "Local storage" - 使用本地存储
pub fn storage_location() -> &'static str {
    if is_config_in_icloud() {
        "iCloud Drive (synced across devices)"
    } else {
        "Local storage"
    }
}

/// 获取详细的存储信息
///
/// 返回包含存储类型、配置路径和工作历史路径的详细信息。
///
/// # 返回
///
/// 返回格式化的存储信息字符串。
///
/// # 错误
///
/// 如果无法获取路径，返回相应的错误信息。
pub fn storage_info() -> Result<String, PathError> {
    let config_dir = config_dir()?;
    let work_history_dir = work_history_dir()?;

    let info = if is_config_in_icloud() {
        format!(
            "Storage Type: iCloud Drive (synced across devices)\n\
             \n\
             Configuration (synced):\n\
             {}\n\
             \n\
             Work History (local only, not synced):\n\
             {}",
            config_dir.display(),
            work_history_dir.display()
        )
    } else {
        format!(
            "Storage Type: Local storage\n\
             \n\
             Configuration:\n\
             {}\n\
             \n\
             Work History:\n\
             {}",
            config_dir.display(),
            work_history_dir.display()
        )
    };

    Ok(info)
}
