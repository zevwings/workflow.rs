//! HTTP 测试工具
//!
//! 提供 HTTP Mock 测试的通用工具函数。

use mockito::{Matcher, Mock, Server};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Mock 服务器包装器
///
/// 提供统一的 Mock 服务器管理接口，支持 GitHub 和 Jira API Mock。
/// 合并了原 `MockServerManager` 的功能，提供基础功能和高级封装。
///
/// `Server::new()` 返回 `ServerGuard`，它实现了 `DerefMut<Target = Server>`
/// 我们直接存储 Server::new() 的返回值
pub struct MockServer {
    // Server::new() 返回 ServerGuard，它实现了 DerefMut
    // 我们使用 Box 来存储，避免类型问题
    pub server: Box<dyn std::ops::DerefMut<Target = Server>>,
    pub base_url: String,
    /// 跟踪创建的 Mock 端点
    mocks: Vec<Mock>,
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
            mocks: Vec::new(),
        }
    }

    /// 获取 Mock 服务器引用（用于设置 Mock 端点）
    pub fn server(&mut self) -> &mut dyn std::ops::DerefMut<Target = Server> {
        self.server.as_mut()
    }

    /// 获取服务器基础 URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 设置 GitHub API Mock 环境
    pub fn setup_github_base_url(&self) {
        env::set_var("GITHUB_API_URL", self.base_url.clone());
    }

    /// 设置 Jira API Mock 环境
    pub fn setup_jira_base_url(&self) {
        env::set_var("JIRA_API_URL", self.base_url.clone());
    }

    /// 设置 GitHub API Mock 环境（别名，保持向后兼容）
    pub fn setup_github_api(&self) {
        self.setup_github_base_url();
    }

    /// 设置 Jira API Mock 环境（别名，保持向后兼容）
    pub fn setup_jira_api(&self) {
        self.setup_jira_base_url();
    }

    /// 创建 GitHub PR Mock 端点
    pub fn mock_github_pr(
        &mut self,
        method: &str,
        path: &str,
        response_body: &str,
        status: u16,
    ) -> &mut Self {
        let mock = self
            .server
            .as_mut()
            .mock(method, path)
            .match_header("authorization", Matcher::Regex(r"token .+".to_string()))
            .match_header("accept", "application/vnd.github.v3+json")
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        self.mocks.push(mock);
        self
    }

    /// 创建 Jira Issue Mock 端点
    pub fn mock_jira_issue(
        &mut self,
        method: &str,
        path: &str,
        response_body: &str,
        status: u16,
    ) -> &mut Self {
        let mock = self
            .server
            .as_mut()
            .mock(method, path)
            .match_header("authorization", Matcher::Regex(r"Basic .+".to_string()))
            .match_header("accept", "application/json")
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(response_body)
            .create();

        self.mocks.push(mock);
        self
    }

    /// 从文件加载 Mock 响应
    #[allow(dead_code)]
    pub fn mock_from_file(
        &mut self,
        method: &str,
        path: &str,
        file_path: &PathBuf,
        status: u16,
    ) -> &mut Self {
        let response_body = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read mock response file: {:?}", file_path));

        self.mock_github_pr(method, path, &response_body, status);
        self
    }

    /// 创建错误响应 Mock
    pub fn mock_error_response(
        &mut self,
        method: &str,
        path: &str,
        error_message: &str,
        status: u16,
    ) -> &mut Self {
        let error_body = json!({
            "message": error_message,
            "errors": []
        })
        .to_string();

        self.mock_github_pr(method, path, &error_body, status);
        self
    }

    /// 验证所有 Mock 是否被调用
    #[allow(dead_code)]
    pub fn assert_all_called(&self) {
        for mock in &self.mocks {
            mock.assert();
        }
    }

    /// 清理所有 Mock 和环境变量
    pub fn cleanup(&mut self) {
        self.mocks.clear();
        env::remove_var("GITHUB_API_URL");
        env::remove_var("JIRA_API_URL");
    }
}

/// GitHub API Mock 预设
impl MockServer {
    /// 设置 GitHub 创建 PR 成功响应
    pub fn setup_github_create_pr_success(
        &mut self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> &mut Self {
        let response_body = format!(
            r#"{{
            "number": {},
            "title": "Test PR",
            "html_url": "https://github.com/{}/{}/pull/{}",
            "state": "open"
        }}"#,
            pr_number, owner, repo, pr_number
        );

