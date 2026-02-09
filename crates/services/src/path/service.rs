// 标准库
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// 项目内部
use domain::{
    Dir, PathError, PathService, COMPLETIONS_DIR, COMPLETIONS_FILE, COMPLETION_CACHE_DIR,
    JIRA_CONFIG_FILE, MAIN_DIR, PROJECT_CONFIG_FILE, USER_CONFIG_FILE, WORKFLOW_CONFIG_DIR,
    WORKFLOW_CONFIG_FILE,
};

/// 路径服务实现
///
/// 管理应用程序的配置文件、数据目录等路径。
/// 支持 iCloud 同步（macOS）和本地存储两种模式。
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

    /// 获取基础目录（iCloud + 本地）。
    ///
    /// - **iCloud**：可选；不可用时回退为空路径，不向调用方报错。
    /// - **本地目录**：必须可用（如 `~/.workflow/`）；若无法解析或创建则返回错误并向上传播。
    fn get_base_dir(&self) -> Result<Dir, PathError> {
        // iCloud 为可选：不可用时回退为空路径，不向调用方报错
        let icloud_base_dir = if self.is_icloud_available {
            self.try_icloud_base_dir().unwrap_or_default()
        } else {
            PathBuf::new()
        };

        let local_base_dir = self.try_local_base_dir()?;
        let is_icloud_available =
            !icloud_base_dir.as_os_str().is_empty() && icloud_base_dir.exists();

        Ok(Dir {
            is_icloud_available,
            icloud_base_dir,
            local_base_dir,
        })
    }

    /// 创建目录并设置权限（700，仅用户可访问）
    ///
    /// 这是一个辅助函数，用于统一处理目录创建和权限设置。
    fn create_dir_with_permissions(&self, dir: &Path, name: &str) -> Result<(), PathError> {
        // 确保目录存在
        fs::create_dir_all(dir).map_err(PathError::Io)?;

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
}

impl PathService for PathServiceImpl {
    fn get_workflow_config_dir(&self) -> Result<PathBuf, PathError> {
        let base_dir = self.get_base_dir()?;
        let config_dir = if base_dir.is_icloud_available {
            base_dir.icloud_base_dir.join(WORKFLOW_CONFIG_DIR)
        } else {
            base_dir.local_base_dir.join(WORKFLOW_CONFIG_DIR)
        };
        self.create_dir_with_permissions(&config_dir, WORKFLOW_CONFIG_DIR)?;
        Ok(config_dir)
    }

    fn get_workflow_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_dir = self.get_workflow_config_dir()?;
        let config_filepath = config_dir.join(WORKFLOW_CONFIG_FILE);
        Ok(config_filepath)
    }

    fn get_jira_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_dir = self.get_workflow_config_dir()?;
        let config_filepath = config_dir.join(JIRA_CONFIG_FILE);
        Ok(config_filepath)
    }

    fn get_jira_work_history_dir(&self) -> Result<PathBuf, PathError> {
        // 强制使用本地路径，不使用 iCloud
        let history_dir = self.try_local_base_dir()?.join("work-history");
        self.create_dir_with_permissions(&history_dir, "work-history")?;
        Ok(history_dir)
    }

    fn get_project_config_dir(&self) -> Result<PathBuf, PathError> {
        let base_dir =
            std::env::current_dir().map_err(PathError::Io).map(|path| path.join(MAIN_DIR))?;
        self.create_dir_with_permissions(&base_dir, MAIN_DIR)?;
        Ok(base_dir)
    }

    fn get_project_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_dir = self.get_project_config_dir()?;
        let config_filepath = config_dir.join(PROJECT_CONFIG_FILE);
        Ok(config_filepath)
    }

    fn get_user_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_dir = self.get_project_config_dir()?;
        let config_filepath = config_dir.join(USER_CONFIG_FILE);
        Ok(config_filepath)
    }

    fn get_mcp_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_dir = self.get_project_config_dir()?;
        let config_filepath = config_dir.join(".cursor").join("mcp.json");
        Ok(config_filepath)
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

    fn get_download_dir(&self) -> Result<PathBuf, PathError> {
        let download_base_dir = dirs::document_dir()
            .map(|h| h.join("Workflow").to_string_lossy().to_string())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    PathBuf::from("C:\\Users\\User\\Documents\\Workflow")
                } else {
                    PathBuf::from("~/Documents/Workflow")
                }
            });
        Ok(download_base_dir)
    }

    fn get_completion_dir(&self) -> Result<PathBuf, PathError> {
        let local_base_dir = self.try_local_base_dir()?;
        let completion_dir = local_base_dir.join(COMPLETIONS_DIR);
        fs::create_dir_all(&completion_dir).map_err(PathError::Io)?;
        Ok(completion_dir)
    }

    fn get_completion_cache_dir(&self) -> Result<PathBuf, PathError> {
        let local_base_dir = self.try_local_base_dir()?;
        let cache_dir = local_base_dir.join(COMPLETION_CACHE_DIR);
        fs::create_dir_all(&cache_dir).map_err(PathError::Io)?;
        Ok(cache_dir)
    }

    fn get_completion_config_filepath(&self) -> Result<PathBuf, PathError> {
        let config_file = self.try_local_base_dir()?.join(COMPLETIONS_FILE);
        Ok(config_file)
    }

    fn get_logs_dir(&self) -> Result<PathBuf, PathError> {
        // 强制使用本地路径，不使用 iCloud
        let logs_dir = self.try_local_base_dir()?.join("logs");
        self.create_dir_with_permissions(&logs_dir, "logs")?;
        Ok(logs_dir)
    }
}
