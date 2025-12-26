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

use crate::common::test_data::cache::{CacheConfig, TestDataCache};
use crate::common::test_data::cleanup::{CleanupStrategy, TestDataCleanupManager};
use crate::common::test_data::version::TestDataVersionManager;
use chrono::{DateTime, Utc};
use color_eyre::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// 测试数据工厂
///
/// 提供统一的测试数据生成接口，支持从模板文件生成测试数据。
/// 集成了缓存、清理和版本管理功能。
pub struct TestDataFactory {
    templates_dir: PathBuf,
    /// 缓存系统（可选）
    cache: Option<Arc<Mutex<TestDataCache>>>,
    /// 清理管理器（可选）
    cleanup_manager: Option<Arc<Mutex<TestDataCleanupManager>>>,
    /// 版本管理器（可选）
    version_manager: Option<Arc<Mutex<TestDataVersionManager>>>,
}

impl TestDataFactory {
    /// 创建新的测试数据工厂
    pub fn new() -> Self {
        let templates_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("templates");

        Self {
            templates_dir,
            cache: None,
            cleanup_manager: None,
            version_manager: None,
        }
    }

    /// 启用缓存
    pub fn enable_cache(&mut self, config: CacheConfig) {
        let cache = TestDataCache::new(config);
        self.cache = Some(Arc::new(Mutex::new(cache)));
    }

    /// 禁用缓存
    #[allow(dead_code)]
    pub fn disable_cache(&mut self) {
        self.cache = None;
    }

    /// 获取缓存统计信息
    pub fn cache_stats(&self) -> Option<crate::common::test_data::cache::CacheStats> {
        self.cache
            .as_ref()
            .and_then(|c| c.lock().ok())
            .map(|cache| cache.get_stats().clone())
    }

    /// 设置清理策略
    pub fn set_cleanup_strategy(&mut self, strategy: CleanupStrategy) {
        let manager = TestDataCleanupManager::new(strategy);
        self.cleanup_manager = Some(Arc::new(Mutex::new(manager)));
    }

    /// 执行清理
    pub fn cleanup(&mut self) -> Result<()> {
        if let Some(ref manager) = self.cleanup_manager {
            let mut mgr = manager.lock().unwrap();
            let items = vec![]; // 实际应该从缓存中获取需要清理的项目
            let retain = vec![]; // 实际应该从配置中获取需要保留的项目
            let _result = mgr.cleanup(&items, &retain);
        }
        Ok(())
    }

    /// 重置到初始状态
    #[allow(dead_code)]
    pub fn reset(&mut self) -> Result<()> {
        if let Some(ref manager) = self.cleanup_manager {
            let mut mgr = manager.lock().unwrap();
            mgr.reset()?;
        }
        if let Some(ref cache) = self.cache {
            let mut c = cache.lock().unwrap();
            c.clear();
        }
        Ok(())
    }

    /// 设置版本
    pub fn with_version(&mut self, version: &str) -> &mut Self {
        let manager = TestDataVersionManager::new(version.to_string());
        self.version_manager = Some(Arc::new(Mutex::new(manager)));
        self
    }

    /// 获取当前版本
    pub fn get_version(&self) -> Option<String> {
        self.version_manager
            .as_ref()
            .and_then(|m| m.lock().ok())
            .map(|mgr| mgr.get_current_version().to_string())
    }

    /// 批量生成测试数据
    pub fn build_batch<F>(&self, count: usize, builder_fn: F) -> Vec<Value>
    where
        F: Fn(&Self) -> Result<Value>,
    {
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            if let Ok(value) = builder_fn(self) {
                results.push(value);
            }
        }
        results
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

    /// 创建分支数据构建器
    pub fn branch(&self) -> BranchBuilder {
        BranchBuilder::new()
    }

    /// 创建用户数据构建器
    pub fn user(&self) -> UserBuilder {
        UserBuilder::new()
    }