        self.mock_github_pr(
            "POST",
            &format!("/repos/{}/{}/pulls", owner, repo),
            &response_body,
            201,
        );
        self
    }

    /// 设置 GitHub 获取 PR 信息响应
    #[allow(dead_code)]
    pub fn setup_github_get_pr(
        &mut self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_data: &Value,
    ) -> &mut Self {
        let response_body = serde_json::to_string(pr_data).expect("operation should succeed");
        self.mock_github_pr(
            "GET",
            &format!("/repos/{}/{}/pulls/{}", owner, repo, pr_number),
            &response_body,
            200,
        );
        self
    }

    /// 设置 GitHub 错误响应
    #[allow(dead_code)]
    pub fn setup_github_error(&mut self, path: &str, status: u16, message: &str) -> &mut Self {
        self.mock_error_response("GET", path, message, status);
        self
    }
}

/// Jira API Mock 预设
impl MockServer {
    /// 设置 Jira 获取 Issue 成功响应
    pub fn setup_jira_get_issue_success(
        &mut self,
        issue_key: &str,
        issue_data: &Value,
    ) -> &mut Self {
        let response_body = serde_json::to_string(issue_data).expect("operation should succeed");
        self.mock_jira_issue(
            "GET",
            &format!("/rest/api/3/issue/{}", issue_key),
            &response_body,
            200,
        );
        self
    }

    /// 设置 Jira Issue 不存在响应
    #[allow(dead_code)]
    pub fn setup_jira_issue_not_found(&mut self, issue_key: &str) -> &mut Self {
        let error_body = json!({
            "errorMessages": [
                format!("Issue {} does not exist or you do not have permission to see it.", issue_key)
            ]
        })
        .to_string();

        self.mock_jira_issue(
            "GET",
            &format!("/rest/api/3/issue/{}", issue_key),
            &error_body,
            404,
        );
        self
    }

    /// 设置 Jira 搜索 Issues 响应
    #[allow(dead_code)]
    pub fn setup_jira_search_issues(&mut self, issues: &[Value]) -> &mut Self {
        let response_body = json!({
            "issues": issues,
            "total": issues.len()
        })
        .to_string();

        self.mock_jira_issue("POST", "/rest/api/3/search", &response_body, 200);
        self
    }

    /// 设置 Jira 获取当前用户（/myself）成功响应
    pub fn setup_jira_get_current_user_success(&mut self, user_data: &Value) -> &mut Self {
        let response_body = serde_json::to_string(user_data).expect("operation should succeed");
        self.mock_jira_issue("GET", "/rest/api/2/myself", &response_body, 200);
        self
    }

