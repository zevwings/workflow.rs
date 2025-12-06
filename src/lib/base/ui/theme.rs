use ratatui::style::{Color, Modifier, Style};

/// 应用主题
pub struct Theme;

impl Theme {
    /// 成功消息样式（绿色）
    pub fn success() -> Style {
        Style::default().fg(Color::Green)
    }

    /// 错误消息样式（红色）
    pub fn error() -> Style {
        Style::default().fg(Color::Red)
    }

    /// 警告消息样式（黄色）
    pub fn warning() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// 信息消息样式（青色）
    pub fn info() -> Style {
        Style::default().fg(Color::Cyan)
    }

    /// 调试消息样式（灰色）
    pub fn debug() -> Style {
        Style::default().fg(Color::Gray)
    }

    /// 标题样式
    pub fn title() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    /// 高亮样式
    pub fn highlight() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }
}
