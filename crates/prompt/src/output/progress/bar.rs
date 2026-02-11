//! 进度条指示器
//!
//! 用于显示有明确进度的操作（如下载文件、处理多个项目等）。
//! 支持已知总数和未知总数两种模式，以及专门的下载模式。

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    output::{
        progress::{
            builder::ProgressBarBuilder,
            render::start_render_thread,
            terminal::{disable_raw_mode, enable_raw_mode, hide_cursor, show_cursor},
        },
        terminal_state,
    },
    style::theme::get_theme,
};

/// 进度条模式
#[derive(Clone, Copy, Debug)]
pub(crate) enum ProgressMode {
    /// 普通模式（显示数量）
    Normal,
    /// 下载模式（显示字节数、速度、ETA）
    Download,
}

/// 完成进度条并显示完成消息
pub(super) fn finish_with_message(bar: ProgressBar, message: impl Into<String>) {
    bar.stop();
    let theme = get_theme();
    let formatted = message.into();
    let styled = theme.progress.apply(&formatted, theme.enable_color);
    eprintln!("{}", styled);
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
/// use prompt::progress_bar;
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
    pub(crate) message: Arc<Mutex<String>>,
    pub(crate) total: Arc<Mutex<Option<u64>>>,
    pub(crate) current: Arc<Mutex<u64>>,
    pub(crate) mode: ProgressMode,
    pub(crate) interval: Duration,
    pub(crate) running: Arc<Mutex<bool>>,
    pub(crate) cursor_hidden: Arc<Mutex<bool>>,
    pub(crate) raw_mode_enabled: Arc<Mutex<bool>>,
    pub(crate) start_time: Arc<Mutex<Option<Instant>>>,
    pub(crate) bar_width: usize,
    pub(crate) progress_chars: String,
}

impl ProgressBar {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(message: impl Into<String>) -> ProgressBarBuilder {
        ProgressBarBuilder::new(message)
    }

    pub(crate) fn start_internal(&self) {
        let mut running = match self.running.lock() {
            Ok(guard) => guard,
            Err(_) => return, // 锁被毒化，无法启动进度条
        };
        if *running {
            return; // 已经在运行
        }
        *running = true;
        drop(running);

        // 记录开始时间
        if let Ok(mut guard) = self.start_time.lock() {
            *guard = Some(Instant::now());
        }

        // 启用原始模式，禁止输入回显
        enable_raw_mode(self);

        // 隐藏光标
        hide_cursor(self);

        // 注册到全局终端状态（渲染线程会自动重绘，不需要复杂的回调）
        terminal_state::register_renderer(|| {});

        start_render_thread(self);
    }

    pub fn stop(&self) {
        let mut running = match self.running.lock() {
            Ok(guard) => guard,
            Err(_) => return, // 锁被毒化，无法停止
        };
        if !*running {
            return;
        }
        *running = false;
        drop(running);

        // 注销全局终端状态
        terminal_state::unregister_renderer();

        // 清除当前行
        super::render::clear_line();

        // 恢复光标
        show_cursor(self);

        // 禁用原始模式，恢复输入回显
        disable_raw_mode(self);
    }

    /// 增加进度
    pub fn inc(&self, delta: u64) {
        if let Ok(mut current) = self.current.lock() {
            *current += delta;
        }
    }

    /// 增加进度（按字节数，用于下载模式）
    pub fn inc_bytes(&self, delta: u64) {
        self.inc(delta);
    }

    /// 设置当前进度
    pub fn set_position(&self, pos: u64) {
        if let Ok(mut guard) = self.current.lock() {
            *guard = pos;
        }
    }

    /// 更新显示的消息
    pub fn update_message(&self, message: impl Into<String>) {
        if let Ok(mut guard) = self.message.lock() {
            *guard = message.into();
        }
    }

    /// 设置总长度（用于动态更新）
    pub fn set_length(&self, len: u64) {
        if let Ok(mut guard) = self.total.lock() {
            *guard = Some(len);
        }
    }

