//! Pull Request 模板变量
//!
//! 描述 PR 的领域属性，用于生成 PR 内容。

use serde::{Deserialize, Serialize};

/// Pull Request 模板变量
///
/// 描述 PR 的领域属性，用于生成 PR 内容。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PullRequestTemplateVars {
    /// JIRA ticket key (optional)
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    pub jira_summary: Option<String>,
    /// JIRA ticket description
    pub jira_description: Option<String>,
    /// JIRA ticket type
    pub jira_type: Option<String>,
    /// JIRA service address (for building links)
    pub jira_service_address: Option<String>,
    /// Change types (array of booleans indicating which types are selected)
    pub change_types: Vec<ChangeTypeItem>,
    /// Short description (optional)
    pub short_description: Option<String>,
    /// Dependency information (optional)
    pub dependency: Option<String>,
}

/// 变更类型项
///
/// 用于 PR 模板中的变更类型选择。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeTypeItem {
    /// Change type name
    pub name: String,
    /// Whether this change type is selected
    pub selected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_type_item_serialize() {
        let item = ChangeTypeItem {
            name: "Feature".to_string(),
            selected: true,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"name\":\"Feature\""));
        assert!(json.contains("\"selected\":true"));
    }

    #[test]
    fn test_change_type_item_deserialize() {
        let json = r#"{"name": "Bug Fix", "selected": false}"#;
        let item: ChangeTypeItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.name, "Bug Fix");
        assert!(!item.selected);
    }

    #[test]
    fn test_pull_request_template_vars_default() {
        let vars = PullRequestTemplateVars::default();
        assert_eq!(vars.jira_key, None);
        assert_eq!(vars.jira_summary, None);
        assert_eq!(vars.jira_description, None);
        assert_eq!(vars.jira_type, None);
        assert_eq!(vars.jira_service_address, None);
        assert!(vars.change_types.is_empty());
        assert_eq!(vars.short_description, None);
        assert_eq!(vars.dependency, None);
    }

    #[test]
    fn test_pull_request_template_vars_serialize() {
        let vars = PullRequestTemplateVars {
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: Some("实现新功能".to_string()),
            jira_description: Some("详细描述".to_string()),
            jira_type: Some("Feature".to_string()),
            jira_service_address: Some("https://jira.example.com".to_string()),
            change_types: vec![
                ChangeTypeItem {
                    name: "Feature".to_string(),
                    selected: true,
                },
                ChangeTypeItem {
                    name: "Bug Fix".to_string(),
                    selected: false,
                },
            ],
            short_description: Some("添加用户认证".to_string()),
            dependency: Some("依赖 PR #100".to_string()),
        };

        let json = serde_json::to_string(&vars).unwrap();
        assert!(json.contains("\"jira_key\":\"PROJ-123\""));
        assert!(json.contains("\"change_types\":["));
    }

    #[test]
    fn test_pull_request_template_vars_deserialize() {
        let json = r#"{
            "jira_key": "PROJ-456",
            "jira_summary": "修复登录问题",
            "jira_description": null,
            "jira_type": "Bug",
            "jira_service_address": "https://jira.example.com",
            "change_types": [
                {"name": "Bug Fix", "selected": true}
            ],
            "short_description": "修复空指针异常",
            "dependency": null
        }"#;

        let vars: PullRequestTemplateVars = serde_json::from_str(json).unwrap();
        assert_eq!(vars.jira_key, Some("PROJ-456".to_string()));
        assert_eq!(vars.jira_summary, Some("修复登录问题".to_string()));
        assert_eq!(vars.jira_description, None);
        assert_eq!(vars.change_types.len(), 1);
        assert!(vars.change_types[0].selected);
    }

    #[test]
    fn test_pull_request_template_vars_deserialize_minimal() {
        let json = r#"{
            "change_types": []
        }"#;

        let vars: PullRequestTemplateVars = serde_json::from_str(json).unwrap();
        assert_eq!(vars.jira_key, None);
        assert!(vars.change_types.is_empty());
    }

    #[test]
    fn test_pull_request_template_vars_equality() {
        let vars1 = PullRequestTemplateVars {
            jira_key: Some("PROJ-123".to_string()),
            change_types: vec![ChangeTypeItem {
                name: "Feature".to_string(),
                selected: true,
            }],
            ..Default::default()
        };

        let vars2 = PullRequestTemplateVars {
            jira_key: Some("PROJ-123".to_string()),
            change_types: vec![ChangeTypeItem {
                name: "Feature".to_string(),
                selected: true,
            }],
            ..Default::default()
        };

        assert_eq!(vars1, vars2);
    }

    #[test]
    fn test_pull_request_template_vars_clone() {
        let vars = PullRequestTemplateVars {
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: Some("Summary".to_string()),
            change_types: vec![ChangeTypeItem {
                name: "Feature".to_string(),
                selected: true,
            }],
            ..Default::default()
        };

        let cloned = vars.clone();
        assert_eq!(vars, cloned);
    }

    #[test]
    fn test_pull_request_template_vars_roundtrip() {
        let original = PullRequestTemplateVars {
            jira_key: Some("TEST-001".to_string()),
            jira_summary: Some("测试摘要".to_string()),
            jira_description: Some("测试描述".to_string()),
            jira_type: Some("Task".to_string()),
            jira_service_address: Some("https://jira.test.com".to_string()),
            change_types: vec![
                ChangeTypeItem {
                    name: "Improvement".to_string(),
                    selected: true,
                },
                ChangeTypeItem {
                    name: "Documentation".to_string(),
                    selected: false,
                },
            ],
            short_description: Some("简短描述".to_string()),
            dependency: Some("依赖信息".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: PullRequestTemplateVars = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_change_type_item_equality() {
        let item1 = ChangeTypeItem {
            name: "Feature".to_string(),
            selected: true,
        };
        let item2 = ChangeTypeItem {
            name: "Feature".to_string(),
            selected: true,
        };
        let item3 = ChangeTypeItem {
            name: "Feature".to_string(),
            selected: false,
        };

        assert_eq!(item1, item2);
        assert_ne!(item1, item3);
    }
}
