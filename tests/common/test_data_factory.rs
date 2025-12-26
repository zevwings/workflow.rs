//! 测试数据工厂
//!
//! 提供统一的测试数据生成和管理功能，简化测试数据的创建和维护。
//!
//! ## 使用示例
//!
//! ```rust
//! use tests::common::test_data_factory::TestDataFactory;
//!
//! let factory = TestDataFactory::new();
//! let pr = factory.github_pr().number(123).title("Test PR").build();
//! ```

use color_eyre::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 测试数据工厂
///
/// 提供统一的测试数据生成接口，支持从模板文件生成测试数据。
pub struct TestDataFactory {
    templates_dir: PathBuf,
}

impl TestDataFactory {
    /// 创建新的测试数据工厂
    pub fn new() -> Self {
        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("templates");

        Self { templates_dir }
    }

    /// 创建 GitHub PR 数据构建器
    pub fn github_pr(&self) -> GitHubPRBuilder {
        GitHubPRBuilder::new(&self.templates_dir)
    }

    /// 创建 Jira Issue 数据构建器
    pub fn jira_issue(&self) -> JiraIssueBuilder {
        JiraIssueBuilder::new(&self.templates_dir)
    }

    /// 创建 Git Commit 数据构建器
    pub fn git_commit(&self) -> GitCommitBuilder {
        GitCommitBuilder::new(&self.templates_dir)
    }

    /// 创建配置数据构建器
    pub fn config(&self) -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// 从模板文件加载并替换变量
    ///
    /// ## 错误处理
    /// 如果模板文件不存在或JSON解析失败，返回包含详细错误上下文的`Result`。
    ///
    /// ## 示例
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let mut vars = HashMap::new();
    /// vars.insert("number".to_string(), "123".to_string());
    /// let data = factory.load_template("github_pr.json", &vars)?;
    /// ```
    fn load_template(&self, filename: &str, vars: &HashMap<String, String>) -> Result<Value> {
        use color_eyre::eyre::Context;

        let template_path = self.templates_dir.join(filename);
        let template_content = fs::read_to_string(&template_path)
            .wrap_err_with(|| format!("Failed to read template: {}", template_path.display()))?;

        let mut result = template_content;
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 解析为 JSON
        serde_json::from_str(&result).wrap_err_with(|| {
            format!(
                "Failed to parse template result as JSON for file: {}",
                filename
            )
        })
    }
}

impl Default for TestDataFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// GitHub PR 数据构建器
pub struct GitHubPRBuilder {
    templates_dir: PathBuf,
    vars: HashMap<String, String>,
}

impl GitHubPRBuilder {
    fn new(templates_dir: &Path) -> Self {
        let mut vars = HashMap::new();
        // 设置默认值
        vars.insert("number".to_string(), "123".to_string());
        vars.insert("title".to_string(), "Test PR".to_string());
        vars.insert("body".to_string(), "Test PR body".to_string());
        vars.insert("state".to_string(), "open".to_string());
        vars.insert("head_ref".to_string(), "feature/test".to_string());
        vars.insert("head_sha".to_string(), "abc123def456".to_string());
        vars.insert("base_ref".to_string(), "main".to_string());
        vars.insert("base_sha".to_string(), "def456abc123".to_string());
        vars.insert("user_login".to_string(), "testuser".to_string());
        vars.insert("user_id".to_string(), "12345".to_string());
        vars.insert("created_at".to_string(), "2024-01-01T10:00:00Z".to_string());
        vars.insert("updated_at".to_string(), "2024-01-02T15:30:00Z".to_string());
        vars.insert("merged".to_string(), "false".to_string());
        vars.insert(
            "html_url".to_string(),
            "https://github.com/owner/repo/pull/123".to_string(),
        );

        Self {
            templates_dir: templates_dir.to_path_buf(),
            vars,
        }
    }

    /// 设置 PR 编号
    pub fn number(mut self, number: u64) -> Self {
        self.vars.insert("number".to_string(), number.to_string());
        self
    }

