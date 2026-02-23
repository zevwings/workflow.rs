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
    println!("Dialog functionality demonstration");
    println!("==============");
    println!();
    println!("This example demonstrates various interactive dialogs.");
    println!("Press Ctrl+C or Esc to cancel any dialog.");
    println!();

    let msg = Message::global();

    // 演示 1：确认对话框
    if let Err(e) = demo_confirm(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("Demo cancelled by user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    // 演示 2：输入对话框
    if let Err(e) = demo_input(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("Demo cancelled by user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    // 演示 3：选择对话框
    if let Err(e) = demo_select(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("Demo cancelled by user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    // 演示 4：多选对话框
    if let Err(e) = demo_multiselect(&msg) {
        if is_cancelled(&e) {
            let _ = msg.warning("Demo cancelled by user");
            return;
        }
        let _ = msg.error(format!("Error: {}", e));
        return;
    }

    let _ = msg.break_line();
    let _ = msg.success("All demos completed!");
}

/// 检查是否是用户取消操作
fn is_cancelled(e: &PromptError) -> bool {
    is_user_cancelled(&e.to_string())
}

/// 演示 1：确认对话框
fn demo_confirm(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.separator_with_text('-', 50, "Demo 1: Confirm");

    // 基本确认
    let continue_demo = confirm!("Do you want to continue the demo?").default(true).prompt()?;

    if !continue_demo {
        let _ = msg.info("User chose not to continue");
        return Ok(());
    }

    // 带格式化消息的确认
    let branch = "feature/demo";
    let confirmed = confirm!("Do you want to merge branch '{}' to main?", branch)
        .default(false)
        .result_title("Merge")
        .prompt()?;

    let _ = msg.info(format!(
        "Merge choice: {}",
        if confirmed { "Yes" } else { "No" }
    ));

    Ok(())
}

/// 演示 2：输入对话框
fn demo_input(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 2: Input");

    // 基本输入
    let name = input!("Please enter your name")
        .default("User")
        .placeholder("Enter your name...")
        .result_title("Name")
        .prompt()?;

    let _ = msg.info(format!("Hello, {}!", name));

    // 带验证的输入（使用 regex 验证邮箱）
    let email_validator = validators::regex(
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
        Some("Please enter a valid email address"),
    )
    .expect("Invalid regex");

    let email = input!("Please enter your email")
        .placeholder("example@email.com")
        .validator(email_validator)
        .result_title("Email")
        .prompt()?;

    let _ = msg.info(format!("Email: {}", email));

    // 带长度验证的输入
    let username = input!("Please enter your username (at least 3 characters)")
        .validator(validators::min_length(3))
        .result_title("Username")
        .prompt()?;

    let _ = msg.info(format!("Username: {}", username));

    // 多行输入
    let changes = input!("Please enter your changes (can be multiple lines)")
        .multiline()
        .placeholder("For example:\n- Fixed XXX\n- Optimized YYY\n- Added ZZZ")
        .result_title("Changes")
        .prompt()?;

    let _ = msg.info(format!("Changes:\n{}", changes));

    Ok(())
}

/// 演示 3：选择对话框
fn demo_select(msg: &prompt::MessageRef) -> prompt::Result<()> {
    let _ = msg.break_line();
    let _ = msg.separator_with_text('-', 50, "Demo 3: Select");

    // 基本选择
    let colors = vec!["Red", "Green", "Blue", "Yellow", "Purple"];
    let color = select!("Please select your favorite color", colors)
        .default(0)
        .result_title("Color")
        .prompt()?;

    let _ = msg.info(format!("You selected: {}", color));

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
    let lang = select!(
        "Please select your favorite programming language",
        languages
    )
    .page_size(5)
    .result_title("Language")
    .prompt()?;

    let _ = msg.info(format!("You selected: {}", lang));

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

    let selected = multiselect!("Please select the features to enable", features)
        .default(vec![0, 2]) // default selected the 1st and 3rd item
        .result_title("Features")
        .prompt()?;

    if selected.is_empty() {
        let _ = msg.warning("No features selected");
    } else {
        let _ = msg.info(format!("Selected {} features:", selected.len()));
        for feature in &selected {
            let _ = msg.print(format!("  - {}", feature));
        }
    }

    // 带分页的多选
    let frameworks = vec![
        "React", "Vue", "Angular", "Svelte", "Next.js", "Nuxt.js", "Remix", "Astro", "SolidJS",
        "Qwik",
    ];

    let selected_frameworks = multiselect!(
        "Please select the frontend frameworks you are familiar with",
        frameworks
    )
    .page_size(5)
    .result_title("Frameworks")
    .prompt()?;

    let _ = msg.info(format!("Selected: {:?}", selected_frameworks));

    Ok(())
}
