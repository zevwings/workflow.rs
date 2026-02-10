//! 平台账户类型定义

/// 账户设置模式
///
/// 控制新添加的账户是否设为当前账户
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccountSetMode {
    /// 设为当前账户
    #[default]
    SetAsCurrent,
    /// 仅添加，不设为当前
    #[allow(dead_code)]
    AddOnly,
}

impl AccountSetMode {
    /// 是否应该设为当前账户
    #[inline]
    pub fn should_set_current(self) -> bool {
        matches!(self, Self::SetAsCurrent)
    }
}

/// 账户操作选项
#[derive(Clone)]
pub enum AccountAction {
    /// 保留当前账户 (Setup 模式)
    KeepCurrent { account_display: String },
    /// 使用已有账户 (Setup 模式)
    UseExisting {
        account_display: String,
        account_name: String,
    },
    /// 添加新账户
    AddNew { platform_name: String },
    /// 切换当前账户 (Command 模式)
    Switch { platform_name: String },
    /// 更新账户信息 (Command 模式)
    Update { platform_name: String },
    /// 删除账户 (Command 模式)
    Remove { platform_name: String },
}

impl std::fmt::Display for AccountAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountAction::KeepCurrent { account_display } => {
                write!(f, "Keep current account {}", account_display)
            }
            AccountAction::UseExisting {
                account_display, ..
            } => {
                write!(f, "Use existing account {}", account_display)
            }
            AccountAction::AddNew { platform_name } => {
                write!(f, "Add new {} account", platform_name)
            }
            AccountAction::Switch { platform_name } => {
                write!(f, "Switch current {} account", platform_name)
            }
            AccountAction::Update { platform_name } => {
                write!(f, "Update {} account information", platform_name)
            }
            AccountAction::Remove { platform_name } => {
                write!(f, "Remove {} account", platform_name)
            }
        }
    }
}
