//! 阶段三：全局总结服务
//!
//! 综合阶段一分类结果与阶段二各分析结果，生成结构化的 commit 总结。

use llm::LLMConversation;

use crate::summary::{prompt, summary::SummaryAnalyzeInput};

// ── Conversation ──────────────────────────────────────────────

/// 阶段三：全局总结对话
pub struct SummaryAnalyzeConversation {
    input: SummaryAnalyzeInput,
}

impl SummaryAnalyzeConversation {
    pub fn new(input: SummaryAnalyzeInput) -> Self {
        Self { input }
    }
}

impl LLMConversation for SummaryAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::summary().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        let i = &self.input;
        format!(
            r##"## Input Information

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
- Line changes: +{} -{}"##,
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
        )
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}

// ── Service ───────────────────────────────────────────────────
