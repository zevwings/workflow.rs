//! Template 配置相关结构体

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Template configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Template engine type (default: "handlebars")
    /// 暂时只支持 handlebars，不保存到配置文件
    #[serde(default = "default_engine", skip_serializing)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BranchTemplates {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub feature: String,
    /// Bugfix branch template
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub bugfix: String,
    /// Hotfix branch template
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub hotfix: String,
    /// Refactoring branch template
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub refactoring: String,
    /// Chore branch template
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub chore: String,
}

impl Default for BranchTemplates {
    fn default() -> Self {
        Self {
            feature: "{{#if prefix}}{{prefix}}/{{/if}}feature/{{#if jira_key}}{{jira_key}}-{{/if}}{{summary_slug}}"
                .to_string(),
            bugfix: "{{#if prefix}}{{prefix}}/{{/if}}bugfix/{{#if jira_key}}{{jira_key}}-{{/if}}{{summary_slug}}"
                .to_string(),
            hotfix: "{{#if prefix}}{{prefix}}/{{/if}}hotfix/{{#if jira_key}}{{jira_key}}-{{/if}}{{summary_slug}}"
                .to_string(),
            refactoring:
                "{{#if prefix}}{{prefix}}/{{/if}}refactoring/{{#if jira_key}}{{jira_key}}-{{/if}}{{summary_slug}}"
                    .to_string(),
            chore: "{{#if prefix}}{{prefix}}/{{/if}}chore/{{#if jira_key}}{{jira_key}}-{{/if}}{{summary_slug}}"
                .to_string(),
        }
    }
}

/// Commit templates configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitTemplates {
    /// Commit message 模板（Handlebars）
    #[serde(
        default = "CommitTemplates::default_message_template",
        skip_serializing_if = "String::is_empty"
    )]
    pub message: String,
}

impl CommitTemplates {
    /// 默认 commit message 模板
    pub fn default_message_template() -> String {
        r#"{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}{{#if use_scope}}{{commit_type}}{{#if scope}}({{scope}}){{/if}}: {{subject}}{{else}}# {{subject}}{{/if}}{{/if}}

{{#if body}}{{body}}{{/if}}

{{#if jira_key}}Closes {{jira_key}}{{/if}}"#
            .to_string()
    }
}

impl Default for CommitTemplates {
    fn default() -> Self {
        Self {
            message: CommitTemplates::default_message_template(),
        }
    }
}

/// PR templates configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PullRequestsTemplates {
    /// PR 标题模板（Handlebars）
    ///
    /// 可用变量：`jira_key`（可选）、`commit_type`、`scope`（可选）、`summary`。
    /// 示例：`{{#if jira_key}}{{jira_key}}: {{/if}}{{commit_type}}{{#if scope}}({{scope}}){{/if}} - {{summary}}`
    #[serde(
        default = "PullRequestsTemplates::default_title_template",
        skip_serializing_if = "String::is_empty"
    )]
    pub title: String,
    /// PR 正文模板（Handlebars）
    #[serde(
        default = "PullRequestsTemplates::default_body_template",
        skip_serializing_if = "String::is_empty"
    )]
    pub body: String,
}

impl PullRequestsTemplates {
    /// 默认 PR 标题模板：`JIRA: type(scope) - summary` 或 `type(scope) - summary`
    pub fn default_title_template() -> String {
        r#"{{#if jira_key}}{{jira_key}}: {{/if}}{{commit_type}}{{#if scope}}({{scope}}){{/if}} - {{summary}}"#
            .to_string()
    }

    /// 默认 PR 正文模板
    pub fn default_body_template() -> String {
        r#"
# PR Ready

## Types of changes

{{#each change_types}}
- [{{#if this.selected}}x{{else}} {{/if}}] {{this.name}}
{{/each}}

{{#if short_description}}

{{short_description}}
{{/if}}

{{#if jira_key}}
{{#if jira_service_address}}
## Jira Link:

{{jira_service_address}}/browse/{{jira_key}}
{{/if}}
{{/if}}

{{#if dependency}}
## Dependency

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
            title: PullRequestsTemplates::default_title_template(),
            body: PullRequestsTemplates::default_body_template(),
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
    /// 检查模板配置是否为空（即是否为默认值）
    ///
    /// 当所有字段都是默认值时，认为配置为空。
    pub fn is_empty(&self) -> bool {
        *self == TemplateConfig::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_config_is_empty_default() {
        let config = TemplateConfig::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_template_config_is_empty_false_when_customized() {
        let config = TemplateConfig {
            engine: default_engine(),
            branch: BranchTemplates {
                feature: "custom-feature".to_string(),
                ..BranchTemplates::default()
            },
            commit: CommitTemplates::default(),
            pull_requests: PullRequestsTemplates::default(),
        };
        assert!(!config.is_empty());
    }
}
