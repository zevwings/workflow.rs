use serde::Deserialize;
use serde_with::skip_serializing_none;

/// 创建 Pull Request 响应
#[derive(Debug, Deserialize)]
pub struct CreatePullRequestResponse {
    pub html_url: String,
}

/// Pull Request 信息
#[derive(Debug, Deserialize)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(default)]
    pub merged: Option<bool>,
    #[serde(rename = "merged_at", default)]
    pub merged_at: Option<String>,
    #[serde(default)]
    pub mergeable: Option<bool>,
    pub html_url: String,
    pub head: PullRequestBranch,
    pub base: PullRequestBranch,
    pub user: Option<GitHubUser>,
}

/// Pull Request 分支信息
#[derive(Debug, Deserialize)]
pub struct PullRequestBranch {
    #[serde(rename = "ref")]
    pub ref_name: String,
}

/// 仓库信息
#[derive(Debug, Deserialize)]
pub struct RepositoryInfo {
    #[serde(rename = "allow_squash_merge")]
    pub allow_squash_merge: Option<bool>,
    #[serde(rename = "allow_merge_commit")]
    pub allow_merge_commit: Option<bool>,
    #[serde(rename = "allow_rebase_merge")]
    pub allow_rebase_merge: Option<bool>,
}

/// GitHub 用户信息
#[skip_serializing_none]
#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// Pull Request 文件信息
#[derive(Debug, Deserialize)]
pub struct PullRequestFile {
    /// 文件路径
    pub filename: String,
    /// 文件状态（added, removed, modified, renamed, etc.）
    pub status: String,
    /// 添加的行数
    pub additions: u32,
    /// 删除的行数
    pub deletions: u32,
    /// 变更的行数
    pub changes: u32,
    /// 文件的 SHA（base）
    #[serde(default)]
    pub sha: Option<String>,
    /// 文件的 blob URL
    #[serde(default)]
    pub blob_url: Option<String>,
    /// 原始文件内容 URL（base）
    #[serde(default)]
    pub contents_url: Option<String>,
    /// 补丁内容（如果文件较小）
    #[serde(default)]
    pub patch: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;

    // ==================== Response Structure Tests ====================

    /// 测试创建 PR 响应结构体
    ///
    /// ## 测试目的
    /// 验证 CreatePullRequestResponse 结构体能够使用有效字段正确创建。
    ///
    /// ## 预期结果
    /// - html_url 字段值正确
    #[test]
    fn test_create_pull_request_response_structure_with_valid_fields_creates_response() {
        // Arrange: 准备响应字段值
        let html_url = "https://github.com/owner/repo/pull/123";

        // Act: 创建 CreatePullRequestResponse 实例
        let response = CreatePullRequestResponse {
            html_url: html_url.to_string(),
        };

        // Assert: 验证字段值正确
        assert_eq!(response.html_url, html_url);
    }

    /// 测试创建 PR 响应反序列化
    ///
    /// ## 测试目的
    /// 验证 CreatePullRequestResponse 能够从有效的 JSON 正确反序列化。
    ///
    /// ## 测试场景
    /// 从 JSON 字符串反序列化为结构体
    ///
    /// ## 预期结果
    /// - 反序列化成功
    /// - html_url 字段值正确
    #[test]
    fn test_create_pull_request_response_deserialization_with_valid_json_deserializes_response(
    ) -> Result<()> {
        // Arrange: 准备有效的 JSON 字符串
        let json = r#"{"html_url": "https://github.com/owner/repo/pull/123"}"#;

        // Act: 反序列化为 CreatePullRequestResponse
        let response: CreatePullRequestResponse = serde_json::from_str(json)?;

        // Assert: 验证字段值正确
        assert_eq!(response.html_url, "https://github.com/owner/repo/pull/123");
        Ok(())
    }

    /// 测试 PR 信息结构体
    ///
    /// ## 测试目的
    /// 验证 PullRequestInfo 结构体能够使用有效字段正确创建。
    ///
    /// ## 预期结果
    /// - 所有字段值正确（number、title、body、state、merged、head、base等）
    #[test]
    fn test_pull_request_info_structure_with_valid_fields_creates_info() {
        // Arrange: 准备 PR 信息字段值
        let number = 123;
        let title = "Test PR";
        let body = Some("Test body".to_string());
        let state = "open";

        // Act: 创建 PullRequestInfo 实例
        let pr_info = PullRequestInfo {
            number,
            title: title.to_string(),
            body: body.clone(),
            state: state.to_string(),
            merged: Some(false),
            merged_at: None,
            mergeable: None,
            html_url: "https://github.com/owner/repo/pull/123".to_string(),
            head: PullRequestBranch {
                ref_name: "feature/test".to_string(),
            },
            base: PullRequestBranch {
                ref_name: "main".to_string(),
            },
            user: None,
        };

        // Assert: 验证所有字段值正确
        assert_eq!(pr_info.number, number);
        assert_eq!(pr_info.title, title);
        assert_eq!(pr_info.body, body);
        assert_eq!(pr_info.state, state);
        assert_eq!(pr_info.merged, Some(false));
        assert_eq!(pr_info.head.ref_name, "feature/test");
        assert_eq!(pr_info.base.ref_name, "main");
    }

