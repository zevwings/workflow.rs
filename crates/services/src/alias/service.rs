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
                "别名名称不能为空".to_string(),
            ));
        }

        // 验证命令
        if command.is_empty() {
            return Err(ServiceError::InvalidInput("命令不能为空".to_string()));
        }

        // 检查别名名称是否包含空格
        if name.contains(' ') {
            return Err(ServiceError::InvalidInput(
                "别名名称不能包含空格".to_string(),
            ));
        }

        // 加载当前配置
        let mut config = self.config_repo.load()?;

        // 检查是否已存在
        let overwritten = config.aliases.contains_key(name);
        if overwritten && !force {
            return Err(ServiceError::InvalidInput(format!(
                "别名 '{}' 已存在，使用 --force 覆盖",
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
                "别名名称不能为空".to_string(),
            ));
        }

        // 加载当前配置
        let mut config = self.config_repo.load()?;

        // 检查别名是否存在
        let command = config.aliases.remove(name).ok_or_else(|| {
            ServiceError::InvalidInput(format!("别名 '{}' 不存在", name))
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
