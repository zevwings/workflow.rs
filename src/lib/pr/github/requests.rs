use serde::Serialize;
use serde_with::skip_serializing_none;

/// 创建 Pull Request 请求
#[derive(Debug, Serialize)]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
}

/// 合并 Pull Request 请求
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct MergePullRequestRequest {
    pub commit_title: Option<String>,
    pub commit_message: Option<String>,
    pub merge_method: String,
}

/// 更新 Pull Request 请求
#[skip_serializing_none]
#[derive(Debug, Serialize)]
pub struct UpdatePullRequestRequest {
    pub title: Option<String>,
    pub body: Option<String>,
    pub state: Option<String>,
    pub base: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};

    // ==================== Fixtures ====================

    #[fixture]
    fn sample_create_request() -> CreatePullRequestRequest {
        CreatePullRequestRequest {
            title: "Test PR".to_string(),
            body: "Test body".to_string(),
            head: "feature/test".to_string(),
            base: "main".to_string(),
        }
    }

    #[fixture]
    fn sample_merge_request() -> MergePullRequestRequest {
        MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method: "squash".to_string(),
        }
    }

    // ==================== Request Structure Tests ====================

    /// 测试创建 PR 请求结构体（参数化测试）
    ///
    /// ## 测试目的
    /// 验证 CreatePullRequestRequest 结构体能够使用有效字段正确创建。
    ///
    /// ## 测试场景
    /// 使用 fixture 提供的示例请求数据
    ///
    /// ## 预期结果
    /// - 所有字段值正确（title、body、head、base）
    #[rstest]
    fn test_create_request_structure_with_valid_fields_creates_request(
        sample_create_request: CreatePullRequestRequest,
    ) {
        // Arrange: 使用 fixture 提供的请求

        // Act: 验证请求结构
        // (结构验证在 Assert 中完成)

        // Assert: 验证所有字段值正确
        assert_eq!(sample_create_request.title, "Test PR");
        assert_eq!(sample_create_request.body, "Test body");
        assert_eq!(sample_create_request.head, "feature/test");
        assert_eq!(sample_create_request.base, "main");
    }

    /// 测试创建 PR 请求序列化（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 CreatePullRequestRequest 能够正确序列化为 JSON。
    ///
    /// ## 测试场景
    /// 测试正常输入、空字符串、特殊字符、多行文本等各种输入
    ///
    /// ## 预期结果
    /// - JSON 字段存在且值正确
    /// - 序列化/反序列化一致
    #[rstest]
    #[case("Test PR", "Test body", "feature/test", "main")]
    #[case("", "", "", "")]
    #[case(
        "Long Title with Special Chars !@#",
        "Long Body\nwith\nmultiple\nlines",
        "feature/long-branch-name",
        "develop"
    )]
    fn test_create_pr_request_serialization_with_various_inputs_serializes_correctly(
        #[case] title: &str,
        #[case] body: &str,
        #[case] head: &str,
        #[case] base: &str,
    ) -> Result<()> {
        // Arrange: 准备 CreatePullRequestRequest 实例
        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: head.to_string(),
            base: base.to_string(),
        };

        // Act: 序列化为 JSON
        let json_str = serde_json::to_string(&request)?;
        let json_value: serde_json::Value = serde_json::from_str(&json_str)?;
        let obj = json_value
            .as_object()
            .ok_or_else(|| color_eyre::eyre::eyre!("Should be a JSON object"))?;

        // Assert: 验证 JSON 字段存在且值正确
        assert_eq!(obj.get("title").and_then(|v| v.as_str()), Some(title));
        assert_eq!(obj.get("body").and_then(|v| v.as_str()), Some(body));
        assert_eq!(obj.get("head").and_then(|v| v.as_str()), Some(head));
        assert_eq!(obj.get("base").and_then(|v| v.as_str()), Some(base));
        Ok(())
    }

    /// 测试合并 PR 请求结构体（参数化测试）
    ///
    /// ## 测试目的
    /// 验证 MergePullRequestRequest 结构体能够使用有效字段正确创建。
    ///
    /// ## 测试场景
    /// 使用 fixture 提供的示例请求数据
    ///
    /// ## 预期结果
    /// - 所有字段值正确（commit_title、commit_message、merge_method）
    #[rstest]
    fn test_merge_request_structure_with_valid_fields_creates_request(
        sample_merge_request: MergePullRequestRequest,
    ) {
        // Arrange: 使用 fixture 提供的请求

        // Act: 验证请求结构
        // (结构验证在 Assert 中完成)

        // Assert: 验证所有字段值正确
        assert_eq!(sample_merge_request.commit_title, None);
        assert_eq!(sample_merge_request.commit_message, None);
        assert_eq!(sample_merge_request.merge_method, "squash");
    }

    /// 测试合并 PR 请求序列化（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 MergePullRequestRequest 能够正确序列化为 JSON，包括可选字段的处理。
    ///
    /// ## 测试场景
    /// 测试不同的合并方法（squash、merge、rebase）和可选字段（commit_title、commit_message）
    ///
    /// ## 预期结果
    /// - JSON 包含 merge_method
    /// - None 字段被跳过（不包含在 JSON 中）
    #[rstest]
    #[case(None, None, "squash")]
    #[case(Some("Merge PR #123"), Some("Merged via workflow"), "merge")]
    #[case(Some("Custom Title"), None, "rebase")]
    fn test_merge_pr_request_serialization_with_various_options_serializes_correctly_return_ok(
        #[case] commit_title: Option<&str>,
        #[case] commit_message: Option<&str>,
        #[case] merge_method: &str,
    ) -> Result<()> {
        // Arrange: 准备 MergePullRequestRequest 实例
        let request = MergePullRequestRequest {
            commit_title: commit_title.map(|s| s.to_string()),
            commit_message: commit_message.map(|s| s.to_string()),
            merge_method: merge_method.to_string(),
        };

        // Act: 序列化为 JSON
        let json_str = serde_json::to_string(&request)?;

        // Assert: 验证 JSON 包含 merge_method 且 None 字段被跳过
        assert!(
            json_str.contains(merge_method),
            "JSON should contain merge_method"
        );
        if commit_title.is_none() {
            assert!(
                !json_str.contains("commit_title"),
                "None fields should be skipped"
            );
        }
        if commit_message.is_none() {
            assert!(
                !json_str.contains("commit_message"),
                "None fields should be skipped"
            );
        }
        Ok(())
    }

    /// 测试更新 PR 请求序列化（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 UpdatePullRequestRequest 能够正确序列化为 JSON，包括可选字段的处理。
    ///
    /// ## 测试场景
    /// 测试不同的更新选项组合（title、body、state、base）
    ///
    /// ## 预期结果
    /// - 存在的字段包含在 JSON 中
    /// - None 字段被跳过（不包含在 JSON 中）
    #[rstest]
    #[case(None, None, Some("closed"), None)]
    #[case(Some("New Title"), None, None, None)]
    #[case(None, Some("New Body"), None, None)]
    #[case(Some("New Title"), Some("New Body"), None, None)]
    fn test_update_pr_request_serialization_with_various_options_serializes_correctly_return_ok(
        #[case] title: Option<&str>,
        #[case] body: Option<&str>,
        #[case] state: Option<&str>,
        #[case] base: Option<&str>,
    ) -> Result<()> {
        // Arrange: 准备 UpdatePullRequestRequest 实例
        let request = UpdatePullRequestRequest {
            title: title.map(|s| s.to_string()),
            body: body.map(|s| s.to_string()),
            state: state.map(|s| s.to_string()),
            base: base.map(|s| s.to_string()),
        };

        // Act: 序列化为 JSON
        let json_str = serde_json::to_string(&request)?;

        // Assert: 验证存在的字段包含在 JSON 中，None 字段被跳过
        if let Some(t) = title {
            assert!(json_str.contains(t), "JSON should contain title");
        }
        if let Some(b) = body {
            assert!(json_str.contains(b), "JSON should contain body");
        }
        if let Some(s) = state {
            assert!(json_str.contains(s), "JSON should contain state");
        }
        if title.is_none() {
            assert!(
                !json_str.contains("\"title\""),
                "None title should be skipped"
            );
        }
        if body.is_none() {
            assert!(
                !json_str.contains("\"body\""),
                "None body should be skipped"
            );
        }
        if state.is_none() {
            assert!(
                !json_str.contains("\"state\""),
                "None state should be skipped"
            );
        }
        Ok(())
    }

    // ==================== Serialization/Deserialization Boundary Tests ====================

    /// 测试请求边界情况（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证 CreatePullRequestRequest 能够处理边界情况（空字符串、单字符等）。
    ///
    /// ## 测试场景
    /// 测试空字符串和单字符输入
    ///
    /// ## 预期结果
    /// - 序列化成功
    /// - 不会panic或返回错误
    #[rstest]
    #[case("", "", "", "")]
    #[case("a", "b", "c", "d")]
    fn test_request_edge_cases(
        #[case] title: &str,
        #[case] body: &str,
        #[case] head: &str,
        #[case] base: &str,
    ) {
        let request = CreatePullRequestRequest {
            title: title.to_string(),
            body: body.to_string(),
            head: head.to_string(),
            base: base.to_string(),
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok(), "Should handle edge cases");
    }

    /// 测试请求长字符串处理
    ///
    /// ## 测试目的
    /// 验证 CreatePullRequestRequest 能够处理长字符串输入。
    ///
    /// ## 测试场景
    /// 测试包含1000个字符的长字符串
    ///
    /// ## 预期结果
    /// - 序列化成功
    /// - 不会panic或返回错误
    #[test]
    fn test_request_long_strings() {
        // Arrange: 准备测试长字符串的处理
        let long_string = "a".repeat(1000);
        let request = CreatePullRequestRequest {
            title: long_string.clone(),
            body: long_string.clone(),
            head: "feature/test".to_string(),
            base: "main".to_string(),
        };

        let json = serde_json::to_string(&request);
        assert!(json.is_ok(), "Should handle long strings");
    }

    // ==================== Type Safety Tests ====================

    /// 测试请求类型安全
    ///
    /// ## 测试目的
    /// 验证不同类型的请求结构体能够正确区分，确保类型安全。
    ///
    /// ## 测试场景
    /// 创建不同类型的请求并验证可以分别序列化
    ///
    /// ## 预期结果
    /// - 类型正确（通过编译验证）
    /// - 可以分别序列化
    #[test]
    fn test_request_type_safety() {
        // Arrange: 准备测试请求类型的安全性
        let create_request: CreatePullRequestRequest = CreatePullRequestRequest {
            title: "Test".to_string(),
            body: "Test".to_string(),
            head: "feature/test".to_string(),
            base: "main".to_string(),
        };

        let merge_request: MergePullRequestRequest = MergePullRequestRequest {
            commit_title: None,
            commit_message: None,
            merge_method: "squash".to_string(),
        };

        // Assert: 验证类型正确（通过编译验证）

        // Assert: 验证可以分别序列化
        assert!(serde_json::to_string(&create_request).is_ok());
        assert!(serde_json::to_string(&merge_request).is_ok());
    }
}
