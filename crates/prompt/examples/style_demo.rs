//! 样式与主题示例
//!
//! 演示 Style 和 Theme 的使用方式。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example style_demo
//! ```

use crossterm::style::Color;
use prompt::{get_theme, set_theme, Style, Theme};

fn main() {
    println!("Style and theme demonstration");
    println!("==================");
    println!();
    println!("This example demonstrates how to use Style and Theme to display text with different styles.");
    println!("Press Ctrl+C or Esc to cancel the style and theme.");
    println!();

    // 演示 1：基本样式使用
    demo_basic_style();

    // 演示 2：样式组合
    demo_style_composition();

    // 演示 3：主题使用
    demo_theme_usage();

    // 演示 4：自定义主题
    demo_custom_theme();

    println!("\n=== All demonstrations completed ===");
}

/// 演示 1：基本样式使用
fn demo_basic_style() {
    println!("\n=== Demo 1: Basic style ===\n");

    // 创建不同颜色的样式
    let red_style = Style::new().fg(Color::Red);
    let green_style = Style::new().fg(Color::Green);
    let blue_style = Style::new().fg(Color::Blue);
    let yellow_style = Style::new().fg(Color::Yellow);

    // 应用样式（enable_color = true）
    println!("{}", red_style.apply("This is red text", true));
    println!("{}", green_style.apply("This is green text", true));
    println!("{}", blue_style.apply("This is blue text", true));
    println!("{}", yellow_style.apply("This is yellow text", true));

    // 禁用颜色时的效果
    println!("\nWhen color is disabled:");
    println!("{}", red_style.apply("Color is disabled, displaying original text", false));
}

/// 演示 2：样式组合
fn demo_style_composition() {
    println!("\n=== Demo 2: Style composition ===\n");

    // 组合前景色、背景色和属性
    let bold_red = Style::new().fg(Color::Red).bold();
    let cyan_on_black = Style::new().fg(Color::Cyan).bg(Color::Black);
    let bold_green = Style::new().fg(Color::Green).bold();

    println!("{}", bold_red.apply("Bold red text", true));
    println!("{}", cyan_on_black.apply("Cyan on black text", true));
    println!("{}", bold_green.apply("Bold green text", true));

    // 链式调用示例
    let complex_style = Style::new().fg(Color::Magenta).bg(Color::White).bold();
    println!("{}", complex_style.apply("Complex style combination", true));
}

/// 演示 3：主题使用
fn demo_theme_usage() {
    println!("\n=== Demo 3: Default theme ===\n");

    let theme = get_theme();

    // 使用主题中的预定义样式
    println!("{}", theme.info.apply("ℹ This is info style", theme.enable_color));
    println!(
        "{}",
        theme.success.apply("✓ This is success style", theme.enable_color)
    );
    println!(
        "{}",
        theme.warning.apply("⚠ This is warning style", theme.enable_color)
    );
    println!(
        "{}",
        theme.error.apply("✗ This is error style", theme.enable_color)
    );
    println!(
        "{}",
        theme.debug.apply("⚙ This is debug style", theme.enable_color)
    );
    println!("{}", theme.hint.apply("This is hint style", theme.enable_color));
}

/// 演示 4：自定义主题
fn demo_custom_theme() {
    println!("\n=== Demo 4: Custom theme ===\n");

    // 创建自定义主题
    let custom_theme = Theme {
        info: Style::new().fg(Color::Blue),
        success: Style::new().fg(Color::Cyan).bold(),
        warning: Style::new().fg(Color::Magenta),
        error: Style::new().fg(Color::Red).bg(Color::Black).bold(),
        debug: Style::new().fg(Color::Grey),
        title: Style::new().fg(Color::White).bold(),
        answer: Style::new().fg(Color::Green),
        hint: Style::new().fg(Color::DarkGrey),
        prefix: Style::new().fg(Color::Yellow),
        progress: Style::new().fg(Color::Cyan),
        spinner: Style::new().fg(Color::Blue),
        enable_color: true,
    };

    // 设置全局主题
    set_theme(custom_theme.clone());

    // 验证主题已更新
    let theme = get_theme();
    println!(
        "{}",
        theme.info.apply("ℹ custom info style", theme.enable_color)
    );
    println!(
        "{}",
        theme.success.apply("✓ custom success style", theme.enable_color)
    );
    println!(
        "{}",
        theme.warning.apply("⚠ custom warning style", theme.enable_color)
    );
    println!(
        "{}",
        theme.error.apply("✗ custom error style", theme.enable_color)
    );

    // 恢复默认主题
    set_theme(Theme::default());
    println!("\n(Default theme restored)");
}
