//! Codeup API 客户端

use serde_json::Value;

use crate::{
    codeup::{CodeupClientError, CodeupResponse},
    http::HttpMethod,
};

pub struct CodeupRequest {
    pub path: String,
    pub method: HttpMethod,
    pub body: Option<Value>,
    pub query: Option<Value>,
}

pub trait CodeupClient: Send + Sync {
    /// 执行 Codeup API 请求（核心方法）
    fn execute(&self, request: CodeupRequest) -> Result<CodeupResponse, CodeupClientError>;

    /// GET 请求
    fn get(&self, path: &str) -> Result<CodeupResponse, CodeupClientError> {
        self.execute(CodeupRequest {
            path: path.to_string(),
            method: HttpMethod::GET,
            body: None,
            query: None,
        })
    }

    /// POST 请求
    fn post(&self, path: &str, body: &Value) -> Result<CodeupResponse, CodeupClientError> {
        self.execute(CodeupRequest {
            path: path.to_string(),
            method: HttpMethod::POST,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// PUT 请求
    fn put(&self, path: &str, body: &Value) -> Result<CodeupResponse, CodeupClientError> {
        self.execute(CodeupRequest {
            path: path.to_string(),
            method: HttpMethod::PUT,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// PATCH 请求
    fn patch(&self, path: &str, body: &Value) -> Result<CodeupResponse, CodeupClientError> {
        self.execute(CodeupRequest {
            path: path.to_string(),
            method: HttpMethod::PATCH,
            body: Some(body.clone()),
            query: None,
        })
    }

    /// DELETE 请求
    fn delete(&self, path: &str) -> Result<CodeupResponse, CodeupClientError> {
        self.execute(CodeupRequest {
            path: path.to_string(),
            method: HttpMethod::DELETE,
            body: None,
            query: None,
        })
    }
}
