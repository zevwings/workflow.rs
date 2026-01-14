//! 配置块处理工具
//!
//! 提供 shell 配置文件中配置块的通用处理功能，包括：
//! - 配置块标记常量
//! - 配置块边界查找
//! - 配置块移除
//! - 配置块内容提取

/// 配置块开始标记
pub const MARKER_START: &str = "# Workflow CLI Configuration - Start";

/// 配置块结束标记
pub const MARKER_END: &str = "# Workflow CLI Configuration - End";

/// 配置块边界位置
#[derive(Debug, Clone, Copy)]
pub struct BlockBoundaries {
    /// 开始标记的起始位置
    pub start_pos: usize,
    /// 结束标记的结束位置
    pub end_pos: usize,
}

/// 使用自定义标记查找配置块边界
///
/// # 参数
///
/// * `content` - 要搜索的内容
/// * `marker_start` - 开始标记
/// * `marker_end` - 结束标记
///
/// # 返回
///
/// 如果找到配置块，返回 `Some(BlockBoundaries)`；否则返回 `None`。
pub fn find_boundaries_with_markers(
    content: &str,
    marker_start: &str,
    marker_end: &str,
) -> Option<BlockBoundaries> {
    if let Some(start_pos) = content.find(marker_start) {
        if let Some(relative_end) = content[start_pos..].find(marker_end) {
            let end_pos = start_pos + relative_end + marker_end.len();
            return Some(BlockBoundaries { start_pos, end_pos });
        }
    }
    None
}

/// 提取配置块内容
///
/// 从内容中提取配置块内的文本（不包括标记行）。
///
/// # 返回
///
/// 如果找到配置块，返回块内容；否则返回空字符串。
pub fn extract_content(content: &str) -> String {
    extract_content_with_markers(content, MARKER_START, MARKER_END)
}

/// 使用自定义标记提取配置块内容
///
/// # 参数
///
/// * `content` - 要搜索的内容
/// * `marker_start` - 开始标记
/// * `marker_end` - 结束标记
///
/// # 返回
///
/// 如果找到配置块，返回块内容；否则返回空字符串。
pub fn extract_content_with_markers(content: &str, marker_start: &str, marker_end: &str) -> String {
    if let Some(boundaries) = find_boundaries_with_markers(content, marker_start, marker_end) {
        let block_start = boundaries.start_pos + marker_start.len();
        let block_end = boundaries.end_pos - marker_end.len();
        return content[block_start..block_end].to_string();
    }
    String::new()
}

/// 移除配置块
///
/// 从内容中移除配置块（包括标记行），保留块前后的内容。
///
/// # 返回
///
/// 返回移除配置块后的内容。
pub fn remove(content: &str) -> String {
    remove_with_markers(content, MARKER_START, MARKER_END)
}

/// 使用自定义标记移除配置块
///
/// # 参数
///
/// * `content` - 要处理的内容
/// * `marker_start` - 开始标记
/// * `marker_end` - 结束标记
///
/// # 返回
///
/// 返回移除配置块后的内容。
pub fn remove_with_markers(content: &str, marker_start: &str, marker_end: &str) -> String {
    if let Some(boundaries) = find_boundaries_with_markers(content, marker_start, marker_end) {
        let before = content[..boundaries.start_pos].trim_end();
        let after = content[boundaries.end_pos..].trim_start();

        if before.is_empty() {
            after.to_string()
        } else if after.is_empty() {
            format!("{}\n", before)
        } else {
            format!("{}\n{}", before, after)
        }
    } else {
        content.to_string()
    }
}

/// 检查行是否可能是配置块相关行
///
/// 检查给定行是否包含 "Workflow CLI" 标记，用于识别配置块区域。
/// 这包括配置块标记和相关的注释行。
///
/// # 参数
///
/// * `line` - 要检查的行
///
/// # 返回
///
/// 如果行可能属于配置块区域，返回 `true`。
pub fn is_related_line(line: &str) -> bool {
    line.contains("# Workflow CLI")
        && (line.contains("completions") || line.contains("Configuration"))
}

/// 检查配置块是否为空
///
/// 检查配置块内是否包含任何非注释、非空的内容。
///
/// # 参数
///
/// * `content` - 配置块内容（不包括标记行）
/// * `predicate` - 用于判断行是否有意义的谓词函数
///
/// # 返回
///
/// 如果配置块为空（只有注释和空行），返回 `true`。
pub fn is_empty<F>(content: &str, predicate: F) -> bool
where
    F: Fn(&str) -> bool,
{
    !content.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.starts_with('#') && predicate(trimmed)
    })
}
