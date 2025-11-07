//! Git 仓库类型定义

/// Git 仓库类型
///
/// 用于标识远程仓库的类型，以便使用不同的 API 或处理逻辑。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoType {
    /// GitHub 仓库
    GitHub,
    /// 阿里云 Codeup 仓库
    Codeup,
    /// 未知类型的仓库
    Unknown,
}
