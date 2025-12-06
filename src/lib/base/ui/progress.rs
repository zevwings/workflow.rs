//! 进度条组件
//!
//! 提供基于 ratatui 的进度条显示功能。

use anyhow::Result;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;
use std::time::Instant;

/// 进度条
pub struct ProgressBar {
    current: u64,
    total: u64,
    message: String,
    start_time: Instant,
    terminal: Option<Terminal<CrosstermBackend<io::Stdout>>>,
    is_tty: bool,
}

impl ProgressBar {
    /// 创建新的进度条
    pub fn new(total: u64, title: impl Into<String>) -> Result<Self> {
        let is_tty = atty::is(atty::Stream::Stdout);
        let mut terminal = if is_tty {
            let term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
            crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;
            crossterm::terminal::enable_raw_mode()?;
            Some(term)
        } else {
            None
        };

        if let Some(ref mut term) = terminal {
            term.clear()?;
        }

        Ok(Self {
            current: 0,
            total,
            message: title.into(),
            start_time: Instant::now(),
            terminal,
            is_tty,
        })
    }

    /// 设置消息
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    /// 更新进度
    pub fn update(&mut self, current: u64) -> Result<()> {
        self.current = current.min(self.total);
        if self.is_tty {
            self.render()?;
        } else {
            // 非 TTY 环境，简单输出进度信息
            let progress = (self.current as f64 / self.total as f64).min(1.0);
            eprintln!(
                "\r{}: {:.1}% ({}/{})",
                self.message,
                progress * 100.0,
                format_bytes(self.current),
                format_bytes(self.total)
            );
        }
        Ok(())
    }

    /// 完成进度条
    pub fn finish(&mut self, message: impl Into<String>) -> Result<()> {
        self.message = message.into();
        self.current = self.total;
        if self.is_tty {
            self.render()?;
            std::thread::sleep(std::time::Duration::from_millis(500));
            if let Some(ref mut term) = self.terminal {
                term.clear()?;
            }
            crossterm::terminal::disable_raw_mode()?;
            crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        } else {
            eprintln!(
                "\r{}: 100% ({}/{})",
                self.message,
                format_bytes(self.total),
                format_bytes(self.total)
            );
        }
        Ok(())
    }

    /// 渲染进度条
    fn render(&mut self) -> Result<()> {
        if let Some(ref mut terminal) = self.terminal {
            terminal.draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(5),
                        Constraint::Min(0),
                    ])
                    .split(area);

                // 标题
                let title = Paragraph::new(self.message.as_str())
                    .style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(title, chunks[0]);

                // 进度条
                let progress = (self.current as f64 / self.total as f64).min(1.0);
                let elapsed = self.start_time.elapsed();
                let bytes_per_sec = if elapsed.as_secs() > 0 {
                    self.current / elapsed.as_secs()
                } else {
                    0
                };
                let eta = if bytes_per_sec > 0 && self.current < self.total {
                    (self.total - self.current) / bytes_per_sec
                } else {
                    0
                };

                let progress_text = format!(
                    "{} / {} ({:.1}%) | {}/s | ETA: {}s",
                    format_bytes(self.current),
                    format_bytes(self.total),
                    progress * 100.0,
                    format_bytes(bytes_per_sec),
                    eta
                );

                let progress_widget = Gauge::default()
                    .block(Block::default().borders(Borders::ALL).title("Progress"))
                    .gauge_style(Style::default().fg(Color::Cyan))
                    .percent((progress * 100.0) as u16)
                    .label(progress_text);
                f.render_widget(progress_widget, chunks[1]);
            })?;
        }
        Ok(())
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.is_tty {
            if let Some(ref mut terminal) = self.terminal {
                let _ = terminal.clear();
            }
            let _ = crossterm::terminal::disable_raw_mode();
            let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        }
    }
}

/// 格式化字节数为可读格式
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}
