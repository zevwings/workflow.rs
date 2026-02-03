//! 别名服务接口
//!
//! 提供别名的增删查功能。

use crate::alias::entity::{AliasAddResult, AliasListResult, AliasRemoveResult};
use crate::errors::ServiceError;

/// 别名服务接口
///
/// 负责别名的管理，包括：
/// - 列出所有别名
/// - 添加新别名
/// - 移除别名
pub trait AliasService: Send + Sync {
    /// 列出所有别名
    ///
    /// # 返回
    /// 别名列表结果，包含所有已定义的别名
    fn list(&self) -> Result<AliasListResult, ServiceError>;

    /// 添加别名
    ///
    /// # 参数
    /// - `name`: 别名名称
    /// - `command`: 对应的命令
    /// - `force`: 是否强制覆盖已存在的别名
    ///
    /// # 返回
    /// 添加结果，包含是否为覆盖操作
    fn add(&self, name: &str, command: &str, force: bool) -> Result<AliasAddResult, ServiceError>;

    /// 移除别名
    ///
    /// # 参数
    /// - `name`: 要移除的别名名称
    ///
    /// # 返回
    /// 移除结果，包含被移除的别名信息
    fn remove(&self, name: &str) -> Result<AliasRemoveResult, ServiceError>;

    /// 检查别名是否存在
    ///
    /// # 参数
    /// - `name`: 别名名称
    ///
    /// # 返回
    /// 如果别名存在返回 Some(command)，否则返回 None
    fn get(&self, name: &str) -> Result<Option<String>, ServiceError>;
}
