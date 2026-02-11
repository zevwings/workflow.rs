//! Jira API 响应 Fixtures
//!
//! 提供预定义的 Jira API 响应数据，用于测试。

use serde_json::Value;

/// Jira API 响应 Fixtures
pub struct JiraFixtures;

impl JiraFixtures {
    /// 获取示例 Issue 响应
    pub fn sample_issue() -> Value {
        serde_json::json!({
            "key": "PROJ-123",
            "id": "10001",
            "fields": {
                "summary": "Test Issue",
                "description": "Test description",
                "issuetype": {
                    "name": "Task"
                },
                "status": {
                    "name": "To Do"
                },
                "project": {
                    "key": "PROJ"
                }
            }
        })
    }

    /// 获取示例转换响应
    pub fn sample_transitions() -> Value {
        serde_json::json!({
            "transitions": [
                {
                    "id": "11",
                    "name": "In Progress",
                    "to": {
                        "name": "In Progress"
                    }
                },
                {
                    "id": "21",
                    "name": "Done",
                    "to": {
                        "name": "Done"
                    }
                }
            ]
        })
    }
}
