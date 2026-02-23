//! 别名实体定义

use serde::{Deserialize, Serialize};

// ============================================================================
// 别名实体
// ============================================================================

/// 别名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasInfo {
    /// 别名名称
    pub name: String,
    /// 对应的命令
    pub command: String,
}

impl AliasInfo {
    /// 创建新的别名信息
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
}

// ============================================================================
// 操作结果
// ============================================================================

/// 别名列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasListResult {
    /// 别名列表
    pub aliases: Vec<AliasInfo>,
    /// 别名总数
    pub count: usize,
}

/// 别名添加结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasAddResult {
    /// 添加的别名名称
    pub name: String,
    /// 对应的命令
    pub command: String,
    /// 是否为覆盖操作（别名已存在）
    pub overwritten: bool,
}

/// 别名移除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasRemoveResult {
    /// 移除的别名名称
    pub name: String,
    /// 对应的命令（移除前的）
    pub command: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // AliasInfo 测试
    #[test]
    fn test_alias_info_new() {
        let alias = AliasInfo::new("ci", "commit");
        assert_eq!(alias.name, "ci");
        assert_eq!(alias.command, "commit");
    }

    #[test]
    fn test_alias_info_new_with_string() {
        let alias = AliasInfo::new(String::from("st"), String::from("status"));
        assert_eq!(alias.name, "st");
        assert_eq!(alias.command, "status");
    }

    #[test]
    fn test_alias_info_serialize() {
        let alias = AliasInfo::new("br", "branch");
        let json = serde_json::to_string(&alias).unwrap();
        assert!(json.contains("\"name\":\"br\""));
        assert!(json.contains("\"command\":\"branch\""));
    }

    #[test]
    fn test_alias_info_deserialize() {
        let json = r#"{"name": "co", "command": "checkout"}"#;
        let alias: AliasInfo = serde_json::from_str(json).unwrap();
        assert_eq!(alias.name, "co");
        assert_eq!(alias.command, "checkout");
    }

    #[test]
    fn test_alias_info_clone() {
        let alias = AliasInfo::new("lg", "log --oneline");
        let cloned = alias.clone();
        assert_eq!(alias.name, cloned.name);
        assert_eq!(alias.command, cloned.command);
    }

    // AliasListResult 测试
    #[test]
    fn test_alias_list_result_empty() {
        let result = AliasListResult {
            aliases: vec![],
            count: 0,
        };
        assert!(result.aliases.is_empty());
        assert_eq!(result.count, 0);
    }

    #[test]
    fn test_alias_list_result_with_aliases() {
        let result = AliasListResult {
            aliases: vec![
                AliasInfo::new("ci", "commit"),
                AliasInfo::new("st", "status"),
            ],
            count: 2,
        };
        assert_eq!(result.aliases.len(), 2);
        assert_eq!(result.count, 2);
    }

    #[test]
    fn test_alias_list_result_serialize() {
        let result = AliasListResult {
            aliases: vec![AliasInfo::new("br", "branch")],
            count: 1,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"count\":1"));
        assert!(json.contains("\"aliases\":["));
    }

    #[test]
    fn test_alias_list_result_deserialize() {
        let json = r#"{
            "aliases": [
                {"name": "ci", "command": "commit"},
                {"name": "st", "command": "status"}
            ],
            "count": 2
        }"#;
        let result: AliasListResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.count, 2);
        assert_eq!(result.aliases.len(), 2);
    }

    // AliasAddResult 测试
    #[test]
    fn test_alias_add_result_new_alias() {
        let result = AliasAddResult {
            name: "new".to_string(),
            command: "new-command".to_string(),
            overwritten: false,
        };
        assert_eq!(result.name, "new");
        assert_eq!(result.command, "new-command");
        assert!(!result.overwritten);
    }

    #[test]
    fn test_alias_add_result_overwritten() {
        let result = AliasAddResult {
            name: "existing".to_string(),
            command: "updated-command".to_string(),
            overwritten: true,
        };
        assert!(result.overwritten);
    }

    #[test]
    fn test_alias_add_result_serialize() {
        let result = AliasAddResult {
            name: "test".to_string(),
            command: "test-cmd".to_string(),
            overwritten: false,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"overwritten\":false"));
    }

    #[test]
    fn test_alias_add_result_deserialize() {
        let json = r#"{
            "name": "added",
            "command": "added-command",
            "overwritten": true
        }"#;
        let result: AliasAddResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.name, "added");
        assert!(result.overwritten);
    }

    // AliasRemoveResult 测试
    #[test]
    fn test_alias_remove_result() {
        let result = AliasRemoveResult {
            name: "removed".to_string(),
            command: "old-command".to_string(),
        };
        assert_eq!(result.name, "removed");
        assert_eq!(result.command, "old-command");
    }

    #[test]
    fn test_alias_remove_result_serialize() {
        let result = AliasRemoveResult {
            name: "del".to_string(),
            command: "delete".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"name\":\"del\""));
        assert!(json.contains("\"command\":\"delete\""));
    }

    #[test]
    fn test_alias_remove_result_deserialize() {
        let json = r#"{"name": "rm", "command": "remove"}"#;
        let result: AliasRemoveResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.name, "rm");
        assert_eq!(result.command, "remove");
    }

    #[test]
    fn test_alias_remove_result_clone() {
        let result = AliasRemoveResult {
            name: "clone".to_string(),
            command: "clone-cmd".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(result.name, cloned.name);
        assert_eq!(result.command, cloned.command);
    }
}
