//! 进度条格式化函数
//!
//! 提供进度条文本格式化的辅助函数

use crate::core::prompt::style::theme::Theme;
use std::time::{Duration, Instant};

use super::bar::ProgressMode;

/// 格式化进度条文本
pub(crate) fn format_progress_text(
    message: &str,
    total: Option<u64>,
    current: u64,
    start_time: Option<Instant>,
    mode: ProgressMode,
    bar_width: usize,
    progress_chars: &str,
    theme: &Theme,
) -> String {
    let chars: Vec<char> = progress_chars.chars().collect();
    if chars.len() < 2 {
        // 如果字符不足，使用默认字符
        return format!(
            "{} {}",
            theme.progress.apply(message, theme.enable_color),
            current
        );
    }

    let filled_char = chars[0];
    let empty_char = chars[chars.len() - 1];

    // 格式化时间信息
    let time_info = if let Some(start) = start_time {
        let elapsed = start.elapsed();
        format_elapsed_time(elapsed)
    } else {
        String::new()
    };

    // 组合所有部分
    let mut parts = Vec::new();

    if let Some(total_val) = total {
        // 已知总数：显示进度条和百分比
        let percent = if total_val > 0 {
            (current as f64 / total_val as f64 * 100.0).min(100.0)
        } else {
            100.0
        };

        let filled_width = (bar_width as f64 * percent / 100.0) as usize;
        let empty_width = bar_width.saturating_sub(filled_width);

        let bar_str = format!(
            "{}{}",
            filled_char.to_string().repeat(filled_width),
            empty_char.to_string().repeat(empty_width)
        );

        let bar_styled = theme.progress.apply(&bar_str, theme.enable_color);
        parts.push(bar_styled);

        // 根据模式显示不同的统计信息
        if matches!(mode, ProgressMode::Download) {
            // 下载模式：显示字节数、速度、ETA
            let bytes_str = format_bytes(current);
            let total_bytes_str = format_bytes(total_val);

            // 计算平均速度（使用总时间和总进度）
            let speed = if let Some(start) = start_time {
                let elapsed = start.elapsed();
                if elapsed.as_secs_f64() > 0.0 && current > 0 {
                    current as f64 / elapsed.as_secs_f64()
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let speed_str = format!("{}/s", format_bytes(speed as u64));

            // 计算 ETA
            let eta_str = if speed > 0.0 && current < total_val {
                let remaining = total_val - current;
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
            let stats_styled = theme.progress.apply(&stats_str, theme.enable_color);
            parts.push(stats_styled);
        } else {
            // 普通模式：显示数量
            let stats_str = format!("{}/{} ({:.0}%)", current, total_val, percent);
            let stats_styled = theme.progress.apply(&stats_str, theme.enable_color);
            parts.push(stats_styled);
        }
    } else {
        // 未知总数：显示 spinner 和当前值
        // 使用 spinner 字符序列
        let spinner_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let elapsed = start_time.map(|s| s.elapsed()).unwrap_or_else(|| Duration::from_secs(0));
        let frame_idx = (elapsed.as_millis() / 100) as usize % spinner_frames.len();
        let spinner_char = spinner_frames[frame_idx];

        let spinner_styled = theme.progress.apply(spinner_char, theme.enable_color);
        parts.push(spinner_styled);

        // 显示当前值
        if current > 0 {
            let current_str = if matches!(mode, ProgressMode::Download) {
                format_bytes(current)
            } else {
                format!("{}", current)
            };
            let current_styled = theme.progress.apply(&current_str, theme.enable_color);
            parts.push(current_styled);
        }
    }

    // 时间信息
    if !time_info.is_empty() {
        let time_styled = theme.progress.apply(&time_info, theme.enable_color);
        parts.push(time_styled);
    }

    // 消息
    if !message.is_empty() {
        let msg_styled = theme.progress.apply(message, theme.enable_color);
        parts.push(msg_styled);
    }

    parts.join(" ")
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
