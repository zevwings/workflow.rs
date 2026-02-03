//! 进度条格式化函数
//!
//! 提供进度条文本格式化的辅助函数

use crate::output::progress::bar::ProgressMode;
use crate::style::theme::Theme;
use std::time::{Duration, Instant};

/// 进度条格式化参数
#[derive(Debug, Clone)]
pub(crate) struct ProgressFormatParams<'a> {
    /// 消息文本
    pub message: &'a str,
    /// 总数（可选）
    pub total: Option<u64>,
    /// 当前值
    pub current: u64,
    /// 开始时间（可选）
    pub start_time: Option<Instant>,
    /// 进度模式
    pub mode: ProgressMode,
    /// 进度条宽度
    pub bar_width: usize,
    /// 进度条字符
    pub progress_chars: &'a str,
    /// 主题样式
    pub theme: &'a Theme,
    /// 终端宽度（可选，用于截断输出）
    pub terminal_width: Option<usize>,
}

/// 格式化进度条文本
pub(crate) fn format_progress_text(params: &ProgressFormatParams<'_>) -> String {
    let chars: Vec<char> = params.progress_chars.chars().collect();
    if chars.len() < 2 {
        // 如果字符不足，使用默认字符
        return format!(
            "{} {}",
            params.theme.progress.apply(params.message, params.theme.enable_color),
            params.current
        );
    }

    let filled_char = chars[0];
    let empty_char = chars[chars.len() - 1];

    // 格式化时间信息
    let time_info = if let Some(start) = params.start_time {
        let elapsed = start.elapsed();
        format_elapsed_time(elapsed)
    } else {
        String::new()
    };

    // 组合所有部分
    let mut parts = Vec::new();

    if let Some(total_val) = params.total {
        // 已知总数：显示进度条和百分比
        let percent = if total_val > 0 {
            (params.current as f64 / total_val as f64 * 100.0).min(100.0)
        } else {
            100.0
        };

        let filled_width = (params.bar_width as f64 * percent / 100.0) as usize;
        let empty_width = params.bar_width.saturating_sub(filled_width);

        let bar_str = format!(
            "{}{}",
            filled_char.to_string().repeat(filled_width),
            empty_char.to_string().repeat(empty_width)
        );

        let bar_styled = params.theme.progress.apply(&bar_str, params.theme.enable_color);
        parts.push(bar_styled);

        // 根据模式显示不同的统计信息
        if matches!(params.mode, ProgressMode::Download) {
            // 下载模式：显示字节数、速度、ETA
            let bytes_str = format_bytes(params.current);
            let total_bytes_str = format_bytes(total_val);

            // 计算平均速度（使用总时间和总进度）
            let speed = if let Some(start) = params.start_time {
                let elapsed = start.elapsed();
                if elapsed.as_secs_f64() > 0.0 && params.current > 0 {
                    params.current as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let speed_str = format!("{}/s", format_bytes(speed as u64));

            // 计算 ETA
            let eta_str = if speed > 0.0 && params.current < total_val {
                let remaining = total_val - params.current;
                let eta_secs = (remaining as f64 / speed) as u64;
                format!("ETA: {}", format_duration(Duration::from_secs(eta_secs)))
            } else {
                String::new()
            };

            let stats_str = if eta_str.is_empty() {
                format!(
                    "{}/{} ({:.0}%) {}",
                    bytes_str, total_bytes_str, percent, speed_str
                )
            } else {
                format!(
                    "{}/{} ({:.0}%) {} {}",
                    bytes_str, total_bytes_str, percent, speed_str, eta_str
                )
            };
            let stats_styled = params.theme.progress.apply(&stats_str, params.theme.enable_color);
            parts.push(stats_styled);
        } else {
            // 普通模式：显示数量
            let stats_str = format!("{}/{} ({:.0}%)", params.current, total_val, percent);
            let stats_styled = params.theme.progress.apply(&stats_str, params.theme.enable_color);
            parts.push(stats_styled);
        }
    } else {
        // 未知总数：显示 spinner 和当前值
        // 使用 spinner 字符序列
        let spinner_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let elapsed =
            params.start_time.map(|s| s.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
        let frame_idx = (elapsed.as_millis() / 100) as usize % spinner_frames.len();
        let spinner_char = spinner_frames[frame_idx];

        let spinner_styled = params.theme.progress.apply(spinner_char, params.theme.enable_color);
        parts.push(spinner_styled);

        // 显示当前值
        if params.current > 0 {
            let current_str = if matches!(params.mode, ProgressMode::Download) {
                format_bytes(params.current)
            } else {
                format!("{}", params.current)
            };
            let current_styled =
                params.theme.progress.apply(&current_str, params.theme.enable_color);
            parts.push(current_styled);
        }
    }

    // 时间信息
    if !time_info.is_empty() {
        let time_styled = params.theme.progress.apply(&time_info, params.theme.enable_color);
        parts.push(time_styled);
    }

    // 消息
    if !params.message.is_empty() {
        let msg_styled = params.theme.progress.apply(params.message, params.theme.enable_color);
        parts.push(msg_styled);
    }

    let result = parts.join(" ");

    // 根据终端宽度截断输出
    if let Some(term_width) = params.terminal_width {
        if term_width > 0 {
            truncate_to_width(&result, term_width)
        } else {
            result
        }
    } else {
        result
    }
}

/// 格式化已用时间
fn format_elapsed_time(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    if secs < 60 {
        format!("[{}.{:02}s]", secs, elapsed.subsec_millis() / 10)
    } else if secs < 3600 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("[{}m{:02}s]", mins, secs)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("[{}h{:02}m{:02}s]", hours, mins, secs)
    }
}

/// 格式化持续时间（用于 ETA）
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m{:02}s", mins, secs)
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;
        format!("{}h{:02}m{:02}s", hours, mins, secs)
    }
}

