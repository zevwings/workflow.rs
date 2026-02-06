use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use domain::{Dir, PathError, PathService};

use super::{JIRA_CONFIG_FILE, MAIN_DIR, WORKFLOW_CONFIG_FILE};

pub struct PathServiceImpl {
    is_icloud_available: bool,
}

impl PathServiceImpl {
    pub fn new() -> Self {
        // 检查用户是否明确禁用 iCloud
        let is_icloud_available = std::env::var("WORKFLOW_DISABLE_ICLOUD").is_err();
        Self {
            is_icloud_available,
        }
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
    fn try_local_base_dir(&self) -> Result<PathBuf, PathError> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| PathError::CannotDetermine("home".to_string()))?;

        let workflow_dir = home_dir.join(MAIN_DIR);

        fs::create_dir_all(&workflow_dir).map_err(PathError::Io)?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            use std::fs;

            fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700)).map_err(|e| {
                toolkit::log_error!("Failed to set workflow directory permissions: {}", e);
                PathError::Permission("Failed to set workflow directory permissions".to_string())
            })?;
        }

        Ok(workflow_dir)
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
    fn try_icloud_base_dir(&self) -> Result<PathBuf, PathError> {
        // 获取主目录
        let home_dir =
            dirs::home_dir().ok_or_else(|| PathError::CannotDetermine("home".to_string()))?;

        // 构建 iCloud Drive 基础路径
        // ~/Library/Mobile Documents/com~apple~CloudDocs
        let icloud_base =
            home_dir.join("Library").join("Mobile Documents").join("com~apple~CloudDocs");

        // 检查 iCloud Drive 是否可用
        if !icloud_base.exists() || !icloud_base.is_dir() {
            return Err(PathError::CannotDetermine("iCloud Drive".to_string()));
        }

        // 尝试创建 .workflow 目录
        let workflow_dir = icloud_base.join(MAIN_DIR);
        fs::create_dir_all(&workflow_dir).map_err(PathError::Io)?;

        // 设置目录权限为 700（尽力而为，iCloud 目录权限设置可能不被支持）
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

    /// 非 macOS 平台：总是返回 None
    ///
    /// 注意：此函数在非 macOS 平台上不会被调用（调用处被 `#[cfg(target_os = "macos")]` 包裹），
    /// 但为了保持 trait 实现的一致性，需要提供此实现。
    #[cfg(not(target_os = "macos"))]
    #[allow(dead_code)] // 在非 macOS 平台上不会被调用，但需要提供实现以保持一致性
    fn try_icloud_base_dir(&self) -> Result<PathBuf, PathError> {
        Err(PathError::CannotDetermine("iCloud Drive".to_string()))
    }

    fn get_base_dir(&self) -> Result<Dir, PathError> {
        let icloud_base_dir = if self.is_icloud_available {
            self.try_icloud_base_dir().unwrap_or_default()
        } else {
            PathBuf::new()
        };

        let local_base_dir = self.try_local_base_dir().unwrap_or_default();
        let is_icloud_available =
            !icloud_base_dir.as_os_str().is_empty() && icloud_base_dir.exists();

        Ok(Dir {
            is_icloud_available,
            icloud_base_dir,
            local_base_dir,
        })
    }
}

impl PathService for PathServiceImpl {
    fn get_workflow_config_filepath(&self) -> Result<PathBuf, PathError> {
        let base_dir = self.get_base_dir()?;
        let config_dir = if base_dir.is_icloud_available {
            base_dir.icloud_base_dir.join(WORKFLOW_CONFIG_FILE)
        } else {
            base_dir.local_base_dir.join(WORKFLOW_CONFIG_FILE)
        };
        Ok(config_dir)
    }

    fn get_jira_config_filepath(&self) -> Result<PathBuf, PathError> {
        let base_dir = self.get_base_dir()?;
        let config_dir = if base_dir.is_icloud_available {
            base_dir.icloud_base_dir.join(JIRA_CONFIG_FILE)
        } else {
            base_dir.local_base_dir.join(JIRA_CONFIG_FILE)
        };
        Ok(config_dir)
    }

    fn get_binary_install_dir(&self) -> Result<PathBuf, PathError> {
        let binary_install_dir = if cfg!(target_os = "windows") {
            // Windows: 使用 dirs::data_local_dir() 获取 %LOCALAPPDATA%
            dirs::data_local_dir()
                .map(|d| d.join("Programs").join("workflow").join("bin"))
                .or_else(|| dirs::home_dir().map(|h| h.join(".local").join("bin")))
                .unwrap_or_else(|| PathBuf::from("C:\\Users\\User\\Programs\\workflow\\bin"))
        } else {
            // Unix-like: 使用 /usr/local/bin
            PathBuf::from("/usr/local/bin")
        };
        Ok(binary_install_dir)
    }

    fn get_binary_name(&self) -> Result<String, PathError> {
        let name = "workflow";
        let binary_name = if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };
        Ok(binary_name)
    }
}
