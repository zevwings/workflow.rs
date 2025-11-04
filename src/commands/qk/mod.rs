pub mod download;
pub mod find;
pub mod search;

// 为了向后兼容，保留 QuickCommand 作为统一接口
pub use download::DownloadCommand;
pub use find::FindCommand;
pub use search::SearchCommand;

use anyhow::Result;

/// Quick 统一命令包装器（向后兼容）
/// 内部调用拆分的命令模块
#[allow(dead_code)]
pub struct QuickCommand;

impl QuickCommand {
    /// 下载日志
    #[allow(dead_code)]
    pub fn download(jira_id: &str) -> Result<()> {
        DownloadCommand::download(jira_id)
    }

    /// 查找请求 ID
    #[allow(dead_code)]
    pub fn find_request_id(jira_id: &str, request_id: Option<String>) -> Result<()> {
        FindCommand::find_request_id(jira_id, request_id)
    }

    /// 搜索关键词
    #[allow(dead_code)]
    pub fn search(jira_id: &str, search_term: Option<String>) -> Result<()> {
        SearchCommand::search(jira_id, search_term)
    }
}
