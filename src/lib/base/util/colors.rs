//! 颜色输出兼容层（使用 console）
//!
//! 提供统一的颜色输出接口，替代 colored crate。
//! 使用 console crate 实现，提供更丰富的功能和更好的终端支持。
//!
//! ## 功能
//!
//! - 支持多种日志级别样式（success, error, warning, info, debug）
//! - 支持分隔线样式（separator, separator_with_text）
//! - 自动 Emoji 支持（终端不支持时回退到 ASCII）
//! - 自动检测终端能力

use console::{style, Emoji};

/// 成功消息样式（绿色 ✅）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含绿色样式和成功图标
///
/// # 示例
/// ```
/// use workflow::base::util::colors::success;
/// let msg = success("Operation completed");
/// println!("{}", msg);
/// ```
pub fn success(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("✅", "✓"), style(text).green().bold())
}

/// 错误消息样式（红色 ❌）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含红色样式和错误图标
///
/// # 示例
/// ```
/// use workflow::base::util::colors::error;
/// let msg = error("Operation failed");
/// println!("{}", msg);
/// ```
pub fn error(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("❌", "✗"), style(text).red().bold())
}

/// 警告消息样式（黄色 ⚠️）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含黄色样式和警告图标
///
/// # 示例
/// ```
/// use workflow::base::util::colors::warning;
/// let msg = warning("This is a warning");
/// println!("{}", msg);
/// ```
pub fn warning(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("⚠️", "⚠"), style(text).yellow().bold())
}

/// 信息消息样式（蓝色 ℹ️）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含蓝色样式和信息图标
///
/// # 示例
/// ```
/// use workflow::base::util::colors::info;
/// let msg = info("Processing data");
/// println!("{}", msg);
/// ```
pub fn info(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("ℹ️", "ℹ"), style(text).blue().bold())
}

/// 调试消息样式（灰色 🔧）
///
/// # 参数
/// * `text` - 要格式化的文本
///
/// # 返回
/// 格式化后的字符串，包含灰色样式和调试图标
///
/// # 示例
/// ```
/// use workflow::base::util::colors::debug;
/// let msg = debug("Debug information");
/// println!("{}", msg);
/// ```
pub fn debug(text: impl std::fmt::Display) -> String {
    format!("{} {}", Emoji("🔧", "⚙"), style(text).bright().black())
}

/// 分隔线样式（灰色）
///
/// # 参数
/// * `char` - 分隔符字符
/// * `length` - 分隔线长度
///
/// # 返回
/// 格式化后的分隔线字符串
///
/// # 示例
/// ```
/// use workflow::base::util::colors::separator;
/// let sep = separator('-', 80);
/// println!("{}", sep);
/// ```
pub fn separator(char: char, length: usize) -> String {
    style(char.to_string().repeat(length))
        .bright()
        .black()
        .to_string()
}

/// 带文本的分隔线样式
///
/// 在分隔线中间插入文本，文本前后用分隔符字符填充。
/// 文本前后会自动添加空格。
///
/// # 参数
/// * `char` - 分隔符字符
/// * `length` - 总长度
/// * `text` - 要插入的文本
///
/// # 返回
/// 格式化后的带文本分隔线字符串
///
/// # 示例
/// ```
/// use workflow::base::util::colors::separator_with_text;
/// let sep = separator_with_text('=', 80, "Section Title");
/// println!("{}", sep);
/// ```
pub fn separator_with_text(char: char, length: usize, text: impl std::fmt::Display) -> String {
    let text_str = format!("  {} ", text);
    let text_len = text_str.chars().count();

    // 如果文本长度大于等于总长度，直接输出文本
    if text_len >= length {
        return style(text_str).bright().black().to_string();
    }

    // 计算左右两侧需要填充的字符数
    let remaining = length - text_len;
    let left_padding = remaining / 2;
    let right_padding = remaining - left_padding;

    // 生成分隔线
    let left_sep = char.to_string().repeat(left_padding);
    let right_sep = char.to_string().repeat(right_padding);

    format!(
        "{}{}{}",
        style(left_sep).bright().black(),
        text_str,
        style(right_sep).bright().black()
    )
}
