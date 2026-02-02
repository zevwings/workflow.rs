//! CNB 实体类型

/// CNB 用户信息
#[derive(Debug, Clone)]
pub struct CNBUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}
