//! 样式与主题示例
//!
//! 演示 Style 和 Theme 的使用方式。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example style_demo
//! ```

use crossterm::style::Color;
use prompt::style::theme::{get_theme, set_theme, Style, Theme};

fn main() {
    println!("样式与主题功能演示");
    println!("==================");
    println!();

    // 演示 1：基本样式使用
    demo_basic_style();

    // 演示 2：样式组合
    demo_style_composition();

    // 演示 3：主题使用
    demo_theme_usage();

    // 演示 4：自定义主题
    demo_custom_theme();

    println!("\n=== 所有演示完成 ===");
}

/// 演示 1：基本样式使用
fn demo_basic_style() {
    println!("\n=== Demo 1: 基本样式 ===\n");

    // 创建不同颜色的样式
    let red_style = Style::new().fg(Color::Red);
    let green_style = Style::new().fg(Color::Green);
    let blue_style = Style::new().fg(Color::Blue);
    let yellow_style = Style::new().fg(Color::Yellow);

    // 应用样式（enable_color = true）
    println!("{}", red_style.apply("这是红色文本", true));
    println!("{}", green_style.apply("这是绿色文本", true));
    println!("{}", blue_style.apply("这是蓝色文本", true));
    println!("{}", yellow_style.apply("这是黄色文本", true));

    // 禁用颜色时的效果
    println!("\n禁用颜色时：");
    println!("{}", red_style.apply("颜色被禁用，显示原始文本", false));
}

/// 演示 2：样式组合
fn demo_style_composition() {
    println!("\n=== Demo 2: 样式组合 ===\n");

    // 组合前景色、背景色和属性
    let bold_red = Style::new().fg(Color::Red).bold();
    let cyan_on_black = Style::new().fg(Color::Cyan).bg(Color::Black);
    let bold_green = Style::new().fg(Color::Green).bold();

    println!("{}", bold_red.apply("粗体红色文本", true));
    println!("{}", cyan_on_black.apply("黑底青色文本", true));
    println!("{}", bold_green.apply("粗体绿色文本", true));

    // 链式调用示例
    let complex_style = Style::new().fg(Color::Magenta).bg(Color::White).bold();
    println!("{}", complex_style.apply("复杂样式组合", true));
}

/// 演示 3：主题使用
fn demo_theme_usage() {
    println!("\n=== Demo 3: 默认主题 ===\n");

    let theme = get_theme();

    // 使用主题中的预定义样式
    println!("{}", theme.info.apply("ℹ 这是信息样式", theme.enable_color));
    println!(
        "{}",
        theme.success.apply("✓ 这是成功样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.warning.apply("⚠ 这是警告样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.error.apply("✗ 这是错误样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.debug.apply("⚙ 这是调试样式", theme.enable_color)
    );
    println!("{}", theme.hint.apply("这是提示样式", theme.enable_color));
}

/// 演示 4：自定义主题
fn demo_custom_theme() {
    println!("\n=== Demo 4: 自定义主题 ===\n");

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
        theme.info.apply("ℹ 自定义信息样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.success.apply("✓ 自定义成功样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.warning.apply("⚠ 自定义警告样式", theme.enable_color)
    );
    println!(
        "{}",
        theme.error.apply("✗ 自定义错误样式", theme.enable_color)
    );

    // 恢复默认主题
    set_theme(Theme::default());
    println!("\n（已恢复默认主题）");
}