/// 格式化字节数（人类可读格式）
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let exp = (bytes_f.ln() / THRESHOLD.ln()).floor() as usize;
    let exp = exp.min(UNITS.len() - 1);
    let value = bytes_f / THRESHOLD.powi(exp as i32);

    if exp == 0 {
        format!("{} {}", bytes, UNITS[exp])
    } else {
        format!("{:.1} {}", value, UNITS[exp])
    }
}

/// 按显示宽度截断字符串（正确处理 ANSI 转义序列）
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut display_width = 0;
    let mut in_escape = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // 开始 ANSI 转义序列
            in_escape = true;
            result.push(c);
        } else if in_escape {
            result.push(c);
            // ANSI 序列以字母结束（a-z, A-Z）
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else {
            // 计算字符的显示宽度
            let char_width = unicode_width(c);
            if display_width + char_width > max_width {
                break;
            }
            result.push(c);
            display_width += char_width;
        }
    }

    // 确保关闭所有 ANSI 样式（重置）
    if result.contains('\x1b') {
        result.push_str("\x1b[0m");
    }

    result
}

/// 计算字符的显示宽度
fn unicode_width(c: char) -> usize {
    // 简单的 Unicode 宽度估算
    // 大多数 CJK 字符和表情符号占 2 个宽度
    // ASCII 和大多数拉丁字符占 1 个宽度
    if c.is_ascii() {
        1
    } else {
        // 使用 Unicode 块来估算宽度
        let code = c as u32;
        match code {
            // CJK 统一表意文字
            0x4E00..=0x9FFF => 2,
            // CJK 扩展
            0x3400..=0x4DBF | 0x20000..=0x2A6DF => 2,
            // 全角字符
            0xFF00..=0xFFEF => 2,
            // 表情符号（大多数）
            0x1F300..=0x1F9FF => 2,
            // 方框绘制字符（如进度条字符）
            0x2580..=0x259F => 1,
            // 其他默认为 1
            _ => 1,
        }
    }
}
