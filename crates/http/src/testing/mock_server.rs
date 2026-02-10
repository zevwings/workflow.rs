//! Mock 服务器管理器
//!
//! 提供统一的 Mock HTTP 服务器管理接口，简化测试中的 Mock 服务器配置。

use mockito::{Matcher, Mock, ServerGuard};
use serde_json::Value;
use std::collections::HashMap;

/// 简单的 Mock HTTP 服务器
///
/// 对 `mockito::ServerGuard` 的轻量封装，用于简单的测试场景。
///
/// # 示例
///
/// ```ignore
/// use http::testing::MockServer;
///
/// let mut server = MockServer::new();
/// let _mock = server.mock("GET", "/test")
///     .with_status(200)
///     .with_body("test")
///     .create();
///
/// let url = server.url();
/// ```
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
    pub fn mock_with_matcher(&mut self, method: &str, matcher: Matcher) -> Mock {
        self.server.mock(method, matcher)
    }
}

impl Default for MockServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock 服务器管理器
///
/// 管理多个 Mock 服务器实例，提供预定义的 Mock 场景。
///
/// # 使用场景
///
/// - 多服务测试：需要同时 mock GitHub 和 Jira API
/// - 预定义场景：使用内置的 `setup_*` 方法快速配置常见场景
/// - 错误场景测试：模拟认证失败、超时等错误情况
///
/// # 示例
///
/// ## 基础使用
///
/// ```ignore
/// use http::testing::{MockServerManager, TestDataFactory};
///
/// let mut manager = MockServerManager::new();
///
/// // 设置 GitHub PR 列表 Mock
/// let pr = TestDataFactory::github_pr().build();
/// let _mock = manager.setup_github_pr_list(vec![pr]);
///
/// // 获取服务器 URL
/// let url = manager.url("github").unwrap();
/// ```
///
/// ## 错误场景测试
///
/// ```ignore
/// let mut manager = MockServerManager::new();
///
/// // 模拟认证失败
/// let _mock = manager.setup_auth_failure("github");
///
/// // 模拟超时
/// let _mock = manager.setup_timeout_scenario("jira");
/// ```
pub struct MockServerManager {
    servers: HashMap<String, ServerGuard>,
}