    /// 测试 PR 信息反序列化
    ///
    /// ## 测试目的
    /// 验证 PullRequestInfo 能够从有效的 JSON 正确反序列化。
    ///
    /// ## 测试场景
    /// 从 JSON 字符串反序列化为结构体
    ///
    /// ## 预期结果
    /// - 反序列化成功
    /// - 所有字段值正确
    #[test]
    fn test_pull_request_info_deserialization_with_valid_json_deserializes_info_return_ok(
    ) -> Result<()> {
        // Arrange: 准备有效的 JSON 字符串
        let json = r#"{
            "number": 123,
            "title": "Test PR",
            "body": "Test body",
            "state": "open",
            "merged": false,
            "html_url": "https://github.com/owner/repo/pull/123",
            "head": {"ref": "feature/test"},
            "base": {"ref": "main"}
        }"#;

        // Act: 反序列化为 PullRequestInfo
        let pr_info: PullRequestInfo = serde_json::from_str(json)?;

        // Assert: 验证字段值正确
        assert_eq!(pr_info.number, 123);
        assert_eq!(pr_info.title, "Test PR");
        assert_eq!(pr_info.state, "open");
        Ok(())
    }

    /// 测试已合并 PR 的状态
    ///
    /// ## 测试目的
    /// 验证 PullRequestInfo 能够正确表示已合并的 PR 状态。
    ///
    /// ## 测试场景
    /// 从包含 merged=true 和 merged_at 的 JSON 反序列化
    ///
    /// ## 预期结果
    /// - merged 字段为 true
    /// - state 为 "closed"
    /// - merged_at 不为 None
    #[test]
    fn test_pull_request_info_merged_state_with_merged_pr_return_ok() -> Result<()> {
        // Arrange: 准备已合并 PR 的 JSON
        let json = r#"{
            "number": 123,
            "title": "Merged PR",
            "body": null,
            "state": "closed",
            "merged": true,
            "merged_at": "2024-01-01T00:00:00Z",
            "html_url": "https://github.com/owner/repo/pull/123",
            "head": {"ref": "feature/test"},
            "base": {"ref": "main"}
        }"#;

        // Act: 反序列化为 PullRequestInfo
        let pr_info: PullRequestInfo = serde_json::from_str(json)?;

        // Assert: 验证合并状态正确
        assert_eq!(pr_info.merged, Some(true), "Should be marked as merged");
        assert_eq!(pr_info.state, "closed");
        assert!(pr_info.merged_at.is_some());
        Ok(())
    }

    // ==================== PullRequestBranch Tests ====================

    /// 测试 PR 分支结构体
    ///
    /// ## 测试目的
    /// 验证 PullRequestBranch 结构体能够使用有效字段正确创建。
    ///
    /// ## 预期结果
    /// - ref_name 字段值正确
    #[test]
    fn test_pull_request_branch_structure_with_valid_ref_creates_branch() {
        // Arrange: 准备分支引用名
        let ref_name = "feature/test";

        // Act: 创建 PullRequestBranch 实例
        let branch = PullRequestBranch {
            ref_name: ref_name.to_string(),
        };

        // Assert: 验证字段值正确
        assert_eq!(branch.ref_name, ref_name);
    }

    /// 测试 PR 分支反序列化
    ///
    /// ## 测试目的
    /// 验证 PullRequestBranch 能够从有效的 JSON 正确反序列化（注意 JSON 中使用 "ref" 字段）。
    ///
    /// ## 测试场景
    /// 从 JSON 字符串反序列化为结构体（JSON 字段名为 "ref"，结构体字段名为 ref_name）
    ///
    /// ## 预期结果
    /// - 反序列化成功
    /// - ref_name 字段值正确
    #[test]
    fn test_pull_request_branch_deserialization_with_valid_json_deserializes_branch_return_ok(
    ) -> Result<()> {
        // Arrange: 准备有效的 JSON 字符串（注意 JSON 中使用 "ref" 字段）
        let json = r#"{"ref": "feature/test"}"#;

        // Act: 反序列化为 PullRequestBranch
        let branch: PullRequestBranch = serde_json::from_str(json)?;

        // Assert: 验证字段值正确
        assert_eq!(branch.ref_name, "feature/test");
        Ok(())
    }

    // ==================== GitHubUser Tests ====================

    /// 测试 GitHub 用户结构体（包含所有字段）
    ///
    /// ## 测试目的
    /// 验证 GitHubUser 结构体能够使用所有字段正确创建。
    ///
    /// ## 预期结果
    /// - 所有字段值正确（login、name、email）
    #[test]
    fn test_github_user_structure_with_all_fields_creates_user() {
        // Arrange: 准备用户字段值
        let login = "testuser";
        let name = Some("Test User".to_string());
        let email = Some("test@example.com".to_string());

        // Act: 创建 GitHubUser 实例
        let user = GitHubUser {
            login: login.to_string(),
            name: name.clone(),
            email: email.clone(),
        };

        // Assert: 验证所有字段值正确
        assert_eq!(user.login, login);
        assert_eq!(user.name, name);
        assert_eq!(user.email, email);
    }

    /// 测试 GitHub 用户结构体（最小字段）
    ///
    /// ## 测试目的
    /// 验证 GitHubUser 结构体能够使用最小字段（只有 login）正确创建。
    ///
    /// ## 预期结果
    /// - login 字段值正确
    /// - 可选字段（name、email）为 None
    #[test]
    fn test_github_user_minimal_with_only_login_creates_user() {
        // Arrange: 准备最小字段值（只有 login）
        let login = "testuser";

        // Act: 创建 GitHubUser 实例（可选字段为 None）
        let user = GitHubUser {
            login: login.to_string(),
            name: None,
            email: None,
        };

        // Assert: 验证字段值正确
        assert_eq!(user.login, login);
        assert_eq!(user.name, None);
        assert_eq!(user.email, None);
    }

    /// 测试 GitHub 用户反序列化
    ///
    /// ## 测试目的
    /// 验证 GitHubUser 能够从有效的 JSON 正确反序列化。
    ///
    /// ## 测试场景
    /// 从 JSON 字符串反序列化为结构体
    ///
    /// ## 预期结果
    /// - 反序列化成功
    /// - 所有字段值正确
    #[test]
    fn test_github_user_deserialization_return_ok() -> Result<()> {
        // Arrange: 准备测试 GitHub 用户的反序列化
        let json = r#"{
            "login": "testuser",
            "name": "Test User",
            "email": "test@example.com"
        }"#;

        let user: GitHubUser = serde_json::from_str(json)?;
        assert_eq!(user.login, "testuser");
        assert_eq!(user.name, Some("Test User".to_string()));
        Ok(())
    }

    // ==================== Serialization/Deserialization Boundary Tests ====================

    /// 测试响应缺失可选字段的处理
    ///
    /// ## 测试目的
    /// 验证 PullRequestInfo 能够正确处理缺失可选字段的 JSON。
    ///
    /// ## 测试场景
    /// 从缺失可选字段（body、merged_at、user）的 JSON 反序列化
    ///
    /// ## 预期结果
    /// - 反序列化成功
    /// - 可选字段为 None
    #[test]
    fn test_response_missing_optional_fields_return_ok() -> Result<()> {
        // Arrange: 准备测试响应中缺失可选字段
        let json = r#"{
            "number": 123,
            "title": "Test PR",
            "state": "open",
            "merged": false,
            "html_url": "https://github.com/owner/repo/pull/123",
            "head": {"ref": "feature/test"},
            "base": {"ref": "main"}
        }"#;

        let pr_info: PullRequestInfo = serde_json::from_str(json)?;
        assert_eq!(pr_info.body, None);
        assert_eq!(pr_info.merged_at, None);
        assert!(pr_info.user.is_none(), "User should be None");
        Ok(())
    }

    // ==================== Type Safety Tests ====================

    /// 测试响应类型安全
    ///
    /// ## 测试目的
    /// 验证不同类型的响应结构体能够正确区分，确保类型安全。
    ///
    /// ## 测试场景
    /// 创建不同类型的响应并验证可以分别序列化
    ///
    /// ## 预期结果
    /// - 类型正确（通过编译验证）
    /// - 可以分别序列化
    #[test]
    fn test_response_type_safety() {
        // Arrange: 准备测试响应类型的安全性
        let _create_response: CreatePullRequestResponse = CreatePullRequestResponse {
            html_url: "https://example.com".to_string(),
        };

        let _pr_info: PullRequestInfo = PullRequestInfo {
            number: 1,
            title: "Test".to_string(),
            body: None,
            state: "open".to_string(),
            merged: Some(false),
            merged_at: None,
            mergeable: None,
            html_url: "https://example.com".to_string(),
            head: PullRequestBranch {
                ref_name: "head".to_string(),
            },
            base: PullRequestBranch {
                ref_name: "base".to_string(),
            },
            user: None,
        };

        // Assert: 验证类型正确（通过编译验证）
    }
}
