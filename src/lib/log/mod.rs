//! 日志处理模块
//! 提供日志文件的搜索、提取和处理功能

use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod clean;
pub mod download;
pub mod extract;
pub mod find;
pub mod parse;
pub mod search;
pub mod utils;
pub mod zip;

// 重新导出公共类型和函数
pub use clean::{clean_dir, get_base_dir_path};
pub use download::download_from_jira;
pub use extract::extract_url_from_line;
pub use find::{extract_response_content, find_log_file, find_request_id, get_log_file_path};
pub use parse::{add_entry_if_not_duplicate, parse_log_entry, LogEntry};
pub use search::search_keyword;
pub use utils::{calculate_dir_info, expand_path, format_size, list_dir_contents, open_log_file};
pub use zip::{extract_zip, merge_split_zips};

// 为了保持 Logs::xxx() 的调用方式，创建一个 Logs 结构体
pub struct Logs;

impl Logs {
    // 重新导出所有方法作为静态方法（使用包装函数）
    pub fn expand_path(path_str: &str) -> Result<PathBuf> {
        utils::expand_path(path_str)
    }

    pub fn open_log_file(log_file: &Path) -> Result<std::io::BufReader<std::fs::File>> {
        utils::open_log_file(log_file)
    }

    pub fn format_size(bytes: u64) -> String {
        utils::format_size(bytes)
    }

    pub fn calculate_dir_info(dir: &Path) -> Result<(u64, usize)> {
        utils::calculate_dir_info(dir)
    }

    pub fn list_dir_contents(dir: &Path) -> Result<Vec<PathBuf>> {
        utils::list_dir_contents(dir)
    }

    pub fn parse_log_entry(line: &str) -> Result<Option<LogEntry>> {
        parse::parse_log_entry(line)
    }

    pub fn extract_url_from_line(line: &str) -> Option<String> {
        extract::extract_url_from_line(line)
    }

    pub fn download_from_jira(
        jira_id: &str,
        log_output_folder_name: Option<&str>,
        download_all_attachments: bool,
    ) -> Result<PathBuf> {
        download::download_from_jira(jira_id, log_output_folder_name, download_all_attachments)
    }

    pub fn merge_split_zips(download_dir: &Path) -> Result<PathBuf> {
        zip::merge_split_zips(download_dir)
    }

    pub fn extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()> {
        zip::extract_zip(zip_path, output_dir)
    }

    pub fn find_request_id(log_file: &Path, request_id: &str) -> Result<Option<LogEntry>> {
        find::find_request_id(log_file, request_id)
    }

    pub fn extract_response_content(log_file: &Path, request_id: &str) -> Result<String> {
        find::extract_response_content(log_file, request_id)
    }

    pub fn get_log_file_path(jira_id: &str) -> Result<PathBuf> {
        find::get_log_file_path(jira_id)
    }

    /// 确保日志文件存在，否则返回错误
    ///
    /// 检查指定 JIRA ID 的日志文件是否存在，如果不存在则返回包含下载提示的错误。
    ///
    /// # 参数
    ///
    /// * `jira_id` - JIRA ticket ID
    ///
    /// # 返回
    ///
    /// 如果日志文件存在，返回文件路径；否则返回错误。
    ///
    /// # 错误
    ///
    /// 如果日志文件不存在，返回包含下载提示的错误信息。
    pub fn ensure_log_file_exists(jira_id: &str) -> Result<PathBuf> {
        let log_file = Self::get_log_file_path(jira_id)?;
        if !log_file.exists() {
            anyhow::bail!(
                "Log file not found at: {:?}\nTry downloading logs first with: workflow qk {} download",
                log_file, jira_id
            );
        }
        Ok(log_file)
    }

    pub fn find_log_file(base_dir: &Path) -> Result<PathBuf> {
        find::find_log_file(base_dir)
    }

    pub fn search_keyword(log_file: &Path, keyword: &str) -> Result<Vec<LogEntry>> {
        search::search_keyword(log_file, keyword)
    }

    pub fn clean_dir(dir: &Path, dir_name: &str, dry_run: bool, list_only: bool) -> Result<bool> {
        clean::clean_dir(dir, dir_name, dry_run, list_only)
    }

    pub fn get_base_dir_path() -> Result<PathBuf> {
        clean::get_base_dir_path()
    }
}