    /// 设置 PR 标题
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.vars.insert("title".to_string(), title.into());
        self
    }

    /// 设置 PR 正文
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn body<S: Into<String>>(mut self, body: S) -> Self {
        self.vars.insert("body".to_string(), body.into());
        self
    }

    /// 设置 PR 状态
    pub fn state<S: Into<String>>(mut self, state: S) -> Self {
        self.vars.insert("state".to_string(), state.into());
        self
    }

    /// 设置源分支
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn head_ref<S: Into<String>>(mut self, ref_name: S) -> Self {
        self.vars.insert("head_ref".to_string(), ref_name.into());
        self
    }

    /// 设置目标分支
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn base_ref<S: Into<String>>(mut self, ref_name: S) -> Self {
        self.vars.insert("base_ref".to_string(), ref_name.into());
        self
    }

    /// 设置合并状态
    pub fn merged(mut self, merged: bool) -> Self {
        self.vars.insert("merged".to_string(), merged.to_string());
        self
    }

    /// 构建 GitHub PR JSON 数据
    ///
    /// ## 错误处理
    /// 如果模板加载或JSON解析失败，返回包含详细上下文的错误。
    ///
    /// ## 示例
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let pr = factory
    ///     .github_pr()
    ///     .number(123)
    ///     .title("Test PR")
    ///     .build()?;
    /// ```
    pub fn build(&self) -> Result<Value> {
        let factory = TestDataFactory {
            templates_dir: self.templates_dir.clone(),
        };
        factory.load_template("github_pr.json", &self.vars)
    }

    /// 构建为 JSON 字符串
    ///
    /// ## 示例
    /// ```rust
    /// let pr_json = factory.github_pr().build_string()?;
    /// ```
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build()?)?)
    }
}

/// Jira Issue 数据构建器
pub struct JiraIssueBuilder {
    templates_dir: PathBuf,
    vars: HashMap<String, String>,
}

impl JiraIssueBuilder {
    fn new(templates_dir: &Path) -> Self {
        let mut vars = HashMap::new();
        // 设置默认值
        vars.insert("id".to_string(), "12345".to_string());
        vars.insert("key".to_string(), "PROJ-123".to_string());
        vars.insert(
            "self_url".to_string(),
            "https://jira.example.com/rest/api/3/issue/12345".to_string(),
        );
        vars.insert("summary".to_string(), "Test Issue Summary".to_string());
        vars.insert(
            "description".to_string(),
            "This is a test issue description".to_string(),
        );
        vars.insert("status_name".to_string(), "In Progress".to_string());
        vars.insert("status_id".to_string(), "3".to_string());
        vars.insert("assignee_name".to_string(), "Test User".to_string());
        vars.insert("assignee_email".to_string(), "test@example.com".to_string());
        vars.insert("reporter_name".to_string(), "Reporter User".to_string());
        vars.insert(
            "reporter_email".to_string(),
            "reporter@example.com".to_string(),
        );
        vars.insert(
            "created".to_string(),
            "2024-01-01T10:00:00.000+0000".to_string(),
        );
        vars.insert(
            "updated".to_string(),
            "2024-01-02T15:30:00.000+0000".to_string(),
        );
        vars.insert("issue_type".to_string(), "Bug".to_string());
        vars.insert("issue_type_id".to_string(), "1".to_string());
        vars.insert("project_key".to_string(), "PROJ".to_string());
        vars.insert("project_name".to_string(), "Test Project".to_string());

        Self {
            templates_dir: templates_dir.to_path_buf(),
            vars,
        }
    }

    /// 设置 Issue Key
    pub fn key<S: Into<String>>(mut self, key: S) -> Self {
        self.vars.insert("key".to_string(), key.into());
        self
    }

    /// 设置 Issue 摘要
    pub fn summary<S: Into<String>>(mut self, summary: S) -> Self {
        self.vars.insert("summary".to_string(), summary.into());
        self
    }

