//! 阶段二 2.1：批量操作分析服务
//!
//! 当阶段一检测到批量重命名、格式化、配置更新等模式时，对样本 diff 进行分析。

use std::collections::HashMap;
use std::sync::Arc;

use domain::git::entity::CommitFileChange;
use domain::summary::entity::{CommitBatchAnalysis, CommitFileClassification};
use domain::errors::ServiceError;
use llm::{LLMConversation, LLMExecutor};

use crate::summary::prompt;

// ── Conversation ──────────────────────────────────────────────

/// 批量操作分析对话
struct BatchAnalyzeConversation {
    user_prompt: String,
}

impl BatchAnalyzeConversation {
    fn new(user_prompt: String) -> Self {
        Self { user_prompt }
    }
}

impl LLMConversation for BatchAnalyzeConversation {
    fn get_system_prompt(&self, _language_code: &str) -> String {
        prompt::analyze_batch().to_string()
    }

    fn get_user_prompt(&self, _language_code: &str) -> String {
        self.user_prompt.clone()
    }

    fn get_execution_params(&self) -> (Option<u32>, f32) {
        (None, 0.3)
    }
}

// ── Helpers ───────────────────────────────────────────────────

fn detect_pattern_type(stage1: &CommitFileClassification) -> &'static str {
    if stage1.patterns.mass_rename.detected {
        "批量重命名"
    } else if stage1.patterns.formatting.detected {
        "批量格式化"
    } else if stage1.patterns.config_update.detected {
        "统一配置更新"
    } else if stage1.patterns.dependency_upgrade.detected {
        "依赖版本升级"
    } else if stage1.patterns.import_path_change.detected {
        "导入路径调整"
    } else {
        "批量操作"
    }
}

/// 综合 5 种批量操作模式的描述
fn build_batch_pattern_description(stage1: &CommitFileClassification) -> String {
    let p = &stage1.patterns;
    // 最多5种批量操作模式
    let mut parts: Vec<String> = Vec::with_capacity(5);
    if p.mass_rename.detected && !p.mass_rename.pattern.is_empty() {
        parts.push(format!(
            "批量重命名：{}（涉及 {} 个文件）",
            p.mass_rename.pattern, p.mass_rename.affected_files
        ));
    }
    if p.formatting.detected && !p.formatting.description.is_empty() {
        parts.push(format!("批量格式化：{}", p.formatting.description));
    }
    if p.config_update.detected && !p.config_update.type_desc.is_empty() {
        parts.push(format!("统一配置更新：{}", p.config_update.type_desc));
    }
    if p.dependency_upgrade.detected && !p.dependency_upgrade.packages.is_empty() {
        parts.push(format!(
            "依赖版本升级：{}",
            p.dependency_upgrade.packages.join(", ")
        ));
    }
    if p.import_path_change.detected && !p.import_path_change.pattern.is_empty() {
        parts.push(format!("导入路径调整：{}", p.import_path_change.pattern));
    }
    if parts.is_empty() {
        "（阶段一未识别到具体模式，请根据样本 diff 归纳）".to_string()
    } else {
        parts.join("；")
    }
}
