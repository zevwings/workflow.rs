//! Spinner 加载指示器核心实现
//!
//! 提供 Spinner 结构体和所有相关方法。

use crate::output::spinner::builder::SpinnerBuilder;
use crate::output::spinner::format::format_spinner_text;
use crate::style::theme::get_theme;
use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
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

                // Check for Ctrl+C event in raw mode
                if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                    if let Ok(Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    })) = event::read()
                    {
                        // Ctrl+C detected - stop spinner and exit process
                        if let Ok(mut running_guard) = running.lock() {
                            *running_guard = false;
                        }

                        // Clean up terminal
                        let mut stderr = io::stderr();
                        let _ = stderr.queue(cursor::MoveToColumn(0));
                        let _ = stderr.queue(Clear(ClearType::CurrentLine));
                        let _ = stderr.queue(Show);
                        let _ = stderr.flush();
                        let _ = terminal::disable_raw_mode();

                        // Exit the process with SIGINT status
                        std::process::exit(130); // 128 + SIGINT(2) = 130
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
