//! Spinner 加载指示器

use crate::core::prompt::style::theme::{get_theme, Theme};
use crossterm::{
    cursor::{self, Hide, Show},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Spinner 构建器
pub struct SpinnerBuilder {
    message: String,
    frames: Option<Vec<String>>,
    interval: Option<Duration>,
}

impl SpinnerBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: None,
            interval: None,
        }
    }

    pub fn with_frames(mut self, frames: Vec<impl Into<String>>) -> Self {
        self.frames = Some(frames.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// 使用 spinner 执行一个操作（便捷方法）
    pub fn with<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let message_str = self.message.clone();
        let spinner = self.start();
        let start = std::time::Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        if elapsed < Duration::from_millis(100) {
            spinner.finish_with_message(message_str);
        } else {
            spinner.stop();
        }

        result
    }

    /// 使用 spinner 执行一个会产生输出的操作
    pub fn with_output<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = self.start();
        std::thread::sleep(Duration::from_millis(250));
        spinner.stop();
        operation()
    }

    pub fn start(self) -> Spinner {
        let default_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            .into_iter()
            .map(String::from)
            .collect();

        let spinner = Spinner::new_internal(
            self.message,
            self.frames.unwrap_or(default_frames),
            self.interval.unwrap_or(Duration::from_millis(100)),
        );

        spinner.start_internal();
        spinner
    }
}

/// Spinner 加载指示器
pub struct Spinner {
    message: Arc<Mutex<String>>,
    frames: Vec<String>,
    interval: Duration,
    running: Arc<Mutex<bool>>,
    current_frame: Arc<Mutex<usize>>,
    cursor_hidden: Arc<Mutex<bool>>,
    raw_mode_enabled: Arc<Mutex<bool>>,
}

impl Spinner {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(message: impl Into<String>) -> SpinnerBuilder {
        SpinnerBuilder::new(message)
    }

    pub(crate) fn start_internal(&self) {
        let mut running = match self.running.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if *running {
            return;
        }
        *running = true;
        drop(running);

        self.enable_raw_mode();
        self.hide_cursor();

        let running = Arc::clone(&self.running);
        let frames = self.frames.clone();
        let message = Arc::clone(&self.message);
        let interval = self.interval;
        let current_frame = Arc::clone(&self.current_frame);

        thread::spawn(move || {
            let mut frame_idx = 0;
            loop {
                {
                    let running_guard = match running.lock() {
                        Ok(guard) => guard,
                        Err(_) => break,
                    };
                    if !*running_guard {
                        break;
                    }
                }

                let frame = &frames[frame_idx % frames.len()];
                let msg = match message.lock() {
                    Ok(guard) => guard.clone(),
                    Err(_) => break,
                };
                let theme = get_theme();
                let styled = format_spinner_text(frame, &msg, &theme);

                let mut stderr = io::stderr();
                let _ = stderr.queue(cursor::MoveToColumn(0));
                let _ = stderr.queue(Clear(ClearType::CurrentLine));
                let _ = write!(stderr, "{}", styled);
                let _ = stderr.flush();

                if let Ok(mut guard) = current_frame.lock() {
                    *guard = frame_idx;
                }
                frame_idx += 1;

                thread::sleep(interval);
            }
        });
    }

    pub fn stop(&self) {
        let mut running = match self.running.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        let mut stderr = io::stderr();
        let _ = stderr.queue(cursor::MoveToColumn(0));
        let _ = stderr.queue(Clear(ClearType::CurrentLine));
        let _ = stderr.flush();

        self.show_cursor();
        self.disable_raw_mode();
    }

    pub fn update_message(&self, message: impl Into<String>) {
        if let Ok(mut guard) = self.message.lock() {
            *guard = message.into();
        }
    }

    fn hide_cursor(&self) {
        if let Ok(mut hidden) = self.cursor_hidden.lock() {
            if !*hidden {
                let mut stderr = io::stderr();
                let _ = stderr.queue(Hide);
                let _ = stderr.flush();
                *hidden = true;
            }
        }
    }

    fn show_cursor(&self) {
        if let Ok(mut hidden) = self.cursor_hidden.lock() {
            if *hidden {
                let mut stderr = io::stderr();
                let _ = stderr.queue(Show);
                let _ = stderr.flush();
                *hidden = false;
            }
        }
    }

    fn enable_raw_mode(&self) {
        if let Ok(mut enabled) = self.raw_mode_enabled.lock() {
            if !*enabled {
                let _ = terminal::enable_raw_mode();
                *enabled = true;
            }
        }
    }

    fn disable_raw_mode(&self) {
        if let Ok(mut enabled) = self.raw_mode_enabled.lock() {
            if *enabled {
                let _ = terminal::disable_raw_mode();
                *enabled = false;
            }
        }
    }

    pub fn finish_with_message(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = message.into();
        let styled = theme.spinner.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn with_success(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("✓ {}", message.into());
        let styled = theme.success.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn with_error(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("✗ {}", message.into());
        let styled = theme.error.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn with_info(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("ℹ {}", message.into());
        let styled = theme.spinner.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn do_work<F, E>(self, work: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        self.start_internal();
        let result = work();
        self.stop();
        result
    }

    pub(crate) fn new_internal(message: String, frames: Vec<String>, interval: Duration) -> Self {
        Self {
            message: Arc::new(Mutex::new(message)),
            frames,
            interval,
            running: Arc::new(Mutex::new(false)),
            current_frame: Arc::new(Mutex::new(0)),
            cursor_hidden: Arc::new(Mutex::new(false)),
            raw_mode_enabled: Arc::new(Mutex::new(false)),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 格式化 spinner 文本
fn format_spinner_text(frame: &str, message: &str, theme: &Theme) -> String {
    if message.is_empty() {
        theme.spinner.apply(frame, theme.enable_color)
    } else {
        let spinner_part = theme.spinner.apply(frame, theme.enable_color);
        let message_part = theme.spinner.apply(message, theme.enable_color);
        format!("{} {}", spinner_part, message_part)
    }
}

/// 便捷函数
pub fn spinner(message: impl Into<String>) -> SpinnerBuilder {
    SpinnerBuilder::new(message)
}

// ============================================================================
// 宏定义
// ============================================================================

/// 格式化加载指示器宏
///
/// 提供格式化字符串的便捷方式，避免手动使用 `format!`。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::spinner;
///
/// # fn main() {
/// let spinner = spinner!("正在处理 {}...", "文件");
/// # }
/// ```
#[macro_export]
macro_rules! spinner {
    ($($arg:tt)*) => {
        $crate::prompt::spinner(format!($($arg)*))
    };
}
