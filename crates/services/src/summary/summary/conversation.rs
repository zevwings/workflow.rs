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
            r##"## 输入信息

### 阶段一：文件分类结果
```json
{}
```

### 阶段二：详细分析结果

#### 批量操作分析
```json
{}
```

#### 核心逻辑分析
```json
{}
```

#### 配置文档分析
```json
{}
```

#### 测试分析
```json
{}
```

### 统计信息
- 总文件数：{}
- 新增：{} 个
- 删除：{} 个
- 修改：{} 个
- 重命名：{} 个
- 代码行变化：+{} -{}"##,
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
