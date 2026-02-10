//! 阶段三：全局总结服务
//!
//! 综合阶段一分类结果与阶段二各分析结果，生成结构化的 commit 总结。

use llm::{LLMConversation, SupportedLanguage};
use toolkit::log_info;

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

    fn get_user_prompt(&self, language_code: &str) -> String {
        let i = &self.input;

        // 构建未提交变更警告（如果存在）
        let uncommitted_warning = if i.has_uncommitted_changes {
            "\n\n### ⚠️ Notice\nThe working directory has uncommitted changes. This summary only covers committed changes."
        } else {
            ""
        };

        let language_name = SupportedLanguage::find(language_code)
            .map(|lang| lang.native_name)
            .unwrap_or("en");

        log_info!("language_name: {}", language_name);
        log_info!("language_code: {}", language_code);

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
            language_name,
            language_code,
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

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}

// ── Service ───────────────────────────────────────────────────
