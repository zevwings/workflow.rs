use serde::{Deserialize, Serialize};

// ========== 提交分析阶段一：文件分类结果 ==========

/// 阶段一文件分类结果（按变更类型 / 性质 / 规模 + 模式 + 分析策略 + 摘要）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitFileClassification {
    pub categories: FileClassificationCategories,
    pub patterns: FileClassificationPatterns,
    #[serde(rename = "analysis_strategy")]
    pub analysis_strategy: AnalysisStrategy,
    pub summary: ClassificationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassificationCategories {
    #[serde(rename = "by_status")]
    pub by_status: ByStatusCategories,
    #[serde(rename = "by_nature")]
    pub by_nature: ByNatureCategories,
    #[serde(rename = "by_scale")]
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
    #[serde(rename = "mass_rename")]
    pub mass_rename: PatternMassRename,
    pub formatting: PatternFormatting,
    #[serde(rename = "config_update")]
    pub config_update: PatternConfigUpdate,
    #[serde(rename = "dependency_upgrade")]
    pub dependency_upgrade: PatternDependencyUpgrade,
    #[serde(rename = "import_path_change")]
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
    #[serde(rename = "批量处理组")]
    pub batch_group: Vec<String>,
    #[serde(rename = "重点分析组")]
    pub focus_group: Vec<String>,
    #[serde(rename = "可跳过组")]
    pub skip_group: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSummary {
    #[serde(rename = "total_files")]
    pub total_files: u32,
    #[serde(rename = "primary_change_type")]
    #[serde(default)]
    pub primary_change_type: String,
    #[serde(default)]
    pub complexity: String,
}

// ========== 提交分析阶段二：分类分析结果 ==========

/// 阶段二 2.1：批量操作分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitBatchAnalysis {
    #[serde(rename = "batch_summary")]
    #[serde(default)]
    pub batch_summary: String,
    #[serde(rename = "common_changes")]
    #[serde(default)]
    pub common_changes: String,
    #[serde(rename = "pattern_consistency")]
    #[serde(default)]
    pub pattern_consistency: String,
    #[serde(default)]
    pub exceptions: Vec<BatchException>,
    #[serde(default)]
    pub impact: BatchImpact,
    #[serde(rename = "total_affected")]
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
    #[serde(rename = "key_changes")]
    #[serde(default)]
    pub key_changes: Vec<String>,
    #[serde(rename = "technical_approach")]
    #[serde(default)]
    pub technical_approach: String,
    #[serde(rename = "impact_scope")]
    #[serde(default)]
    pub impact_scope: LogicImpactScope,
    #[serde(rename = "related_files")]
    #[serde(default)]
    pub related_files: Vec<RelatedFile>,
    #[serde(rename = "risk_assessment")]
    #[serde(default)]
    pub risk_assessment: RiskAssessment,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogicImpactScope {
    #[serde(rename = "api_changes")]
    #[serde(default)]
    pub api_changes: bool,
    #[serde(rename = "database_changes")]
    #[serde(default)]
    pub database_changes: bool,
    #[serde(rename = "module_dependencies")]
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
    #[serde(rename = "config_changes")]
    #[serde(default)]
    pub config_changes: Vec<ConfigChange>,
    #[serde(rename = "doc_updates")]
    #[serde(default)]
    pub doc_updates: Vec<DocUpdate>,
    #[serde(rename = "deployment_notes")]
    #[serde(default)]
    pub deployment_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    #[serde(default)]
    pub file: String,
    #[serde(rename = "change_type")]
    #[serde(default)]
    pub change_type: String,
    #[serde(default)]
    pub items: Vec<ConfigItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    #[serde(default)]
    pub key: String,
    #[serde(rename = "old_value")]
    #[serde(default)]
    pub old_value: String,
    #[serde(rename = "new_value")]
    #[serde(default)]
    pub new_value: String,
    #[serde(default)]
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocUpdate {
    #[serde(default)]
    pub file: String,
    #[serde(rename = "update_type")]
    #[serde(default)]
    pub update_type: String,
    #[serde(default)]
    pub summary: String,
}

/// 阶段二 2.4：测试文件分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitTestAnalysis {
    #[serde(rename = "test_summary")]
    #[serde(default)]
    pub test_summary: TestSummary,
    #[serde(rename = "alignment_with_code")]
    #[serde(default)]
    pub alignment_with_code: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestSummary {
    #[serde(rename = "new_tests")]
    #[serde(default)]
    pub new_tests: Vec<String>,
    #[serde(rename = "modified_tests")]
    #[serde(default)]
    pub modified_tests: Vec<String>,
    #[serde(rename = "deleted_tests")]
    #[serde(default)]
    pub deleted_tests: Vec<String>,
    #[serde(rename = "coverage_modules")]
    #[serde(default)]
    pub coverage_modules: Vec<String>,
}

// ========== 提交分析阶段三：全局总结结果 ==========

/// 阶段三：全局总结结果（commit_message + structured_summary + impact_analysis + statistics + metadata）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummaryAnalysis {
    #[serde(rename = "commit_message")]
    pub commit_message: CommitMessagePart,
    #[serde(rename = "structured_summary")]
    pub structured_summary: StructuredSummary,
    #[serde(rename = "impact_analysis")]
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

/// 结构化总结（type / scope / subject / main_purpose / key_changes / details_by_category）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredSummary {
    #[serde(default)]
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub subject: String,
    #[serde(rename = "main_purpose")]
    #[serde(default)]
    pub main_purpose: String,
    #[serde(rename = "key_changes")]
    #[serde(default)]
    pub key_changes: Vec<String>,
    #[serde(rename = "details_by_category")]
    #[serde(default)]
    pub details_by_category: DetailsByCategory,
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
    #[serde(rename = "breaking_changes")]
    #[serde(default)]
    pub breaking_changes: SummaryBreakingChanges,
    #[serde(rename = "affected_modules")]
    #[serde(default)]
    pub affected_modules: Vec<AffectedModule>,
    #[serde(rename = "risk_assessment")]
    #[serde(default)]
    pub risk_assessment: SummaryRiskAssessment,
    #[serde(rename = "testing_suggestions")]
    #[serde(default)]
    pub testing_suggestions: Vec<String>,
}

/// 破坏性变更说明
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SummaryBreakingChanges {
    #[serde(rename = "has_breaking")]
    #[serde(default)]
    pub has_breaking: bool,
    #[serde(default)]
    pub description: String,
    #[serde(rename = "migration_guide")]
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
    #[serde(rename = "overall_risk")]
    #[serde(default)]
    pub overall_risk: String,
    #[serde(rename = "risk_factors")]
    #[serde(default)]
    pub risk_factors: Vec<String>,
    #[serde(default)]
    pub mitigation: Vec<String>,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStatistics {
    #[serde(rename = "total_files")]
    #[serde(default)]
    pub total_files: u32,
    #[serde(default)]
    pub additions: u32,
    #[serde(default)]
    pub deletions: u32,
    #[serde(rename = "net_change")]
    #[serde(default)]
    pub net_change: i64,
    #[serde(rename = "file_breakdown")]
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
    #[serde(rename = "review_priority")]
    #[serde(default)]
    pub review_priority: String,
    #[serde(rename = "estimated_review_time")]
    #[serde(default)]
    pub estimated_review_time: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
