//! GitHub 实体类型

/// GitHub 用户信息
#[derive(Debug, Clone)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}
