//! Template 配置测试
//!
//! 测试 Template 配置相关的功能，包括：
//! - 加载配置
//! - 验证配置
//! - 默认值

use pretty_assertions::assert_eq;
use workflow::template::config::TemplateConfig;

// ==================== 默认配置测试 ====================

#[test]
fn test_template_config_default() {
    // 测试默认配置
    let config = TemplateConfig::default();

    assert_eq!(config.engine, "handlebars");
    assert!(!config.branch.default.is_empty());
}

#[test]
fn test_branch_templates_default() {
    // 测试分支模板默认值
    let branch_templates = workflow::template::config::BranchTemplates::default();

    assert_eq!(branch_templates.default, "{{jira_key}}-{{summary_slug}}");
    assert!(branch_templates.feature.is_some());
    assert!(branch_templates.bugfix.is_some());
    assert!(branch_templates.hotfix.is_some());
    assert!(branch_templates.refactoring.is_some());
    assert!(branch_templates.chore.is_some());
}

#[test]
fn test_commit_templates_default() {
    // 测试提交模板默认值
    let commit_templates = workflow::template::config::CommitTemplates::default();

    assert!(!commit_templates.default.is_empty());
    assert!(!commit_templates.use_scope); // 默认应该是 false
}

#[test]
fn test_commit_templates_default_template() {
    // 测试默认提交模板内容
    let default_template = workflow::template::config::CommitTemplates::default_commit_template();

    // 应该包含模板变量
    assert!(default_template.contains("{{jira_key}}"));
    assert!(default_template.contains("{{subject}}"));
}

// ==================== 配置序列化测试 ====================

#[test]
fn test_template_config_serialization() {
    // 测试配置序列化
    let config = TemplateConfig::default();
    let serialized = toml::to_string(&config).unwrap();

    // 验证序列化结果包含关键字段
    assert!(serialized.contains("engine"));
    assert!(serialized.contains("branch"));
    assert!(serialized.contains("commit"));
}

#[test]
fn test_template_config_deserialization() {
    // 测试配置反序列化
    let toml_str = r#"
engine = "handlebars"

[branch]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"

[commit]
default = "{{jira_key}}: {{subject}}"
use_scope = false
"#;

    let config: TemplateConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.engine, "handlebars");
    assert_eq!(config.branch.default, "{{jira_key}}-{{summary_slug}}");
    assert!(config.branch.feature.is_some());
    assert!(!config.commit.use_scope);
}

// ==================== 配置验证测试 ====================

#[test]
fn test_template_config_with_custom_engine() {
    // 测试自定义引擎配置
    let toml_str = r#"
engine = "handlebars"

[branch]
default = "custom-{{jira_key}}"
"#;

    let config: TemplateConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(config.engine, "handlebars");
    assert_eq!(config.branch.default, "custom-{{jira_key}}");
}

#[test]
fn test_commit_templates_with_use_scope() {
    // 测试 use_scope 配置
    let toml_str = r#"
[commit]
default = "{{commit_type}}({{scope}}): {{subject}}"
use_scope = true
"#;

    let config: TemplateConfig = toml::from_str(toml_str).unwrap();
    assert!(config.commit.use_scope);
}