    /// 设置 Jira 获取当前用户失败响应
    pub fn setup_jira_get_current_user_error(
        &mut self,
        status: u16,
        error_message: &str,
    ) -> &mut Self {
        let error_body = json!({
            "errorMessages": [error_message]
        })
        .to_string();

        self.mock_jira_issue("GET", "/rest/api/2/myself", &error_body, status);
        self
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// 创建 Mock 服务器（公共函数）
///
/// 用于所有需要 Mock HTTP 服务器的测试。
/// 这个函数统一了 Mock 服务器的创建方式，避免在多个测试文件中重复定义。
///
/// # 返回
///
/// 返回新创建的 `MockServer` 实例
///
/// # 示例
///
/// ```rust
/// use crate::common::http_helpers::setup_mock_server;
///
/// #[test]
/// fn test_http_request_return_result() -> Result<()> {
///     let mut mock_server = setup_mock_server();
///     let url = format!("{}/test", mock_server.base_url);
///     // ...
///     Ok(())
/// }
/// ```
pub fn setup_mock_server() -> MockServer {
    MockServer::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_data_factory::TestDataFactory;

    /// 测试MockServer创建
    ///
    /// ## 测试目的
    /// 验证 `MockServer::new()` 能够成功创建Mock服务器，并生成有效的base URL。
    ///
    /// ## 测试场景
    /// 1. 创建MockServer实例
    /// 2. 获取base URL
    /// 3. 验证base URL不为空
    ///
    /// ## 预期结果
    /// - Mock服务器创建成功
    /// - base URL不为空
    #[test]
    fn test_mock_server_creation() {
        let server = MockServer::new();
        assert!(!server.base_url().is_empty());
    }

    /// 测试设置GitHub API环境变量
    ///
    /// ## 测试目的
    /// 验证 `MockServer::setup_github_api()` 方法能够设置GITHUB_API_URL环境变量。
    ///
    /// ## 测试场景
    /// 1. 创建MockServer
    /// 2. 调用setup_github_api设置环境变量
    /// 3. 验证环境变量已设置
    ///
    /// ## 预期结果
    /// - GITHUB_API_URL环境变量已设置
    #[test]
    fn test_setup_github_api() {
        let server = MockServer::new();
        server.setup_github_api();
        // 验证环境变量已设置
        assert!(env::var("GITHUB_API_URL").is_ok());
    }

    /// 测试设置Jira API环境变量
    ///
    /// ## 测试目的
    /// 验证 `MockServer::setup_jira_api()` 方法能够设置JIRA_API_URL环境变量。
    ///
    /// ## 测试场景
    /// 1. 创建MockServer
    /// 2. 调用setup_jira_api设置环境变量
    /// 3. 验证环境变量已设置
    ///
    /// ## 预期结果
    /// - JIRA_API_URL环境变量已设置
    #[test]
    fn test_setup_jira_api() {
        let server = MockServer::new();
        server.setup_jira_api();
        // 验证环境变量已设置
        assert!(env::var("JIRA_API_URL").is_ok());
    }

    /// 测试Mock GitHub创建PR端点
    ///
    /// ## 测试目的
    /// 验证 `MockServer::setup_github_create_pr_success()` 方法能够创建GitHub创建PR的Mock端点。
    ///
    /// ## 测试场景
    /// 1. 创建MockServer并设置GitHub API环境变量
    /// 2. 调用setup_github_create_pr_success创建Mock端点
    /// 3. 验证Mock端点已创建
    ///
    /// ## 预期结果
    /// - Mock端点创建成功
    /// - mocks列表长度为1
    #[test]
    fn test_mock_github_create_pr() {
        let mut server = MockServer::new();
        server.setup_github_api();
        server.setup_github_create_pr_success("owner", "repo", 123);

        // Mock 已创建
        assert_eq!(server.mocks.len(), 1);
    }

    /// 测试Mock Jira获取Issue端点
    ///
    /// ## 测试目的
    /// 验证 `MockServer::setup_jira_get_issue_success()` 方法能够创建Jira获取Issue的Mock端点。
    ///
    /// ## 测试场景
    /// 1. 使用TestDataFactory创建Issue数据
    /// 2. 创建MockServer并设置Jira API环境变量
    /// 3. 调用setup_jira_get_issue_success创建Mock端点
    /// 4. 验证Mock端点已创建
    ///
    /// ## 预期结果
    /// - Mock端点创建成功
    /// - mocks列表长度为1
    #[test]
    fn test_mock_jira_get_issue_return_result() -> color_eyre::Result<()> {
        let factory = TestDataFactory::new();
        let issue_data = factory.jira_issue().key("PROJ-123").build()?;

        let mut server = MockServer::new();
        server.setup_jira_api();
        server.setup_jira_get_issue_success("PROJ-123", &issue_data);

        assert_eq!(server.mocks.len(), 1);
        Ok(())
    }

    /// 测试Mock错误响应
    ///
    /// ## 测试目的
    /// 验证 `MockServer::mock_error_response()` 方法能够创建错误响应的Mock端点。
    ///
    /// ## 测试场景
    /// 1. 创建MockServer并设置GitHub API环境变量
    /// 2. 调用mock_error_response创建错误响应Mock（404状态码）
    /// 3. 验证Mock端点已创建
    ///
    /// ## 预期结果
    /// - Mock端点创建成功
    /// - mocks列表长度为1
    #[test]
    fn test_mock_error_response() {
        let mut server = MockServer::new();
        server.setup_github_api();
        server.mock_error_response("GET", "/test", "Not Found", 404);

        assert_eq!(server.mocks.len(), 1);
    }
}
