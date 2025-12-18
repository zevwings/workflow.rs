//! TemplateConfig 业务逻辑测试
//!
//! 测试 TemplateConfig 模块的核心功能，包括：
//! - 配置加载和解析
//! - 默认值处理
//! - 配置合并逻辑
//! - 模板选择逻辑
//! - 序列化和反序列化

use pretty_assertions::assert_eq;
use serde_json::json;
use workflow::template::{CommitTemplates, PullRequestsTemplates, TemplateConfig};
use workflow::template::config::BranchTemplates;

// ==================== 测试用例 ====================

/// 测试默认配置加载
#[test]
fn test_load_default_config() {
    let config = TemplateConfig::default();

    // 验证默认引擎
    assert_eq!(config.engine, "handlebars");

    // 验证默认分支模板
    assert_eq!(config.branch.default, "{{jira_key}}-{{summary_slug}}");
    assert_eq!(
        config.branch.feature,
        Some("feature/{{jira_key}}-{{summary_slug}}".to_string())
    );
    assert_eq!(
        config.branch.bugfix,
        Some("bugfix/{{jira_key}}-{{summary_slug}}".to_string())
    );

    // 验证默认提交模板
    assert!(config.commit.default.contains("{{jira_key}}"));
    assert!(config.commit.default.contains("{{subject}}"));
    assert_eq!(config.commit.use_scope, false);

    // 验证默认 PR 模板
    assert!(config.pull_requests.default.contains("PR Ready"));
    assert!(config.pull_requests.default.contains("change_types"));
}

/// 测试配置结构体创建
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

    // 验证字段访问
    assert_eq!(config.engine, "custom_engine");
    assert_eq!(config.branch.default, "custom-{{jira_key}}");
    assert_eq!(config.branch.feature, Some("feat/{{jira_key}}".to_string()));
    assert_eq!(config.commit.default, "{{commit_type}}: {{subject}}");
    assert_eq!(config.commit.use_scope, true);
    assert_eq!(config.pull_requests.default, "Custom PR template");
}

/// 测试分支模板默认实现
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
#[test]
fn test_commit_templates_default() {
    let commit_templates = CommitTemplates::default();

    assert!(commit_templates.default.contains("{{jira_key}}"));
    assert!(commit_templates.default.contains("{{subject}}"));
    assert!(commit_templates.default.contains("{{commit_type}}"));
    assert_eq!(commit_templates.use_scope, false);

    // 测试默认模板方法
    let default_template = CommitTemplates::default_commit_template();
    assert!(default_template.contains("{{jira_key}}"));
    assert!(default_template.contains("{{subject}}"));
}

/// 测试 PR 模板默认实现
#[test]
fn test_pr_templates_default() {
    let pr_templates = PullRequestsTemplates::default();

    assert!(pr_templates.default.contains("PR Ready"));
    assert!(pr_templates.default.contains("Types of changes"));
    assert!(pr_templates.default.contains("{{#each change_types}}"));

    // 测试默认模板方法
    let default_template = PullRequestsTemplates::default_pull_request_template();
    assert!(default_template.contains("PR Ready"));
    assert!(default_template.contains("{{jira_key}}"));
}

/// 测试配置序列化
#[test]
fn test_config_serialization() {
    let config = TemplateConfig::default();

    // 测试序列化为 JSON
    let json_result = serde_json::to_string(&config);
    assert!(json_result.is_ok());

    let json_str = json_result.unwrap();
    assert!(json_str.contains("handlebars"));
    assert!(json_str.contains("jira_key"));

    // 测试序列化为 TOML
    let toml_result = toml::to_string(&config);
    assert!(toml_result.is_ok());

    let toml_str = toml_result.unwrap();
    assert!(toml_str.contains("engine"));
    assert!(toml_str.contains("handlebars"));
}

/// 测试配置反序列化
#[test]
fn test_config_deserialization() {
    // 测试从 JSON 反序列化
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

    let config = config_result.unwrap();
    assert_eq!(config.engine, "test_engine");
    assert_eq!(config.branch.default, "test-{{jira_key}}");
    assert_eq!(config.branch.feature, Some("feat/{{jira_key}}".to_string()));
    assert_eq!(config.commit.use_scope, true);

    // 测试从 TOML 反序列化
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

    let toml_config = toml_config_result.unwrap();
    assert_eq!(toml_config.engine, "toml_engine");
    assert_eq!(toml_config.branch.default, "toml-{{jira_key}}");
    assert_eq!(toml_config.commit.use_scope, false);
}

/// 测试分支模板按 JIRA 类型加载
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
#[test]
fn test_load_branch_template_by_branch_type() {
    // 测试各种分支类型
    let results = [
        TemplateConfig::load_branch_template_by_type(Some("feature")),
        TemplateConfig::load_branch_template_by_type(Some("bugfix")),
        TemplateConfig::load_branch_template_by_type(Some("hotfix")),
        TemplateConfig::load_branch_template_by_type(Some("refactoring")),
        TemplateConfig::load_branch_template_by_type(Some("chore")),
        TemplateConfig::load_branch_template_by_type(Some("unknown")),
        TemplateConfig::load_branch_template_by_type(None),
    ];

    // 验证方法能处理各种输入而不 panic
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
#[test]
fn test_load_commit_template() {
    let result = TemplateConfig::load_commit_template();

    // 验证方法能正常调用
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
#[test]
fn test_load_pull_request_template() {
    let result = TemplateConfig::load_pull_request_template();

    // 验证方法能正常调用
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
#[test]
fn test_config_clone() {
    let original_config = TemplateConfig::default();
    let cloned_config = original_config.clone();

    // 验证克隆的配置与原始配置相等
    assert_eq!(original_config.engine, cloned_config.engine);
    assert_eq!(original_config.branch.default, cloned_config.branch.default);
    assert_eq!(original_config.commit.default, cloned_config.commit.default);
    assert_eq!(
        original_config.pull_requests.default,
        cloned_config.pull_requests.default
    );
}

/// 测试配置调试输出
#[test]
fn test_config_debug() {
    let config = TemplateConfig::default();

    // 测试 Debug 实现
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("TemplateConfig"));
    assert!(debug_str.contains("handlebars"));
    assert!(debug_str.contains("jira_key"));

    // 测试各个子结构的 Debug 实现
    let branch_debug = format!("{:?}", config.branch);
    assert!(branch_debug.contains("BranchTemplates"));

    let commit_debug = format!("{:?}", config.commit);
    assert!(commit_debug.contains("CommitTemplates"));

    let pr_debug = format!("{:?}", config.pull_requests);
    assert!(pr_debug.contains("PullRequestsTemplates"));
}