    /// 创建日期时间数据构建器
    ///
    /// 用于生成测试用的日期时间字符串。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let dt = factory.date_time().build();
    /// // 返回 ISO 8601 格式的日期时间字符串，如 "2024-01-01T10:00:00Z"
    /// ```
    pub fn date_time(&self) -> DateTimeBuilder {
        DateTimeBuilder::new()
    }

    /// 创建 UUID 数据构建器
    ///
    /// 用于生成测试用的 UUID 字符串。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// let factory = TestDataFactory::new();
    /// let uuid = factory.uuid().build();
    /// // 返回 UUID v4 格式的字符串，如 "550e8400-e29b-41d4-a716-446655440000"
    /// ```
    pub fn uuid(&self) -> UuidBuilder {
        UuidBuilder::new()
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
            cache: None,
            cleanup_manager: None,
            version_manager: None,
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
            cache: None,
            cleanup_manager: None,
            version_manager: None,
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
            cache: None,
            cleanup_manager: None,
            version_manager: None,
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
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build())
            .map_err(|e| color_eyre::eyre::eyre!("JSON serialization should succeed: {}", e))?)
    }
}

/// 分支数据构建器
///
/// ## 使用示例
///
/// ```rust
/// let factory = TestDataFactory::new();
/// let branch_name = factory.branch().name("feature/test").build_name();
/// let branch_data = factory.branch().name("feature/test").build();
/// ```
pub struct BranchBuilder {
    name: String,
    prefix: Option<String>,
    jira_key: Option<String>,
}

impl BranchBuilder {
    fn new() -> Self {
        Self {
            name: "feature/test".to_string(),
            prefix: Some("feature".to_string()),
            jira_key: None,
        }
    }

    /// 设置分支名称
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// 设置分支前缀（feature, bugfix, hotfix等）
    pub fn prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// 设置关联的 Jira Key
    pub fn jira_key<S: Into<String>>(mut self, jira_key: S) -> Self {
        self.jira_key = Some(jira_key.into());
        self
    }

    /// 构建分支名称字符串
    ///
    /// ## 示例
    /// ```rust
    /// let branch_name = factory.branch().name("test").build_name();
    /// ```
    pub fn build_name(&self) -> String {
        self.name.clone()
    }

    /// 构建分支 JSON 数据
    ///
    /// ## 示例
    /// ```rust
    /// let branch_data = factory.branch().name("feature/test").build();
    /// ```
    pub fn build(&self) -> Value {
        let mut branch = json!({
            "name": self.name,
            "ref": format!("refs/heads/{}", self.name),
            "sha": "abc123def456789",
        });

        if let Some(ref prefix) = self.prefix {
            branch["prefix"] = json!(prefix);
        }

        if let Some(ref jira_key) = self.jira_key {
            branch["jira_key"] = json!(jira_key);
        }

        branch
    }

    /// 构建为 JSON 字符串
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build())
            .map_err(|e| color_eyre::eyre::eyre!("JSON serialization should succeed: {}", e))?)
    }
}

impl Default for BranchBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 用户数据构建器
///
/// ## 使用示例
///
/// ```rust
/// let factory = TestDataFactory::new();
/// // GitHub 用户
/// let github_user = factory.user().github().login("testuser").build();
/// // Jira 用户
/// let jira_user = factory.user().jira().account_id("test-123").build();
/// ```
pub struct UserBuilder {
    user_type: UserType,
    login: Option<String>,
    id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    account_id: Option<String>,
}

enum UserType {
    GitHub,
    Jira,
}

impl UserBuilder {
    fn new() -> Self {
        Self {
            user_type: UserType::GitHub,
            login: Some("testuser".to_string()),
            id: Some("12345".to_string()),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            account_id: None,
        }
    }

    /// 设置为 GitHub 用户
    pub fn github(mut self) -> Self {
        self.user_type = UserType::GitHub;
        self
    }

    /// 设置为 Jira 用户
    pub fn jira(mut self) -> Self {
        self.user_type = UserType::Jira;
        self
    }

    /// 设置登录名（GitHub）
    pub fn login<S: Into<String>>(mut self, login: S) -> Self {
        self.login = Some(login.into());
        self
    }

