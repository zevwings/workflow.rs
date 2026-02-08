use serde::{Deserialize, Serialize};

// ========== 通用结构 ==========

/// 行为差异分析
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BehaviorDiff {
    /// 修改前的行为描述
    #[serde(default)]
    pub before: String,
    /// 修改后的行为描述
    #[serde(default)]
    pub after: String,
    /// 行为变更的原因
    #[serde(default)]
    pub reason: String,
}

/// 功能域（跨文件类型的功能性聚合）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDomain {
    /// 功能域名称（如 "HTTP 客户端统一化"、"LLM 集成"）
    #[serde(default)]
    pub domain: String,
    /// 该功能域的整体目的
    #[serde(default)]
    pub purpose: String,
    /// 涉及的文件路径列表
    #[serde(default)]
    pub files: Vec<String>,
    /// 该功能域下的变更描述（可跨 features/config/tests 等类别）
    #[serde(default)]
    pub changes: Vec<String>,
}

// ========== 提交分析阶段一：文件分类结果 ==========

/// 阶段一文件分类结果（按变更类型 / 性质 / 规模 + 模式 + 分析策略 + 摘要）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFileClassification {
    pub categories: FileClassificationCategories,
    pub patterns: FileClassificationPatterns,
    pub analysis_strategy: AnalysisStrategy,
    pub summary: ClassificationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassificationCategories {
    pub by_status: ByStatusCategories,
    pub by_nature: ByNatureCategories,
    pub by_scale: ByScaleCategories,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByStatusCategories {
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub renamed: Vec<RenamedFileEntry>,
    pub modified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamedFileEntry {
    pub old: String,
    pub new: String,
    pub changes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByNatureCategories {
    pub business_logic: Vec<String>,
    pub configuration: Vec<String>,
    pub tests: Vec<String>,
    pub documentation: Vec<String>,
    pub dependencies: Vec<String>,
    pub ui_style: Vec<String>,
    pub infrastructure: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByScaleCategories {
    pub large: Vec<String>,
    pub medium: Vec<String>,
    pub small: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassificationPatterns {
    pub mass_rename: PatternMassRename,
    pub formatting: PatternFormatting,
    pub config_update: PatternConfigUpdate,
    pub dependency_upgrade: PatternDependencyUpgrade,
    pub import_path_change: PatternImportPathChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMassRename {
    pub detected: bool,
    #[serde(default)]
    pub pattern: String,
    #[serde(default)]
    pub affected_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFormatting {
    pub detected: bool,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfigUpdate {
    pub detected: bool,
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDependencyUpgrade {
    pub detected: bool,
    #[serde(default)]
    pub packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternImportPathChange {
    pub detected: bool,
    #[serde(default)]
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisStrategy {
    pub batch_group: Vec<String>,
    pub focus_group: Vec<String>,
    pub skip_group: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSummary {
    pub total_files: u32,
    #[serde(default)]
    pub primary_change_type: String,
    #[serde(default)]
    pub complexity: String,
}

// ========== 提交分析阶段二：分类分析结果 ==========

/// 阶段二 2.1：批量操作分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBatchAnalysis {
    #[serde(default)]
    pub batch_summary: String,
    #[serde(default)]
    pub common_changes: String,
    #[serde(default)]
    pub pattern_consistency: String,
    #[serde(default)]
    pub exceptions: Vec<BatchException>,
    #[serde(default)]
    pub impact: BatchImpact,
    #[serde(default)]
    pub total_affected: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchException {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BatchImpact {
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub breaking: bool,
    #[serde(default)]
    pub description: String,
}

/// 阶段二 2.2：核心逻辑分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLogicAnalysis {
    #[serde(default)]
    pub files: Vec<LogicFileAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFileAnalysis {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub key_changes: Vec<String>,
    #[serde(default)]
    pub technical_approach: String,
    #[serde(default)]
    pub impact_scope: LogicImpactScope,
    #[serde(default)]
    pub related_files: Vec<RelatedFile>,
    #[serde(default)]
    pub risk_assessment: RiskAssessment,
    /// 行为差异分析
    #[serde(default)]
    pub behavior_diff: BehaviorDiff,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogicImpactScope {
    #[serde(default)]
    pub api_changes: bool,
    #[serde(default)]
    pub database_changes: bool,
    #[serde(default)]
    pub module_dependencies: Vec<String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedFile {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub relationship: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskAssessment {
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub concerns: Vec<String>,
    #[serde(default)]
    pub recommendations: Vec<String>,
}

/// 阶段二 2.3：配置/文档分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitConfigAnalysis {
    #[serde(default)]
    pub config_changes: Vec<ConfigChange>,
    #[serde(default)]
    pub doc_updates: Vec<DocUpdate>,
    #[serde(default)]
    pub deployment_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub change_type: String,
    #[serde(default)]
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    #[serde(default)]
    pub key: String,
    #[serde(default, deserialize_with = "deserialize_value_as_string")]
    pub old_value: String,
    #[serde(default, deserialize_with = "deserialize_value_as_string")]
    pub new_value: String,
    #[serde(default)]
    pub purpose: String,
}

/// Custom deserializer to convert any JSON value to a string
fn deserialize_value_as_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde_json::Value;
    let value = Value::deserialize(deserializer)?;
    Ok(match value {
        Value::String(s) => s,
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocUpdate {
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub update_type: String,
    #[serde(default)]
    pub summary: String,
}

/// 阶段二 2.4：测试文件分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTestAnalysis {
    #[serde(default)]
    pub test_summary: TestSummary,
    #[serde(default)]
    pub alignment_with_code: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestSummary {
    #[serde(default)]
    pub new_tests: Vec<String>,
    #[serde(default)]
    pub modified_tests: Vec<String>,
    #[serde(default)]
    pub deleted_tests: Vec<String>,
    #[serde(default)]
    pub coverage_modules: Vec<String>,
}

// ========== 提交分析阶段三：全局总结结果 ==========

/// 阶段三：全局总结结果（commit_message + structured_summary + impact_analysis + statistics + metadata）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummaryAnalysis {
    pub commit_message: CommitMessagePart,
    pub structured_summary: StructuredSummary,
    pub impact_analysis: ImpactAnalysis,
    pub statistics: SummaryStatistics,
    pub metadata: SummaryMetadata,
}

/// Commit 消息部分（title / body / footer）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMessagePart {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub footer: String,
}

/// 结构化总结（type / scope / subject / main_purpose / key_changes / details_by_category / changes_by_domain）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSummary {
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub main_purpose: String,
    #[serde(default)]
    pub key_changes: Vec<String>,
    #[serde(default)]
    pub details_by_category: DetailsByCategory,
    /// 按功能域聚合的变更（跨文件类型）
    #[serde(default)]
    pub changes_by_domain: Vec<FeatureDomain>,
}

/// 按类别划分的变更详情
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetailsByCategory {
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub fixes: Vec<String>,
    #[serde(default)]
    pub refactors: Vec<String>,
    #[serde(default)]
    pub config: Vec<String>,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(default)]
    pub tests: Vec<String>,
    #[serde(default)]
    pub others: Vec<String>,
}

/// 影响分析（breaking_changes / affected_modules / risk_assessment / testing_suggestions）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    #[serde(default)]
    pub breaking_changes: SummaryBreakingChanges,
    #[serde(default)]
    pub affected_modules: Vec<AffectedModule>,
    #[serde(default)]
    pub risk_assessment: SummaryRiskAssessment,
    #[serde(default)]
    pub testing_suggestions: Vec<String>,
}

/// 破坏性变更说明
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryBreakingChanges {
    #[serde(default)]
    pub has_breaking: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub migration_guide: String,
}

/// 受影响模块
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedModule {
    #[serde(default)]
    pub module: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub severity: String,
}

/// 阶段三风险评估
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryRiskAssessment {
    #[serde(default)]
    pub overall_risk: String,
    #[serde(default)]
    pub risk_factors: Vec<String>,
    #[serde(default)]
    pub mitigation: Vec<String>,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStatistics {
    #[serde(default)]
    pub total_files: u32,
    #[serde(default)]
    pub additions: u32,
    #[serde(default)]
    pub deletions: u32,
    #[serde(default)]
    pub net_change: i64,
    #[serde(default)]
    pub file_breakdown: FileBreakdown,
}

/// 文件变更数量分布
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileBreakdown {
    #[serde(default)]
    pub added: u32,
    #[serde(default)]
    pub modified: u32,
    #[serde(default)]
    pub deleted: u32,
    #[serde(default)]
    pub renamed: u32,
}

/// 阶段三元数据（complexity / review_priority / estimated_review_time / tags）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetadata {
    #[serde(default)]
    pub complexity: String,
    #[serde(default)]
    pub review_priority: String,
    #[serde(default)]
    pub estimated_review_time: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
