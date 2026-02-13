//! 阶段三：全局总结服务
//!
//! 综合阶段一分类结果与阶段二各分析结果，生成结构化的 commit 总结。

use client::{LLMConversation, SupportedLanguage};

use crate::summary::{prompt, summary::SummaryAnalyzeInput};

// ── Conversation ──────────────────────────────────────────────

/// 阶段三：全局总结对话
pub struct SummaryAnalyzeConversation {
    input: SummaryAnalyzeInput,
    language: SupportedLanguage,
}

impl SummaryAnalyzeConversation {
    pub fn new(input: SummaryAnalyzeInput, language: SupportedLanguage) -> Self {
        Self { input, language }
    }
}

impl LLMConversation for SummaryAnalyzeConversation {
    fn get_system_prompt(&self) -> String {
        prompt::summary().to_string()
    }

    fn get_user_prompt(&self) -> String {
        let i = &self.input;

        // 构建未提交变更警告（如果存在）
        let uncommitted_warning = if i.has_uncommitted_changes {
            "\n\n### ⚠️ Notice\nThe working directory has uncommitted changes. This summary only covers committed changes."
        } else {
            ""
        };

        format!(
            r##"🌐 LANGUAGE REQUIREMENT:
The RESPOND LANGUAGE MUST use the {}({})

## Input Information

### Commit History (Total: {} commits)
```
{}
```

### Stage 1: File Classification Results
```json
{}
```

### Stage 2: Detailed Analysis Results

#### Batch Operation Analysis
```json
{}
```

#### Core Logic Analysis
```json
{}
```

#### Configuration/Documentation Analysis
```json
{}
```

#### Test Analysis
```json
{}
```

### Statistics
- Total files: {}
- Added: {}
- Deleted: {}
- Modified: {}
- Renamed: {}
- Line changes: +{} -{}{}"##,
            self.language.name,
            self.language.code,
            i.commit_count,
            i.commit_history_summary,
            i.stage1_classification,
            i.stage2_batch_analysis,
            i.stage2_logic_analysis,
            i.stage2_config_analysis,
            i.stage2_test_analysis,
            i.total_files,
            i.added_count,
            i.deleted_count,
            i.modified_count,
            i.renamed_count,
            i.total_additions,
            i.total_deletions,
            uncommitted_warning,
        )
    }

    fn get_max_tokens(&self) -> Option<u32> {
        None
    }

    fn get_temperature(&self) -> f32 {
        0.3
    }
}

// ── Service ───────────────────────────────────────────────────
