//! 全局配置仓储实现
//!
//! 提供 GlobalConfig 的加载、保存和缓存管理功能。

use std::sync::{Arc, Mutex, OnceLock};

use domain::{GlobalConfig, GlobalConfigRepository, PathService, ServiceError};
use toolkit::file;

/// 全局配置缓存
static GLOBAL_CONFIG: OnceLock<Mutex<Option<GlobalConfig>>> = OnceLock::new();

/// 全局配置仓储实现
pub struct GlobalConfigRepositoryImpl {
    path_service: Arc<dyn PathService>,
}

impl GlobalConfigRepositoryImpl {
    /// 创建新的全局配置存储服务实例
    pub fn new(path_service: Arc<dyn PathService>) -> Self {
        Self { path_service }
    }

    /// 清除全局配置缓存
    ///
    /// 用于在配置保存后清除缓存，确保下次加载时读取最新配置。
    fn clear(&self) {
        if let Some(cache) = GLOBAL_CONFIG.get() {
            if let Ok(mut cached) = cache.lock() {
                *cached = None;
            }
        }
    }
}

impl GlobalConfigRepository for GlobalConfigRepositoryImpl {
    /// 加载全局配置（带缓存）
    ///
    /// 使用缓存机制，首次加载后后续调用返回缓存的配置。
    /// 保存配置后会自动清除缓存，下次加载时会重新读取文件。
    fn load(&self) -> Result<GlobalConfig, ServiceError> {
        let cache = GLOBAL_CONFIG.get_or_init(|| Mutex::new(None));

        // 尝试从缓存获取
        {
            let cached = cache.lock().map_err(|_| {
                ServiceError::OperationFailed("Failed to acquire cache lock".to_string())
            })?;
            if let Some(config) = cached.as_ref() {
                return Ok(config.clone());
            }
        }

        // 缓存未命中，加载配置（get_config_path 返回配置目录，全局配置文件为 workflow.toml）
        let config_filepath = self.path_service.get_workflow_config_filepath().map_err(|e| {
            toolkit::log_error!("Failed to get config path: {}", e);
            ServiceError::OperationFailed("Failed to get config path".to_string())
        })?;

        let settings = if !config_filepath.exists() {
            GlobalConfig::default()
        } else {
            let content = file::read_string(&config_filepath).map_err(|e| {
                toolkit::log_error!("Failed to read config: {}", e);
                ServiceError::OperationFailed("Failed to read config".to_string())
            })?;

            toml::from_str(&content).map_err(|e| {
                toolkit::log_error!("Failed to parse config: {}", e);
                ServiceError::OperationFailed("Failed to parse config".to_string())
            })?
        };

        // 更新缓存
        {
            let mut cached = cache.lock().map_err(|_| {
                ServiceError::OperationFailed("Failed to acquire cache lock".to_string())
            })?;
            *cached = Some(settings.clone());
        }

        Ok(settings)
    }

    /// 保存全局配置
    ///
    /// 保存配置后会自动清除缓存，下次加载时会重新读取文件。
    /// 在 Unix 系统上会自动设置文件权限为 600 以确保安全性。
    fn save(&self, settings: &GlobalConfig) -> Result<(), ServiceError> {
        let config_filepath = self.path_service.get_workflow_config_filepath().map_err(|e| {
            toolkit::log_error!("Failed to get config path: {}", e);
            ServiceError::OperationFailed("Failed to get config path".to_string())
        })?;

        let content = toml::to_string(settings).map_err(|e| {
            toolkit::log_error!("Failed to serialize settings: {}", e);
            ServiceError::OperationFailed("Failed to serialize settings".to_string())
        })?;

        file::write_string(&config_filepath, &content).map_err(|e| {
            toolkit::log_error!("Failed to write config: {}", e);
            ServiceError::OperationFailed("Failed to write config".to_string())
        })?;

        // 设置文件权限为 600（仅 Unix 系统）
        #[cfg(unix)]
        {
            file::set_permissions(&config_filepath, 0o600).map_err(|e| {
                toolkit::log_error!("Failed to set config file permissions: {}", e);
                ServiceError::OperationFailed("Failed to set config file permissions".to_string())
            })?;
        }

        // 清除缓存，下次加载时会重新读取
        self.clear();

        Ok(())
    }

    /// 检查配置文件权限（仅 Unix 系统）
    fn check_permissions(&self) -> Option<String> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(config_filepath) = self.path_service.get_workflow_config_filepath() {
                if config_filepath.exists() {
                    if let Ok(metadata) = config_filepath.metadata() {
                        let permissions = metadata.permissions();
                        let mode = permissions.mode();
                        // 检查是否有组或其他用户权限（非 600）
                        if (mode & 0o077) != 0 {
                            return Some(format!(
                                "Warning: Configuration file has overly permissive permissions (current: {:o}). Consider setting to 600 for better security.",
                                mode & 0o777
                            ));
                        }
                    }
                }
            }
        }
        None
    }
}
