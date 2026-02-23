//! Merge 相关实体

/// 合并策略
///
/// 定义不同的 Git 合并策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeStrategy {
    /// 普通合并（创建合并提交）
    Merge,
    /// Squash 合并（将分支的所有提交压缩为一个提交）
    Squash,
    /// 只允许 fast-forward 合并（如果无法 fast-forward 则失败）
    FastForwardOnly,
}
