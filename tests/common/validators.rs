//! 测试数据验证器
//!
//! 提供测试数据验证功能，确保测试数据的完整性和正确性。
//!
//! ## 使用示例
//!
//! ```rust
//! use tests::common::validators::TestDataValidator;
//! use serde_json::json;
//!
//! let data = json!({"key": "value"});
//! data.validate()?;
//! ```

use color_eyre::Result;
use serde_json::Value;

/// 测试数据验证器 trait
///
/// 为测试数据提供验证功能，确保数据的完整性和正确性。
pub trait TestDataValidator {
    /// 验证测试数据
    ///
    /// ## 返回
    /// - `Ok(())` - 验证通过
    /// - `Err` - 验证失败，包含错误信息
    ///
    /// ## 示例
    /// ```rust
    /// let data = json!({"number": 123});
    /// data.validate()?;
    /// ```
    fn validate(&self) -> Result<()>;
}

impl TestDataValidator for Value {
    fn validate(&self) -> Result<()> {
        use color_eyre::eyre::Context;

        // 验证 JSON 值不为空
        if self.is_null() {
            return Err(color_eyre::eyre::eyre!("Test data cannot be null"))
                .wrap_err("Validation failed: null value");
        }

        // 验证对象类型的基本结构
        if let Some(obj) = self.as_object() {
            if obj.is_empty() {
                return Err(color_eyre::eyre::eyre!("Test data object cannot be empty"))
                    .wrap_err("Validation failed: empty object");
            }
        }

        // 验证数组类型的基本结构
        if let Some(arr) = self.as_array() {
            if arr.is_empty() {
                return Err(color_eyre::eyre::eyre!("Test data array cannot be empty"))
                    .wrap_err("Validation failed: empty array");
            }
        }

        Ok(())
    }
}

/// GitHub PR 数据验证器
pub struct GitHubPRValidator;

impl GitHubPRValidator {
    /// 验证 GitHub PR 数据
    ///
    /// ## 参数
    /// * `pr` - GitHub PR JSON 数据
    ///
    /// ## 返回
    /// - `Ok(())` - 验证通过
    /// - `Err` - 验证失败，包含错误信息
    ///
    /// ## 示例
    /// ```rust
    /// let pr = factory.github_pr().build()?;
    /// GitHubPRValidator::validate(&pr)?;
    /// ```
    pub fn validate(pr: &Value) -> Result<()> {
        use color_eyre::eyre::Context;

        // 验证必需字段
        let required_fields = ["number", "title", "state"];
        for field in &required_fields {
            if !pr.get(field).is_some() {
                return Err(color_eyre::eyre::eyre!("Missing required field: {}", field))
                    .wrap_err("GitHub PR validation failed");
            }
        }

        // 验证 number 字段类型
        if let Some(number) = pr.get("number") {
            if !number.is_number() && !number.is_string() {
                return Err(color_eyre::eyre::eyre!(
                    "Field 'number' must be a number or string"
                ))
                .wrap_err("GitHub PR validation failed");
            }
        }

        // 验证 state 字段值
        if let Some(state) = pr.get("state").and_then(|s| s.as_str()) {
            let valid_states = ["open", "closed"];
            if !valid_states.contains(&state) {
                return Err(color_eyre::eyre::eyre!(
                    "Invalid state value: {}. Must be one of: {:?}",
                    state,
                    valid_states
                ))
                .wrap_err("GitHub PR validation failed");
            }
        }

        Ok(())
    }
}

/// Jira Issue 数据验证器
pub struct JiraIssueValidator;

impl JiraIssueValidator {
    /// 验证 Jira Issue 数据
    ///
    /// ## 参数
    /// * `issue` - Jira Issue JSON 数据
    ///
    /// ## 返回
    /// - `Ok(())` - 验证通过
    /// - `Err` - 验证失败，包含错误信息
    ///
    /// ## 示例
    /// ```rust
    /// let issue = factory.jira_issue().build()?;
    /// JiraIssueValidator::validate(&issue)?;
    /// ```
    pub fn validate(issue: &Value) -> Result<()> {
        use color_eyre::eyre::Context;

        // 验证必需字段
        let required_fields = ["key", "fields"];
        for field in &required_fields {
            if !issue.get(field).is_some() {
                return Err(color_eyre::eyre::eyre!("Missing required field: {}", field))
                    .wrap_err("Jira Issue validation failed");
            }
        }

        // 验证 fields 对象
        if let Some(fields) = issue.get("fields") {
            if !fields.is_object() {
                return Err(color_eyre::eyre::eyre!("Field 'fields' must be an object"))
                    .wrap_err("Jira Issue validation failed");
            }

            // 验证 fields 中的必需字段
            let required_field_fields = ["summary", "status"];
            for field in &required_field_fields {
                if !fields.get(field).is_some() {
                    return Err(color_eyre::eyre::eyre!(
                        "Missing required field in 'fields': {}",
                        field
                    ))
                    .wrap_err("Jira Issue validation failed");
                }
            }
        }

        Ok(())
    }
}

