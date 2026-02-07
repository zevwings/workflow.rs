//! Mock HTTP 服务器

use mockito::{Matcher, Mock, ServerGuard};

/// Mock HTTP 服务器
///
/// 用于测试 HTTP 请求。
pub struct MockServer {
    server: ServerGuard,
}

impl MockServer {
    /// 创建新的 Mock 服务器
    pub fn new() -> Self {
        Self {
            server: mockito::Server::new(),
        }
    }

    /// 获取服务器 URL
    pub fn url(&self) -> String {
        self.server.url()
    }

    /// 创建 Mock
    pub fn mock(&mut self, method: &str, path: &str) -> Mock {
        self.server.mock(method, path)
    }

    /// 创建带匹配器的 Mock
    #[allow(dead_code)]
    pub fn mock_with_matcher(&mut self, method: &str, matcher: Matcher) -> Mock {
        self.server.mock(method, matcher)
    }
}

impl Default for MockServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_url() {
        let server = MockServer::new();
        let url = server.url();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_mock_server_mock() {
        let mut server = MockServer::new();
        let _mock = server.mock("GET", "/test").with_status(200).with_body("test").create();
    }
}
