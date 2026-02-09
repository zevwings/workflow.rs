//! 提交消息模板变量
//!
//! 描述提交的领域属性，用于生成提交消息。

use serde::{Deserialize, Serialize};

/// 提交消息模板变量
///
/// 描述提交的领域属性，用于生成提交消息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitTemplateVars {
    /// 提交类型（例如 "feat", "fix", "docs"）
    pub commit_type: String,
    /// 提交范围（可选）
    pub scope: Option<String>,
    /// 提交主题
    pub subject: String,
    /// 提交正文（可选）
    pub body: Option<String>,
    /// JIRA 工单键（可选）
    pub jira_key: Option<String>,
    /// 是否使用范围（当没有工单 ID 时）
    ///
    /// 该值来自配置并传递给模板
    pub use_scope: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_template_vars_serialize() -> Result<(), serde_json::Error> {
        let vars = CommitTemplateVars {
            commit_type: "feat".to_string(),
            scope: Some("auth".to_string()),
            subject: "add login feature".to_string(),
            body: Some("Implement OAuth2 login flow".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            use_scope: true,
        };

        let json = serde_json::to_string(&vars)?;
        assert!(json.contains("\"commit_type\":\"feat\""));
        assert!(json.contains("\"scope\":\"auth\""));
        assert!(json.contains("\"subject\":\"add login feature\""));
        assert!(json.contains("\"use_scope\":true"));

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_deserialize() -> Result<(), serde_json::Error> {
        let json = r#"{
            "commit_type": "fix",
            "scope": "api",
            "subject": "fix null pointer exception",
            "body": "Handle edge case when user is null",
            "jira_key": "BUG-456",
            "use_scope": false
        }"#;

        let vars: CommitTemplateVars = serde_json::from_str(json)?;
        assert_eq!(vars.commit_type, "fix");
        assert_eq!(vars.scope, Some("api".to_string()));
        assert_eq!(vars.subject, "fix null pointer exception");
        assert_eq!(
            vars.body,
            Some("Handle edge case when user is null".to_string())
        );
        assert_eq!(vars.jira_key, Some("BUG-456".to_string()));
        assert!(!vars.use_scope);

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_deserialize_minimal() -> Result<(), serde_json::Error> {
        let json = r#"{
            "commit_type": "docs",
            "subject": "update README",
            "use_scope": false
        }"#;

        let vars: CommitTemplateVars = serde_json::from_str(json)?;
        assert_eq!(vars.commit_type, "docs");
        assert_eq!(vars.scope, None);
        assert_eq!(vars.subject, "update README");
        assert_eq!(vars.body, None);
        assert_eq!(vars.jira_key, None);

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_equality() -> Result<(), serde_json::Error> {
        let vars1 = CommitTemplateVars {
            commit_type: "feat".to_string(),
            scope: None,
            subject: "add feature".to_string(),
            body: None,
            jira_key: None,
            use_scope: false,
        };

        let vars2 = CommitTemplateVars {
            commit_type: "feat".to_string(),
            scope: None,
            subject: "add feature".to_string(),
            body: None,
            jira_key: None,
            use_scope: false,
        };

        assert_eq!(vars1, vars2);

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_clone() -> Result<(), serde_json::Error> {
        let vars = CommitTemplateVars {
            commit_type: "refactor".to_string(),
            scope: Some("core".to_string()),
            subject: "improve performance".to_string(),
            body: Some("Optimize database queries".to_string()),
            jira_key: Some("PERF-001".to_string()),
            use_scope: true,
        };

        let cloned = vars.clone();
        assert_eq!(vars, cloned);

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_roundtrip() -> Result<(), serde_json::Error> {
        let original = CommitTemplateVars {
            commit_type: "test".to_string(),
            scope: Some("unit".to_string()),
            subject: "add unit tests".to_string(),
            body: Some("Cover edge cases".to_string()),
            jira_key: Some("TEST-123".to_string()),
            use_scope: true,
        };

        let json = serde_json::to_string(&original)?;
        let deserialized: CommitTemplateVars = serde_json::from_str(&json)?;
        assert_eq!(original, deserialized);

        Ok(())
    }

    #[test]
    fn test_commit_template_vars_with_chinese_content() -> Result<(), serde_json::Error> {
        let vars = CommitTemplateVars {
            commit_type: "feat".to_string(),
            scope: Some("用户模块".to_string()),
            subject: "实现用户注册功能".to_string(),
            body: Some("包含邮箱验证和密码加密".to_string()),
            jira_key: Some("PROJ-123".to_string()),
            use_scope: true,
        };

        let json = serde_json::to_string(&vars)?;
        let deserialized: CommitTemplateVars = serde_json::from_str(&json)?;
        assert_eq!(vars, deserialized);

        Ok(())
    }
}