    /// 设置用户 ID
    pub fn id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 设置显示名称
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置邮箱
    pub fn email<S: Into<String>>(mut self, email: S) -> Self {
        self.email = Some(email.into());
        self
    }

    /// 设置账户 ID（Jira）
    pub fn account_id<S: Into<String>>(mut self, account_id: S) -> Self {
        self.account_id = Some(account_id.into());
        self
    }

    /// 构建用户 JSON 数据
    ///
    /// ## 示例
    /// ```rust
    /// let user = factory.user().github().login("testuser").build();
    /// ```
    pub fn build(&self) -> Value {
        match self.user_type {
            UserType::GitHub => {
                let mut user = json!({
                    "login": self.login.as_ref().unwrap_or(&"testuser".to_string()),
                    "id": self.id.as_ref().unwrap_or(&"12345".to_string()),
                });

                if let Some(ref name) = self.name {
                    user["name"] = json!(name);
                }

                if let Some(ref email) = self.email {
                    user["email"] = json!(email);
                }

                user
            }
            UserType::Jira => {
                let mut user = json!({
                    "accountId": self.account_id.as_ref().unwrap_or(&"test-account-id-123".to_string()),
                    "displayName": self.name.as_ref().unwrap_or(&"Test User".to_string()),
                });

                if let Some(ref email) = self.email {
                    user["emailAddress"] = json!(email);
                }

                if let Some(ref login) = self.login {
                    user["name"] = json!(login);
                }

                user
            }
        }
    }

    /// 构建为 JSON 字符串
    #[allow(dead_code)]
    pub fn build_string(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.build())
            .map_err(|e| color_eyre::eyre::eyre!("JSON serialization should succeed: {}", e))?)
    }
}

