//! User 服务实现
//!
//! 提供 Jira 用户信息获取的业务逻辑实现。

use std::sync::Arc;

use domain::{JiraError, JiraUser};

use crate::jira::client::core::JiraClient;
use crate::jira::client::types::JiraResponseSerializable;
use crate::jira::types::JiraUser as StorageJiraUser;

pub trait UserService: Send + Sync {
    fn get_user_info(&self) -> Result<JiraUser, JiraError>;
}

pub struct UserServiceImpl {
    jira_client: Arc<dyn JiraClient>,
}

impl UserService for UserServiceImpl {
    fn get_user_info(&self) -> Result<JiraUser, JiraError> {
        // 1. 调用 API 获取用户信息
        let response = self
            .jira_client
            .get("myself", None)
            .map_err(|e| JiraError::ApiError(format!("Failed to get user info: {}", e)))?;

        // 2. 解析响应为 Storage DTO
        let user_dto = response
            .as_model::<StorageJiraUser>()
            .map_err(|e| JiraError::ApiError(format!("Failed to parse user info: {}", e)))?;

        // 3. DTO → Domain 映射
        Ok(JiraUser {
            display_name: user_dto.display_name,
            account_id: user_dto.account_id,
        })
    }
}

impl UserServiceImpl {
    pub fn new(jira_client: Arc<dyn JiraClient>) -> Self {
        Self { jira_client }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jira::client::types::JiraResponse;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    struct MockJiraClient {
        responses: HashMap<String, JiraResponse>,
    }

    impl MockJiraClient {
        fn new(responses: HashMap<String, JiraResponse>) -> Self {
            Self { responses }
        }
    }

    impl JiraClient for MockJiraClient {
        fn get(
            &self,
            path: &str,
            _query: Option<&[(String, String)]>,
        ) -> Result<JiraResponse, JiraError> {
            self.responses
                .get(path)
                .cloned()
                .ok_or_else(|| JiraError::ApiError(format!("missing response for {}", path)))
        }

        fn post(
            &self,
            _path: &str,
            _body: &serde_json::Value,
            _query: Option<&[(String, String)]>,
        ) -> Result<JiraResponse, JiraError> {
            Ok(JiraResponse::new(json!({})))
        }
    }

    #[test]
    fn test_get_user_info_maps_fields() {
        let mut responses = HashMap::new();
        responses.insert(
            "myself".to_string(),
            JiraResponse::new(json!({
                "accountId": "123",
                "displayName": "User"
            })),
        );
        let client = Arc::new(MockJiraClient::new(responses));
        let service = UserServiceImpl::new(client);

        let user = service.get_user_info().unwrap();
        assert_eq!(user.display_name, "User");
        assert_eq!(user.account_id, "123");
    }
}
