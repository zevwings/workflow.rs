//! 工具包（Toolkit）
//!
//! 通用工具、HTTP 客户端、日志、配置等
//! 可以被所有层级使用
//!
//! ## 使用方式
//!
//! 所有 API 都通过 `toolkit::{xx}` 直接访问：
//! ```rust
//! use toolkit::{HttpClient, TemplateEngine, Platform, log_info};
//! ```

pub mod http;
pub mod logger;
pub mod paths;
pub mod rollback;
pub mod shell;
pub mod template;
pub mod terminal;
pub mod util;

// Private module to re-export tracing for use in exported macros
// This allows macros to use $crate::__tracing::debug! etc.
#[doc(hidden)]
pub mod __tracing {
    pub use tracing::{debug, error, info, warn};
}

// ============================================================================
// 统一重新导出主要公共 API
// ============================================================================

// HTTP 客户端模块
pub use http::{
    Authorization, HttpClient, HttpClientConfig, HttpError, HttpMethod, HttpMethodError,
    HttpResponse, HttpRetry, HttpRetryConfig, HttpRetryError, IntoHeaderMap,
    MultipartRequestConfig, RequestConfig, RetryResult,
};

// Logger 模块
pub use logger::init as logger_init;
pub use logger::{LoggerConfig, LoggerError};

// Paths 模块 - 导出路径相关函数
pub use paths::{
    binary_install_dir, binary_name, binary_paths, command_names, commands_config_path,
    completion_cache_dir, completion_cache_dir_shell_path, completion_dir,
    completion_dir_shell_path, completion_file_shell_path, completion_source_shell_path,
    config_dir, default_download_base_dir, expand, is_config_in_icloud, jira_config_path,
    llm_config_path, logs_dir, project_config_file, project_config_path, repo_dir,
    repo_workflow_dir, repository_config_path, storage_info, storage_location, user_config_file,
    work_history_dir, workflow_config_path, workflow_dir, PathError,
};

// Template 模块
pub use template::{TemplateEngine, TemplateEngineType, TemplateError};

// Shell 模块
pub use shell::{
    add_source, config_file_path, detect_shell, has_source, is_configured, reload_hint,
    remove_source, shell_from_string, shell_to_string, supported_shells, ShellError,
};

// Util 模块 - 导出所有工具类型和函数
pub use util::{
    build_checksum_url, calculate_sha256, parse_hash_from_content, verify_checksum,
    verify_checksum_lenient, Browser, BrowserError, BrowserExt, ChecksumError,
    ChecksumVerifyResult, ClipboardError, ClipboardExt, FileError, PathExt, Platform,
    PlatformError, Sensitive, SizeExt, Truncate,
};
// FS 模块 - 导出文件系统操作模块
pub use util::fs::{archive, directory, file};

// Rollback 模块 - 导出回滚相关类型和函数
pub use rollback::{
    cleanup_backup, create_backup, get_all_completion_files, get_completion_filename,
    get_completion_files_for_shell, reload_shell, rollback, BackupInfo, BackupResult,
    CompletionHelperError, ReloadError, ReloadResult, RollbackError, RollbackResult,
};

// Terminal 模块 - 导出终端协调相关类型和函数
pub use terminal::{register_spinner_handlers, SpinnerAwareLayer};
