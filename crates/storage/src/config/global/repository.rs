//! 全局配置仓储实现
//!
//! 提供 GlobalConfig 的加载、保存和缓存管理功能。

use std::sync::{Mutex, OnceLock};

use domain::{GlobalConfig, GlobalConfigRepository, ServiceError};
use toolkit::{file, workflow_config_path};

/// 全局配置缓存
static GLOBAL_CONFIG: OnceLock<Mutex<Option<GlobalConfig>>> = OnceLock::new();

/// 全局配置仓储实现
pub struct GlobalConfigRepositoryImpl;

impl Default for GlobalConfigRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalConfigRepositoryImpl {
    /// 创建新的全局配置存储服务实例
    pub fn new() -> Self {
        Self
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

        // 缓存未命中，加载配置
        let config_path = workflow_config_path().map_err(|e| {
            ServiceError::OperationFailed(format!("Failed to get config path: {}", e))
        })?;

        let settings = if !config_path.exists() {
            GlobalConfig::default()
        } else {
            let content = file::read_string(&config_path).map_err(|e| {
                ServiceError::OperationFailed(format!("Failed to read config: {}", e))
            })?;

            toml::from_str(&content).map_err(|e| {
                ServiceError::OperationFailed(format!("Failed to parse config: {}", e))
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
        let config_path = workflow_config_path().map_err(|e| {
            ServiceError::OperationFailed(format!("Failed to get config path: {}", e))
        })?;

        let content = toml::to_string(settings).map_err(|e| {
            ServiceError::OperationFailed(format!("Failed to serialize settings: {}", e))
        })?;

        file::write_string(&config_path, &content)
            .map_err(|e| ServiceError::OperationFailed(format!("Failed to write config: {}", e)))?;

        // 设置文件权限为 600（仅 Unix 系统）
        #[cfg(unix)]
        {
            file::set_permissions(&config_path, 0o600).map_err(|e| {
                ServiceError::OperationFailed(format!(
                    "Failed to set config file permissions: {}",
                    e
                ))
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
            if let Ok(config_path) = workflow_config_path() {
                if config_path.exists() {
                    if let Ok(metadata) = config_path.metadata() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::env;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use toolkit::file;
    use toolkit::workflow_config_path;

    static TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

    fn reset_cache() {
        let cache = GLOBAL_CONFIG.get_or_init(|| Mutex::new(None));
        if let Ok(mut cached) = cache.lock() {
            *cached = None;
        }
    }

    fn with_temp_home<F: FnOnce()>(f: F) {
        let _lock = TEST_ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        let original_disable = env::var("WORKFLOW_DISABLE_ICLOUD").ok();

        env::set_var("HOME", tmp.path());
        env::set_var("WORKFLOW_DISABLE_ICLOUD", "1");
        reset_cache();

        f();

        if let Some(value) = original_home {
            env::set_var("HOME", value);
        } else {
            env::remove_var("HOME");
        }
        if let Some(value) = original_disable {
            env::set_var("WORKFLOW_DISABLE_ICLOUD", value);
        } else {
            env::remove_var("WORKFLOW_DISABLE_ICLOUD");
        }
        reset_cache();
    }

    #[test]
    fn test_load_default_when_missing() {
        with_temp_home(|| {
            let repo = GlobalConfigRepositoryImpl::new();
            let config = repo.load().unwrap();
            let toml = toml::to_string(&config).unwrap();
            assert!(toml.trim().is_empty());
        });
    }

    #[test]
    fn test_load_uses_cache() {
        with_temp_home(|| {
            let repo = GlobalConfigRepositoryImpl::new();
            let config_path = workflow_config_path().unwrap();

            let mut aliases = HashMap::new();
            aliases.insert("co".to_string(), "checkout".to_string());
            let config = GlobalConfig {
                aliases,
                ..GlobalConfig::default()
            };
            repo.save(&config).unwrap();

            let loaded = repo.load().unwrap();
            assert!(loaded.aliases.contains_key("co"));

            let mut aliases = HashMap::new();
            aliases.insert("br".to_string(), "branch".to_string());
            let updated = GlobalConfig {
                aliases,
                ..GlobalConfig::default()
            };
            let content = toml::to_string(&updated).unwrap();
            file::write_string(&config_path, &content).unwrap();

            let cached = repo.load().unwrap();
            assert!(cached.aliases.contains_key("co"));
            assert!(!cached.aliases.contains_key("br"));
        });
    }

    #[test]
    fn test_save_clears_cache_and_sets_permissions() {
        with_temp_home(|| {
            let repo = GlobalConfigRepositoryImpl::new();
            let config_path = workflow_config_path().unwrap();

            let mut aliases = HashMap::new();
            aliases.insert("co".to_string(), "checkout".to_string());
            let config = GlobalConfig {
                aliases,
                ..GlobalConfig::default()
            };
            repo.save(&config).unwrap();

            let loaded = repo.load().unwrap();
            assert!(loaded.aliases.contains_key("co"));

            let mut aliases = HashMap::new();
            aliases.insert("st".to_string(), "status".to_string());
            let updated = GlobalConfig {
                aliases,
                ..GlobalConfig::default()
            };
            repo.save(&updated).unwrap();

            let refreshed = repo.load().unwrap();
            assert!(refreshed.aliases.contains_key("st"));
            assert!(!refreshed.aliases.contains_key("co"));

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = config_path.metadata().unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, 0o600);
            }
        });
    }
}
