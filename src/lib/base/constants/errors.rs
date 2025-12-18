//! 通用错误消息常量
//!
//! 统一管理项目中使用的错误消息，确保错误信息的一致性和用户体验。

/// 文件操作错误消息
pub mod file_operations {
    /// 创建目录失败
    pub const CREATE_DIR_FAILED: &str = "Failed to create directory";

    /// 创建临时目录失败
    pub const CREATE_TEMP_DIR_FAILED: &str = "Failed to create temp directory";

    /// 创建父目录失败
    pub const CREATE_PARENT_DIR_FAILED: &str = "Failed to create parent dir";

    /// 创建配置目录失败
    pub const CREATE_CONFIG_DIR_FAILED: &str = "Failed to create config dir";

    /// 读取文件失败
    pub const READ_FILE_FAILED: &str = "Failed to read file";

    /// 读取配置文件失败
    pub const READ_CONFIG_FAILED: &str = "Failed to read config file";

    /// 读取完成文件失败
    pub const READ_COMPLETION_FILE_FAILED: &str = "Failed to read completion file";

    /// 读取夹具文件失败
    pub const READ_FIXTURE_FAILED: &str = "Failed to read fixture";

    /// 写入文件失败
    pub const WRITE_FILE_FAILED: &str = "Failed to write file";

    /// 写入配置失败
    pub const WRITE_CONFIG_FAILED: &str = "Failed to write config";

    /// 写入序列编辑器脚本失败
    pub const WRITE_SEQUENCE_EDITOR_SCRIPT_FAILED: &str = "Failed to write sequence editor script";

    /// 写入消息编辑器脚本失败
    pub const WRITE_MESSAGE_EDITOR_SCRIPT_FAILED: &str = "Failed to write message editor script";

    /// 写入 rebase todo 文件失败
    pub const WRITE_REBASE_TODO_FAILED: &str = "Failed to write rebase todo file";

    /// 写入提交消息文件失败
    pub const WRITE_COMMIT_MESSAGE_FAILED: &str = "Failed to write commit message file";

    /// 写入工作流完成配置文件失败
    pub const WRITE_WORKFLOW_COMPLETION_CONFIG_FAILED: &str =
        "Failed to write workflow completion config file";
}

/// HTTP 客户端错误消息
pub mod http_client {
    /// 创建 HTTP 客户端失败
    pub const CREATE_CLIENT_FAILED: &str = "Failed to create HTTP client";
}

/// 输入读取错误消息
pub mod input_reading {
    /// 读取 Jira 票据 ID 失败
    pub const READ_JIRA_TICKET_ID_FAILED: &str = "Failed to read Jira ticket ID";

    /// 读取分支名称失败
    pub const READ_BRANCH_NAME_FAILED: &str = "Failed to read branch name";
}

/// 生成器创建错误消息
pub mod generator_creation {
    /// 创建生成器失败（带格式化参数）
    pub const CREATE_GENERATOR_FAILED_FORMAT: &str = "Failed to create generator for {}";

    /// 创建 zsh 生成器失败
    pub const CREATE_ZSH_GENERATOR_FAILED: &str = "Failed to create zsh generator";
}

/// 验证错误消息
pub mod validation_errors {
    /// 无效的 PR 编号
    pub const INVALID_PR_NUMBER: &str = "Invalid PR number";

    /// 无效的仓库格式
    pub const INVALID_REPO_FORMAT: &str = "Invalid repo format";

    /// 无效的 JIRA ID 格式
    pub const INVALID_JIRA_ID_FORMAT: &str = "Invalid JIRA ID format";

    /// JIRA ID 格式说明
    pub const JIRA_ID_FORMAT_HELP: &str = "Expected formats:\n\
        • Ticket ID: PROJ-123 (project code + hyphen + number)\n\
        • Project name: PROJ (letters, numbers, underscores only)";

    /// JIRA ID 不能为空
    pub const JIRA_ID_EMPTY: &str = "JIRA ID cannot be empty";

    /// JIRA ID 格式验证失败的完整消息模板
    pub const JIRA_ID_VALIDATION_ERROR_TEMPLATE: &str =
        "Invalid JIRA ID format.\n{}\n\nError details: {}";
}
