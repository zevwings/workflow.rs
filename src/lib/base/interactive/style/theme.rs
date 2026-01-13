//! 主题和样式定义

use crossterm::style::{Attribute, Color};

/// 样式定义
#[derive(Clone, Debug)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub attributes: Vec<Attribute>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            foreground: None,
            background: None,
            attributes: Vec::new(),
        }
    }

    pub fn fg(mut self, color: Color) -> Self {
        self.foreground = Some(color);
        self
    }

    #[allow(dead_code)]
    pub fn bg(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.attributes.push(Attribute::Bold);
        self
    }

    /// 应用样式到文本
    pub fn apply(&self, text: &str, enable_color: bool) -> String {
        use crossterm::style::Stylize;

        if !enable_color {
            return text.to_string();
        }
        let mut styled = text.stylize();

        if let Some(fg) = self.foreground {
            styled = styled.with(fg);
        }

        if let Some(bg) = self.background {
            styled = styled.on(bg);
        }

        for attr in &self.attributes {
            match attr {
                Attribute::Bold => styled = styled.bold(),
                Attribute::Italic => styled = styled.italic(),
                Attribute::Underlined => styled = styled.underlined(),
                _ => {}
            }
        }

        format!("{}", styled)
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

/// 主题配置
#[derive(Clone, Debug)]
pub struct Theme {
    pub info: Style,
    pub success: Style,
    pub warning: Style,
    pub error: Style,
    pub debug: Style,
    pub title: Style,
    pub answer: Style,
    pub hint: Style,
    pub prefix: Style,
    pub progress: Style,
    pub spinner: Style,
    pub enable_color: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            info: Style::new().fg(Color::Cyan),
            success: Style::new().fg(Color::Green),
            warning: Style::new().fg(Color::Yellow),
            error: Style::new().fg(Color::Red).bold(),
            debug: Style::new().fg(Color::DarkGrey),
            title: Style::new().fg(Color::White),
            answer: Style::new().fg(Color::Cyan),
            hint: Style::new().fg(Color::DarkGrey),
            prefix: Style::new().fg(Color::Green),
            progress: Style::new().fg(Color::White),
            spinner: Style::new().fg(Color::Cyan),
            enable_color: true,
        }
    }
}

/// 全局主题（使用 OnceLock 实现线程安全的单例）
use std::sync::OnceLock;

static THEME: OnceLock<Mutex<Theme>> = OnceLock::new();
use std::sync::Mutex;

/// 设置全局主题
#[allow(dead_code)]
pub fn set_theme(theme: Theme) {
    let theme_mutex = THEME.get_or_init(|| Mutex::new(Theme::default()));
    *theme_mutex.lock().unwrap() = theme;
}

/// 获取当前主题
pub fn get_theme() -> Theme {
    let theme_mutex = THEME.get_or_init(|| Mutex::new(Theme::default()));
    theme_mutex.lock().unwrap().clone()
}
