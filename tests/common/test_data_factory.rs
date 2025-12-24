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

    /// 创建配置数据构建器
    pub fn config(&self) -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// 从模板文件加载并替换变量
    fn load_template(&self, filename: &str, vars: &HashMap<String, String>) -> Value {
        let template_path = self.templates_dir.join(filename);
        let template_content = fs::read_to_string(&template_path)
            .unwrap_or_else(|_| panic!("Failed to read template: {}", template_path.display()));

        let mut result = template_content.clone();
        for (key, value) in vars {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // 解析为 JSON
        serde_json::from_str(&result)
            .unwrap_or_else(|_| panic!("Failed to parse template result as JSON"))
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
    pub fn build(&self) -> Value {
        let factory = TestDataFactory {
            templates_dir: self.templates_dir.clone(),
        };
        factory.load_template("github_pr.json", &self.vars)
    }

    /// 构建为 JSON 字符串
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> String {
        serde_json::to_string_pretty(&self.build()).unwrap()
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
    pub fn build(&self) -> Value {
        let factory = TestDataFactory {
            templates_dir: self.templates_dir.clone(),
        };
        factory.load_template("jira_issue.json", &self.vars)
    }

    /// 构建为 JSON 字符串
    ///
    /// 注意：此方法目前未被使用，但保留作为测试工具函数，供未来测试使用。
    #[allow(dead_code)]
    pub fn build_string(&self) -> String {
        serde_json::to_string_pretty(&self.build()).unwrap()
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
        serde_json::to_string_pretty(&self.build()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_pr_builder_default() {
        let factory = TestDataFactory::new();
        let pr = factory.github_pr().build();

        assert_eq!(pr["number"], 123);
        assert_eq!(pr["title"], "Test PR");
        assert_eq!(pr["state"], "open");
        assert_eq!(pr["merged"], false);
    }

    #[test]
    fn test_github_pr_builder_custom() {
        let factory = TestDataFactory::new();
        let pr = factory
            .github_pr()
            .number(456)
            .title("Custom PR")
            .state("closed")
            .merged(true)
            .build();

        assert_eq!(pr["number"], 456);
        assert_eq!(pr["title"], "Custom PR");
        assert_eq!(pr["state"], "closed");
        assert_eq!(pr["merged"], true);
    }

    #[test]
    fn test_jira_issue_builder_default() {
        let factory = TestDataFactory::new();
        let issue = factory.jira_issue().build();

        assert_eq!(issue["key"], "PROJ-123");
        assert_eq!(issue["fields"]["summary"], "Test Issue Summary");
        assert_eq!(issue["fields"]["status"]["name"], "In Progress");
    }

    #[test]
    fn test_jira_issue_builder_custom() {
        let factory = TestDataFactory::new();
        let issue = factory
            .jira_issue()
            .key("PROJ-456")
            .summary("Custom Issue")
            .status("Done")
            .issue_type("Feature")
            .build();

        assert_eq!(issue["key"], "PROJ-456");
        assert_eq!(issue["fields"]["summary"], "Custom Issue");
        assert_eq!(issue["fields"]["status"]["name"], "Done");
        assert_eq!(issue["fields"]["issuetype"]["name"], "Feature");
    }

    #[test]
    fn test_config_builder_default() {
        let factory = TestDataFactory::new();
        let config = factory.config().build();

        assert!(config["github"].is_object());
        assert!(config["jira"].is_object());
        assert!(config["llm"].is_object());
        assert!(config["log"].is_object());
    }

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
}
