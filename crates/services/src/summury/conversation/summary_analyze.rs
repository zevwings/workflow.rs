//! 阶段三：全局总结对话
//!
//! 综合阶段一分类结果与阶段二各分析结果，生成结构化的 commit 总结（标题、描述、影响分析等）。

use domain::{CommitSummaryAnalysis, LLMError};

use crate::llm::services::{parsers::JsonParser, LLMConversation};

/// 阶段三全局总结的输入参数
#[derive(Clone)]
pub(crate) struct SummaryAnalyzeInput {
    pub stage1_classification: String,
    pub stage2_batch_analysis: String,
    pub stage2_logic_analysis: String,
    pub stage2_config_analysis: String,
    pub stage2_test_analysis: String,
    pub total_files: u32,
    pub added_count: u32,
    pub deleted_count: u32,
    pub modified_count: u32,
    pub renamed_count: u32,
    pub total_additions: u32,
    pub total_deletions: u32,
}

/// 阶段三：全局总结对话
///
/// 输入为阶段一、阶段二的分析结果 JSON 及统计信息，输出为结构化的 commit 总结。
pub(crate) struct SummaryAnalyzeConversation {
    input: SummaryAnalyzeInput,
}

impl SummaryAnalyzeConversation {
    pub fn new(input: SummaryAnalyzeInput) -> Self {
        Self { input }
    }
}

impl LLMConversation for SummaryAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        super::summary_analyze().to_string()
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

    fn parse_response(&self, response: String) -> Result<Self::Output, LLMError> {
        JsonParser::to_model(response)
            .map_err(|e| LLMError::ApiError(format!("JSON parse error: {}", e)))
    }
}