impl Default for UserBuilder {
    fn default() -> Self {
        Self::new()
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

    /// 测试BranchBuilder使用默认值构建
    ///
    /// ## 测试目的
    /// 验证 `BranchBuilder` 使用默认值能够成功构建分支数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值构建分支
    /// 3. 验证构建的数据包含预期的默认值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - name为"feature/test"，ref为"refs/heads/feature/test"
    #[test]
    fn test_branch_builder_default() {
        let factory = TestDataFactory::new();
        let branch = factory.branch().build();

        assert_eq!(branch["name"], "feature/test");
        assert_eq!(branch["ref"], "refs/heads/feature/test");
    }

    /// 测试BranchBuilder使用自定义值构建
    ///
    /// ## 测试目的
    /// 验证 `BranchBuilder` 使用自定义值能够成功构建分支数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用自定义值构建分支（name, prefix, jira_key）
    /// 3. 验证构建的数据包含自定义值
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 所有自定义值都正确设置
    #[test]
    fn test_branch_builder_custom() {
        let factory = TestDataFactory::new();
        let branch = factory
            .branch()
            .name("bugfix/PROJ-456")
            .prefix("bugfix")
            .jira_key("PROJ-456")
            .build();

        assert_eq!(branch["name"], "bugfix/PROJ-456");
        assert_eq!(branch["prefix"], "bugfix");
        assert_eq!(branch["jira_key"], "PROJ-456");
    }

    /// 测试BranchBuilder构建分支名称字符串
    ///
    /// ## 测试目的
    /// 验证 `BranchBuilder::build_name()` 能够返回分支名称字符串。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 设置分支名称
    /// 3. 调用build_name()获取名称字符串
    ///
    /// ## 预期结果
    /// - 返回正确的分支名称字符串
    #[test]
    fn test_branch_builder_build_name() {
        let factory = TestDataFactory::new();
        let branch_name = factory.branch().name("feature/custom").build_name();

        assert_eq!(branch_name, "feature/custom");
    }

    /// 测试UserBuilder构建GitHub用户
    ///
    /// ## 测试目的
    /// 验证 `UserBuilder` 能够成功构建GitHub用户数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用github()方法设置为GitHub用户
    /// 3. 设置用户信息（login, id, name, email）
    /// 4. 验证构建的数据包含GitHub用户字段
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 包含login、id等GitHub用户字段
    #[test]
    fn test_user_builder_github() {
        let factory = TestDataFactory::new();
        let user = factory
            .user()
            .github()
            .login("testuser")
            .id("12345")
            .name("Test User")
            .email("test@example.com")
            .build();

        assert_eq!(user["login"], "testuser");
        assert_eq!(user["id"], "12345");
        assert_eq!(user["name"], "Test User");
        assert_eq!(user["email"], "test@example.com");
    }

    /// 测试UserBuilder构建Jira用户
    ///
    /// ## 测试目的
    /// 验证 `UserBuilder` 能够成功构建Jira用户数据。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用jira()方法设置为Jira用户
    /// 3. 设置用户信息（account_id, name, email）
    /// 4. 验证构建的数据包含Jira用户字段
    ///
    /// ## 预期结果
    /// - 构建成功
    /// - 包含accountId、displayName等Jira用户字段
    #[test]
    fn test_user_builder_jira() {
        let factory = TestDataFactory::new();
        let user = factory
            .user()
            .jira()
            .account_id("test-account-id-123")
            .name("Test User")
            .email("test@example.com")
            .build();

        assert_eq!(user["accountId"], "test-account-id-123");
        assert_eq!(user["displayName"], "Test User");
        assert_eq!(user["emailAddress"], "test@example.com");
    }

    /// 测试DateTimeBuilder生成日期时间
    ///
    /// ## 测试目的
    /// 验证 `DateTimeBuilder` 能够成功生成日期时间字符串。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 使用默认值生成日期时间
    /// 3. 验证生成的日期时间格式正确
    ///
    /// ## 预期结果
    /// - 生成成功
    /// - 日期时间格式为 ISO 8601 格式
    #[test]
    fn test_date_time_builder_default() {
        let factory = TestDataFactory::new();
        let dt = factory.date_time().build();

        // 验证格式为 ISO 8601
        assert!(dt.contains('T'));
        assert!(dt.ends_with('Z') || dt.contains('+') || dt.contains('-'));
    }

    /// 测试DateTimeBuilder使用自定义日期时间
    ///
    /// ## 测试目的
    /// 验证 `DateTimeBuilder` 使用自定义日期时间能够成功生成。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 设置自定义日期时间
    /// 3. 验证生成的日期时间正确
    ///
    /// ## 预期结果
    /// - 生成成功
    /// - 日期时间与设置的值一致
    #[test]
    fn test_date_time_builder_custom() {
        let factory = TestDataFactory::new();
        let custom_dt = DateTime::parse_from_rfc3339("2024-06-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let dt = factory.date_time().datetime(custom_dt).build();

        assert_eq!(dt, "2024-06-01T12:00:00Z");
    }

    /// 测试UuidBuilder生成UUID
    ///
    /// ## 测试目的
    /// 验证 `UuidBuilder` 能够成功生成UUID字符串。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 生成UUID
    /// 3. 验证UUID格式正确
    ///
    /// ## 预期结果
    /// - 生成成功
    /// - UUID格式为标准的UUID v4格式
    #[test]
    fn test_uuid_builder_default() {
        let factory = TestDataFactory::new();
        let uuid = factory.uuid().build();

        // 验证UUID格式: 8-4-4-4-12
        let parts: Vec<&str> = uuid.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);
    }

    /// 测试UuidBuilder使用自定义UUID
    ///
    /// ## 测试目的
    /// 验证 `UuidBuilder` 使用自定义UUID能够成功生成。
    ///
    /// ## 测试场景
    /// 1. 创建TestDataFactory
    /// 2. 设置自定义UUID
    /// 3. 验证生成的UUID正确
    ///
    /// ## 预期结果
    /// - 生成成功
    /// - UUID与设置的值一致
    #[test]
    fn test_uuid_builder_custom() {
        let factory = TestDataFactory::new();
        let custom_uuid = "550e8400-e29b-41d4-a716-446655440000";
        let uuid = factory.uuid().uuid(custom_uuid).build();

        assert_eq!(uuid, custom_uuid);
    }
}

/// 日期时间数据构建器
///
/// 用于生成测试用的日期时间字符串（ISO 8601 格式）。
///
/// ## 使用示例
///
/// ```rust
/// let factory = TestDataFactory::new();
///
/// // 使用默认值（当前时间）
/// let dt = factory.date_time().build();
///
/// // 使用自定义日期时间
/// let custom_dt = DateTime::parse_from_rfc3339("2024-06-01T12:00:00Z")
///     .unwrap()
///     .with_timezone(&Utc);
/// let dt = factory.date_time().datetime(custom_dt).build();
/// ```
pub struct DateTimeBuilder {
    datetime: Option<DateTime<Utc>>,
}

impl DateTimeBuilder {
    fn new() -> Self {
        Self { datetime: None }
    }

