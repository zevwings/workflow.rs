//! 测试数据工厂
//!
//! 提供统一的测试数据创建接口，使用构建器模式创建常用的测试数据。

use serde_json::{json, Value};

/// 测试数据工厂
///
/// 提供各种测试数据构建器的入口点。
///
/// # 示例
///
/// ```ignore
/// use http::testing::TestDataFactory;
///
/// let pr = TestDataFactory::github_pr()
///     .with_title("Add feature X")
///     .with_head("feature/x")
///     .build();
/// ```
pub struct TestDataFactory;

impl TestDataFactory {
    /// 创建 GitHub PR 构建器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let pr = TestDataFactory::github_pr()
    ///     .with_title("My PR")
    ///     .build();
    /// ```
    pub fn github_pr() -> GitHubPRBuilder {
        GitHubPRBuilder::default()
    }

    /// 创建 Jira Issue 构建器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let issue = TestDataFactory::jira_issue()
    ///     .with_summary("Bug fix")
    ///     .build();
    /// ```
    pub fn jira_issue() -> JiraIssueBuilder {
        JiraIssueBuilder::default()
    }

    /// 创建配置构建器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let config = TestDataFactory::config()
    ///     .with_github_token("token123")
    ///     .build();
    /// ```
    pub fn config() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

/// GitHub PR 构建器
///
/// 用于创建 GitHub Pull Request 测试数据。
#[derive(Default)]
pub struct GitHubPRBuilder {
    title: Option<String>,
    body: Option<String>,
    head: Option<String>,
    base: Option<String>,
    state: Option<String>,
    number: Option<u64>,
    user_login: Option<String>,
    user_id: Option<u64>,
}

impl GitHubPRBuilder {
    /// 设置 PR 标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置 PR 描述
    pub fn with_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// 设置源分支
    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.head = Some(head.into());
        self
    }

    /// 设置目标分支
    pub fn with_base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    /// 设置 PR 状态
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// 设置 PR 编号
    pub fn with_number(mut self, number: u64) -> Self {
        self.number = Some(number);
        self
    }

    /// 设置用户登录名
    pub fn with_user_login(mut self, login: impl Into<String>) -> Self {
        self.user_login = Some(login.into());
        self
    }

    /// 设置用户 ID
    pub fn with_user_id(mut self, id: u64) -> Self {
        self.user_id = Some(id);
        self
    }

