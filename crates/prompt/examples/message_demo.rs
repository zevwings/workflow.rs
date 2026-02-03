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
    println!("消息输出功能演示");
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

    println!("\n=== 所有演示完成 ===");
    Ok(())
}

/// 演示 1：基本消息类型
fn demo_basic_messages(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 1: 基本消息类型 ===\n");

    // 信息消息
    msg.info("这是一条普通信息消息")?;

    // 成功消息
    msg.success("操作已成功完成")?;

    // 警告消息
    msg.warning("请注意：这是一条警告消息")?;

    // 错误消息
    msg.error("发生错误：无法完成操作")?;

    // 调试消息
    msg.debug("调试信息：变量值 = 42")?;

    // 纯文本输出（无 emoji 前缀）
    msg.print("这是纯文本输出，不带任何前缀")?;

    Ok(())
}

/// 演示 2：分隔线
fn demo_separators(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 2: 分隔线 ===\n");

    // 空行
    msg.break_line()?;

    // 简单分隔线
    msg.separator('-', 40)?;

    msg.info("分隔线上方的内容")?;

    // 带文本的分隔线
    msg.separator_with_text('=', 50, "Section Title")?;

    msg.info("分隔线下方的内容")?;

    // 不同字符的分隔线
    msg.separator('*', 30)?;
    msg.separator('#', 30)?;

    Ok(())
}

/// 演示 3：实际应用场景
fn demo_real_world_usage(msg: &prompt::MessageRef) -> prompt::Result<()> {
    println!("\n=== Demo 3: 实际应用场景 ===\n");

    // 模拟一个安装过程
    msg.separator_with_text('-', 50, "Installation")?;

    msg.info("正在检查系统环境...")?;
    msg.success("系统环境检查通过")?;

    msg.info("正在下载依赖...")?;
    msg.success("依赖下载完成 (23 packages)")?;

    msg.info("正在编译项目...")?;
    msg.warning("发现 2 个弃用警告，建议后续修复")?;
    msg.success("编译完成")?;

    msg.info("正在运行测试...")?;
    msg.success("所有测试通过 (15/15)")?;

    msg.separator('-', 50)?;
    msg.success("安装完成！")?;

    msg.break_line()?;

    // 模拟错误处理
    msg.separator_with_text('-', 50, "Error Handling")?;

    msg.info("正在连接数据库...")?;
    msg.error("连接失败：Connection refused")?;
    msg.warning("将使用本地缓存模式")?;
    msg.debug("Retry count: 3, Last error: ECONNREFUSED")?;

    Ok(())
}