    /// 设置 Issue 描述
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.vars.insert("description".to_string(), description.into());
        self
    }

    /// 设置状态
    pub fn status<S: Into<String>>(mut self, status: S) -> Self {
        self.vars.insert("status_name".to_string(), status.into());
        self
    }

    /// 设置 Issue 类型
    pub fn issue_type<S: Into<String>>(mut self, issue_type: S) -> Self {
        self.vars.insert("issue_type".to_string(), issue_type.into());
        self
    }

    /// 构建 Jira Issue JSON 数据
    ///
    /// ## 错误处理
    /// 如果模板加载或JSON解析失败，返回包含详细上下文的错误。
    ///
    /// ## 示例
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let issue = factory
    ///     .jira_issue()
    ///     .key("PROJ-123")
    ///     .summary("Test Issue")
    ///     .build()?;
    /// ```
    pub fn build(&self) -> Result<Value> {
        let factory = TestDataFactory {
            templates_dir: self.templates_dir.clone(),
        };
        factory.load_template("jira_issue.json", &self.vars)
    }

    /// 构建为 JSON 字符串
    ///
    /// ## 示例
    /// ```rust
    /// let issue_json = factory.jira_issue().build_string()?;
    /// ```
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build()?)?)
    }
}

/// Git Commit 数据构建器
///
/// ## 使用示例
///
/// ```rust
/// let factory = TestDataFactory::new();
/// let commit = factory
///     .git_commit()
///     .sha("abc123def456")
///     .message("feat: add new feature")
///     .author_name("John Doe")
///     .author_email("john@example.com")
///     .build();
/// ```
pub struct GitCommitBuilder {
    templates_dir: PathBuf,
    vars: HashMap<String, String>,
}

impl GitCommitBuilder {
    fn new(templates_dir: &Path) -> Self {
        let mut vars = HashMap::new();
        // 设置默认值
        vars.insert("sha".to_string(), "abc123def456789".to_string());
        vars.insert(
            "message".to_string(),
            "feat: test commit message".to_string(),
        );
        vars.insert("author_name".to_string(), "Test Author".to_string());
        vars.insert("author_email".to_string(), "author@example.com".to_string());
        vars.insert(
            "author_date".to_string(),
            "2024-01-01T10:00:00Z".to_string(),
        );
        vars.insert("committer_name".to_string(), "Test Committer".to_string());
        vars.insert(
            "committer_email".to_string(),
            "committer@example.com".to_string(),
        );
        vars.insert(
            "committer_date".to_string(),
            "2024-01-01T10:00:00Z".to_string(),
        );
        vars.insert("author_login".to_string(), "testauthor".to_string());
        vars.insert("author_id".to_string(), "12345".to_string());
        vars.insert("committer_login".to_string(), "testcommitter".to_string());
        vars.insert("committer_id".to_string(), "12345".to_string());
        vars.insert("parent_sha".to_string(), "parent123def456".to_string());
        vars.insert(
            "html_url".to_string(),
            "https://github.com/owner/repo/commit/abc123def456789".to_string(),
        );

        Self {
            templates_dir: templates_dir.to_path_buf(),
            vars,
        }
    }

    /// 设置 commit SHA
    pub fn sha<S: Into<String>>(mut self, sha: S) -> Self {
        self.vars.insert("sha".to_string(), sha.into());
        self
    }

    /// 设置 commit 消息
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.vars.insert("message".to_string(), message.into());
        self
    }

    /// 设置作者名称
    pub fn author_name<S: Into<String>>(mut self, name: S) -> Self {
        self.vars.insert("author_name".to_string(), name.into());
        self
    }

    /// 设置作者邮箱
    pub fn author_email<S: Into<String>>(mut self, email: S) -> Self {
        self.vars.insert("author_email".to_string(), email.into());
        self
    }

    /// 设置作者日期
    pub fn author_date<S: Into<String>>(mut self, date: S) -> Self {
        self.vars.insert("author_date".to_string(), date.into());
        self
    }

    /// 设置提交者名称
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn committer_name<S: Into<String>>(mut self, name: S) -> Self {
        self.vars.insert("committer_name".to_string(), name.into());
        self
    }

    /// 设置提交者邮箱
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn committer_email<S: Into<String>>(mut self, email: S) -> Self {
        self.vars.insert("committer_email".to_string(), email.into());
        self
    }

    /// 设置提交者日期
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn committer_date<S: Into<String>>(mut self, date: S) -> Self {
        self.vars.insert("committer_date".to_string(), date.into());
        self
    }

    /// 设置父提交 SHA
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn parent_sha<S: Into<String>>(mut self, sha: S) -> Self {
        self.vars.insert("parent_sha".to_string(), sha.into());
        self
    }

    /// 构建 Git Commit JSON 数据
    ///
    /// ## 错误处理
    /// 如果模板加载或JSON解析失败，返回包含详细上下文的错误。
    ///
    /// ## 示例
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let commit = factory
    ///     .git_commit()
    ///     .sha("abc123")
    ///     .message("feat: add feature")
    ///     .build()?;
    /// ```
    pub fn build(&self) -> Result<Value> {
        let factory = TestDataFactory {
            templates_dir: self.templates_dir.clone(),
        };
        factory.load_template("git_commit.json", &self.vars)
    }

    /// 构建为 JSON 字符串
    ///
    /// ## 示例
    /// ```rust
    /// let commit_json = factory.git_commit().build_string()?;
    /// ```
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build()?)?)
    }
}

