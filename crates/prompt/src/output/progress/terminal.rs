//! 终端控制（光标和原始模式）

use std::io::Write;

use crossterm::{
    cursor::{Hide, Show},
    terminal, QueueableCommand,
};

use crate::output::progress::bar::ProgressBar;

/// 隐藏光标
pub(super) fn hide_cursor(bar: &ProgressBar) {
    if let Ok(mut hidden) = bar.cursor_hidden.lock() {
        if !*hidden {
            let mut stderr = std::io::stderr();
            let _ = stderr.queue(Hide);
            let _ = stderr.flush();
            *hidden = true;
        }
    }
}

/// 显示光标
pub(super) fn show_cursor(bar: &ProgressBar) {
    if let Ok(mut hidden) = bar.cursor_hidden.lock() {
        if *hidden {
            let mut stderr = std::io::stderr();
            let _ = stderr.queue(Show);
            let _ = stderr.flush();
            *hidden = false;
        }
    }
}

/// 启用原始模式
pub(super) fn enable_raw_mode(bar: &ProgressBar) {
    if let Ok(mut enabled) = bar.raw_mode_enabled.lock() {
        if !*enabled {
            let _ = terminal::enable_raw_mode();
            *enabled = true;
        }
    }
}

/// 禁用原始模式
pub(super) fn disable_raw_mode(bar: &ProgressBar) {
    if let Ok(mut enabled) = bar.raw_mode_enabled.lock() {
        if *enabled {
            let _ = terminal::disable_raw_mode();
            *enabled = false;
        }
    }
}
