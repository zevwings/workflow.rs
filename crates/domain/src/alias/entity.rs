//! 别名实体定义

use serde::{Deserialize, Serialize};

// ============================================================================
// 别名实体
// ============================================================================

/// 别名信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasInfo {
    /// 别名名称
    pub name: String,
    /// 对应的命令
    pub command: String,
}

impl AliasInfo {
    /// 创建新的别名信息
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
        }
    }
}

// ============================================================================
// 操作结果
// ============================================================================

/// 别名列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasListResult {
    /// 别名列表
    pub aliases: Vec<AliasInfo>,
    /// 别名总数
    pub count: usize,
}

/// 别名添加结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasAddResult {
    /// 添加的别名名称
    pub name: String,
    /// 对应的命令
    pub command: String,
    /// 是否为覆盖操作（别名已存在）
    pub overwritten: bool,
}

/// 别名移除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasRemoveResult {
    /// 移除的别名名称
    pub name: String,
    /// 对应的命令（移除前的）
    pub command: String,
}
