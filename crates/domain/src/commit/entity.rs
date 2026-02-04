//! 提交操作实体

use serde::{Deserialize, Serialize};

/// Amend 预览信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendPreview {
    /// 原始 commit SHA
    pub original_sha: String,
    /// 新提交消息
    pub new_message: Option<String>,
    /// 原始提交消息
    pub original_message: String,
    /// 要添加的文件列表
    pub files_to_add: Vec<String>,
    /// 操作类型
    pub operation_type: String,
    /// 是否已推送到远程
    pub is_pushed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amend_preview_basic() {
        let preview = AmendPreview {
            original_sha: "abc1234".to_string(),
            new_message: Some("新提交消息".to_string()),
            original_message: "原始提交消息".to_string(),
            files_to_add: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            operation_type: "amend".to_string(),
            is_pushed: false,
        };

        assert_eq!(preview.original_sha, "abc1234");
        assert_eq!(preview.new_message, Some("新提交消息".to_string()));
        assert_eq!(preview.files_to_add.len(), 2);
        assert!(!preview.is_pushed);
    }

    #[test]
    fn test_amend_preview_without_new_message() {
        let preview = AmendPreview {
            original_sha: "def5678".to_string(),
            new_message: None,
            original_message: "保持原消息".to_string(),
            files_to_add: vec!["new_file.rs".to_string()],
            operation_type: "amend_files".to_string(),
            is_pushed: false,
        };

        assert_eq!(preview.new_message, None);
        assert_eq!(preview.original_message, "保持原消息");
    }

    #[test]
    fn test_amend_preview_pushed_commit() {
        let preview = AmendPreview {
            original_sha: "pushed123".to_string(),
            new_message: Some("修改已推送的提交".to_string()),
            original_message: "原始消息".to_string(),
            files_to_add: vec![],
            operation_type: "amend".to_string(),
            is_pushed: true,
        };

        assert!(preview.is_pushed);
        assert!(preview.files_to_add.is_empty());
    }

    #[test]
    fn test_amend_preview_serialize() {
        let preview = AmendPreview {
            original_sha: "sha123".to_string(),
            new_message: Some("test message".to_string()),
            original_message: "original".to_string(),
            files_to_add: vec!["file.rs".to_string()],
            operation_type: "amend".to_string(),
            is_pushed: false,
        };

        let json = serde_json::to_string(&preview).unwrap();
        assert!(json.contains("\"original_sha\":\"sha123\""));
        assert!(json.contains("\"is_pushed\":false"));
        assert!(json.contains("\"files_to_add\":[\"file.rs\"]"));
    }

    #[test]
    fn test_amend_preview_deserialize() {
        let json = r#"{
            "original_sha": "deserialize123",
            "new_message": "deserialized message",
            "original_message": "original message",
            "files_to_add": ["a.rs", "b.rs"],
            "operation_type": "amend",
            "is_pushed": true
        }"#;

        let preview: AmendPreview = serde_json::from_str(json).unwrap();
        assert_eq!(preview.original_sha, "deserialize123");
        assert_eq!(preview.new_message, Some("deserialized message".to_string()));
        assert_eq!(preview.files_to_add.len(), 2);
        assert!(preview.is_pushed);
    }

    #[test]
    fn test_amend_preview_deserialize_with_null_new_message() {
        let json = r#"{
            "original_sha": "null_test",
            "new_message": null,
            "original_message": "keep this",
            "files_to_add": [],
            "operation_type": "amend_files",
            "is_pushed": false
        }"#;

        let preview: AmendPreview = serde_json::from_str(json).unwrap();
        assert_eq!(preview.new_message, None);
        assert!(preview.files_to_add.is_empty());
    }

    #[test]
    fn test_amend_preview_clone() {
        let preview = AmendPreview {
            original_sha: "clone_test".to_string(),
            new_message: Some("cloned".to_string()),
            original_message: "original".to_string(),
            files_to_add: vec!["cloned.rs".to_string()],
            operation_type: "amend".to_string(),
            is_pushed: false,
        };

        let cloned = preview.clone();
        assert_eq!(preview.original_sha, cloned.original_sha);
        assert_eq!(preview.new_message, cloned.new_message);
        assert_eq!(preview.files_to_add, cloned.files_to_add);
    }

    #[test]
    fn test_amend_preview_roundtrip() {
        let original = AmendPreview {
            original_sha: "roundtrip".to_string(),
            new_message: Some("round trip test".to_string()),
            original_message: "original message".to_string(),
            files_to_add: vec!["file1.rs".to_string(), "file2.rs".to_string()],
            operation_type: "amend".to_string(),
            is_pushed: true,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AmendPreview = serde_json::from_str(&json).unwrap();

        assert_eq!(original.original_sha, deserialized.original_sha);
        assert_eq!(original.new_message, deserialized.new_message);
        assert_eq!(original.original_message, deserialized.original_message);
        assert_eq!(original.files_to_add, deserialized.files_to_add);
        assert_eq!(original.operation_type, deserialized.operation_type);
        assert_eq!(original.is_pushed, deserialized.is_pushed);
    }
}