    /// 构建 JSON 数据
    pub fn build(self) -> Value {
        json!({
            "title": self.title.unwrap_or_else(|| "Test PR".to_string()),
            "body": self.body.unwrap_or_else(|| "Test body".to_string()),
            "head": self.head.unwrap_or_else(|| "feature".to_string()),
            "base": self.base.unwrap_or_else(|| "main".to_string()),
            "state": self.state.unwrap_or_else(|| "open".to_string()),
            "number": self.number.unwrap_or(1),
            "user": {
                "login": self.user_login.unwrap_or_else(|| "testuser".to_string()),
                "id": self.user_id.unwrap_or(1)
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }
}

/// Jira Issue 构建器
///
/// 用于创建 Jira Issue 测试数据。
#[derive(Default)]
pub struct JiraIssueBuilder {
    key: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    issue_type: Option<String>,
    project: Option<String>,
    status: Option<String>,
}

impl JiraIssueBuilder {
    /// 设置 Issue Key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// 设置 Issue 标题
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// 设置 Issue 描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 设置 Issue 类型
    pub fn with_issue_type(mut self, issue_type: impl Into<String>) -> Self {
        self.issue_type = Some(issue_type.into());
        self
    }

    /// 设置项目 Key
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// 设置 Issue 状态
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// 构建 JSON 数据
    pub fn build(self) -> Value {
        json!({
            "key": self.key.unwrap_or_else(|| "TEST-1".to_string()),
            "fields": {
                "summary": self.summary.unwrap_or_else(|| "Test Issue".to_string()),
                "description": self.description.unwrap_or_else(|| "Test description".to_string()),
                "issuetype": {
                    "name": self.issue_type.unwrap_or_else(|| "Task".to_string())
                },
                "project": {
                    "key": self.project.unwrap_or_else(|| "TEST".to_string())
                },
                "status": {
                    "name": self.status.unwrap_or_else(|| "To Do".to_string())
                }
            }
        })
    }
}

/// 配置构建器
///
/// 用于创建测试配置数据。
#[derive(Default)]
pub struct ConfigBuilder {
    github_token: Option<String>,
    github_url: Option<String>,
    jira_url: Option<String>,
    jira_email: Option<String>,
    jira_token: Option<String>,
}

impl ConfigBuilder {
    /// 设置 GitHub Token
    pub fn with_github_token(mut self, token: impl Into<String>) -> Self {
        self.github_token = Some(token.into());
        self
    }

    /// 设置 GitHub URL
    pub fn with_github_url(mut self, url: impl Into<String>) -> Self {
        self.github_url = Some(url.into());
        self
    }

    /// 设置 Jira URL
    pub fn with_jira_url(mut self, url: impl Into<String>) -> Self {
        self.jira_url = Some(url.into());
        self
    }

    /// 设置 Jira Email
    pub fn with_jira_email(mut self, email: impl Into<String>) -> Self {
        self.jira_email = Some(email.into());
        self
    }

    /// 设置 Jira Token
    pub fn with_jira_token(mut self, token: impl Into<String>) -> Self {
        self.jira_token = Some(token.into());
        self
    }

    /// 构建 JSON 数据
    pub fn build(self) -> Value {
        json!({
            "github": {
                "token": self.github_token.unwrap_or_else(|| "test_token".to_string()),
                "url": self.github_url.unwrap_or_else(|| "https://api.github.com".to_string())
            },
            "jira": {
                "url": self.jira_url.unwrap_or_else(|| "https://test.atlassian.net".to_string()),
                "email": self.jira_email.unwrap_or_else(|| "test@example.com".to_string()),
                "token": self.jira_token.unwrap_or_else(|| "test_jira_token".to_string())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pr_builder_defaults() {
        let pr = TestDataFactory::github_pr().build();

        assert_eq!(pr["title"], "Test PR");
        assert_eq!(pr["body"], "Test body");
        assert_eq!(pr["head"], "feature");
        assert_eq!(pr["base"], "main");
        assert_eq!(pr["state"], "open");
        assert_eq!(pr["number"], 1);
        assert_eq!(pr["user"]["login"], "testuser");
    }

    #[test]
    fn test_github_pr_builder_custom() {
        let pr = TestDataFactory::github_pr()
            .with_title("My PR")
            .with_head("feature-branch")
            .with_number(42)
            .build();

        assert_eq!(pr["title"], "My PR");
        assert_eq!(pr["head"], "feature-branch");
        assert_eq!(pr["number"], 42);
        assert_eq!(pr["base"], "main"); // 默认值
    }

    #[test]
    fn test_jira_issue_builder_defaults() {
        let issue = TestDataFactory::jira_issue().build();

        assert_eq!(issue["key"], "TEST-1");
        assert_eq!(issue["fields"]["summary"], "Test Issue");
        assert_eq!(issue["fields"]["issuetype"]["name"], "Task");
        assert_eq!(issue["fields"]["project"]["key"], "TEST");
    }

    #[test]
    fn test_jira_issue_builder_custom() {
        let issue = TestDataFactory::jira_issue()
            .with_key("PROJ-123")
            .with_summary("My Issue")
            .with_issue_type("Bug")
            .build();

        assert_eq!(issue["key"], "PROJ-123");
        assert_eq!(issue["fields"]["summary"], "My Issue");
        assert_eq!(issue["fields"]["issuetype"]["name"], "Bug");
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = TestDataFactory::config().build();

        assert_eq!(config["github"]["token"], "test_token");
        assert_eq!(config["jira"]["url"], "https://test.atlassian.net");
    }

    #[test]
    fn test_config_builder_custom() {
        let config = TestDataFactory::config()
            .with_github_token("custom_token")
            .with_jira_url("https://custom.atlassian.net")
            .build();

        assert_eq!(config["github"]["token"], "custom_token");
        assert_eq!(config["jira"]["url"], "https://custom.atlassian.net");
    }
}