    /// 设置日期时间
    ///
    /// # 参数
    ///
    /// * `datetime` - UTC 日期时间
    pub fn datetime(mut self, datetime: DateTime<Utc>) -> Self {
        self.datetime = Some(datetime);
        self
    }

    /// 设置日期时间（从字符串解析）
    ///
    /// # 参数
    ///
    /// * `s` - RFC 3339 格式的日期时间字符串
    ///
    /// # 示例
    ///
    /// ```rust
    /// let dt = factory.date_time().from_str("2024-06-01T12:00:00Z").build();
    /// ```
    #[allow(dead_code)]
    pub fn from_str<S: AsRef<str>>(mut self, s: S) -> Self {
        if let Ok(dt) = DateTime::parse_from_rfc3339(s.as_ref()) {
            self.datetime = Some(dt.with_timezone(&Utc));
        }
        self
    }

    /// 构建日期时间字符串
    ///
    /// 返回 ISO 8601 格式的日期时间字符串（如 "2024-01-01T10:00:00Z"）。
    pub fn build(&self) -> String {
        let dt = self.datetime.unwrap_or_else(|| Utc::now());
        dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    }
}

impl Default for DateTimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// UUID 数据构建器
///
/// 用于生成测试用的 UUID 字符串。
///
/// ## 使用示例
///
/// ```rust
/// let factory = TestDataFactory::new();
///
/// // 使用默认值（随机生成）
/// let uuid = factory.uuid().build();
///
/// // 使用自定义UUID
/// let uuid = factory.uuid().uuid("550e8400-e29b-41d4-a716-446655440000").build();
/// ```
pub struct UuidBuilder {
    uuid: Option<String>,
}

impl UuidBuilder {
    fn new() -> Self {
        Self { uuid: None }
    }

    /// 设置 UUID
    ///
    /// # 参数
    ///
    /// * `uuid` - UUID 字符串
    pub fn uuid<S: Into<String>>(mut self, uuid: S) -> Self {
        self.uuid = Some(uuid.into());
        self
    }

    /// 构建 UUID 字符串
    ///
    /// 如果没有设置自定义 UUID，则生成一个随机的 UUID v4。
    pub fn build(&self) -> String {
        self.uuid.clone().unwrap_or_else(|| {
            // 生成简单的UUID v4格式字符串（用于测试）
            // 注意：这不是真正的UUID生成，只是格式匹配
            // 如果需要真正的UUID，可以使用uuid crate
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::sync::atomic::AtomicU64::fetch_add(
                &std::sync::atomic::AtomicU64::new(0),
                1,
                std::sync::atomic::Ordering::Relaxed,
            )
            .hash(&mut hasher);
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .hash(&mut hasher);
            let hash = hasher.finish();

            format!(
                "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                (hash >> 32) as u32,
                (hash >> 16) as u16,
                ((hash >> 16) as u16) | 0x4000,          // version 4
                ((hash >> 16) as u16) & 0x3fff | 0x8000, // variant
                hash & 0xffffffffffff,
            )
        })
    }
}

impl Default for UuidBuilder {
    fn default() -> Self {
        Self::new()
    }
}
