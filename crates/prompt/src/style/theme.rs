//! 主题和样式定义

use std::sync::{Mutex, OnceLock};

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
static THEME: OnceLock<Mutex<Theme>> = OnceLock::new();

/// 设置全局主题
pub fn set_theme(theme: Theme) {
    let theme_mutex = THEME.get_or_init(|| Mutex::new(Theme::default()));
    // 对于主题设置，如果锁被毒化，使用默认主题是合理的选择
    if let Ok(mut guard) = theme_mutex.lock() {
        *guard = theme;
    }
}

/// 获取当前主题
pub fn get_theme() -> Theme {
    let theme_mutex = THEME.get_or_init(|| Mutex::new(Theme::default()));
    // 如果锁被毒化，返回默认主题是合理的选择
    theme_mutex
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_else(|_| Theme::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Style 测试 ====================

    #[test]
    fn test_style_new() {
        let style = Style::new();
        assert!(style.foreground.is_none());
        assert!(style.background.is_none());
        assert!(style.attributes.is_empty());
    }

    #[test]
    fn test_style_default() {
        let style = Style::default();
        assert!(style.foreground.is_none());
        assert!(style.background.is_none());
        assert!(style.attributes.is_empty());
    }

    #[test]
    fn test_style_fg() {
        let style = Style::new().fg(Color::Red);
        assert_eq!(style.foreground, Some(Color::Red));
        assert!(style.background.is_none());
    }

    #[test]
    fn test_style_bg() {
        let style = Style::new().bg(Color::Blue);
        assert_eq!(style.background, Some(Color::Blue));
        assert!(style.foreground.is_none());
    }

    #[test]
    fn test_style_bold() {
        let style = Style::new().bold();
        assert_eq!(style.attributes.len(), 1);
        assert_eq!(style.attributes[0], Attribute::Bold);
    }

    #[test]
    fn test_style_apply_with_color_enabled() {
        let style = Style::new().fg(Color::Red).bold();
        let result = style.apply("test", true);
        // 当颜色启用时，应该包含样式信息（虽然格式可能因平台而异）
        assert!(!result.is_empty());
        // 至少应该包含原始文本
        assert!(result.contains("test") || result == "test");
    }

    #[test]
    fn test_style_apply_with_color_disabled() {
        let style = Style::new().fg(Color::Red).bold();
        let result = style.apply("test", false);
        // 当颜色禁用时，应该返回原始文本
        assert_eq!(result, "test");
    }

    #[test]
    fn test_style_chain() {
        let style = Style::new().fg(Color::Green).bg(Color::Blue).bold();
        assert_eq!(style.foreground, Some(Color::Green));
        assert_eq!(style.background, Some(Color::Blue));
        assert_eq!(style.attributes.len(), 1);
    }

    // ==================== Theme 测试 ====================

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        // 验证所有样式都已初始化
        assert!(theme.enable_color);
        // 验证样式不为空（至少应该有前景色设置）
        // 注意：我们只验证结构存在，不验证具体的颜色值
    }

    #[test]
    fn test_theme_get() {
        let theme1 = get_theme();
        let theme2 = get_theme();
        // 应该返回相同的主题（单例）
        assert_eq!(theme1.enable_color, theme2.enable_color);
    }

    #[test]
    fn test_theme_set_and_get() {
        let custom_theme = Theme {
            enable_color: false,
            ..Theme::default()
        };

        set_theme(custom_theme.clone());
        let retrieved = get_theme();
        assert_eq!(retrieved.enable_color, custom_theme.enable_color);
    }

    #[test]
    fn test_theme_clone() {
        let theme1 = Theme::default();
        let theme2 = theme1.clone();
        assert_eq!(theme1.enable_color, theme2.enable_color);
    }

    // ==================== 样式应用测试 ====================

    #[test]
    fn test_style_apply_empty_text() {
        let style = Style::new().fg(Color::Red);
        // 当颜色禁用时，应该返回空字符串
        let result = style.apply("", false);
        assert_eq!(result, "");

        // 当颜色启用时，可能会包含 ANSI 转义码（即使文本为空）
        let result = style.apply("", true);
        // 只验证不为空（可能包含转义码）或为空（取决于实现）
        assert!(result.is_empty() || result.contains('\u{1b}'));
    }

    #[test]
    fn test_style_apply_unicode() {
        let style = Style::new().fg(Color::Cyan);
        let result = style.apply("你好世界", false);
        assert_eq!(result, "你好世界");
    }

    #[test]
    fn test_style_apply_multiple_attributes() {
        let style = Style::new().bold().fg(Color::Yellow);
        let result = style.apply("test", false);
        assert_eq!(result, "test");
    }
}
