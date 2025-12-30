//! Template configuration
//!
//! Loads templates from configuration files (global and project-level).

use crate::base::fs::FileReader;
use crate::base::settings::paths::Paths;
use color_eyre::{eyre::WrapErr, Result};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template engine type (default: "handlebars")
    #[serde(default = "default_engine")]
    pub engine: String,
    /// Branch templates
    #[serde(default)]
    pub branch: BranchTemplates,
    /// Commit templates
    #[serde(default)]
    pub commit: CommitTemplates,
    /// PR templates
    #[serde(default)]
    pub pull_requests: PullRequestsTemplates,
}

fn default_engine() -> String {
    "handlebars".to_string()
}

/// Branch templates configuration
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchTemplates {
    /// Default branch template
    #[serde(default = "default_branch_template")]
    pub default: String,
    /// Feature branch template
    pub feature: Option<String>,
    /// Bugfix branch template
    pub bugfix: Option<String>,
    /// Hotfix branch template
    pub hotfix: Option<String>,
    /// Refactoring branch template
    pub refactoring: Option<String>,
    /// Chore branch template
    pub chore: Option<String>,
}

fn default_branch_template() -> String {
    "{{jira_key}}-{{summary_slug}}".to_string()
}

impl Default for BranchTemplates {
    fn default() -> Self {
        Self {
            default: "{{jira_key}}-{{summary_slug}}".to_string(),
            feature: Some("feature/{{jira_key}}-{{summary_slug}}".to_string()),
            bugfix: Some("bugfix/{{jira_key}}-{{summary_slug}}".to_string()),
            hotfix: Some("hotfix/{{jira_key}}-{{summary_slug}}".to_string()),
            refactoring: Some("refactoring/{{jira_key}}-{{summary_slug}}".to_string()),
            chore: Some("chore/{{jira_key}}-{{summary_slug}}".to_string()),
        }
    }
}

/// Commit templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTemplates {
    /// Default commit template
    #[serde(default = "CommitTemplates::default_commit_template")]
    pub default: String,
    /// Whether to use scope for commit messages (when no ticket id)
    ///
    /// When `true`, uses Conventional Commits format: `{commit_type}({scope}): {title}`
    /// When `false`, uses simple format: `# {title}`
    #[serde(default = "default_use_scope")]
    pub use_scope: bool,
}

fn default_use_scope() -> bool {
    false // Keep backward compatibility
}

impl CommitTemplates {
    /// Get default commit template
    pub fn default_commit_template() -> String {
        r#"{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}{{#if use_scope}}{{commit_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira_key}}Closes {{jira_key}}{{/if}}"#
            .to_string()
    }
}

impl Default for CommitTemplates {
    fn default() -> Self {
        Self {
            default: CommitTemplates::default_commit_template(),
            use_scope: default_use_scope(),
        }
    }
}

/// PR templates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestsTemplates {
    /// Default PR template
    #[serde(default = "PullRequestsTemplates::default_pull_request_template")]
    pub default: String,
}