/// 配置数据构建器
pub struct ConfigBuilder {
    config: Value,
}

impl ConfigBuilder {
    fn new() -> Self {
        Self {
            config: json!({
                "github": {
                    "accounts": [],
                    "current_account": null
                },
                "jira": {
                    "service_address": null,
                    "username": null,
                    "api_token": null
                },
                "llm": {
                    "providers": [],
                    "current_provider": null
                },
                "log": {
                    "level": "info",
                    "method": "console"
                }
            }),
        }
    }

    /// 设置 GitHub 配置
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn github(mut self, github_config: Value) -> Self {
        self.config["github"] = github_config;
        self
    }

    /// 设置 Jira 配置
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn jira(mut self, jira_config: Value) -> Self {
        self.config["jira"] = jira_config;
        self
    }

    /// 设置 LLM 配置
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn llm(mut self, llm_config: Value) -> Self {
        self.config["llm"] = llm_config;
        self
    }

    /// 设置日志配置
    pub fn log(mut self, log_config: Value) -> Self {
        self.config["log"] = log_config;
        self
    }

    /// 构建配置 JSON 数据
    pub fn build(&self) -> Value {
        self.config.clone()
    }

    /// 构建为 JSON 字符串
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> String {
        serde_json::to_string_pretty(&self.build()).expect("JSON serialization should succeed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试GitHubPRBuilder使用默认值构建
    ///
    /// ## 测试目的
    /// 验证 `GitHubPRBuilder` 使用默认值能够成功构建GitHub PR JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值构建GitHub PR
    /// 3. 验证构建的数据包含预期的默认值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - number为123，title为"Test PR"，state为"open"，merged为false
    #[test]
    fn test_github_pr_builder_default_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let pr = factory.github_pr().build()?;

        assert_eq!(pr["number"], 123);
        assert_eq!(pr["title"], "Test PR");
        assert_eq!(pr["state"], "open");
        assert_eq!(pr["merged"], false);
        Ok(())
    }

    /// 测试GitHubPRBuilder使用自定义值构建
    ///
    /// ## 测试目的
    /// 验证 `GitHubPRBuilder` 使用自定义值能够成功构建GitHub PR JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用自定义值构建GitHub PR（number, title, state, merged）
    /// 3. 验证构建的数据包含自定义值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 所有自定义值都正确设置
    #[test]
    fn test_github_pr_builder_custom_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let pr = factory
            .github_pr()
            .number(456)
            .title("Custom PR")
            .state("closed")
            .merged(true)
            .build()?;

        assert_eq!(pr["number"], 456);
        assert_eq!(pr["title"], "Custom PR");
        assert_eq!(pr["state"], "closed");
        assert_eq!(pr["merged"], true);
        Ok(())
    }

    /// 测试JiraIssueBuilder使用默认值构建
    ///
    /// ## 测试目的
    /// 验证 `JiraIssueBuilder` 使用默认值能够成功构建Jira Issue JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值构建Jira Issue
    /// 3. 验证构建的数据包含预期的默认值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - key为"PROJ-123"，summary为"Test Issue Summary"，status为"In Progress"
    #[test]
    fn test_jira_issue_builder_default_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let issue = factory.jira_issue().build()?;

