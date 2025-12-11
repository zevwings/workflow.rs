//! Commit 业务逻辑模块
//!
//! 本模块提供了 Commit 相关的业务逻辑，包括：
//! - Amend 操作的业务逻辑
//! - Reword 操作的业务逻辑
//! - 格式化显示逻辑
//! - 预览信息生成

mod amend;
mod reword;

pub use amend::{AmendPreview, CommitAmend};
pub use reword::{CommitReword, RewordHistoryOptions, RewordHistoryResult, RewordPreview};