    /// 完成并清除进度条（不需要 move，用于 Mutex 中）
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use prompt::progress_bar;
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
    /// use prompt::progress_bar;
    ///
    /// let pb = progress_bar("Downloading...").with_total(100).start();
    /// // 执行操作
    /// pb.finish_with_message("Download completed!");
    /// ```
    pub fn finish_with_message(self, message: impl Into<String>) {
        finish_with_message(self, message);
    }

    pub fn with_success(self, message: impl Into<String>) {
        self.stop();
        let theme = crate::style::theme::get_theme();
        let formatted = format!("✓ {}", message.into());
        let styled = theme.success.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn with_error(self, message: impl Into<String>) {
        self.stop();
        let theme = crate::style::theme::get_theme();
        let formatted = format!("✗ {}", message.into());
        let styled = theme.error.apply(&formatted, theme.enable_color);
        eprintln!("{}", styled);
    }

    pub fn with_info(self, message: impl Into<String>) {
        self.stop();
        let theme = crate::style::theme::get_theme();
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

    pub(crate) fn new_internal(
        message: String,
        total: Option<u64>,
        mode: ProgressMode,
        interval: Duration,
        bar_width: usize,
        progress_chars: String,
    ) -> Self {
        Self {
            message: Arc::new(Mutex::new(message)),
            total: Arc::new(Mutex::new(total)),
            current: Arc::new(Mutex::new(0)),
            mode,
            interval,
            running: Arc::new(Mutex::new(false)),
            cursor_hidden: Arc::new(Mutex::new(false)),
            raw_mode_enabled: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            bar_width,
            progress_chars,
        }
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bar() -> ProgressBar {
        ProgressBar::new_internal(
            "Test".to_string(),
            Some(100),
            ProgressMode::Normal,
            Duration::from_millis(100),
            40,
            "█░".to_string(),
        )
    }

    #[test]
    fn test_progress_bar_new_internal() {
        let bar = ProgressBar::new_internal(
            "Test message".to_string(),
            Some(100),
            ProgressMode::Normal,
            Duration::from_millis(100),
            40,
            "█▓▒░".to_string(),
        );

        assert_eq!(*bar.message.lock().unwrap(), "Test message");
        assert_eq!(*bar.total.lock().unwrap(), Some(100));
        assert_eq!(*bar.current.lock().unwrap(), 0);
        assert!(matches!(bar.mode, ProgressMode::Normal));
        assert!(!*bar.running.lock().unwrap());

        // 下载模式
        let bar = ProgressBar::new_internal(
            "Downloading...".to_string(),
            Some(1024 * 1024),
            ProgressMode::Download,
            Duration::from_millis(50),
            30,
            "=>-".to_string(),
        );
        assert!(matches!(bar.mode, ProgressMode::Download));

        // 未知总数
        let bar = ProgressBar::new_internal(
            "Processing...".to_string(),
            None,
            ProgressMode::Normal,
            Duration::from_millis(100),
            40,
            "█░".to_string(),
        );
        assert_eq!(*bar.total.lock().unwrap(), None);
    }

    #[test]
    fn test_progress_bar_updates() {
        let bar = create_test_bar();

        // inc
        bar.inc(10);
        assert_eq!(*bar.current.lock().unwrap(), 10);
        bar.inc(5);
        assert_eq!(*bar.current.lock().unwrap(), 15);

        // set_position
        bar.set_position(50);
        assert_eq!(*bar.current.lock().unwrap(), 50);

        // set_length
        bar.set_length(200);
        assert_eq!(*bar.total.lock().unwrap(), Some(200));

        // update_message
        bar.update_message("Updated");
        assert_eq!(*bar.message.lock().unwrap(), "Updated");
        bar.update_message("下载中 📥");
        assert_eq!(*bar.message.lock().unwrap(), "下载中 📥");
    }

    #[test]
    fn test_progress_bar_concurrent_access() {
        use std::thread;

        let bar = ProgressBar::new_internal(
            "Concurrent".to_string(),
            Some(1000),
            ProgressMode::Normal,
            Duration::from_millis(100),
            40,
            "█░".to_string(),
        );

        let current = Arc::clone(&bar.current);

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let current = Arc::clone(&current);
                thread::spawn(move || {
                    for _ in 0..100 {
                        if let Ok(mut c) = current.lock() {
                            *c += 1;
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(*bar.current.lock().unwrap(), 1000);
    }
}
