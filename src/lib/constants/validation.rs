//! 验证相关常量
//!
//! 统一管理各种验证场景的错误消息和规则。

/// 分支名称验证错误消息
pub mod branch {
    /// 分支名称不能为空
    pub const EMPTY_NAME: &str = "Branch name cannot be empty";

    /// 分支名称不能以 '.' 开头或结尾
    pub const INVALID_DOT_POSITION: &str = "Branch name cannot start or end with '.'";

    /// 分支名称不能包含 '..'
    pub const DOUBLE_DOT: &str = "Branch name cannot contain '..'";

    /// 分支名称不能包含空格
    pub const CONTAINS_SPACES: &str = "Branch name cannot contain spaces";

    /// 分支名称不能包含特殊字符
    pub const INVALID_SPECIAL_CHAR: &str = "Branch name cannot contain special character";

    /// 分支名称不能以 '/' 结尾
    pub const TRAILING_SLASH: &str = "Branch name cannot end with '/'";

    /// 分支名称不能包含连续的斜杠 '//'
    pub const DOUBLE_SLASH: &str = "Branch name cannot contain consecutive slashes '//'";

    /// 分支名称不能是保留名称
    pub const RESERVED_NAME: &str = "Branch name cannot be reserved name";
}

/// 配置验证消息
pub mod config {
    /// 配置验证失败
    pub const VALIDATION_FAILED: &str = "Configuration validation failed";

    /// 配置标题
    pub const HEADER: &str = "Configuration";

    /// 不支持的 shell 类型
    pub const UNSUPPORTED_SHELL: &str = "Unsupported shell type";
}
