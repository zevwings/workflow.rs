use crate::{
    jira::{JiraClientError, JiraResponse},
    HttpMethod,
};

pub struct JiraRequest {
    /// API 路径（相对路径，如 "/issue/PROJ-123"）
    pub path: String,
    pub method: HttpMethod,
    pub body: Option<serde_json::Value>,
    pub query: Option<serde_json::Value>,
}

pub trait JiraClient: Send + Sync {
    /// 执行 Jira API 请求（核心方法）
    fn execute(&self, request: JiraRequest) -> Result<JiraResponse, JiraClientError>;

    /// GET 请求（便捷方法）
    fn get(
        &self,
        path: &str,
        query: Option<serde_json::Value>,
    ) -> Result<JiraResponse, JiraClientError> {
        self.execute(JiraRequest {
            path: path.to_string(),
            method: HttpMethod::GET,
            body: None,
            query,
        })
    }

    /// POST 请求（便捷方法）
    ///
    /// 接受 `serde_json::Value` 作为 body，保证 trait 的 dyn-compatibility
    fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
        query: Option<serde_json::Value>,
    ) -> Result<JiraResponse, JiraClientError> {
        self.execute(JiraRequest {
            path: path.to_string(),
            method: HttpMethod::POST,
            body: Some(body.clone()),
            query,
        })
    }

    /// PUT 请求（便捷方法）
    ///
    /// 接受 `serde_json::Value` 作为 body，保证 trait 的 dyn-compatibility
    fn put(
        &self,
        path: &str,
        body: &serde_json::Value,
        query: Option<serde_json::Value>,
    ) -> Result<JiraResponse, JiraClientError> {
        self.execute(JiraRequest {
            path: path.to_string(),
            method: HttpMethod::PUT,
            body: Some(body.clone()),
            query,
        })
    }

    /// DELETE 请求（便捷方法）
    fn delete(&self, path: &str) -> Result<JiraResponse, JiraClientError> {
        self.execute(JiraRequest {
            path: path.to_string(),
            method: HttpMethod::DELETE,
            body: None,
            query: None,
        })
    }
}
