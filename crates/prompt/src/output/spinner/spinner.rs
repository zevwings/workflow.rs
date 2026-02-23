//! Spinner 加载指示器核心实现
//!
//! 提供 Spinner 结构体和所有相关方法。

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

use crate::{
    output::{
        spinner::{builder::SpinnerBuilder, format::format_spinner_text},
        terminal_state,
    },
    style::theme::get_theme,
};

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

        // 注册到全局终端状态（渲染线程会自动重绘，不需要复杂的回调）
        terminal_state::register_renderer(|| {});

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

                // 在原始模式下检查 Ctrl+C 事件
                if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                    if let Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    })) = event::read()
                    {
                        // 检测到 Ctrl+C - 停止 spinner 并退出进程
                        if let Ok(mut running_guard) = running.lock() {
                            *running_guard = false;
                        }

                        // 注销全局终端状态
                        terminal_state::unregister_renderer();

                        // 清理终端
                        let mut stderr = io::stderr();
                        let _ = stderr.queue(cursor::MoveToColumn(0));
                        let _ = stderr.queue(Clear(ClearType::CurrentLine));
                        let _ = stderr.queue(Show);
                        let _ = stderr.flush();
                        let _ = terminal::disable_raw_mode();

                        // 以 SIGINT 状态退出进程
                        std::process::exit(130); // 128 + SIGINT(2) = 130
                    }
                }

                // 如果处于暂停状态，跳过渲染
                if terminal_state::is_suspended() {
                    thread::sleep(interval);
                    continue;
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

        // 注销全局终端状态
        terminal_state::unregister_renderer();

        // 等待渲染线程退出，避免竞态：stop 清行后渲染线程再写一帧导致残留
        thread::sleep(self.interval.saturating_add(Duration::from_millis(50)));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn default_frames() -> Vec<String> {
        vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    #[test]
    fn test_spinner_new_internal() {
        let spinner = Spinner::new_internal(
            "Loading...".to_string(),
            default_frames(),
            Duration::from_millis(80),
        );

        assert_eq!(*spinner.message.lock().unwrap(), "Loading...");
        assert_eq!(spinner.frames.len(), 10);
        assert_eq!(spinner.interval, Duration::from_millis(80));
        assert!(!*spinner.running.lock().unwrap());

        // 自定义帧
        let frames = vec!["-".to_string(), "|".to_string()];
        let spinner = Spinner::new_internal(
            "Custom".to_string(),
            frames.clone(),
            Duration::from_millis(100),
        );
        assert_eq!(spinner.frames, frames);

        // Unicode 帧
        let frames: Vec<String> = vec!["🌑", "🌕"].into_iter().map(String::from).collect();
        let spinner = Spinner::new_internal("Moon".to_string(), frames, Duration::from_millis(100));
        assert_eq!(spinner.frames[0], "🌑");
    }

    #[test]
    fn test_spinner_update_message() {
        let spinner = Spinner::new_internal(
            "Initial".to_string(),
            default_frames(),
            Duration::from_millis(80),
        );

        spinner.update_message("Updated");
        assert_eq!(*spinner.message.lock().unwrap(), "Updated");

        spinner.update_message("Processing 🔄");
        assert_eq!(*spinner.message.lock().unwrap(), "Processing 🔄");

        spinner.update_message("");
        assert_eq!(*spinner.message.lock().unwrap(), "");
    }

    #[test]
    fn test_spinner_concurrent_message_update() {
        let spinner = Spinner::new_internal(
            "Initial".to_string(),
            default_frames(),
            Duration::from_millis(80),
        );

        let message = Arc::clone(&spinner.message);

        let handles: Vec<_> = (0..5)
            .map(|i| {
                let message = Arc::clone(&message);
                thread::spawn(move || {
                    for j in 0..10 {
                        if let Ok(mut m) = message.lock() {
                            *m = format!("Thread {} iteration {}", i, j);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let final_message = spinner.message.lock().unwrap();
        assert!(final_message.starts_with("Thread "));
    }

    #[test]
    fn test_spinner_stop_safe() {
        let spinner = Spinner::new_internal(
            "Test".to_string(),
            default_frames(),
            Duration::from_millis(80),
        );

        // 未运行时停止和多次停止都应该安全
        spinner.stop();
        spinner.stop();
        assert!(!*spinner.running.lock().unwrap());
    }
}
