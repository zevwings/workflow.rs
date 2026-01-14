//! Jira 附件下载常量定义

/// 默认输出文件夹名称
pub const DEFAULT_OUTPUT_FOLDER: &str = "merged";

/// 下载文件夹名称
pub const DOWNLOADS_FOLDER: &str = "downloads";

/// 日志 ZIP 文件名
pub const LOG_ZIP_FILENAME: &str = "log.zip";

/// 合并后的 ZIP 文件名
pub const MERGED_ZIP_FILENAME: &str = "merged.zip";

/// 日志 ZIP 分片文件前缀
pub const LOG_ZIP_SPLIT_PREFIX: &str = "log.z";

/// 日志文件扩展名
pub const LOG_EXTENSIONS: &[&str] = &[".log", ".txt", ".zip"];
