//! 全局配置仓储接口
//!
//! 负责管理全局配置（GlobalConfig）的加载、保存和持久化。

use crate::config::global::config::GlobalConfig;
use crate::errors::ServiceError;

/// 全局配置仓储接口
///
/// 负责管理全局配置（GlobalConfig）的加载、保存和持久化。
pub trait GlobalConfigRepository: Send + Sync {
    /// 加载全局配置
    fn load(&self) -> Result<GlobalConfig, ServiceError>;

    /// 保存全局配置
    fn save(&self, settings: &GlobalConfig) -> Result<(), ServiceError>;

    /// 检查配置文件权限（仅 Unix 系统）
    fn check_permissions(&self) -> Option<String>;
}
