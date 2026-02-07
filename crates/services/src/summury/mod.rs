//! Commit 分析对话子模块
//!
//! 包含阶段一文件分类、阶段二各类型分析（批量/逻辑/配置/测试）、阶段三全局总结，
//! 以及对应的 prompt 文本（本目录下 `.md` 文件）。

mod batch_analyze;
mod config_analyze;
mod file_classify;
mod logic_analyze;
mod summary_analyze;
mod test_analyze;

// 子模块类型统一导出
pub(crate) use batch_analyze::BatchAnalyzeConversation;
pub(crate) use config_analyze::ConfigAnalyzeConversation;
pub(crate) use file_classify::FileClassifyConversation;
pub(crate) use logic_analyze::LogicAnalyzeConversation;
pub(crate) use summary_analyze::{SummaryAnalyzeConversation, SummaryAnalyzeInput};
pub(crate) use test_analyze::TestAnalyzeConversation;

// ---------- Prompt 访问（本目录 .md，供子模块使用） ----------

/// 提交文件分类 prompt（阶段一）
pub(super) const fn file_classify() -> &'static str {
    include_str!("file_classify.md")
}

/// 阶段二 2.1：批量操作分析 prompt
pub(super) const fn batch_analyze() -> &'static str {
    include_str!("batch_analyze.md")
}

/// 阶段二 2.2：核心逻辑分析 prompt
pub(super) const fn logic_analyze() -> &'static str {
    include_str!("logic_analyze.md")
}

/// 阶段二 2.3：配置/文档分析 prompt
pub(super) const fn config_analyze() -> &'static str {
    include_str!("config_analyze.md")
}

/// 阶段二 2.4：测试文件分析 prompt
pub(super) const fn test_analyze() -> &'static str {
    include_str!("test_analyze.md")
}

/// 阶段三：全局总结 prompt
pub(super) const fn summary_analyze() -> &'static str {
    include_str!("summary_analyze.md")
}
