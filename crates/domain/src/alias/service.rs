//! 别名服务接口
//!
//! 提供别名的增删查功能。

use crate::alias::entity::{AliasAddResult, AliasListResult, AliasRemoveResult};
use crate::alias::error::AliasError;

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
    fn list(&self) -> Result<AliasListResult, AliasError>;

    /// 添加别名
    ///
    /// # 参数
    /// - `name`: 别名名称
    /// - `command`: 对应的命令
    /// - `force`: 是否强制覆盖已存在的别名
    ///
    /// # 返回
    /// 添加结果，包含是否为覆盖操作
    fn add(&self, name: &str, command: &str, force: bool) -> Result<AliasAddResult, AliasError>;

    /// 移除别名
    ///
    /// # 参数
    /// - `name`: 要移除的别名名称
    ///
    /// # 返回
    /// 移除结果，包含被移除的别名信息
    fn remove(&self, name: &str) -> Result<AliasRemoveResult, AliasError>;

    /// 检查别名是否存在
    ///
    /// # 参数
    /// - `name`: 别名名称
    ///
    /// # 返回
    /// 如果别名存在返回 Some(command)，否则返回 None
    fn get(&self, name: &str) -> Result<Option<String>, AliasError>;

    /// 展开别名（支持嵌套）
    ///
    /// 递归展开别名，支持嵌套别名（别名引用别名）。
    ///
    /// # 参数
    /// - `name`: 要展开的别名名称
    ///
    /// # 返回
    /// 展开后的完整命令字符串
    ///
    /// # 错误
    /// - 如果别名不存在，返回错误
    /// - 如果检测到循环引用，返回错误
    /// - 如果超过最大展开深度，返回错误
    fn expand(&self, name: &str) -> Result<String, AliasError>;

    /// 展开命令行参数（如果第一个参数是别名）
    ///
    /// 检查参数列表中的第一个参数（子命令）是否是别名，
    /// 如果是则展开为对应的完整命令。
    ///
    /// # 参数
    /// - `args`: 命令行参数列表（第一个元素是程序名）
    ///
    /// # 返回
    /// 展开后的参数列表。如果第一个参数不是别名，返回原参数列表。
    fn expand_args(&self, args: Vec<String>) -> Result<Vec<String>, AliasError>;
}
