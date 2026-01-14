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
        let mut running = self.running.lock().expect("Spinner running lock poisoned");
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
                    let running_guard =
                        running.lock().expect("Spinner running lock poisoned in thread");
                    if !*running_guard {
                        break;
                    }
                }

                let frame = &frames[frame_idx % frames.len()];
                let msg = message.lock().expect("Spinner message lock poisoned in thread").clone();
                let theme = get_theme();
                let styled = format_spinner_text(frame, &msg, &theme);

                // 清除当前行并输出到 stderr（使用 crossterm 而不是直接使用 ANSI 转义序列）
                // 输出到 stderr 避免与 stdout 的日志输出冲突
                let mut stderr = io::stderr();
                let _ = stderr.queue(cursor::MoveToColumn(0));
                let _ = stderr.queue(Clear(ClearType::CurrentLine));
                let _ = write!(stderr, "{}", styled);
                let _ = stderr.flush();

                *current_frame.lock().expect("Spinner current_frame lock poisoned in thread") =
                    frame_idx;
                frame_idx += 1;

                thread::sleep(interval);
            }
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().expect("Spinner running lock poisoned");
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        // 清除当前行（使用 crossterm 而不是直接使用 ANSI 转义序列）
        // 输出到 stderr 避免与 stdout 的日志输出冲突
        let mut stderr = io::stderr();
        let _ = stderr.queue(cursor::MoveToColumn(0));
        let _ = stderr.queue(Clear(ClearType::CurrentLine));
        let _ = stderr.flush();

        // 恢复光标
        self.show_cursor();

        // 禁用原始模式，恢复输入回显
        self.disable_raw_mode();
    }

    pub fn update_message(&self, message: impl Into<String>) {
        *self.message.lock().expect("Spinner message lock poisoned") = message.into();
    }

    fn hide_cursor(&self) {
        let mut hidden = self.cursor_hidden.lock().expect("Spinner cursor_hidden lock poisoned");
        if !*hidden {
            let mut stderr = io::stderr();
            let _ = stderr.queue(Hide);
            let _ = stderr.flush();
            *hidden = true;
        }
    }

    fn show_cursor(&self) {
        let mut hidden = self.cursor_hidden.lock().expect("Spinner cursor_hidden lock poisoned");
        if *hidden {
            let mut stderr = io::stderr();
            let _ = stderr.queue(Show);
            let _ = stderr.flush();
            *hidden = false;
        }
    }

    fn enable_raw_mode(&self) {
        let mut enabled =
            self.raw_mode_enabled.lock().expect("Spinner raw_mode_enabled lock poisoned");
        if !*enabled {
            let _ = terminal::enable_raw_mode();
            *enabled = true;
        }
    }

    fn disable_raw_mode(&self) {
        let mut enabled =
            self.raw_mode_enabled.lock().expect("Spinner raw_mode_enabled lock poisoned");
        if *enabled {
            let _ = terminal::disable_raw_mode();
            *enabled = false;
        }
    }

    /// 完成 spinner 并显示完成消息
    ///
    /// 停止 spinner 动画并显示完成消息，然后清除。
    ///
    /// # 参数
    ///
    /// * `message` - 完成消息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::interactive::spinner;
    ///
    /// let spinner = spinner("Creating PR...").start();
    /// // 执行操作
    /// spinner.finish_with_message("PR created successfully!");
    /// ```
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

    /// 使用 spinner 执行一个操作（便捷方法）
    ///
    /// 自动创建并启动 spinner，执行操作，然后清理 spinner。
    /// 如果操作很快完成（< 100ms），会使用 `finish_with_message` 显示完成消息，
    /// 确保用户至少能看到一次输出。
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作（闭包）
    ///
    /// # 返回
    ///
    /// 返回操作的结果
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::interactive::spinner;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result: Result<i32, Box<dyn std::error::Error>> = spinner("Creating PR...").with(|| {
    ///     // 执行操作
    ///     Ok(42)
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let message_str = self.message.clone();
        let spinner = self.start();
        let start = std::time::Instant::now();
        let result = operation();
        let elapsed = start.elapsed();

        // 如果操作很快完成（< 100ms），使用 finish_with_message 显示消息
        // 确保用户至少能看到一次输出
        if elapsed < Duration::from_millis(100) {
            spinner.finish_with_message(&message_str);
        } else {
            spinner.stop();
        }

        result
    }

    /// 使用 spinner 执行一个会产生输出的操作
    ///
    /// 先显示 spinner 消息（250ms），然后完成 spinner，再执行操作。
    /// 这样可以确保用户能看到消息，同时让子进程的输出正常显示。
    ///
    /// 这个方法适用于执行会产生 stdout/stderr 输出的操作（如 `git push`），
    /// 可以避免子进程的输出与 spinner 动画混合。
    ///
    /// **注意**：操作完成后，建议使用 `info!` 或 `success!` 显示完成状态。
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作（闭包）
    ///
    /// # 返回
    ///
    /// 返回操作的结果
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::base::interactive::spinner;
    /// use workflow::success;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let result: Result<(), Box<dyn std::error::Error>> = spinner("Pushing to remote...").with_output(|| {
    ///     // 执行操作
    ///     Ok(())
    /// })?;
    /// result?;
    /// success!("Pushed to remote successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_output<F, T, E>(self, operation: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let spinner = self.start();
        // 让 spinner 显示足够长的时间（250ms），确保用户能看到消息
        std::thread::sleep(Duration::from_millis(250));
        // 完成 spinner（清除它），然后执行操作
        spinner.stop();
        // 执行操作，让子进程的输出正常显示
        operation()
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

/// 格式化加载指示器宏
///
/// 提供格式化字符串的便捷方式，避免手动使用 `format!`。
///
/// # Examples
///
/// ```rust,no_run
/// use workflow::spinner;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// spinner!("Getting ticket info for {}...", "PROJ-123")
///     .with(|| {
///         // 执行操作
///         Ok(())
///     })?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! spinner {
    ($($arg:tt)*) => {
        $crate::base::interactive::spinner(format!($($arg)*))
    };
}
