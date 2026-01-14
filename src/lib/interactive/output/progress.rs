//! 进度条模块
//!
//! 提供统一的进度条功能，用于显示有明确进度的操作（如下载、上传等）。
//!
//! # 示例
//!
//! ```rust,no_run
//! use workflow::interactive::{progress_bar, Progress};
//!
//! // 方式 1：已知总数
//! let pb = progress_bar("Downloading files...")
//!     .with_total(100)
//!     .start();
//! for i in 0..100 {
//!     pb.inc(1);
//!     std::thread::sleep(std::time::Duration::from_millis(10));
//! }
//! pb.finish_with_message("Download completed!");
//!
//! // 方式 2：下载模式（显示速度和 ETA）
//! let pb = Progress::new_download(1024 * 1024, "Downloading...");
//! pb.set_position(512 * 1024);
//! pb.finish_with_message("Download completed!");
//!
//! // 方式 3：未知总数（使用 spinner 模式）
//! let pb = Progress::new_unknown("Downloading...");
//! pb.inc(1);
//! pb.finish_with_message("Download completed!");
//! ```

use crate::interactive::style::get_theme;
use crossterm::{
    cursor::{self, Hide, Show},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// 进度条模式
#[derive(Clone, Copy, Debug)]
enum ProgressMode {
    /// 普通模式（显示数量）
    Normal,
    /// 下载模式（显示字节数、速度、ETA）
    Download,
}

/// 进度条指示器
///
/// 用于显示有明确进度的操作（如下载文件、处理多个项目等）。
/// 支持已知总数和未知总数两种模式，以及专门的下载模式。
///
/// # 线程安全
///
/// `ProgressBar` 内部使用 `Arc<Mutex<>>` 进行线程安全的状态管理，
/// 可以在多线程环境中安全使用。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::interactive::progress_bar;
///
/// let pb = progress_bar("Processing...")
///     .with_total(100)
///     .start();
///
/// // 在另一个线程中更新进度
/// std::thread::spawn(move || {
///     for i in 0..100 {
///         pb.inc(1);
///         std::thread::sleep(std::time::Duration::from_millis(10));
///     }
///     pb.finish_with_message("Completed!");
/// });
/// ```
pub struct ProgressBar {
    message: Arc<Mutex<String>>,
    total: Arc<Mutex<Option<u64>>>,
    current: Arc<Mutex<u64>>,
    mode: ProgressMode,
    interval: Duration,
    running: Arc<Mutex<bool>>,
    cursor_hidden: Arc<Mutex<bool>>,
    raw_mode_enabled: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<Instant>>>,
    bar_width: usize,
    progress_chars: String,
}

impl ProgressBar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(message: impl Into<String>) -> ProgressBarBuilder {
        ProgressBarBuilder::new(message)
    }

    fn start_internal(&self) {
        let mut running = self.running.lock().expect("ProgressBar running lock poisoned");
        if *running {
            return; // 已经在运行
        }
        *running = true;
        drop(running);

        // 记录开始时间
        *self.start_time.lock().expect("ProgressBar start_time lock poisoned") =
            Some(Instant::now());

        // 启用原始模式，禁止输入回显
        self.enable_raw_mode();

        // 隐藏光标
        self.hide_cursor();

        let running = Arc::clone(&self.running);
        let message = Arc::clone(&self.message);
        let total = Arc::clone(&self.total);
        let current = Arc::clone(&self.current);
        let mode = self.mode;
        let interval = self.interval;
        let bar_width = self.bar_width;
        let progress_chars = self.progress_chars.clone();
        let start_time = Arc::clone(&self.start_time);

        thread::spawn(move || {
            loop {
                {
                    let running_guard =
                        running.lock().expect("ProgressBar running lock poisoned in thread");
                    if !*running_guard {
                        break;
                    }
                }

                let msg =
                    message.lock().expect("ProgressBar message lock poisoned in thread").clone();
                let total_val = *total.lock().expect("ProgressBar total lock poisoned in thread");
                let current_val =
                    *current.lock().expect("ProgressBar current lock poisoned in thread");
                let start =
                    *start_time.lock().expect("ProgressBar start_time lock poisoned in thread");

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
                let _ = stderr.queue(Clear(ClearType::CurrentLine));
                let _ = write!(stderr, "{}", styled);
                let _ = stderr.flush();

                thread::sleep(interval);
            }
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().expect("ProgressBar running lock poisoned");
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

    /// 增加进度
    pub fn inc(&self, delta: u64) {
        let mut current = self.current.lock().expect("ProgressBar current lock poisoned");
        *current += delta;
    }

    /// 增加进度（按字节数，用于下载模式）
    pub fn inc_bytes(&self, delta: u64) {
        self.inc(delta);
    }

    /// 设置当前进度
    pub fn set_position(&self, pos: u64) {
        *self.current.lock().expect("ProgressBar current lock poisoned") = pos;
    }

    /// 更新显示的消息
    pub fn update_message(&self, message: impl Into<String>) {
        *self.message.lock().expect("ProgressBar message lock poisoned") = message.into();
    }

    /// 设置总长度（用于动态更新）
    pub fn set_length(&self, len: u64) {
        *self.total.lock().expect("ProgressBar total lock poisoned") = Some(len);
    }

    fn hide_cursor(&self) {
        let mut hidden =
            self.cursor_hidden.lock().expect("ProgressBar cursor_hidden lock poisoned");
        if !*hidden {
            let mut stderr = io::stderr();
            let _ = stderr.queue(Hide);
            let _ = stderr.flush();
            *hidden = true;
        }
    }

    fn show_cursor(&self) {
        let mut hidden =
            self.cursor_hidden.lock().expect("ProgressBar cursor_hidden lock poisoned");
        if *hidden {
            let mut stderr = io::stderr();
            let _ = stderr.queue(Show);
            let _ = stderr.flush();
            *hidden = false;
        }
    }

    fn enable_raw_mode(&self) {
        let mut enabled = self
            .raw_mode_enabled
            .lock()
            .expect("ProgressBar raw_mode_enabled lock poisoned");
        if !*enabled {
            let _ = terminal::enable_raw_mode();
            *enabled = true;
        }
    }

    fn disable_raw_mode(&self) {
        let mut enabled = self
            .raw_mode_enabled
            .lock()
            .expect("ProgressBar raw_mode_enabled lock poisoned");
        if *enabled {
            let _ = terminal::disable_raw_mode();
            *enabled = false;
        }
    }

    /// 完成并清除进度条（不需要 move，用于 Mutex 中）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::interactive::progress_bar;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let progress = Arc::new(Mutex::new(progress_bar("Processing...")
    ///     .with_total(100)
    ///     .start()));
    /// {
    ///     let pb = progress.lock().unwrap();
    ///     pb.finish_ref();
    /// }
    /// ```
    pub fn finish_ref(&self) {
        self.stop();
    }

    /// 完成进度条并显示完成消息
    ///
    /// 停止进度条动画并显示完成消息，然后清除。
    ///
    /// # 参数
    ///
    /// * `message` - 完成消息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::interactive::progress_bar;
    ///
    /// let pb = progress_bar("Downloading...").with_total(100).start();
    /// // 执行操作
    /// pb.finish_with_message("Download completed!");
    /// ```
    pub fn finish_with_message(self, message: impl Into<String>) {
        self.stop();
        let theme = get_theme();
        let formatted = message.into();
        let styled = theme.progress.apply(&formatted, theme.enable_color);
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
        let styled = theme.progress.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    /// 使用进度条执行一个操作
    ///
    /// 自动创建并启动进度条，执行操作，然后清理进度条。
    ///
    /// # 参数
    ///
    /// * `work` - 要执行的操作（闭包）
    ///
    /// # 返回
    ///
    /// 返回操作的结果
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

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 进度条构建器
///
/// 使用构建器模式创建和配置进度条。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::interactive::progress_bar;
/// use std::time::Duration;
///
/// let pb = progress_bar("Downloading...")
///     .with_total(1024 * 1024)
///     .with_interval(Duration::from_millis(50))
///     .with_bar_width(40)
///     .with_progress_chars("█░")
///     .start();
/// ```
pub struct ProgressBarBuilder {
    message: String,
    total: Option<u64>,
    interval: Option<Duration>,
    bar_width: Option<usize>,
    progress_chars: Option<String>,
    mode: ProgressMode,
}

impl ProgressBarBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            total: None,
            interval: None,
            bar_width: None,
            progress_chars: None,
            mode: ProgressMode::Normal,
        }
    }

    /// 设置总长度（已知总数）
    pub fn with_total(mut self, total: u64) -> Self {
        self.total = Some(total);
        self
    }

    /// 设置刷新间隔
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = Some(interval);
        self
    }

    /// 设置进度条宽度
    pub fn with_bar_width(mut self, width: usize) -> Self {
        self.bar_width = Some(width);
        self
    }

    /// 设置进度条字符（如 "█░" 或 "#>-"）
    pub fn with_progress_chars(mut self, chars: impl Into<String>) -> Self {
        self.progress_chars = Some(chars.into());
        self
    }

    /// 设置为下载模式（显示字节数、速度、ETA）
    pub fn with_download_mode(mut self) -> Self {
        self.mode = ProgressMode::Download;
        self
    }

    /// 启动进度条
    pub fn start(self) -> ProgressBar {
        let progress_bar = ProgressBar {
            message: Arc::new(Mutex::new(self.message)),
            total: Arc::new(Mutex::new(self.total)),
            current: Arc::new(Mutex::new(0)),
            mode: self.mode,
            interval: self.interval.unwrap_or(Duration::from_millis(100)),
            running: Arc::new(Mutex::new(false)),
            cursor_hidden: Arc::new(Mutex::new(false)),
            raw_mode_enabled: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            bar_width: self.bar_width.unwrap_or(30),
            progress_chars: self.progress_chars.unwrap_or_else(|| "█░".to_string()),
        };

        progress_bar.start_internal();
        progress_bar
    }
}

/// Progress 结构体（兼容原 indicator::Progress API）
///
/// 提供与原有 `indicator::Progress` 相同的 API，已完全替代原实现。
pub struct Progress {
    inner: ProgressBar,
}

impl Progress {
    /// 创建一个新的进度条（已知总数）
    ///
    /// # 参数
    ///
    /// * `total` - 总数量（文件数、字节数等）
    /// * `message` - 要显示的消息文本
    pub fn new(total: u64, message: impl AsRef<str>) -> Self {
        Self {
            inner: progress_bar(message.as_ref()).with_total(total).start(),
        }
    }

    /// 创建一个新的进度条（用于下载，显示字节数）
    ///
    /// # 参数
    ///
    /// * `total_bytes` - 总字节数
    /// * `message` - 要显示的消息文本
    pub fn new_download(total_bytes: u64, message: impl AsRef<str>) -> Self {
        Self {
            inner: progress_bar(message.as_ref())
                .with_total(total_bytes)
                .with_download_mode()
                .start(),
        }
    }

    /// 创建一个新的进度条（未知总数，使用 spinner 模式）
    ///
    /// # 参数
    ///
    /// * `message` - 要显示的消息文本
    pub fn new_unknown(message: impl AsRef<str>) -> Self {
        Self {
            inner: progress_bar(message.as_ref()).start(),
        }
    }

    /// 增加进度（按单位数）
    pub fn inc(&self, delta: u64) {
        self.inner.inc(delta);
    }

    /// 增加进度（按字节数）
    pub fn inc_bytes(&self, delta: u64) {
        self.inner.inc_bytes(delta);
    }

    /// 设置当前位置
    pub fn set_position(&self, pos: u64) {
        self.inner.set_position(pos);
    }

    /// 更新显示的消息
    pub fn update_message(&self, message: impl AsRef<str>) {
        self.inner.update_message(message.as_ref());
    }

    /// 完成并清除进度条
    pub fn finish(self) {
        self.inner.stop();
    }

    /// 完成并清除进度条（不需要 move，用于 Mutex 中）
    pub fn finish_ref(&self) {
        self.inner.finish_ref();
    }

    /// 完成进度条并显示完成消息
    pub fn finish_with_message(self, message: impl AsRef<str>) {
        self.inner.finish_with_message(message.as_ref());
    }
}

/// 格式化进度条文本
fn format_progress_text(
    message: &str,
    total: Option<u64>,
    current: u64,
    start_time: Option<Instant>,
    mode: ProgressMode,
    bar_width: usize,
    progress_chars: &str,
    theme: &crate::interactive::style::Theme,
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

/// 便捷函数
pub fn progress_bar(message: impl Into<String>) -> ProgressBarBuilder {
    ProgressBarBuilder::new(message)
}

/// 格式化进度条宏
///
/// 提供格式化字符串的便捷方式，避免手动使用 `format!`。
/// 使用 `progress!` 作为宏名，与 `spinner!` 保持一致。
///
/// # Examples
///
/// ```rust,no_run
/// use workflow::progress;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let pb = progress!("Downloading {}...", "file.zip")
///     .with_total(100)
///     .start();
/// // 使用进度条
/// pb.finish_with_message("Download completed!");
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! progress {
    ($($arg:tt)*) => {
        $crate::interactive::progress_bar(format!($($arg)*))
    };
}