impl PullRequestsTemplates {
    /// Get default PR template
    pub fn default_pull_request_template() -> String {
        r#"
# PR Ready

## Types of changes

{{#each change_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short_description}}
#### Short description:

{{short_description}}
{{/if}}

{{#if jira_key}}
{{#if jira_service_address}}
#### Jira Link:

{{jira_service_address}}/browse/{{jira_key}}
{{/if}}
{{/if}}

{{#if dependency}}
#### Dependency

{{dependency}}
{{/if}}
"#
        .trim_start()
        .to_string()
    }
}

impl Default for PullRequestsTemplates {
    fn default() -> Self {
        Self {
            default: PullRequestsTemplates::default_pull_request_template(),
        }
    }
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            engine: default_engine(),
            branch: BranchTemplates::default(),
            commit: CommitTemplates::default(),
            pull_requests: PullRequestsTemplates::default(),
        }
    }
}

impl TemplateConfig {
    /// Load template configuration
    ///
    /// Loads templates from configuration file, with project-level config overriding global config.
    pub fn load() -> Result<Self> {
        // Try to load from project-level config first
        if let Ok(project_config) = Self::load_project() {
            return Ok(project_config);
        }

        // Fall back to global config
        if let Ok(global_config) = Self::load_global() {
            return Ok(global_config);
        }

        // Return default config if no config file exists
        Ok(Self::default())
    }

    /// Load project-level template config
    fn load_project() -> Result<Self> {
        // Check for .workflow/config.toml in project root
        let project_config_path = std::env::current_dir()
            .wrap_err("Failed to get current directory")?
            .join(".workflow")
            .join("config.toml");

        if !project_config_path.exists() {
            color_eyre::eyre::bail!("Project config not found");
        }

        // Parse TOML and extract template section
        let value: toml::Value = FileReader::new(&project_config_path).toml()?;

        // Extract template section if exists
        if let Some(template_section) = value.get("template") {
            let config: TemplateConfig = toml::from_str(
                &toml::to_string(template_section)
                    .wrap_err("Failed to serialize template section")?,
            )
            .wrap_err("Failed to parse template config")?;
            Ok(config)
        } else {
            color_eyre::eyre::bail!("No template section in project config")
        }
    }

    /// Load global template config
    fn load_global() -> Result<Self> {
        // Try to load from workflow.toml
        let config_path =
            Paths::workflow_config().wrap_err("Failed to get workflow config path")?;

        if !config_path.exists() {
            color_eyre::eyre::bail!("Global config not found");
        }

        let value: toml::Value = FileReader::new(&config_path).toml()?;

        // Extract template section if exists
        if let Some(template_section) = value.get("template") {
            let config: TemplateConfig = toml::from_str(
                &toml::to_string(template_section)
                    .wrap_err("Failed to serialize template section")?,
            )
            .wrap_err("Failed to parse template config")?;
            Ok(config)
        } else {
            color_eyre::eyre::bail!("No template section in global config")
        }
    }

    /// Load branch template
    ///
    /// Loads branch template based on JIRA ticket type (feature/bugfix/hotfix) or uses default.
    /// This is the legacy function for backward compatibility.
    pub fn load_branch_template(jira_type: Option<&str>) -> Result<String> {
        let config = Self::load()?;

        // Select template based on JIRA type
        let template = match jira_type {
            Some("Feature") | Some("Story") | Some("Epic") => {
                config.branch.feature.as_ref().unwrap_or(&config.branch.default)
            }
            Some("Bug") => config.branch.bugfix.as_ref().unwrap_or(&config.branch.default),
            Some("Hotfix") => config.branch.hotfix.as_ref().unwrap_or(&config.branch.default),
            _ => &config.branch.default,
        };

        Ok(template.clone())
    }

    /// Load branch template by branch type
    ///
    /// Loads branch template based on branch type string (feature/bugfix/refactoring/hotfix/chore).
    pub fn load_branch_template_by_type(branch_type: Option<&str>) -> Result<String> {
        let config = Self::load()?;

        // Select template based on branch type
        let template = match branch_type {
            Some("feature") => config.branch.feature.as_ref().unwrap_or(&config.branch.default),
            Some("bugfix") => config.branch.bugfix.as_ref().unwrap_or(&config.branch.default),
            Some("hotfix") => config.branch.hotfix.as_ref().unwrap_or(&config.branch.default),
            Some("refactoring") => {
                config.branch.refactoring.as_ref().unwrap_or(&config.branch.default)
            }
            Some("chore") => config.branch.chore.as_ref().unwrap_or(&config.branch.default),
            _ => &config.branch.default,
        };

        Ok(template.clone())
    }

    /// Load commit template
    pub fn load_commit_template() -> Result<String> {
        let config = Self::load()?;
        Ok(config.commit.default.clone())
    }

    /// Load PR template
    pub fn load_pull_request_template() -> Result<String> {
        let config = Self::load()?;
        Ok(config.pull_requests.default.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // ==================== Test Cases ====================

    /// 测试默认配置加载
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig::default() 能够创建包含所有默认值的配置实例。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 方法创建默认配置
    /// 2. 验证默认引擎为 "handlebars"
    /// 3. 验证默认分支模板格式正确
    /// 4. 验证默认提交模板包含必要的占位符
    /// 5. 验证默认 PR 模板包含必要的内容
    ///
    /// ## 预期结果
    /// - 默认引擎为 "handlebars"
    /// - 默认分支模板为 "{{jira_key}}-{{summary_slug}}"
    /// - feature、bugfix、hotfix 分支模板格式正确
    /// - 提交模板包含 {{jira_key}} 和 {{subject}}
    /// - PR 模板包含 "PR Ready" 和 "change_types"
    #[test]
    fn test_load_default_config() {
        let config = TemplateConfig::default();

        // Assert: 验证默认引擎
        assert_eq!(config.engine, "handlebars");

        // Assert: 验证默认分支模板
        assert_eq!(config.branch.default, "{{jira_key}}-{{summary_slug}}");
        assert_eq!(
            config.branch.feature,
            Some("feature/{{jira_key}}-{{summary_slug}}".to_string())
        );
        assert_eq!(
            config.branch.bugfix,
            Some("bugfix/{{jira_key}}-{{summary_slug}}".to_string())
        );

        // Assert: 验证默认提交模板
        assert!(config.commit.default.contains("{{jira_key}}"));
        assert!(config.commit.default.contains("{{subject}}"));
        assert_eq!(config.commit.use_scope, false);

        // Assert: 验证默认 PR 模板
        assert!(config.pull_requests.default.contains("PR Ready"));
        assert!(config.pull_requests.default.contains("change_types"));
    }

    /// 测试配置结构体创建
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig 结构体能够使用自定义值创建配置实例。
    ///
    /// ## 测试场景
    /// 1. 创建自定义的分支模板、提交模板和 PR 模板
    /// 2. 使用这些模板创建 TemplateConfig 实例
    /// 3. 验证所有字段值正确设置
    ///
    /// ## 预期结果
    /// - 配置实例创建成功
    /// - 所有模板字段值与创建时传入的值一致
    /// - 引擎类型正确设置
    #[test]
    fn test_config_struct_creation() {
        let branch_templates = BranchTemplates {
            default: "custom-{{jira_key}}".to_string(),
            feature: Some("feat/{{jira_key}}".to_string()),
            bugfix: Some("fix/{{jira_key}}".to_string()),
            hotfix: Some("hotfix/{{jira_key}}".to_string()),
            refactoring: Some("refactor/{{jira_key}}".to_string()),
            chore: Some("chore/{{jira_key}}".to_string()),
        };

        let commit_templates = CommitTemplates {
            default: "{{commit_type}}: {{subject}}".to_string(),
            use_scope: true,
        };

        let pr_templates = PullRequestsTemplates {
            default: "Custom PR template".to_string(),
        };

        let config = TemplateConfig {
            engine: "custom_engine".to_string(),
            branch: branch_templates.clone(),
            commit: commit_templates.clone(),
            pull_requests: pr_templates.clone(),
        };

        // Assert: 验证字段访问
        assert_eq!(config.engine, "custom_engine");
        assert_eq!(config.branch.default, "custom-{{jira_key}}");
        assert_eq!(config.branch.feature, Some("feat/{{jira_key}}".to_string()));
        assert_eq!(config.commit.default, "{{commit_type}}: {{subject}}");
        assert_eq!(config.commit.use_scope, true);
        assert_eq!(config.pull_requests.default, "Custom PR template");
    }

    /// 测试分支模板默认实现
    ///
    /// ## 测试目的
    /// 验证 BranchTemplates::default() 能够创建包含所有默认分支模板的实例。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 方法创建默认分支模板
    /// 2. 验证默认模板格式正确
    /// 3. 验证所有分支类型（feature、bugfix、hotfix、refactoring、chore）的模板都存在
    ///
    /// ## 预期结果
    /// - 默认模板为 "{{jira_key}}-{{summary_slug}}"
    /// - feature 模板为 "feature/{{jira_key}}-{{summary_slug}}"
    /// - bugfix 模板为 "bugfix/{{jira_key}}-{{summary_slug}}"
    /// - hotfix 模板为 "hotfix/{{jira_key}}-{{summary_slug}}"
    /// - refactoring 和 chore 模板格式正确
    #[test]
    fn test_branch_templates_default() {
        let branch_templates = BranchTemplates::default();

        assert_eq!(branch_templates.default, "{{jira_key}}-{{summary_slug}}");
        assert_eq!(
            branch_templates.feature,
            Some("feature/{{jira_key}}-{{summary_slug}}".to_string())
        );
        assert_eq!(
            branch_templates.bugfix,
            Some("bugfix/{{jira_key}}-{{summary_slug}}".to_string())
        );
        assert_eq!(
            branch_templates.hotfix,
            Some("hotfix/{{jira_key}}-{{summary_slug}}".to_string())
        );
        assert_eq!(
            branch_templates.refactoring,
            Some("refactoring/{{jira_key}}-{{summary_slug}}".to_string())
        );
        assert_eq!(
            branch_templates.chore,
            Some("chore/{{jira_key}}-{{summary_slug}}".to_string())
        );
    }

    /// 测试提交模板默认实现
    ///
    /// ## 测试目的
    /// 验证 CommitTemplates::default() 能够创建包含默认提交模板的实例。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 方法创建默认提交模板
    /// 2. 验证默认模板包含必要的占位符
    /// 3. 验证 use_scope 默认值为 false
    /// 4. 验证 default_commit_template() 方法返回的模板格式正确
    ///
    /// ## 预期结果
    /// - 默认模板包含 {{jira_key}}、{{subject}}、{{commit_type}} 占位符
    /// - use_scope 为 false
    /// - default_commit_template() 返回的模板包含必要的占位符
    #[test]
    fn test_commit_templates_default() {
        let commit_templates = CommitTemplates::default();

        assert!(commit_templates.default.contains("{{jira_key}}"));
        assert!(commit_templates.default.contains("{{subject}}"));
        assert!(commit_templates.default.contains("{{commit_type}}"));
        assert_eq!(commit_templates.use_scope, false);

        // Arrange: 准备测试默认模板方法
        let default_template = CommitTemplates::default_commit_template();
        assert!(default_template.contains("{{jira_key}}"));
        assert!(default_template.contains("{{subject}}"));
    }

    /// 测试 PR 模板默认实现
    ///
    /// ## 测试目的
    /// 验证 PullRequestsTemplates::default() 能够创建包含默认 PR 模板的实例。
    ///
    /// ## 测试场景
    /// 1. 调用 default() 方法创建默认 PR 模板
    /// 2. 验证默认模板包含必要的内容和占位符
    /// 3. 验证 default_pull_request_template() 方法返回的模板格式正确
    ///
    /// ## 预期结果
    /// - 默认模板包含 "PR Ready"、"Types of changes" 和 "{{#each change_types}}" 占位符
    /// - default_pull_request_template() 返回的模板包含 "PR Ready" 和 "{{jira_key}}"
    #[test]
    fn test_pr_templates_default() {
        let pr_templates = PullRequestsTemplates::default();

        assert!(pr_templates.default.contains("PR Ready"));
        assert!(pr_templates.default.contains("Types of changes"));
        assert!(pr_templates.default.contains("{{#each change_types}}"));

        // Arrange: 准备测试默认模板方法
        let default_template = PullRequestsTemplates::default_pull_request_template();
        assert!(default_template.contains("PR Ready"));
        assert!(default_template.contains("{{jira_key}}"));
    }

    /// 测试配置序列化
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig 能够正确序列化为 JSON 格式。
    ///
    /// ## 测试场景
    /// 1. 创建默认配置实例
    /// 2. 使用 serde_json::to_string() 序列化为 JSON
    /// 3. 验证序列化成功
    /// 4. 验证 JSON 字符串包含预期的字段
    ///
    /// ## 预期结果
    /// - 序列化成功（返回 Ok）
    /// - JSON 字符串包含 "engine"、"branch"、"commit"、"pull_requests" 字段
    #[test]
    fn test_config_serialization() -> Result<()> {
        let config = TemplateConfig::default();

        // Arrange: 准备测试序列化为 JSON
        let json_result = serde_json::to_string(&config);
        assert!(json_result.is_ok());

        let json_str =
            json_result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
        assert!(json_str.contains("handlebars"));
        assert!(json_str.contains("jira_key"));

        // Arrange: 准备测试序列化为 TOML
        let toml_result = toml::to_string(&config);
        assert!(toml_result.is_ok());

        let toml_str =
            toml_result.map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
        assert!(toml_str.contains("engine"));
        assert!(toml_str.contains("handlebars"));
        Ok(())
    }

    /// 测试配置反序列化
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig 能够从 JSON 格式正确反序列化。
    ///
    /// ## 测试场景
    /// 1. 准备包含所有配置字段的 JSON 数据
    /// 2. 使用 serde_json::from_str() 反序列化为 TemplateConfig
    /// 3. 验证反序列化成功
    /// 4. 验证所有字段值正确
    ///
    /// ## 预期结果
    /// - 反序列化成功（返回 Ok）
    /// - 所有字段值与 JSON 数据一致
    /// - use_scope 等布尔值正确解析
    #[test]
    fn test_config_deserialization() -> Result<()> {
        // Arrange: 准备测试从 JSON 反序列化
        let json_config = json!({
            "engine": "test_engine",
            "branch": {
                "default": "test-{{jira_key}}",
                "feature": "feat/{{jira_key}}"
            },
            "commit": {
                "default": "{{commit_type}}: {{subject}}",
                "use_scope": true
            },
            "pull_requests": {
                "default": "Test PR template"
            }
        });

        let config_result: Result<TemplateConfig, _> = serde_json::from_value(json_config);
        assert!(config_result.is_ok());

        let config = config_result
            .map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
        assert_eq!(config.engine, "test_engine");
        assert_eq!(config.branch.default, "test-{{jira_key}}");
        assert_eq!(config.branch.feature, Some("feat/{{jira_key}}".to_string()));
        assert_eq!(config.commit.use_scope, true);

        // Arrange: 准备测试从 TOML 反序列化
        let toml_str = r#"
engine = "toml_engine"

[branch]
default = "toml-{{jira_key}}"

[commit]
default = "TOML: {{subject}}"
use_scope = false

[pull_requests]
default = "TOML PR template"
"#;

        let toml_config_result: Result<TemplateConfig, _> = toml::from_str(toml_str);
        assert!(toml_config_result.is_ok());

        let toml_config = toml_config_result
            .map_err(|e| color_eyre::eyre::eyre!("operation should succeed: {}", e))?;
        assert_eq!(toml_config.engine, "toml_engine");
        assert_eq!(toml_config.branch.default, "toml-{{jira_key}}");
        assert_eq!(toml_config.commit.use_scope, false);
        Ok(())
    }

    /// 测试分支模板按 JIRA 类型加载
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig::load_branch_template() 能够根据 JIRA 类型加载对应的分支模板。
    ///
    /// ## 测试场景
    /// 1. 使用不同的 JIRA 类型（Feature、Bug、Hotfix）调用 load_branch_template()
    /// 2. 使用 None 调用 load_branch_template() 获取默认模板
    /// 3. 验证方法能够正常处理各种输入（不 panic）
    ///
    /// ## 预期结果
    /// - 方法执行不产生 panic
    /// - 如果成功返回模板，模板不为空
    /// - 如果返回错误，错误是预期的（测试环境中可能没有配置文件）
    ///
    /// ## 注意
    /// 此方法需要实际的配置文件，在测试环境中可能返回错误，这是正常的。
    #[test]
    fn test_load_branch_template_by_jira_type() {
        // 由于这个方法需要实际的配置文件，我们主要测试它不会 panic
        // 在非配置环境中，它应该返回错误或默认值

        let result_feature = TemplateConfig::load_branch_template(Some("Feature"));
        let result_bug = TemplateConfig::load_branch_template(Some("Bug"));
        let result_hotfix = TemplateConfig::load_branch_template(Some("Hotfix"));
        let result_default = TemplateConfig::load_branch_template(None);

        // 在没有配置文件的环境中，这些调用可能会失败，但不应该 panic
        // 我们主要验证方法能正常处理各种输入
        for result in [result_feature, result_bug, result_hotfix, result_default] {
            match result {
                Ok(template) => assert!(!template.is_empty()),
                Err(_) => {
                    // 在测试环境中没有配置文件是正常的
                }
            }
        }
    }

    /// 测试分支模板按分支类型加载
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig::load_branch_template_by_type() 能够根据分支类型加载对应的模板。
    ///
    /// ## 测试场景
    /// 1. 使用不同的分支类型（feature、bugfix、hotfix）调用 load_branch_template_by_type()
    /// 2. 使用 None 调用获取默认模板
    /// 3. 验证方法能够正常处理各种输入（不 panic）
    ///
    /// ## 预期结果
    /// - 方法执行不产生 panic
    /// - 如果成功返回模板，模板不为空
    /// - 如果返回错误，错误是预期的（测试环境中可能没有配置文件）
    ///
    /// ## 注意
    /// 此方法需要实际的配置文件，在测试环境中可能返回错误，这是正常的。
    #[test]
    fn test_load_branch_template_by_branch_type() {
        // Arrange: 准备测试各种分支类型
        let results = [
            TemplateConfig::load_branch_template_by_type(Some("feature")),
            TemplateConfig::load_branch_template_by_type(Some("bugfix")),
            TemplateConfig::load_branch_template_by_type(Some("hotfix")),
            TemplateConfig::load_branch_template_by_type(Some("refactoring")),
            TemplateConfig::load_branch_template_by_type(Some("chore")),
            TemplateConfig::load_branch_template_by_type(Some("unknown")),
            TemplateConfig::load_branch_template_by_type(None),
        ];

        // Assert: 验证方法能处理各种输入而不 panic
        for result in results {
            match result {
                Ok(template) => assert!(!template.is_empty()),
                Err(_) => {
                    // 在测试环境中没有配置文件是正常的
                }
            }
        }
    }

    /// 测试提交模板加载
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig::load_commit_template() 能够正确加载提交模板。
    ///
    /// ## 测试场景
    /// 1. 调用 load_commit_template() 方法加载提交模板
    /// 2. 验证方法能够正常调用（不 panic）
    /// 3. 如果成功返回模板，验证模板包含必要的占位符
    ///
    /// ## 预期结果
    /// - 方法执行不产生 panic
    /// - 如果成功返回模板，模板不为空且包含占位符
    /// - 如果返回错误，错误是预期的（测试环境中可能没有配置文件）
    ///
    /// ## 注意
    /// 此方法需要实际的配置文件，在测试环境中可能返回错误，这是正常的。
    #[test]
    fn test_load_commit_template() {
        let result = TemplateConfig::load_commit_template();

        // Assert: 验证方法能正常调用
        match result {
            Ok(template) => {
                assert!(!template.is_empty());
                // 默认模板应该包含这些占位符
                assert!(template.contains("{{") && template.contains("}}"));
            }
            Err(_) => {
                // 在测试环境中没有配置文件是正常的
            }
        }
    }

    /// 测试 PR 模板加载
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig::load_pull_request_template() 能够正确加载 PR 模板。
    ///
    /// ## 测试场景
    /// 1. 调用 load_pull_request_template() 方法加载 PR 模板
    /// 2. 验证方法能够正常调用（不 panic）
    /// 3. 如果成功返回模板，验证模板包含必要的占位符
    ///
    /// ## 预期结果
    /// - 方法执行不产生 panic
    /// - 如果成功返回模板，模板不为空且包含占位符（{{ 和 }}）
    /// - 如果返回错误，错误是预期的（测试环境中可能没有配置文件）
    ///
    /// ## 注意
    /// 此方法需要实际的配置文件，在测试环境中可能返回错误，这是正常的。
    #[test]
    fn test_load_pull_request_template() {
        let result = TemplateConfig::load_pull_request_template();

        // Assert: 验证方法能正常调用
        match result {
            Ok(template) => {
                assert!(!template.is_empty());
                // 默认模板应该包含这些占位符
                assert!(template.contains("{{") && template.contains("}}"));
            }
            Err(_) => {
                // 在测试环境中没有配置文件是正常的
            }
        }
    }

    /// 测试配置克隆功能
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig 实现了 Clone trait，能够正确克隆配置实例。
    ///
    /// ## 测试场景
    /// 1. 创建默认配置实例
    /// 2. 调用 clone() 方法克隆配置
    /// 3. 验证克隆实例的所有字段与原始实例一致
    ///
    /// ## 预期结果
    /// - 成功克隆配置实例
    /// - 克隆实例的 engine、branch、commit、pull_requests 字段与原始实例相同
    #[test]
    fn test_config_clone() {
        let original_config = TemplateConfig::default();
        let cloned_config = original_config.clone();

        // Assert: 验证克隆的配置与原始配置相等
        assert_eq!(original_config.engine, cloned_config.engine);
        assert_eq!(original_config.branch.default, cloned_config.branch.default);
        assert_eq!(original_config.commit.default, cloned_config.commit.default);
        assert_eq!(
            original_config.pull_requests.default,
            cloned_config.pull_requests.default
        );
    }

    /// 测试配置调试输出
    ///
    /// ## 测试目的
    /// 验证 TemplateConfig 实现了 Debug trait，能够通过 format!("{:?}", config) 格式化输出调试信息。
    ///
    /// ## 测试场景
    /// 1. 创建默认配置实例
    /// 2. 使用 Debug 格式化输出
    /// 3. 验证输出包含配置类型名称
    ///
    /// ## 预期结果
    /// - Debug 格式化输出包含 "TemplateConfig"
    /// - 输出不为空
    #[test]
    fn test_config_debug() {
        let config = TemplateConfig::default();

        // Arrange: 准备测试 Debug 实现
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("TemplateConfig"));
        assert!(debug_str.contains("handlebars"));
        assert!(debug_str.contains("jira_key"));

        // Arrange: 准备测试各个子结构的 Debug 实现
        let branch_debug = format!("{:?}", config.branch);
        assert!(branch_debug.contains("BranchTemplates"));

        let commit_debug = format!("{:?}", config.commit);
        assert!(commit_debug.contains("CommitTemplates"));

        let pr_debug = format!("{:?}", config.pull_requests);
        assert!(pr_debug.contains("PullRequestsTemplates"));
    }
}
