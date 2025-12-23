//! Git 仓库类型定义

/// Git 仓库类型
///
/// 用于标识远程仓库的类型，以便使用不同的 API 或处理逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoType {
    /// GitHub 仓库
    GitHub,
    /// Codeup 仓库（检测支持，但 PR 功能不支持）
    Codeup,
    /// 未知类型的仓库
    Unknown,
}
