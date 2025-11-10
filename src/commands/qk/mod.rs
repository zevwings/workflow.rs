pub mod clean;
pub mod download;
pub mod find;
pub mod info;
pub mod search;

// 为了向后兼容，保留 QuickCommand 作为统一接口
pub use clean::CleanCommand;
pub use download::DownloadCommand;
pub use find::FindCommand;
pub use info::InfoCommand;
pub use search::SearchCommand;

use anyhow::Result;

/// Quick 统一命令包装器（向后兼容）
/// 内部调用拆分的命令模块
#[allow(dead_code)]
pub struct QuickCommand;

impl QuickCommand {
    /// 下载日志
    #[allow(dead_code)]
    pub fn download(jira_id: &str, download_all: bool) -> Result<()> {
        DownloadCommand::download(jira_id, download_all)
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

    /// 清理日志目录
    #[allow(dead_code)]
    pub fn clean(jira_id: &str, dry_run: bool, list_only: bool) -> Result<()> {
        CleanCommand::clean(jira_id, dry_run, list_only)
    }

    /// 显示 ticket 信息
    #[allow(dead_code)]
    pub fn show(jira_id: &str) -> Result<()> {
        InfoCommand::show(jira_id)
    }
}
