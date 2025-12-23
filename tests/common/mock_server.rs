//! Mock 服务器管理器
//!
//! 提供统一的 Mock 服务器管理功能，简化 HTTP API Mock 测试的设置和维护。
//!
//! ## 使用示例
//!
//! ```rust
//! use tests::common::mock_server::MockServerManager;
//!
//! let mut manager = MockServerManager::new();
//! manager.setup_github_api();
//! // 使用 manager.server() 设置 Mock 端点
//! ```

use crate::common::http_helpers::MockServer;
use mockito::{Matcher, Mock};
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Mock 服务器管理器
///
/// 提供统一的 Mock 服务器管理接口，支持 GitHub 和 Jira API Mock。
pub struct MockServerManager {
    server: MockServer,
    mocks: Vec<Mock>,
}

impl MockServerManager {
    /// 创建新的 Mock 服务器管理器
    pub fn new() -> Self {
        Self {
            server: MockServer::new(),
            mocks: Vec::new(),
        }
    }

    /// 获取 Mock 服务器引用（用于设置 Mock 端点）
    pub fn server(&mut self) -> &mut dyn std::ops::DerefMut<Target = mockito::Server> {
        self.server.server.as_mut()
    }

    /// 获取服务器基础 URL
    pub fn base_url(&self) -> &str {
        &self.server.base_url
    }

    /// 设置 GitHub API Mock 环境
    pub fn setup_github_api(&self) {
        self.server.setup_github_base_url();
    }

    /// 设置 Jira API Mock 环境
    pub fn setup_jira_api(&self) {
        self.server.setup_jira_base_url();
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
    pub fn assert_all_called(&self) {
        for mock in &self.mocks {
            mock.assert();
        }
    }

    /// 清理所有 Mock
    pub fn cleanup(&mut self) {
        self.mocks.clear();
        self.server.cleanup();
    }
}

impl Drop for MockServerManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// GitHub API Mock 预设
impl MockServerManager {
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
    pub fn setup_github_get_pr(
        &mut self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_data: &Value,
    ) -> &mut Self {
        let response_body = serde_json::to_string(pr_data).unwrap();
        self.mock_github_pr(
            "GET",
            &format!("/repos/{}/{}/pulls/{}", owner, repo, pr_number),
            &response_body,
            200,
        );
        self
    }

    /// 设置 GitHub 错误响应
    pub fn setup_github_error(&mut self, path: &str, status: u16, message: &str) -> &mut Self {
        self.mock_error_response("GET", path, message, status);
        self
    }
}

/// Jira API Mock 预设
impl MockServerManager {
    /// 设置 Jira 获取 Issue 成功响应
    pub fn setup_jira_get_issue_success(
        &mut self,
        issue_key: &str,
        issue_data: &Value,
    ) -> &mut Self {
        let response_body = serde_json::to_string(issue_data).unwrap();
        self.mock_jira_issue(
            "GET",
            &format!("/rest/api/3/issue/{}", issue_key),
            &response_body,
            200,
        );
        self
    }

    /// 设置 Jira Issue 不存在响应
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
        let response_body = serde_json::to_string(user_data).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_data_factory::TestDataFactory;

    #[test]
    fn test_mock_server_manager_creation() {
        let manager = MockServerManager::new();
        assert!(!manager.base_url().is_empty());
    }

    #[test]
    fn test_setup_github_api() {
        let manager = MockServerManager::new();
        manager.setup_github_api();
        // 验证环境变量已设置
        assert!(env::var("GITHUB_API_URL").is_ok());
    }

    #[test]
    fn test_setup_jira_api() {
        let manager = MockServerManager::new();
        manager.setup_jira_api();
        // 验证环境变量已设置
        assert!(env::var("JIRA_API_URL").is_ok());
    }

    #[test]
    fn test_mock_github_create_pr() {
        let mut manager = MockServerManager::new();
        manager.setup_github_api();
        manager.setup_github_create_pr_success("owner", "repo", 123);

        // Mock 已创建，可以用于测试
        assert_eq!(manager.mocks.len(), 1);
    }

    #[test]
    fn test_mock_jira_get_issue() {
        let factory = TestDataFactory::new();
        let issue_data = factory.jira_issue().key("PROJ-123").build();

        let mut manager = MockServerManager::new();
        manager.setup_jira_api();
        manager.setup_jira_get_issue_success("PROJ-123", &issue_data);

        assert_eq!(manager.mocks.len(), 1);
    }

    #[test]
    fn test_mock_error_response() {
        let mut manager = MockServerManager::new();
        manager.setup_github_api();
        manager.mock_error_response("GET", "/test", "Not Found", 404);

        assert_eq!(manager.mocks.len(), 1);
    }
}
