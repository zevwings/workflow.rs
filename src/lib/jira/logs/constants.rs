//! Jira 日志处理常量定义

/// 默认输出文件夹名称
pub(crate) const DEFAULT_OUTPUT_FOLDER: &str = "merged";

/// 下载文件夹名称
pub(crate) const DOWNLOADS_FOLDER: &str = "downloads";

/// 日志 ZIP 文件名
pub(crate) const LOG_ZIP_FILENAME: &str = "log.zip";

/// 合并后的 ZIP 文件名
pub(crate) const MERGED_ZIP_FILENAME: &str = "merged.zip";

/// 日志 ZIP 分片文件前缀
pub(crate) const LOG_ZIP_SPLIT_PREFIX: &str = "log.z";

/// Flutter API 日志文件前缀
pub(crate) const FLUTTER_API_LOG_PREFIX: &str = "flutter-api";

/// Flutter API 日志文件后缀
pub(crate) const FLUTTER_API_LOG_SUFFIX: &str = ".log";

/// 日志文件扩展名
pub(crate) const LOG_EXTENSIONS: &[&str] = &[".log", ".txt", ".zip"];

/// 响应关键字
pub(crate) const RESPONSE_KEYWORD: &str = "response:";

/// 响应关键字长度
pub(crate) const RESPONSE_KEYWORD_LEN: usize = 9;
