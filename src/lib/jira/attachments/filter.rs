//! 附件过滤逻辑

use crate::{trace_debug, JiraAttachment};
use regex::Regex;

/// 附件过滤器
///
/// 提供附件过滤功能，主要用于识别和过滤日志附件。
pub struct AttachmentFilter;

impl AttachmentFilter {
    /// 过滤日志附件
    ///
    /// 根据以下规则过滤日志附件：
    /// 1. log.zip 或 log.z[0-9]+ 格式的文件（如 log.zip, log.z01, log.z02 等）
    /// 2. 以 .log 结尾的文件（如 any_file.log, error.log 等）
    /// 3. 以 .txt 结尾的文件（如 metric0.txt, log0.txt, network3.txt 等）
    ///
    /// # 参数
    ///
    /// * `attachments` - 要过滤的附件列表
    ///
    /// # 返回
    ///
    /// 返回过滤后的日志附件列表。
    pub fn filter_log_attachments(attachments: &[JiraAttachment]) -> Vec<JiraAttachment> {
        // 预先编译正则表达式，避免在循环中重复编译
        let log_zip_pattern = Regex::new(r"^log\.(zip|z\d+)$").unwrap();

        let log_attachments: Vec<_> = attachments
            .iter()
            .filter(|a| {
                // 匹配 log.zip 或 log.z[0-9]+ 格式（与 shell 脚本一致）
                let matches_log_zip = log_zip_pattern.is_match(&a.filename);
                // 匹配所有以 .log 结尾的文件
                let matches_log_ext = a.filename.ends_with(".log");
                // 匹配所有以 .txt 结尾的文件
                let matches_txt_ext = a.filename.ends_with(".txt");

                matches_log_zip || matches_log_ext || matches_txt_ext
            })
            .cloned()
            .collect();

        // 调试：显示过滤后的日志附件
        if !log_attachments.is_empty() {
            trace_debug!("Filtered {} log attachment(s):", log_attachments.len());
            for attachment in &log_attachments {
                trace_debug!("  - {}", attachment.filename);
            }
        }

        log_attachments
    }

    /// 检查是否为日志附件
    ///
    /// 判断给定的文件名是否为日志附件。
    ///
    /// # 参数
    ///
    /// * `filename` - 文件名
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果是日志附件，否则返回 `false`。
    #[allow(dead_code)]
    pub fn is_log_attachment(filename: &str) -> bool {
        let log_zip_pattern = Regex::new(r"^log\.(zip|z\d+)$").unwrap();
        log_zip_pattern.is_match(filename)
            || filename.ends_with(".log")
            || filename.ends_with(".txt")
    }
}
