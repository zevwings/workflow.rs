//! CNB Provider 接口
//!
//! 定义 CNB API 所需的配置和操作接口，实现依赖倒置原则。

use crate::cnb::CNBError;

/// CNB Context trait
///
/// 提供 CNB API 所需的配置信息，包括账号、Token、项目路径等。
/// 通过此 trait，CNB API 模块可以独立于具体的配置实现。
pub trait CNBContext: Send + Sync {
    /// 获取账号名称
    fn get_name(&self) -> Result<String, CNBError>;

    /// 获取用户登录名
    fn get_login(&self) -> Result<String, CNBError>;

    /// 获取账号邮箱
    fn get_email(&self) -> Result<String, CNBError>;

    /// 获取 API Token
    fn get_api_token(&self) -> Result<String, CNBError>;

    /// 获取项目路径（格式：owner/project）
    fn get_project_path(&self) -> Result<String, CNBError>;
}
