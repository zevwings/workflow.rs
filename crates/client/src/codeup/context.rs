//! Codeup Provider 接口

use crate::codeup::CodeupClientError;

/// Codeup Context trait
///
/// 提供 Codeup API 所需的配置信息
pub trait CodeupConfigContext: Send + Sync {
    /// 获取项目 ID
    fn get_project_id(&self) -> Result<String, CodeupClientError>;
    /// 获取 CSRF Token
    fn get_csrf_token(&self) -> Result<String, CodeupClientError>;
    /// 获取 Cookie
    fn get_cookie(&self) -> Result<String, CodeupClientError>;
}
