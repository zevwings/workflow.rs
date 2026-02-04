//! 对话框示例
//!
//! 演示 input, confirm, select, multiselect 对话框。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example dialog_demo
//! ```
//!
//! 注意：此示例需要交互式终端，不能在非 TTY 环境下运行。

use prompt::{
    confirm, input, is_user_cancelled, multiselect, select, validators, Message, PromptError,
};

fn main() {
    println!("对话框功能演示");
    println!("==============");
    println!();
    println!("此示例演示各种交互式对话框。");
    println!("按 Ctrl+C 或 Esc 可以取消任何对话框。");
    println!();

    let msg = Message::global();

    // 演示 1：确认对话框
    if let Err(e) = demo_confirm(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("演示被用户取消");
            return;
        }
        let _ = msg.error(format!("错误: {}", e));
        return;
    }

    // 演示 2：输入对话框
    if let Err(e) = demo_input(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("演示被用户取消");
            return;
        }
        let _ = msg.error(format!("错误: {}", e));
        return;
    }

    // 演示 3：选择对话框
    if let Err(e) = demo_select(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("演示被用户取消");
            return;
        }
        let _ = msg.error(format!("错误: {}", e));
        return;
    }

    // 演示 4：多选对话框
    if let Err(e) = demo_multiselect(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("演示被用户取消");
            return;
        }
        let _ = msg.error(format!("错误: {}", e));
        return;
    }

    let _ = msg.break_line();
    let _ = msg.success("所有演示完成！");
}

/// 检查是否是用户取消操作
fn is_cancelled(e: &PromptError) -> bool {
    is_user_cancelled(&e.to_string())
}

/// 演示 1：确认对话框
fn demo_confirm(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.separator_with_text('-', 50, "Demo 1: Confirm");

    // 基本确认
    let continue_demo = confirm!("是否继续演示？").default(true).prompt()?;

    if !continue_demo {
        let _ = msg.info("用户选择不继续");
        return Ok(());
    }

    // 带格式化消息的确认
    let branch = "feature/demo";
    let confirmed = confirm!("是否要合并分支 '{}' 到 main？", branch)
        .default(false)
        .result_title("Merge")
        .prompt()?;

    let _ = msg.info(format!("合并选择: {}", if confirmed { "是" } else { "否" }));

    Ok(())
}

/// 演示 2：输入对话框
fn demo_input(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 2: Input");

    // 基本输入
    let name = input!("请输入您的姓名")
        .default("User")
        .placeholder("输入姓名...")
        .result_title("Name")
        .prompt()?;

    let _ = msg.info(format!("您好, {}!", name));

    // 带验证的输入（使用 regex 验证邮箱）
    let email_validator = validators::regex(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
        Some("请输入有效的邮箱地址"),
    )
    .expect("Invalid regex");

    let email = input!("请输入您的邮箱")
        .placeholder("example@email.com")
        .validator(email_validator)
        .result_title("Email")
        .prompt()?;

    let _ = msg.info(format!("邮箱: {}", email));

    // 带长度验证的输入
    let username = input!("请输入用户名（至少 3 个字符）")
        .validator(validators::min_length(3))
        .result_title("Username")
        .prompt()?;

    let _ = msg.info(format!("用户名: {}", username));

    Ok(())
}

/// 演示 3：选择对话框
fn demo_select(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 3: Select");

    // 基本选择
    let colors = vec!["Red", "Green", "Blue", "Yellow", "Purple"];
    let color = select!("请选择您喜欢的颜色", colors)
        .default(0)
        .result_title("Color")
        .prompt()?;

    let _ = msg.info(format!("您选择了: {}", color));

    // 带分页的选择（当选项多时）
    let languages = vec![
        "Rust",
        "Python",
        "JavaScript",
        "TypeScript",
        "Go",
        "Java",
        "C++",
        "C#",
        "Swift",
        "Kotlin",
        "Ruby",
        "PHP",
    ];
    let lang = select!("请选择您最喜欢的编程语言", languages)
        .page_size(5)
        .result_title("Language")
        .prompt()?;

    let _ = msg.info(format!("您选择了: {}", lang));

    Ok(())
}

/// 演示 4：多选对话框
fn demo_multiselect(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 4: MultiSelect");

    // 基本多选
    let features = vec![
        "Authentication",
        "Authorization",
        "Logging",
        "Caching",
        "Rate Limiting",
        "Monitoring",
    ];

    let selected = multiselect!("请选择要启用的功能", features)
        .default(vec![0, 2]) // 默认选中第 1 和第 3 项
        .result_title("Features")
        .prompt()?;

    if selected.is_empty() {
        let _ = msg.warning("未选择任何功能");
    } else {
        let _ = msg.info(format!("已选择 {} 个功能:", selected.len()));
        for feature in &selected {
            let _ = msg.print(format!("  - {}", feature));
        }
    }

    // 带分页的多选
    let frameworks = vec![
        "React", "Vue", "Angular", "Svelte", "Next.js", "Nuxt.js", "Remix", "Astro", "SolidJS",
        "Qwik",
    ];

    let selected_frameworks = multiselect!("请选择您熟悉的前端框架", frameworks)
        .page_size(5)
        .result_title("Frameworks")
        .prompt()?;

    let _ = msg.info(format!("已选择: {:?}", selected_frameworks));

    Ok(())
}
