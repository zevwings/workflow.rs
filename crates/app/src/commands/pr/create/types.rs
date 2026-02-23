//! PR 创建相关类型定义

use domain::GitRepository;

/// 分支处理方式选项
#[derive(Clone)]
pub enum BranchHandleOption {
    /// 直接使用当前分支
    UseCurrentBranch(String),
    /// 基于当前分支创建新分支
    CreateFromCurrent(String),
    /// 切换到默认分支，创建新分支
    SwitchToDefault(String),
}

impl std::fmt::Display for BranchHandleOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchHandleOption::UseCurrentBranch(branch) => {
                write!(f, "Use current branch directly ({})", branch)
            }
            BranchHandleOption::CreateFromCurrent(branch) => {
                write!(f, "Create new branch from current ({})", branch)
            }
            BranchHandleOption::SwitchToDefault(branch) => {
                write!(f, "Switch to default branch and create new ({})", branch)
            }
        }
    }
}

/// 目标分支选项
#[derive(Clone)]
pub enum TargetBranchOption {
    /// 合并到当前分支
    Current(String),
    /// 合并到推断的分支
    Inferred(String),
    /// 合并到默认分支
    Default(String),
}

impl std::fmt::Display for TargetBranchOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetBranchOption::Current(branch) => {
                write!(f, "Merge to current branch: {}", branch)
            }
            TargetBranchOption::Inferred(branch) => {
                write!(f, "Merge to inferred branch: {}", branch)
            }
            TargetBranchOption::Default(branch) => {
                write!(f, "Merge to default branch: {}", branch)
            }
        }
    }
}

impl TargetBranchOption {
    /// 获取分支名
    pub fn branch_name(&self) -> &str {
        match self {
            TargetBranchOption::Current(branch)
            | TargetBranchOption::Inferred(branch)
            | TargetBranchOption::Default(branch) => branch,
        }
    }
}

/// 确认操作选项
#[derive(Clone, PartialEq)]
pub enum ConfirmOption {
    /// 确认执行
    Yes,
    /// 取消操作
    No,
}

impl std::fmt::Display for ConfirmOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmOption::Yes => write!(f, "Yes, update PR"),
            ConfirmOption::No => write!(f, "No, cancel"),
        }
    }
}

/// 分支处理上下文
///
/// 封装处理非默认分支时所需的所有参数
pub struct BranchHandleContext<'a> {
    /// Git 仓库引用
    pub branch_repo: &'a dyn GitRepository,
    /// 当前分支名
    pub current_branch: &'a str,
    /// 默认分支名
    pub default_branch: &'a str,
    /// 生成的新分支名
    pub generated_branch_name: &'a str,
    /// JIRA ID（可选）
    pub jira_id: &'a Option<String>,
    /// 描述信息（可选）
    pub description: Option<&'a str>,
}
