//! GitHub API 响应 Fixtures
//!
//! 提供预定义的 GitHub API 响应数据，用于测试。

use serde_json::Value;

/// GitHub API 响应 Fixtures
///
/// 提供预定义的 GitHub API 响应数据，用于测试。
pub struct GitHubFixtures;

impl GitHubFixtures {
    /// 获取示例 Pull Request 响应
    pub fn sample_pull_request() -> Value {
        serde_json::json!({
            "id": 1,
            "number": 123,
            "state": "open",
            "title": "Add new feature",
            "body": "This PR adds a new feature",
            "user": {
                "login": "octocat",
                "id": 1
            },
            "head": {
                "ref": "feature-branch",
                "sha": "abc123"
            },
            "base": {
                "ref": "main",
                "sha": "def456"
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }

    /// 获取示例 Pull Request 列表响应
    pub fn sample_pull_request_list() -> Value {
        serde_json::json!([Self::sample_pull_request()])
    }

    /// 获取示例 Issue 响应
    pub fn sample_issue() -> Value {
        serde_json::json!({
            "id": 1,
            "number": 456,
            "state": "open",
            "title": "Bug report",
            "body": "Found a bug",
            "user": {
                "login": "octocat",
                "id": 1
            },
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        })
    }
}
