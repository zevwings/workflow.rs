//! User 服务实现
//!
//! 提供 Jira 用户信息获取的业务逻辑实现。

use std::sync::Arc;

use domain::{JiraError, JiraUser};

use crate::jira::client::core::JiraClient;
use crate::jira::client::types::JiraResponseSerializable;

pub trait UserService: Send + Sync {
    fn me(&self) -> Result<JiraUser, JiraError>;
}

pub struct UserServiceImpl {
    jira_client: Arc<dyn JiraClient>,
}

impl UserService for UserServiceImpl {
    fn me(&self) -> Result<JiraUser, JiraError> {
        // 1. 调用 API 获取用户信息
        let response = self
            .jira_client
            .get("myself", None)
            .map_err(|e| JiraError::ApiError(format!("Failed to get user info: {}", e)))?;

        // 2. 解析响应为 Storage DTO
        let user = response
            .as_model::<JiraUser>()
            .map_err(|e| JiraError::ApiError(format!("Failed to parse user info: {}", e)))?;

        // 3. DTO → Domain 映射
        Ok(user)
    }
}

impl UserServiceImpl {
    pub fn new(jira_client: Arc<dyn JiraClient>) -> Self {
        Self { jira_client }
    }
}
