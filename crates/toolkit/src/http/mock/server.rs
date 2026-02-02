//! HTTP Mock 服务器实现
//!
//! 提供轻量级的 HTTP mock 服务器，用于测试 HTTP 客户端和响应处理。
//! 与集成测试的 `MockServer` 不同，此服务器不设置环境变量，专注于单元测试需求。
//!
//! ⚠️ **注意**：此文件仅在测试时编译，不会被打包到正式代码中。
//! 由 `mock/mod.rs` 中的 `#[cfg(test)] pub mod server;` 控制。

use mockito::Server;
use reqwest::blocking::Client;

use crate::http::HttpResponse;

/// HTTP Mock 服务器（用于单元测试）
///
/// 提供轻量级的 HTTP mock 服务器，用于测试 HTTP 客户端和响应处理。
/// 与集成测试的 `MockServer` 不同，此服务器不设置环境变量，专注于单元测试需求。
///
/// # 示例
///
/// ```rust,no_run
/// use crate::http::mock::HttpMockServer;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut mock_server = HttpMockServer::new();
///
/// // 创建 mock 响应
/// let _mock = mock_server
///     .mock("GET", "/test")
///     .with_status(200)
///     .with_body(r#"{"key": "value"}"#)
///     .create();
///
/// // 发送请求并获取响应
/// let response = mock_server.request("GET", "/test")?;
/// # Ok(())
/// # }
/// ```
pub struct HttpMockServer {
    server: Box<dyn std::ops::DerefMut<Target = Server>>,
    client: Client,
    base_url: String,
}

impl HttpMockServer {
    /// 创建新的 Mock 服务器
    ///
    /// # 返回
    ///
    /// 返回新的 `HttpMockServer` 实例。
    pub fn new() -> Self {
        let server = Server::new();
        let base_url = server.url();
        Self {
            server: Box::new(server),
            client: Client::new(),
            base_url,
        }
    }

    /// 获取服务器 URL
    ///
    /// # 返回
    ///
    /// 返回 mock 服务器的 base URL。
    pub fn url(&self) -> &str {
        &self.base_url
    }

    /// 创建 Mock 响应
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法（如 "GET", "POST"）
    /// * `path` - 请求路径
    ///
    /// # 返回
    ///
    /// 返回 `mockito::Mock` 构建器，可以链式调用设置响应。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// let mut mock_server = HttpMockServer::new();
    /// let _mock = mock_server
    ///     .mock("GET", "/api/data")
    ///     .with_status(200)
    ///     .with_body(r#"{"data": "test"}"#)
    ///     .create();
    /// ```
    pub fn mock(&mut self, method: &str, path: &str) -> mockito::Mock {
        self.server.as_mut().mock(method, path)
    }

    /// 发送 HTTP 请求
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `path` - 请求路径
    ///
    /// # 返回
    ///
    /// 返回 `reqwest::blocking::Response`。
    ///
    /// # 错误
    ///
    /// 如果请求失败，返回相应的错误。
    pub fn request(
        &self,
        method: &str,
        path: &str,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        match method {
            "GET" => self.client.get(&url).send(),
            "POST" => self.client.post(&url).send(),
            "PUT" => self.client.put(&url).send(),
            "DELETE" => self.client.delete(&url).send(),
            "PATCH" => self.client.patch(&url).send(),
            _ => panic!("Unsupported HTTP method: {}", method),
        }
    }

    /// 创建 HttpResponse 从 mock 响应
    ///
    /// 发送请求到 mock 服务器，并将响应转换为 `HttpResponse`。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `path` - 请求路径
    /// * `max_body_size` - 最大响应体大小（字节）
    ///
    /// # 返回
    ///
    /// 返回 `HttpResponse`。
    ///
    /// # 错误
    ///
    /// 如果请求失败或响应体过大，返回相应的错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use crate::http::mock::HttpMockServer;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut mock_server = HttpMockServer::new();
    /// let _mock = mock_server
    ///     .mock("GET", "/test")
    ///     .with_status(200)
    ///     .with_body(r#"{"key": "value"}"#)
    ///     .create();
    ///
    /// let response = mock_server.create_http_response("GET", "/test", 1024)?;
    /// assert!(response.is_success());
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_http_response(
        &self,
        method: &str,
        path: &str,
        max_body_size: usize,
    ) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        let response = self.request(method, path)?;
        Ok(HttpResponse::from_reqwest_response(
            response,
            max_body_size,
        )?)
    }
}

impl Default for HttpMockServer {
    fn default() -> Self {
        Self::new()
    }
}
