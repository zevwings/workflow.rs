//! 分支命名模板变量
//!
//! 描述分支的领域属性，用于生成分支名。

use serde::{Deserialize, Serialize};

/// 分支命名模板变量
///
/// 描述分支的领域属性，用于生成分支名。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchTemplateVars {
    /// User prefix (e.g., "zw")
    pub prefix: Option<String>,
    /// JIRA ticket key (e.g., "PROJ-123")
    pub jira_key: Option<String>,
    /// JIRA ticket summary
    pub jira_summary: Option<String>,
    /// JIRA ticket summary as slug (URL-friendly format)
    pub summary_slug: Option<String>,
    /// JIRA ticket type (e.g., "Feature", "Bug")
    pub jira_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_template_vars_serialize() {
        let vars = BranchTemplateVars {
            prefix: Some("zw".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: Some("implement user login function".to_string()),
            summary_slug: Some("implement-user-login".to_string()),
            jira_type: Some("Feature".to_string()),
        };

        let json = serde_json::to_string(&vars).unwrap();
        assert!(json.contains("\"prefix\":\"zw\""));
        assert!(json.contains("\"jira_key\":\"PROJ-123\""));
        assert!(json.contains("\"summary_slug\":\"implement-user-login\""));
    }

    #[test]
    fn test_branch_template_vars_deserialize() {
        let json = r#"{
            "prefix": "zw",
            "jira_key": "PROJ-123",
            "jira_summary": "implement user login function",
            "summary_slug": "implement-user-login",
            "jira_type": "Feature"
        }"#;

        let vars: BranchTemplateVars = serde_json::from_str(json).unwrap();
        assert_eq!(vars.prefix, Some("zw".to_string()));
        assert_eq!(vars.jira_key, Some("PROJ-123".to_string()));
        assert_eq!(vars.jira_summary, Some("implement user login function".to_string()));
        assert_eq!(vars.summary_slug, Some("implement-user-login".to_string()));
        assert_eq!(vars.jira_type, Some("Feature".to_string()));
    }

    #[test]
    fn test_branch_template_vars_deserialize_with_null_fields() {
        let json = r#"{
            "prefix": null,
            "jira_key": "PROJ-456",
            "jira_summary": null,
            "summary_slug": null,
            "jira_type": null
        }"#;

        let vars: BranchTemplateVars = serde_json::from_str(json).unwrap();
        assert_eq!(vars.prefix, None);
        assert_eq!(vars.jira_key, Some("PROJ-456".to_string()));
        assert_eq!(vars.jira_summary, None);
    }

    #[test]
    fn test_branch_template_vars_deserialize_missing_fields() {
        let json = r#"{
            "jira_key": "PROJ-789"
        }"#;

        let vars: BranchTemplateVars = serde_json::from_str(json).unwrap();
        assert_eq!(vars.prefix, None);
        assert_eq!(vars.jira_key, Some("PROJ-789".to_string()));
        assert_eq!(vars.jira_summary, None);
        assert_eq!(vars.summary_slug, None);
        assert_eq!(vars.jira_type, None);
    }

    #[test]
    fn test_branch_template_vars_equality() {
        let vars1 = BranchTemplateVars {
            prefix: Some("zw".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: None,
            summary_slug: None,
            jira_type: None,
        };

        let vars2 = BranchTemplateVars {
            prefix: Some("zw".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: None,
            summary_slug: None,
            jira_type: None,
        };

        assert_eq!(vars1, vars2);
    }

    #[test]
    fn test_branch_template_vars_clone() {
        let vars = BranchTemplateVars {
            prefix: Some("zw".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            jira_summary: Some("Test".to_string()),
            summary_slug: Some("test".to_string()),
            jira_type: Some("Bug".to_string()),
        };

        let cloned = vars.clone();
        assert_eq!(vars, cloned);
    }

    #[test]
    fn test_branch_template_vars_roundtrip() {
        let original = BranchTemplateVars {
            prefix: Some("dev".to_string()),
            jira_key: Some("TEST-001".to_string()),
            jira_summary: Some("test task".to_string()),
            summary_slug: Some("test-task".to_string()),
            jira_type: Some("Task".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: BranchTemplateVars = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }
}
