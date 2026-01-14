//! 进度条渲染逻辑

use super::bar::ProgressBar;
use super::format::format_progress_text;
use crate::core::prompt::style::theme::get_theme;
use crossterm::{cursor, terminal::ClearType, QueueableCommand};
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;

/// 启动渲染线程
pub(super) fn start_render_thread(bar: &ProgressBar) {
    let running = Arc::clone(&bar.running);
    let message = Arc::clone(&bar.message);
    let total = Arc::clone(&bar.total);
    let current = Arc::clone(&bar.current);
    let mode = bar.mode;
    let interval = bar.interval;
    let bar_width = bar.bar_width;
    let progress_chars = bar.progress_chars.clone();
    let start_time = Arc::clone(&bar.start_time);

    thread::spawn(move || {
        loop {
            {
                let running_guard = match running.lock() {
                    Ok(guard) => guard,
                    Err(_) => break, // 锁被毒化，退出线程
                };
                if !*running_guard {
                    break;
                }
            }

            // 获取所有需要的数据（需要克隆 message，因为需要在 guard 释放后使用）
            let msg = match message.lock() {
                Ok(guard) => guard.clone(),
                Err(_) => break, // 锁被毒化，退出线程
            };
            let total_val = match total.lock() {
                Ok(guard) => *guard,
                Err(_) => break, // 锁被毒化，退出线程
            };
            let current_val = match current.lock() {
                Ok(guard) => *guard,
                Err(_) => break, // 锁被毒化，退出线程
            };
            let start = match start_time.lock() {
                Ok(guard) => *guard,
                Err(_) => break, // 锁被毒化，退出线程
            };

            let theme = get_theme();
            let styled = format_progress_text(
                &msg,
                total_val,
                current_val,
                start,
                mode,
                bar_width,
                &progress_chars,
                &theme,
            );

            // 清除当前行并输出到 stderr（使用 crossterm 而不是直接使用 ANSI 转义序列）
            // 输出到 stderr 避免与 stdout 的日志输出冲突
            let mut stderr = io::stderr();
            let _ = stderr.queue(cursor::MoveToColumn(0));
            let _ = stderr.queue(crossterm::terminal::Clear(ClearType::CurrentLine));
            let _ = write!(stderr, "{}", styled);
            let _ = stderr.flush();

            thread::sleep(interval);
        }
    });
}

/// 清除当前行
pub(super) fn clear_line() {
    let mut stderr = io::stderr();
    let _ = stderr.queue(cursor::MoveToColumn(0));
    let _ = stderr.queue(crossterm::terminal::Clear(ClearType::CurrentLine));
    let _ = stderr.flush();
}
