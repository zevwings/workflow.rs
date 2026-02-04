//! GitHub API 响应和错误处理
//!
//! 提供 GitHub API 响应封装和错误类型定义

use serde::Deserialize;
use toolkit::{HttpError, Response};

/// HTTP 响应格式
///
/// 封装 HTTP 响应的状态码、状态文本、响应数据和 Headers。
/// 响应体延迟解析，通过方法（json, text 等）来解析。
#[derive(Debug)]
pub struct GitHubResponse {
    response: Response,
}

impl GitHubResponse {
    pub fn new(response: Response) -> Self {
        Self { response }
    }

    /// 解析为 JSON（便捷方法）
    ///
    /// 将响应体解析为 JSON 并反序列化为类型 `T`。
    ///
    /// # 类型参数
    ///
    /// * `T` - 目标类型，必须实现 `Deserialize` trait
    ///
    /// # 返回
    ///
    /// 返回解析后的数据。
    ///
    /// # 错误
    ///
    /// 如果 JSON 解析失败，返回相应的错误信息。
    pub fn json<T>(&self) -> Result<T, HttpError>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.response.json()
    }

    /// 解析为文本（便捷方法）
    ///
    /// 将响应体解析为 UTF-8 文本字符串。
    ///
    /// # 返回
    ///
    /// 返回响应体的文本内容。
    ///
    /// # 错误
    ///
    /// 如果读取响应体失败或不是有效的 UTF-8，返回相应的错误信息。
    pub fn text(&self) -> Result<&str, HttpError> {
        self.response.text()
    }

    pub fn status_code(&self) -> u16 {
        self.response.status
    }
}

/// GitHub 错误响应结构
#[derive(Debug, Deserialize)]
pub struct GitHubErrorResponse {
    pub message: String,
    pub errors: Option<Vec<GitHubErrorResource>>,
}

/// GitHub 错误详情
#[derive(Debug, Deserialize)]
pub struct GitHubErrorResource {
    pub resource: Option<String>,
    pub field: Option<String>,
    pub code: Option<String>,
}
