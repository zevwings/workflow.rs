//! Commit Message 对话构建器
//!
//! 构建用于生成 commit message 的 LLM 对话。

use llm::LLMConversation;

use super::service::FileStatistics;

/// Commit Message 对话构建器
pub struct CommitMessageConversation {
    file_summary: String,
    diff_content: String,
    stats: FileStatistics,
}

impl CommitMessageConversation {
    /// 创建新的对话构建器
    pub fn new(file_summary: String, diff_content: String, stats: FileStatistics) -> Self {
        Self {
            file_summary,
            diff_content,
            stats,
        }
    }

    /// 构建用户 prompt
    fn build_user_prompt(&self) -> String {
        format!(
            r#"## LANGUAGE REQUIREMENT

Please respond in the language code specified below. All textual content in the output MUST be in this language.

Language Code: {{{{language_code}}}}

---

## Changed Files Summary

{}

---

## Statistics

- Total files: {}
- Added: {}
- Modified: {}
- Deleted: {}
- Renamed: {}
- Total additions: +{}
- Total deletions: -{}

---

## Diff Content

```diff
{}
```

---

Please analyze the above changes and generate a high-quality commit message following the Conventional Commits specification. Output in JSON format only."#,
            self.file_summary,
            self.stats.total_files,
            self.stats.added_count,
            self.stats.modified_count,
            self.stats.deleted_count,
            self.stats.renamed_count,
            self.stats.total_additions,
            self.stats.total_deletions,
            self.diff_content
        )
    }
}

impl LLMConversation for CommitMessageConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        include_str!("prompt.md").to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.build_user_prompt()
    }
}
