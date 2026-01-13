//! 加载指示器模块

use crate::base::interactive::style::get_theme;
use crossterm::{
    cursor::{self, Hide, Show},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

    fn start_internal(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return; // 已经在运行
        }
        *running = true;
        drop(running);

        // 启用原始模式，禁止输入回显
        self.enable_raw_mode();

        // 隐藏光标
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
                    let running_guard = running.lock().unwrap();
                    if !*running_guard {
                        break;
                    }
                }

                let frame = &frames[frame_idx % frames.len()];
                let msg = message.lock().unwrap().clone();
                let theme = get_theme();
                let styled = format_spinner_text(frame, &msg, &theme);

                // 清除当前行并输出（使用 crossterm 而不是直接使用 ANSI 转义序列）
                let mut stdout = io::stdout();
                let _ = stdout.queue(cursor::MoveToColumn(0));
                let _ = stdout.queue(Clear(ClearType::CurrentLine));
                let _ = write!(stdout, "{}", styled);
                let _ = stdout.flush();

                *current_frame.lock().unwrap() = frame_idx;
                frame_idx += 1;

                thread::sleep(interval);
            }
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        // 清除当前行（使用 crossterm 而不是直接使用 ANSI 转义序列）
        let mut stdout = io::stdout();
        let _ = stdout.queue(cursor::MoveToColumn(0));
        let _ = stdout.queue(Clear(ClearType::CurrentLine));
        let _ = stdout.flush();

        // 恢复光标
        self.show_cursor();

        // 禁用原始模式，恢复输入回显
        self.disable_raw_mode();
    }

    pub fn update_message(&self, message: impl Into<String>) {
        *self.message.lock().unwrap() = message.into();
    }

    fn hide_cursor(&self) {
        let mut hidden = self.cursor_hidden.lock().unwrap();
        if !*hidden {
            let mut stdout = io::stdout();
            let _ = stdout.queue(Hide);
            let _ = stdout.flush();
            *hidden = true;
        }
    }

    fn show_cursor(&self) {
        let mut hidden = self.cursor_hidden.lock().unwrap();
        if *hidden {
            let mut stdout = io::stdout();
            let _ = stdout.queue(Show);
            let _ = stdout.flush();
            *hidden = false;
        }
    }

    fn enable_raw_mode(&self) {
        let mut enabled = self.raw_mode_enabled.lock().unwrap();
        if !*enabled {
            let _ = terminal::enable_raw_mode();
            *enabled = true;
        }
    }

    fn disable_raw_mode(&self) {
        let mut enabled = self.raw_mode_enabled.lock().unwrap();
        if *enabled {
            let _ = terminal::disable_raw_mode();
            *enabled = false;
        }
    }

    pub fn with_success(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("✓ {}", message.into());
        let styled = theme.success.apply(&formatted, theme.enable_color);
        println!("{}", styled);
    }

    pub fn with_error(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("✗ {}", message.into());
        let styled = theme.error.apply(&formatted, theme.enable_color);
        println!("{}", styled);
    }

    pub fn with_info(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = format!("ℹ {}", message.into());
        let styled = theme.info.apply(&formatted, theme.enable_color);
        println!("{}", styled);
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
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

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

    pub fn start(self) -> Spinner {
        let default_frames = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            .into_iter()
            .map(String::from)
            .collect();

        let spinner = Spinner {
            message: Arc::new(Mutex::new(self.message)),
            frames: self.frames.unwrap_or(default_frames),
            interval: self.interval.unwrap_or(Duration::from_millis(100)),
            running: Arc::new(Mutex::new(false)),
            current_frame: Arc::new(Mutex::new(0)),
            cursor_hidden: Arc::new(Mutex::new(false)),
            raw_mode_enabled: Arc::new(Mutex::new(false)),
        };

        spinner.start_internal();
        spinner
    }
}

/// 格式化 spinner 文本
fn format_spinner_text(
    frame: &str,
    message: &str,
    theme: &crate::base::interactive::style::Theme,
) -> String {
    if message.is_empty() {
        theme.info.apply(frame, theme.enable_color)
    } else {
        let spinner_part = theme.info.apply(frame, theme.enable_color);
        let message_part = theme.info.apply(message, theme.enable_color);
        format!("{} {}", spinner_part, message_part)
    }
}

/// 便捷函数
pub fn spinner(message: impl Into<String>) -> SpinnerBuilder {
    SpinnerBuilder::new(message)
}