/// Git Commit 数据验证器
pub struct GitCommitValidator;

impl GitCommitValidator {
    /// 验证 Git Commit 数据
    ///
    /// ## 参数
    /// * `commit` - Git Commit JSON 数据
    ///
    /// ## 返回
    /// - `Ok(())` - 验证通过
    /// - `Err` - 验证失败，包含错误信息
    ///
    /// ## 示例
    /// ```rust
    /// let commit = factory.git_commit().build()?;
    /// GitCommitValidator::validate(&commit)?;
    /// ```
    pub fn validate(commit: &Value) -> Result<()> {
        use color_eyre::eyre::Context;

        // 验证必需字段
        let required_fields = ["sha", "commit"];
        for field in &required_fields {
            if !commit.get(field).is_some() {
                return Err(color_eyre::eyre::eyre!("Missing required field: {}", field))
                    .wrap_err("Git Commit validation failed");
            }
        }

        // 验证 commit 对象
        if let Some(commit_obj) = commit.get("commit") {
            if !commit_obj.is_object() {
                return Err(color_eyre::eyre::eyre!("Field 'commit' must be an object"))
                    .wrap_err("Git Commit validation failed");
            }

            // 验证 commit 中的必需字段
            let required_commit_fields = ["message", "author"];
            for field in &required_commit_fields {
                if !commit_obj.get(field).is_some() {
                    return Err(color_eyre::eyre::eyre!(
                        "Missing required field in 'commit': {}",
                        field
                    ))
                    .wrap_err("Git Commit validation failed");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_data::factory::TestDataFactory;

    /// 测试 TestDataValidator trait 的基本验证
    #[test]
    fn test_validator_basic() -> Result<()> {
        let data = serde_json::json!({"key": "value"});
        data.validate()?;
        Ok(())
    }

    /// 测试 TestDataValidator 拒绝 null 值
    #[test]
    fn test_validator_rejects_null() {
        let data = serde_json::json!(null);
        assert!(data.validate().is_err());
    }

    /// 测试 TestDataValidator 拒绝空对象
    #[test]
    fn test_validator_rejects_empty_object() {
        let data = serde_json::json!({});
        assert!(data.validate().is_err());
    }

    /// 测试 GitHubPRValidator 验证有效数据
    #[test]
    fn test_github_pr_validator_valid() -> Result<()> {
        let factory = TestDataFactory::new();
        let pr = factory.github_pr().build()?;
        GitHubPRValidator::validate(&pr)?;
        Ok(())
    }

    /// 测试 GitHubPRValidator 拒绝无效数据
    #[test]
    fn test_github_pr_validator_invalid() {
        let invalid_pr = serde_json::json!({});
        assert!(GitHubPRValidator::validate(&invalid_pr).is_err());
    }

    /// 测试 JiraIssueValidator 验证有效数据
    #[test]
    fn test_jira_issue_validator_valid() -> Result<()> {
        let factory = TestDataFactory::new();
        let issue = factory.jira_issue().build()?;
        JiraIssueValidator::validate(&issue)?;
        Ok(())
    }

    /// 测试 JiraIssueValidator 拒绝无效数据
    #[test]
    fn test_jira_issue_validator_invalid() {
        let invalid_issue = serde_json::json!({});
        assert!(JiraIssueValidator::validate(&invalid_issue).is_err());
    }

    /// 测试 GitCommitValidator 验证有效数据
    #[test]
    fn test_git_commit_validator_valid() -> Result<()> {
        let factory = TestDataFactory::new();
        let commit = factory.git_commit().build()?;
        GitCommitValidator::validate(&commit)?;
        Ok(())
    }

    /// 测试 GitCommitValidator 拒绝无效数据
    #[test]
    fn test_git_commit_validator_invalid() {
        let invalid_commit = serde_json::json!({});
        assert!(GitCommitValidator::validate(&invalid_commit).is_err());
    }
}