impl MockServerManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    /// 获取或创建 GitHub Mock 服务器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let server = manager.github();
    /// ```
    pub fn github(&mut self) -> &mut ServerGuard {
        self.servers
            .entry("github".to_string())
            .or_insert_with(|| mockito::Server::new())
    }

    /// 获取或创建 Jira Mock 服务器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let server = manager.jira();
    /// ```
    pub fn jira(&mut self) -> &mut ServerGuard {
        self.servers.entry("jira".to_string()).or_insert_with(|| mockito::Server::new())
    }

    /// 获取服务器 URL
    ///
    /// # 参数
    ///
    /// * `name` - 服务器名称（"github" 或 "jira"）
    ///
    /// # 返回
    ///
    /// 返回服务器 URL，如果服务器不存在则返回 None。
    pub fn url(&self, name: &str) -> Option<String> {
        self.servers.get(name).map(|s| s.url())
    }

    /// 设置 GitHub PR 列表 Mock
    ///
    /// # 参数
    ///
    /// * `prs` - PR 数据列表
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let pr = TestDataFactory::github_pr().build();
    /// let _mock = manager.setup_github_pr_list(vec![pr]);
    /// ```
    pub fn setup_github_pr_list(&mut self, prs: Vec<Value>) -> Mock {
        let body = serde_json::json!(prs).to_string();
        self.github()
            .mock("GET", "/repos/owner/repo/pulls")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 GitHub PR 创建 Mock
    ///
    /// # 参数
    ///
    /// * `pr` - 创建成功后返回的 PR 数据
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let pr = TestDataFactory::github_pr().build();
    /// let _mock = manager.setup_github_pr_create(pr);
    /// ```
    pub fn setup_github_pr_create(&mut self, pr: Value) -> Mock {
        let body = pr.to_string();
        self.github()
            .mock("POST", "/repos/owner/repo/pulls")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 GitHub PR 获取 Mock
    ///
    /// # 参数
    ///
    /// * `number` - PR 编号
    /// * `pr` - PR 数据
    pub fn setup_github_pr_get(&mut self, number: u64, pr: Value) -> Mock {
        let body = pr.to_string();
        let path = format!("/repos/owner/repo/pulls/{}", number);
        self.github()
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 Jira Issue 创建 Mock
    ///
    /// # 参数
    ///
    /// * `issue` - 创建成功后返回的 Issue 数据
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let issue = TestDataFactory::jira_issue().build();
    /// let _mock = manager.setup_jira_issue_create(issue);
    /// ```
    pub fn setup_jira_issue_create(&mut self, issue: Value) -> Mock {
        let body = issue.to_string();
        self.jira()
            .mock("POST", "/rest/api/2/issue")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置 Jira Issue 获取 Mock
    ///
    /// # 参数
    ///
    /// * `key` - Issue Key
    /// * `issue` - Issue 数据
    pub fn setup_jira_issue_get(&mut self, key: &str, issue: Value) -> Mock {
        let body = issue.to_string();
        let path = format!("/rest/api/2/issue/{}", key);
        self.jira()
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置错误响应 Mock
    ///
    /// # 参数
    ///
    /// * `service` - 服务名称（"github" 或 "jira"）
    /// * `status` - HTTP 状态码
    /// * `message` - 错误消息
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let _mock = manager.setup_error_response("github", 401, "Bad credentials");
    /// ```
    pub fn setup_error_response(&mut self, service: &str, status: usize, message: &str) -> Mock {
        let body = serde_json::json!({
            "message": message
        })
        .to_string();

        let server = match service {
            "github" => self.github(),
            "jira" => self.jira(),
            _ => panic!("Unknown service: {}", service),
        };

        server
            .mock("GET", Matcher::Any)
            .with_status(status)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create()
    }

    /// 设置认证失败场景
    ///
    /// # 参数
    ///
    /// * `service` - 服务名称（"github" 或 "jira"）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let mut manager = MockServerManager::new();
    /// let _mock = manager.setup_auth_failure("github");
    /// ```
    pub fn setup_auth_failure(&mut self, service: &str) -> Mock {
        match service {
            "github" => self.setup_error_response("github", 401, "Bad credentials"),
            "jira" => self.setup_error_response(
                "jira",
                401,
                "Client must be authenticated to access this resource.",
            ),
            _ => panic!("Unknown service: {}", service),
        }
    }

    /// 设置网络超时场景（返回 504 Gateway Timeout）
    ///
    /// # 参数
    ///
    /// * `service` - 服务名称
    ///
    /// # 注意
    ///
    /// 由于 mockito 1.x 不支持延迟响应，我们通过返回 504 状态码来模拟超时。
    pub fn setup_timeout_scenario(&mut self, service: &str) -> Mock {
        self.setup_error_response(service, 504, "Gateway Timeout")
    }
}

impl Default for MockServerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::TestDataFactory;

    #[test]
    fn test_mock_server_manager_new() {
        let manager = MockServerManager::new();
        assert!(manager.servers.is_empty());
    }

    #[test]
    fn test_github_server() {
        let mut manager = MockServerManager::new();
        let server = manager.github();
        let url = server.url();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_jira_server() {
        let mut manager = MockServerManager::new();
        let server = manager.jira();
        let url = server.url();
        assert!(url.starts_with("http://"));
    }

    #[test]
    fn test_url() {
        let mut manager = MockServerManager::new();
        assert!(manager.url("github").is_none());

        let _ = manager.github();
        assert!(manager.url("github").is_some());
    }

    #[test]
    fn test_setup_github_pr_list() {
        let mut manager = MockServerManager::new();
        let pr = TestDataFactory::github_pr().build();
        let _mock = manager.setup_github_pr_list(vec![pr]);

        assert!(manager.url("github").is_some());
    }

    #[test]
    fn test_setup_github_pr_create() {
        let mut manager = MockServerManager::new();
        let pr = TestDataFactory::github_pr().build();
        let _mock = manager.setup_github_pr_create(pr);

        assert!(manager.url("github").is_some());
    }

    #[test]
    fn test_setup_jira_issue_create() {
        let mut manager = MockServerManager::new();
        let issue = TestDataFactory::jira_issue().build();
        let _mock = manager.setup_jira_issue_create(issue);

        assert!(manager.url("jira").is_some());
    }

    #[test]
    fn test_setup_error_response() {
        let mut manager = MockServerManager::new();
        let _mock = manager.setup_error_response("github", 404, "Not found");

        assert!(manager.url("github").is_some());
    }

    #[test]
    fn test_setup_auth_failure() {
        let mut manager = MockServerManager::new();
        let _mock = manager.setup_auth_failure("github");

        assert!(manager.url("github").is_some());
    }

    #[test]
    fn test_setup_timeout_scenario() {
        let mut manager = MockServerManager::new();
        let _mock = manager.setup_timeout_scenario("github");

        assert!(manager.url("github").is_some());
    }
}
