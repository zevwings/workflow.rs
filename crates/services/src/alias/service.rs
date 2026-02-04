//! 别名服务实现
//!
//! 实现 `AliasService` trait，负责别名的管理。

use std::sync::Arc;

use domain::{
    alias::{AliasAddResult, AliasInfo, AliasListResult, AliasRemoveResult, AliasService},
    errors::ServiceError,
    GlobalConfigRepository,
};

/// 别名服务实现
pub struct AliasServiceImpl {
    /// 全局配置仓储
    config_repo: Arc<dyn GlobalConfigRepository>,
}

impl AliasServiceImpl {
    /// 创建新的 AliasServiceImpl 实例
    pub fn new(config_repo: Arc<dyn GlobalConfigRepository>) -> Self {
        Self { config_repo }
    }
}

impl AliasService for AliasServiceImpl {
    fn list(&self) -> Result<AliasListResult, ServiceError> {
        let config = self.config_repo.load()?;

        let aliases: Vec<AliasInfo> = config
            .aliases
            .iter()
            .map(|(name, command)| AliasInfo::new(name, command))
            .collect();

        let count = aliases.len();

        Ok(AliasListResult { aliases, count })
    }

    fn add(&self, name: &str, command: &str, force: bool) -> Result<AliasAddResult, ServiceError> {
        // 验证别名名称
        if name.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Alias name cannot be empty".to_string(),
            ));
        }

        // 验证命令
        if command.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Command cannot be empty".to_string(),
            ));
        }

        // 检查别名名称是否包含空格
        if name.contains(' ') {
            return Err(ServiceError::InvalidInput(
                "Alias name cannot contain spaces".to_string(),
            ));
        }

        // 加载当前配置
        let mut config = self.config_repo.load()?;

        // 检查是否已存在
        let overwritten = config.aliases.contains_key(name);
        if overwritten && !force {
            return Err(ServiceError::InvalidInput(format!(
                "Alias '{}' already exists, use --force to overwrite",
                name
            )));
        }

        // 添加或更新别名
        config.aliases.insert(name.to_string(), command.to_string());

        // 保存配置
        self.config_repo.save(&config)?;

        Ok(AliasAddResult {
            name: name.to_string(),
            command: command.to_string(),
            overwritten,
        })
    }

    fn remove(&self, name: &str) -> Result<AliasRemoveResult, ServiceError> {
        // 验证别名名称
        if name.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Alias name cannot be empty".to_string(),
            ));
        }

        // 加载当前配置
        let mut config = self.config_repo.load()?;

        // 检查别名是否存在
        let command = config.aliases.remove(name).ok_or_else(|| {
            ServiceError::InvalidInput(format!("Alias '{}' does not exist", name))
        })?;

        // 保存配置
        self.config_repo.save(&config)?;

        Ok(AliasRemoveResult {
            name: name.to_string(),
            command,
        })
    }

    fn get(&self, name: &str) -> Result<Option<String>, ServiceError> {
        let config = self.config_repo.load()?;
        Ok(config.aliases.get(name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    use domain::config::global::config::GlobalConfig;

    struct MockGlobalConfigRepository {
        config: Mutex<GlobalConfig>,
    }

    impl MockGlobalConfigRepository {
        fn new(config: GlobalConfig) -> Self {
            Self {
                config: Mutex::new(config),
            }
        }
    }

    impl GlobalConfigRepository for MockGlobalConfigRepository {
        fn load(&self) -> Result<GlobalConfig, ServiceError> {
            self.config
                .lock()
                .map(|cfg| cfg.clone())
                .map_err(|_| ServiceError::Other("Failed to lock config repository".to_string()))
        }

        fn save(&self, settings: &GlobalConfig) -> Result<(), ServiceError> {
            let mut config = self
                .config
                .lock()
                .map_err(|_| ServiceError::Other("Failed to lock config repository".to_string()))?;
            *config = settings.clone();
            Ok(())
        }

        fn check_permissions(&self) -> Option<String> {
            None
        }
    }

    struct FailingGlobalConfigRepository {
        config: Mutex<GlobalConfig>,
        fail_load: bool,
        fail_save: bool,
    }

    impl FailingGlobalConfigRepository {
        fn new(config: GlobalConfig, fail_load: bool, fail_save: bool) -> Self {
            Self {
                config: Mutex::new(config),
                fail_load,
                fail_save,
            }
        }
    }

    impl GlobalConfigRepository for FailingGlobalConfigRepository {
        fn load(&self) -> Result<GlobalConfig, ServiceError> {
            if self.fail_load {
                return Err(ServiceError::Other("load failed".to_string()));
            }
            self.config
                .lock()
                .map(|cfg| cfg.clone())
                .map_err(|_| ServiceError::Other("Failed to lock config repository".to_string()))
        }

        fn save(&self, settings: &GlobalConfig) -> Result<(), ServiceError> {
            if self.fail_save {
                return Err(ServiceError::Other("save failed".to_string()));
            }
            let mut config = self
                .config
                .lock()
                .map_err(|_| ServiceError::Other("Failed to lock config repository".to_string()))?;
            *config = settings.clone();
            Ok(())
        }

        fn check_permissions(&self) -> Option<String> {
            None
        }
    }

    fn build_service(config: GlobalConfig) -> AliasServiceImpl {
        let repo = Arc::new(MockGlobalConfigRepository::new(config));
        AliasServiceImpl::new(repo)
    }

    fn build_service_with_repo(repo: Arc<dyn GlobalConfigRepository>) -> AliasServiceImpl {
        AliasServiceImpl::new(repo)
    }

    #[test]
    fn list_returns_empty_when_no_aliases() {
        let service = build_service(GlobalConfig::default());
        let result = service.list().unwrap();
        assert_eq!(result.count, 0);
        assert!(result.aliases.is_empty());
    }

    #[test]
    fn add_stores_alias_and_returns_result() {
        let service = build_service(GlobalConfig::default());
        let result = service.add("st", "status", false).unwrap();
        assert_eq!(result.name, "st");
        assert_eq!(result.command, "status");
        assert!(!result.overwritten);

        let list = service.list().unwrap();
        assert_eq!(list.count, 1);
        assert_eq!(list.aliases[0].name, "st");
        assert_eq!(list.aliases[0].command, "status");
    }

    #[test]
    fn add_rejects_empty_name() {
        let service = build_service(GlobalConfig::default());
        let err = service.add("", "status", false).unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn add_rejects_empty_command() {
        let service = build_service(GlobalConfig::default());
        let err = service.add("st", "", false).unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn add_rejects_name_with_spaces() {
        let service = build_service(GlobalConfig::default());
        let err = service.add("git st", "status", false).unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn add_rejects_existing_without_force() {
        let service = build_service(GlobalConfig::default());
        service.add("st", "status", false).unwrap();
        let err = service.add("st", "status -sb", false).unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn add_overwrites_existing_with_force() {
        let service = build_service(GlobalConfig::default());
        service.add("st", "status", false).unwrap();
        let result = service.add("st", "status -sb", true).unwrap();
        assert!(result.overwritten);

        let command = service.get("st").unwrap().unwrap();
        assert_eq!(command, "status -sb");
    }

    #[test]
    fn remove_deletes_existing_alias() {
        let service = build_service(GlobalConfig::default());
        service.add("st", "status", false).unwrap();
        let result = service.remove("st").unwrap();
        assert_eq!(result.name, "st");
        assert_eq!(result.command, "status");

        let list = service.list().unwrap();
        assert_eq!(list.count, 0);
    }

    #[test]
    fn remove_rejects_empty_name() {
        let service = build_service(GlobalConfig::default());
        let err = service.remove("").unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn remove_rejects_missing_alias() {
        let service = build_service(GlobalConfig::default());
        let err = service.remove("missing").unwrap_err();
        assert!(matches!(err, ServiceError::InvalidInput(_)));
    }

    #[test]
    fn get_returns_none_for_missing_alias() {
        let service = build_service(GlobalConfig::default());
        let result = service.get("missing").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_returns_alias_for_existing_name() {
        let mut config = GlobalConfig::default();
        config.aliases.insert("st".to_string(), "status".to_string());
        let service = build_service(config);

        let result = service.get("st").unwrap();
        assert_eq!(result, Some("status".to_string()));
    }

    #[test]
    fn test_list_propagates_load_error() {
        let repo = Arc::new(FailingGlobalConfigRepository::new(
            GlobalConfig::default(),
            true,
            false,
        ));
        let service = build_service_with_repo(repo);
        let err = service.list().unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
    }

    #[test]
    fn test_add_propagates_save_error() {
        let repo = Arc::new(FailingGlobalConfigRepository::new(
            GlobalConfig::default(),
            false,
            true,
        ));
        let service = build_service_with_repo(repo);
        let err = service.add("st", "status", false).unwrap_err();
        assert!(matches!(err, ServiceError::Other(_)));
    }
}
