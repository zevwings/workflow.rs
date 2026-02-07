/// 阶段三全局总结的输入参数
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
