//! 文件操作错误消息

/// 创建目录失败
pub const FILE_CREATE_DIR_FAILED: &str = "Failed to create directory";

/// 创建临时目录失败
pub const FILE_CREATE_TEMP_DIR_FAILED: &str = "Failed to create temp directory";

/// 创建父目录失败
pub const FILE_CREATE_PARENT_DIR_FAILED: &str = "Failed to create parent dir";

/// 创建配置目录失败
pub const FILE_CREATE_CONFIG_DIR_FAILED: &str = "Failed to create config dir";

/// 读取文件失败
pub const FILE_READ_FILE_FAILED: &str = "Failed to read file";

/// 读取配置文件失败
pub const FILE_READ_CONFIG_FAILED: &str = "Failed to read config file";

/// 读取完成文件失败
pub const FILE_READ_COMPLETION_FILE_FAILED: &str = "Failed to read completion file";

/// 读取夹具文件失败
pub const FILE_READ_FIXTURE_FAILED: &str = "Failed to read fixture";

/// 写入文件失败
pub const FILE_WRITE_FILE_FAILED: &str = "Failed to write file";

/// 写入配置失败
pub const FILE_WRITE_CONFIG_FAILED: &str = "Failed to write config";

// 向后兼容的常量别名
/// 创建临时目录失败（别名）
pub const CREATE_TEMP_DIR_FAILED: &str = FILE_CREATE_TEMP_DIR_FAILED;

/// 创建父目录失败（别名）
pub const CREATE_PARENT_DIR_FAILED: &str = FILE_CREATE_PARENT_DIR_FAILED;

/// 写入文件失败（别名）
pub const WRITE_FILE_FAILED: &str = FILE_WRITE_FILE_FAILED;

/// 创建配置目录失败（别名）
pub const CREATE_CONFIG_DIR_FAILED: &str = FILE_CREATE_CONFIG_DIR_FAILED;

/// 写入配置失败（别名）
pub const WRITE_CONFIG_FAILED: &str = FILE_WRITE_CONFIG_FAILED;

/// 写入序列编辑器脚本失败
pub const FILE_WRITE_SEQUENCE_EDITOR_SCRIPT_FAILED: &str = "Failed to write sequence editor script";

/// 写入消息编辑器脚本失败
pub const FILE_WRITE_MESSAGE_EDITOR_SCRIPT_FAILED: &str = "Failed to write message editor script";

/// 写入 rebase todo 文件失败
pub const FILE_WRITE_REBASE_TODO_FAILED: &str = "Failed to write rebase todo file";

/// 写入提交消息文件失败
pub const FILE_WRITE_COMMIT_MESSAGE_FAILED: &str = "Failed to write commit message file";

/// 写入工作流完成配置文件失败
pub const FILE_WRITE_WORKFLOW_COMPLETION_CONFIG_FAILED: &str =
    "Failed to write workflow completion config file";