        assert_eq!(issue["key"], "PROJ-123");
        assert_eq!(issue["fields"]["summary"], "Test Issue Summary");
        assert_eq!(issue["fields"]["status"]["name"], "In Progress");
        Ok(())
    }

    /// 测试JiraIssueBuilder使用自定义值构建
    ///
    /// ## 测试目的
    /// 验证 `JiraIssueBuilder` 使用自定义值能够成功构建Jira Issue JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用自定义值构建Jira Issue（key, summary, status, issue_type）
    /// 3. 验证构建的数据包含自定义值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 所有自定义值都正确设置
    #[test]
    fn test_jira_issue_builder_custom_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let issue = factory
            .jira_issue()
            .key("PROJ-456")
            .summary("Custom Issue")
            .status("Done")
            .issue_type("Feature")
            .build()?;

        assert_eq!(issue["key"], "PROJ-456");
        assert_eq!(issue["fields"]["summary"], "Custom Issue");
        assert_eq!(issue["fields"]["status"]["name"], "Done");
        assert_eq!(issue["fields"]["issuetype"]["name"], "Feature");
        Ok(())
    }

    /// 测试ConfigBuilder使用默认值构建
    ///
    /// ## 测试目的
    /// 验证 `ConfigBuilder` 使用默认值能够成功构建配置JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值构建配置
    /// 3. 验证构建的配置包含所有必需的节（github, jira, llm, log）
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 配置包含所有必需的节
    #[test]
    fn test_config_builder_default() {
        let factory = TestDataFactory::new();
        let config = factory.config().build();

        assert!(config["github"].is_object());
        assert!(config["jira"].is_object());
        assert!(config["llm"].is_object());
        assert!(config["log"].is_object());
    }

    /// 测试ConfigBuilder使用自定义值构建
///
/// ## 测试目的
/// 验证 `ConfigBuilder` 使用自定义值能够成功构建配置JSON数据。
///
/// ## 测试场景
/// 1. 创建TestDataFactory
/// 2. 使用自定义值构建配置（log配置）
/// 3. 验证构建的配置包含自定义值
///
/// ## 预期结果
/// - 构建成功
/// - 自定义值正确设置
    #[test]
    fn test_config_builder_custom() {
        let factory = TestDataFactory::new();
        let config = factory
            .config()
            .log(json!({
                "level": "debug",
                "method": "file"
            }))
            .build();

        assert_eq!(config["log"]["level"], "debug");
        assert_eq!(config["log"]["method"], "file");
    }

    /// 测试GitCommitBuilder使用默认值构建
    ///
    /// ## 测试目的
    /// 验证 `GitCommitBuilder` 使用默认值能够成功构建Git Commit JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值构建Git Commit
    /// 3. 验证构建的数据包含预期的默认值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - sha、message、author等字段包含预期的默认值
    #[test]
    fn test_git_commit_builder_default_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let commit = factory.git_commit().build()?;

        assert_eq!(commit["sha"], "abc123def456789");
        assert_eq!(commit["commit"]["message"], "feat: test commit message");
        assert_eq!(commit["commit"]["author"]["name"], "Test Author");
        assert_eq!(commit["commit"]["author"]["email"], "author@example.com");
        Ok(())
    }

    /// 测试GitCommitBuilder使用自定义值构建
    ///
    /// ## 测试目的
    /// 验证 `GitCommitBuilder` 使用自定义值能够成功构建Git Commit JSON数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用自定义值构建Git Commit（sha, message, author_name, author_email, author_date）
    /// 3. 验证构建的数据包含自定义值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 所有自定义值都正确设置
    #[test]
    fn test_git_commit_builder_custom_return_ok() -> Result<()> {
        let factory = TestDataFactory::new();
        let commit = factory
            .git_commit()
            .sha("custom123sha")
            .message("fix: custom commit message")
            .author_name("Custom Author")
            .author_email("custom@example.com")
            .author_date("2024-06-01T12:00:00Z")
            .build()?;

        assert_eq!(commit["sha"], "custom123sha");
        assert_eq!(commit["commit"]["message"], "fix: custom commit message");
        assert_eq!(commit["commit"]["author"]["name"], "Custom Author");
        assert_eq!(commit["commit"]["author"]["email"], "custom@example.com");
        assert_eq!(commit["commit"]["author"]["date"], "2024-06-01T12:00:00Z");
        Ok(())
    }
}
