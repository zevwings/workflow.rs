//! Blame 相关实体

/// Blame 行信息
///
/// 表示文件中某一行的 blame 信息，包括该行的作者、提交信息等。
#[derive(Debug, Clone)]
pub struct BlameLineInfo {
    /// 行号（从 1 开始）
    pub line_number: usize,
    /// 行内容
    pub line_content: String,
    /// 提交 SHA
    pub commit_sha: String,
    /// 作者名称
    pub author: String,
    /// 作者邮箱
    pub author_email: String,
    /// 提交时间（Unix 时间戳）
    pub commit_time: i64,
    /// 提交消息（第一行）
    pub commit_message: String,
    /// 原始提交 SHA（如果该行是从其他文件移动过来的）
    pub original_commit_sha: Option<String>,
    /// 原始文件路径（如果该行是从其他文件移动过来的）
    pub original_file_path: Option<String>,
}
