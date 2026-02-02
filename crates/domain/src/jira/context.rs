//! Jira 配置上下文接口
//!
//! 定义 Jira 配置上下文的 trait，用于封装配置获取逻辑。

/// Jira 配置上下文接口
///
/// 封装 Jira 配置获取逻辑，提供认证信息和 API URL 的统一接口。
pub trait JiraConfigContext: Send + Sync {
    /// 获取 Jira 用户邮箱
    ///
    /// # 返回
    ///
    /// 返回 Jira 用户邮箱。
    fn get_jira_email(&self) -> String;

    /// 获取 Jira API Token
    ///
    /// # 返回
    ///
    /// 返回 Jira API Token。
    fn get_jira_api_token(&self) -> String;

    /// 获取 Jira 服务地址
    ///
    /// # 返回
    ///
    /// 返回 Jira 服务地址。
    fn get_jira_service_address(&self) -> String;
}
