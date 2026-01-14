//! 验证相关常量
//!
//! 统一管理各种验证场景的错误消息和规则。

// 分支名称验证错误消息

/// 分支名称不能为空
pub const BRANCH_EMPTY_NAME: &str = "Branch name cannot be empty";

/// 分支名称不能以 '.' 开头或结尾
pub const BRANCH_INVALID_DOT_POSITION: &str = "Branch name cannot start or end with '.'";

/// 分支名称不能包含 '..'
pub const BRANCH_DOUBLE_DOT: &str = "Branch name cannot contain '..'";

/// 分支名称不能包含空格
pub const BRANCH_CONTAINS_SPACES: &str = "Branch name cannot contain spaces";

/// 分支名称不能包含特殊字符
pub const BRANCH_INVALID_SPECIAL_CHAR: &str = "Branch name cannot contain special character";

/// 分支名称不能以 '/' 结尾
pub const BRANCH_TRAILING_SLASH: &str = "Branch name cannot end with '/'";

/// 分支名称不能包含连续的斜杠 '//'
pub const BRANCH_DOUBLE_SLASH: &str = "Branch name cannot contain consecutive slashes '//'";

/// 分支名称不能是保留名称
pub const BRANCH_RESERVED_NAME: &str = "Branch name cannot be reserved name";

// 配置验证消息

/// 配置验证失败
pub const CONFIG_VALIDATION_FAILED: &str = "Configuration validation failed";

/// 配置标题
pub const CONFIG_HEADER: &str = "Configuration";

/// 不支持的 shell 类型
pub const CONFIG_UNSUPPORTED_SHELL: &str = "Unsupported shell type";
