//! HTTP 测试工具
//!
//! 提供 HTTP Mock 测试的通用工具函数。

use mockito::Server;
use std::env;

/// Mock 服务器包装器
///
/// `Server::new()` 返回 `ServerGuard`，它实现了 `DerefMut<Target = Server>`
/// 我们直接存储 Server::new() 的返回值
pub struct MockServer {
    // Server::new() 返回 ServerGuard，它实现了 DerefMut
    // 我们使用 Box 来存储，避免类型问题
    pub server: Box<dyn std::ops::DerefMut<Target = Server>>,
    pub base_url: String,
}

impl MockServer {
    /// 创建新的 Mock 服务器
    pub fn new() -> Self {
        let server = Server::new();
        let base_url = server.url();
        // 将 ServerGuard 包装在 Box 中
        Self {
            server: Box::new(server),
            base_url,
        }
    }

    /// 设置 GitHub API Mock
    pub fn setup_github_base_url(&self) {
        env::set_var("GITHUB_API_URL", self.base_url.clone());
    }

    /// 设置 Jira API Mock
    pub fn setup_jira_base_url(&self) {
        env::set_var("JIRA_API_URL", self.base_url.clone());
    }

    /// 清理环境变量
    pub fn cleanup(&self) {
        env::remove_var("GITHUB_API_URL");
        env::remove_var("JIRA_API_URL");
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}
