//! 消息输出示例
//!
//! 演示 Message 的各种输出类型。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example message_demo
//! ```

use prompt::Message;

fn main() -> prompt::Result<()> {
    println!("Message output demo");
    println!("================");
    println!();

    // 获取全局 Message 单例
    let msg = Message::global();

    // 演示 1：基本消息类型
    demo_basic_messages(&msg)?;

    // 演示 2：分隔线
    demo_separators(&msg)?;

    // 演示 3：实际应用场景
    demo_real_world_usage(&msg)?;

    println!("\n=== All demos completed ===");
    Ok(())
}

/// 演示 1：基本消息类型
fn demo_basic_messages(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 1: Basic message types ===\n");

    // 信息消息
    msg.info("This is a normal info message")?;

    // 成功消息
    msg.success("Operation completed successfully")?;

    // 警告消息
    msg.warning("Warning: This is a warning message")?;

    // 错误消息
    msg.error("Error: Failed to complete operation")?;

    // 调试消息
    msg.debug("Debug info: variable value = 42")?;

    // 纯文本输出（无 emoji 前缀）
    msg.print("This is plain text output without any prefix")?;

    Ok(())
}

/// 演示 2：分隔线
fn demo_separators(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 2: Separators ===\n");

    // 空行
    msg.break_line()?;

    // 简单分隔线
    msg.separator('-', 40)?;

    msg.info("Content above the separator")?;

    // 带文本的分隔线
    msg.separator_with_text('=', 50, "Section Title")?;

    msg.info("Content below the separator")?;

    // 不同字符的分隔线
    msg.separator('*', 30)?;
    msg.separator('#', 30)?;

    Ok(())
}

/// 演示 3：实际应用场景
fn demo_real_world_usage(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 3: Real-world usage ===\n");

    // 模拟一个安装过程
    msg.separator_with_text('-', 50, "Installation")?;

    msg.info("Checking system environment...")?;
    msg.success("System environment check passed")?;

    msg.info("Downloading dependencies...")?;
    msg.success("Dependencies downloaded (23 packages)")?;

    msg.info("Compiling project...")?;
    msg.warning("Found 2 deprecation warnings, consider fixing later")?;
    msg.success("Compilation completed")?;

    msg.info("Running tests...")?;
    msg.success("All tests passed (15/15)")?;

    msg.separator('-', 50)?;
    msg.success("Installation completed!")?;

    msg.break_line()?;

    // 模拟错误处理
    msg.separator_with_text('-', 50, "Error Handling")?;

    msg.info("Connecting to database...")?;
    msg.error("Connection failed: Connection refused")?;
    msg.warning("Will use local cache mode")?;
    msg.debug("Retry count: 3, Last error: ECONNREFUSED")?;

    Ok(())
}
