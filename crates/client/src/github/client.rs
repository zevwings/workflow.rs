//! GitHub API 客户端
//!
//! 封装 GitHub API 的公共 HTTP 请求逻辑，包括：
//! - 请求头构建
//! - 错误处理
//! - API 基础 URL 管理

use serde_json::Value;

use crate::{
    github::{GitHubClientError, GitHubResponse},
    http::HttpMethod,
};

pub struct GitHubRequest {
    pub path: String,
    pub method: HttpMethod,
    pub body: Option<Value>,
    pub query: Option<Value>,
}

pub trait GitHubClient: Send + Sync {
    /// 执行 GitHub API 请求（核心方法）
    fn execute(&self, request: GitHubRequest) -> Result<GitHubResponse, GitHubClientError>;

    /// GET 请求（便捷方法）
    fn get(&self, path: &str) -> Result<GitHubResponse, GitHubClientError> {
        self.execute(GitHubRequest {
            path: path.to_string(),
            method: HttpMethod::GET,
            body: None,
            query: None,
        })
    }

    /// POST 请求（便捷方法）
    ///
    /// 接受 `serde_json::Value` 作为 body，保证 trait 的 dyn-compatibility
    fn post(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubClientError> {
        self.execute(GitHubRequest {
            path: path.to_string(),
            method: HttpMethod::POST,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// PUT 请求（便捷方法）
    ///
    /// 接受 `serde_json::Value` 作为 body，保证 trait 的 dyn-compatibility
    fn put(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubClientError> {
        self.execute(GitHubRequest {
            path: path.to_string(),
            method: HttpMethod::PUT,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// PATCH 请求（便捷方法）
    ///
    /// 接受 `serde_json::Value` 作为 body，保证 trait 的 dyn-compatibility
    fn patch(&self, path: &str, body: &Value) -> Result<GitHubResponse, GitHubClientError> {
        self.execute(GitHubRequest {
            path: path.to_string(),
            method: HttpMethod::PATCH,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// DELETE 请求（便捷方法）
    fn delete(&self, path: &str) -> Result<GitHubResponse, GitHubClientError> {
        self.execute(GitHubRequest {
            path: path.to_string(),
            method: HttpMethod::DELETE,
            body: None,
            query: None,
        })
    }
}
