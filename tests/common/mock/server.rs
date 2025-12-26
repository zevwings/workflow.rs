//! HTTP 测试工具
//!
//! 提供 HTTP Mock 测试的通用工具函数。

use mockito::{Matcher, Mock, Server};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Mock 期望信息
///
/// 记录每个 Mock 端点的期望信息，用于在验证失败时提供详细的错误信息。
#[derive(Debug, Clone)]
pub struct MockExpectation {
    /// HTTP 方法
    pub method: String,
    /// 请求路径
    pub path: String,
    /// 期望的状态码
    pub status: u16,
    /// Mock 索引（用于关联实际的 Mock 对象）
    mock_index: usize,
}

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
    /// 跟踪 Mock 期望信息（用于错误信息增强）
    expectations: Vec<MockExpectation>,
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
            expectations: Vec::new(),
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
    ///
    /// 同时设置 `GITHUB_API_URL`（用于 API 调用）和 `GITHUB_BASE_URL`（用于网络检查）
    pub fn setup_github_base_url(&self) {
        env::set_var("GITHUB_API_URL", self.base_url.clone());
        env::set_var("GITHUB_BASE_URL", self.base_url.clone());
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
        let mock_index = self.mocks.len();
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
        self.expectations.push(MockExpectation {
            method: method.to_string(),
            path: path.to_string(),
            status,
            mock_index,
        });
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
        let mock_index = self.mocks.len();
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
        self.expectations.push(MockExpectation {
            method: method.to_string(),
            path: path.to_string(),
            status,
            mock_index,
        });
        self
    }

    /// 从文件加载 Mock 响应（通用方法）
    ///
    /// 从文件加载响应体，创建通用的 Mock 端点。
    /// 适用于需要从文件加载复杂响应数据的场景。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法（GET, POST, PUT, DELETE 等）
    /// * `path` - 请求路径
    /// * `file_path` - 响应文件路径（相对于项目根目录或绝对路径）
    /// * `status` - HTTP 状态码
    ///
    /// # 返回
    ///
    /// 返回 `&mut Self` 以支持链式调用
    ///
    /// # 示例
    ///
    /// ```rust
    /// use std::path::PathBuf;
    ///
    /// let response_file = PathBuf::from("tests/fixtures/mock_responses/jira/issue.json");
    /// mock_server.mock_from_file("GET", "/rest/api/3/issue/PROJ-123", &response_file, 200);
    /// ```
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

        // 创建通用的 Mock 端点（不限制为 GitHub 或 Jira）
        let mock_index = self.mocks.len();
        let mock = self
            .server
            .as_mut()
            .mock(method, path)
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(&response_body)
            .create();

        self.mocks.push(mock);
        self.expectations.push(MockExpectation {
            method: method.to_string(),
            path: path.to_string(),
            status,
            mock_index,
        });
        self
    }

    /// 从文件加载 GitHub PR Mock 响应（便捷方法）
    ///
    /// 从文件加载响应体，创建 GitHub PR Mock 端点（自动匹配 GitHub API 请求头）。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `path` - 请求路径
    /// * `file_path` - 响应文件路径
    /// * `status` - HTTP 状态码
    ///
    /// # 示例
    ///
    /// ```rust
    /// let response_file = PathBuf::from("tests/fixtures/mock_responses/github/pr.json");
    /// mock_server.mock_github_pr_from_file("GET", "/repos/owner/repo/pulls/123", &response_file, 200);
    /// ```
    #[allow(dead_code)]
    pub fn mock_github_pr_from_file(
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

    /// 从文件加载 Jira Issue Mock 响应（便捷方法）
    ///
    /// 从文件加载响应体，创建 Jira Issue Mock 端点（自动匹配 Jira API 请求头）。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `path` - 请求路径
    /// * `file_path` - 响应文件路径
    /// * `status` - HTTP 状态码
    ///
    /// # 示例
    ///
    /// ```rust
    /// let response_file = PathBuf::from("tests/fixtures/mock_responses/jira/issue.json");
    /// mock_server.mock_jira_issue_from_file("GET", "/rest/api/3/issue/PROJ-123", &response_file, 200);
    /// ```
    #[allow(dead_code)]
    pub fn mock_jira_issue_from_file(
        &mut self,
        method: &str,
        path: &str,
        file_path: &PathBuf,
        status: u16,
    ) -> &mut Self {
        let response_body = fs::read_to_string(file_path)
            .unwrap_or_else(|_| panic!("Failed to read mock response file: {:?}", file_path));

        self.mock_jira_issue(method, path, &response_body, status);
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

    /// 使用模板创建 Mock 端点
    ///
    /// 支持变量替换的模板系统，模板中使用 `{{variable}}` 格式的占位符。
    ///
    /// # 参数
    ///
    /// * `method` - HTTP 方法
    /// * `path` - 请求路径（支持路径参数，如 `/repos/{owner}/{repo}/pulls/{pr_number}`）
    /// * `template` - 响应模板字符串，支持 `{{variable}}` 格式的变量占位符
    /// * `variables` - 变量映射表
    /// * `status` - HTTP 状态码
    ///
    /// # 示例
    ///
    /// ```rust
    /// use std::collections::HashMap;
    ///
    /// let mut vars = HashMap::new();
    /// vars.insert("pr_number".to_string(), "123".to_string());
    /// vars.insert("owner".to_string(), "test-owner".to_string());
    /// mock_server.mock_with_template(
    ///     "GET",
    ///     "/repos/{owner}/repo/pulls/{pr_number}",
    ///     r#"{"number": {{pr_number}}, "owner": "{{owner}}"}"#,
    ///     vars,
    ///     200,
    /// );
    /// ```
    pub fn mock_with_template(
        &mut self,
        method: &str,
        path: &str,
        template: &str,
        variables: HashMap<String, String>,
        status: u16,
    ) -> &mut Self {
        // 替换模板中的变量
        let mut response_body = template.to_string();
        for (key, value) in &variables {
            let placeholder = format!("{{{{{}}}}}", key);
            response_body = response_body.replace(&placeholder, value);
        }

        // 创建通用的 Mock 端点
        let mock_index = self.mocks.len();
        let mock = self
            .server
            .as_mut()
            .mock(method, path)
            .with_status(status as usize)
            .with_header("content-type", "application/json")
            .with_body(&response_body)
            .create();

        self.mocks.push(mock);
        self.expectations.push(MockExpectation {
            method: method.to_string(),
            path: path.to_string(),
            status,
            mock_index,
        });
        self
    }

    /// 验证所有 Mock 是否被调用
    ///
    /// 如果验证失败，会输出详细的错误信息，包括每个未调用的 Mock 的期望信息。
    ///
    /// # 错误信息增强
    ///
    /// 当 Mock 验证失败时，会输出以下信息：
    /// - Mock 索引
    /// - 期望的 HTTP 方法
    /// - 期望的请求路径
    /// - 期望的响应状态码
    ///
    /// # 示例
    ///
    /// ```rust
    /// let mut mock_server = MockServer::new();
    /// mock_server.setup_github_api();
    /// mock_server.setup_github_create_pr_success("owner", "repo", 123);
    ///
    /// // 执行测试...
    ///
    /// // 验证所有 Mock 被调用（如果失败会输出详细错误信息）
    /// mock_server.assert_all_called();
    /// ```
    #[allow(dead_code)]
    pub fn assert_all_called(&self) {
        // 先输出所有 Mock 的期望信息，这样在 assert() 失败时也能看到
        if !self.expectations.is_empty() {
            eprintln!("\n📋 Mock 期望信息 (共 {} 个):", self.expectations.len());
            for (idx, exp) in self.expectations.iter().enumerate() {
                eprintln!(
                    "   Mock #{}: {} {} -> 状态码 {}",
                    idx + 1,
                    exp.method,
                    exp.path,
                    exp.status
                );
            }
            eprintln!("");
        }

        // 验证所有 Mock
        for (index, mock) in self.mocks.iter().enumerate() {
            if let Some(expectation) = self.expectations.iter().find(|e| e.mock_index == index) {
                // 在验证前输出当前 Mock 信息，这样如果失败可以看到是哪个 Mock
                eprintln!(
                    "验证 Mock #{}: {} {}",
                    index + 1,
                    expectation.method,
                    expectation.path
                );
            }
            mock.assert();
        }
    }

    /// 清理所有 Mock 和环境变量
    pub fn cleanup(&mut self) {
        self.mocks.clear();
        self.expectations.clear();
        env::remove_var("GITHUB_API_URL");
        env::remove_var("JIRA_API_URL");
    }

    /// 重置 Mock 服务器
    ///
    /// 清除所有 Mock 端点、期望信息和请求历史，但保留服务器实例。
    /// 用于在同一个测试中重新配置 Mock。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let mut mock_server = MockServer::new();
    /// mock_server.setup_github_api();
    /// mock_server.mock_github_pr("GET", "/test", r#"{"ok": true}"#, 200);
    ///
    /// // 执行测试...
    ///
    /// // 重置并重新配置
    /// mock_server.reset();
    /// mock_server.mock_github_pr("POST", "/test", r#"{"ok": false}"#, 400);
    /// ```
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.mocks.clear();
        self.expectations.clear();
        // 注意：不清理环境变量，因为可能还需要使用
    }

    /// 验证所有 Mock 端点都被调用（返回 Result）
    ///
    /// 与 `assert_all_called()` 类似，但返回 `Result` 而不是 panic。
    /// 适用于需要优雅处理验证失败的场景。
    ///
    /// # 返回
    ///
    /// 如果所有 Mock 都被调用，返回 `Ok(())`；否则返回包含详细错误信息的 `Err`。
    ///
    /// # 示例
    ///
    /// ```rust
    /// match mock_server.verify_all_called() {
    ///     Ok(_) => println!("All mocks called"),
    ///     Err(e) => eprintln!("Some mocks not called: {}", e),
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn verify_all_called(&self) -> color_eyre::Result<()> {
        use color_eyre::eyre::Context;

        let mut errors = Vec::new();

        // 先输出所有 Mock 的期望信息
        if !self.expectations.is_empty() {
            eprintln!("\n📋 Mock 期望信息 (共 {} 个):", self.expectations.len());
            for (idx, exp) in self.expectations.iter().enumerate() {
                eprintln!(
                    "   Mock #{}: {} {} -> 状态码 {}",
                    idx + 1,
                    exp.method,
                    exp.path,
                    exp.status
                );
            }
            eprintln!("");
        }

        // 尝试验证所有 Mock，收集错误
        for (index, mock) in self.mocks.iter().enumerate() {
            if let Some(expectation) = self.expectations.iter().find(|e| e.mock_index == index) {
                // 使用 catch_unwind 捕获 assert() 的 panic
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    mock.assert();
                }));

                if result.is_err() {
                    errors.push(format!(
                        "Mock #{}: {} {} -> 状态码 {}",
                        index + 1,
                        expectation.method,
                        expectation.path,
                        expectation.status
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(color_eyre::eyre::eyre!(
                "{} mock(s) were not called",
                errors.len()
            ))
            .context(format!("Failed mocks:\n{}", errors.join("\n")))
        }
    }

    /// 获取 Mock 期望信息（用于调试）
    ///
    /// 返回所有 Mock 端点的期望信息，包括方法、路径和状态码。
    ///
    /// # 示例
    ///
    /// ```rust
    /// let expectations = mock_server.get_expectations();
    /// for exp in expectations {
    ///     println!("期望: {} {} -> {}", exp.method, exp.path, exp.status);
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn get_expectations(&self) -> &[MockExpectation] {
        &self.expectations
    }

    /// 打印所有 Mock 期望信息（用于调试）
    ///
    /// 在测试失败时调用此方法，可以查看所有 Mock 的期望信息，帮助调试。
    ///
    /// # 示例
    ///
    /// ```rust
    /// // 在测试失败时调用
    /// mock_server.print_expectations();
    /// ```
    #[allow(dead_code)]
    pub fn print_expectations(&self) {
        if self.expectations.is_empty() {
            eprintln!("📋 没有配置 Mock 期望");
            return;
        }

        eprintln!("\n📋 Mock 期望信息 (共 {} 个):", self.expectations.len());
        for (idx, exp) in self.expectations.iter().enumerate() {
            eprintln!(
                "   Mock #{}: {} {} -> 状态码 {}",
                idx + 1,
                exp.method,
                exp.path,
                exp.status
            );
        }
        eprintln!("");
    }
}

/// GitHub API Mock 预设
impl MockServer {
    /// 设置 GitHub 创建 PR 成功响应
    ///
    /// 使用 TestDataFactory 生成 PR 数据，提供类型安全的数据生成。
    pub fn setup_github_create_pr_success(
        &mut self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> &mut Self {
        use crate::common::test_data::TestDataFactory;
        use serde_json::json;

        // 使用 TestDataFactory 生成 PR 数据
        let factory = TestDataFactory::new();
        let mut pr_data = factory
            .github_pr()
            .number(pr_number)
            .title("Test PR")
            .state("open")
            .build()
            .unwrap_or_else(|_| json!({"number": pr_number}));

        // 设置 html_url
        pr_data["html_url"] = json!(format!(
            "https://github.com/{}/{}/pull/{}",
            owner, repo, pr_number
        ));

        let response_body = serde_json::to_string(&pr_data).unwrap_or_else(|_| {
            format!(
                r#"{{"number": {}, "html_url": "https://github.com/{}/{}/pull/{}"}}"#,
                pr_number, owner, repo, pr_number
            )
        });

        self.mock_github_pr(
            "POST",
            &format!("/repos/{}/{}/pulls", owner, repo),
            &response_body,
            201,
        );
        self
    }

    /// 设置 GitHub 获取 PR 信息响应
    ///
    /// 如果提供了 `pr_data`，使用提供的数据；否则使用 TestDataFactory 生成默认数据。
    #[allow(dead_code)]
    pub fn setup_github_get_pr(
        &mut self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        pr_data: Option<&Value>,
    ) -> &mut Self {
        use crate::common::test_data::TestDataFactory;
        use serde_json::json;

        let response_body = if let Some(data) = pr_data {
            serde_json::to_string(data)
                .unwrap_or_else(|e| panic!("operation should succeed: {}", e))
        } else {
            // 使用 TestDataFactory 生成默认 PR 数据
            let factory = TestDataFactory::new();
            let mut pr_data = factory
                .github_pr()
                .number(pr_number)
                .build()
                .unwrap_or_else(|_| json!({"number": pr_number}));

            pr_data["html_url"] = json!(format!(
                "https://github.com/{}/{}/pull/{}",
                owner, repo, pr_number
            ));
            serde_json::to_string(&pr_data)
                .unwrap_or_else(|e| panic!("operation should succeed: {}", e))
        };

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
    ///
    /// 如果提供了 `issue_data`，使用提供的数据；否则使用 TestDataFactory 生成默认数据。
    pub fn setup_jira_get_issue_success(
        &mut self,
        issue_key: &str,
        issue_data: Option<&Value>,
    ) -> &mut Self {
        use crate::common::test_data::TestDataFactory;

        let response_body = if let Some(data) = issue_data {
            serde_json::to_string(data)
                .unwrap_or_else(|e| panic!("operation should succeed: {}", e))
        } else {
            // 使用 TestDataFactory 生成默认 Issue 数据
            let factory = TestDataFactory::new();
            let issue_data = factory
                .jira_issue()
                .key(issue_key)
                .build()
                .unwrap_or_else(|_| serde_json::json!({"key": issue_key}));

            serde_json::to_string(&issue_data)
                .unwrap_or_else(|e| panic!("operation should succeed: {}", e))
        };

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
        let response_body = serde_json::to_string(user_data)
            .unwrap_or_else(|e| panic!("operation should succeed: {}", e));
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
/// fn test_http_request_return_ok() -> Result<()> {
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
    use crate::common::test_data::factory::TestDataFactory;

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
    fn test_mock_jira_get_issue_return_ok() -> color_eyre::Result<()> {
        let factory = TestDataFactory::new();
        let issue_data = factory.jira_issue().key("PROJ-123").build()?;

        let mut server = MockServer::new();
        server.setup_jira_api();
        server.setup_jira_get_issue_success("PROJ-123", Some(&issue_data));

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
