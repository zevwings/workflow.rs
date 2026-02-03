//! Spinner 加载指示器示例
//!
//! 演示 Spinner 的各种使用方式。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example spinner_demo
//! ```

use prompt::output::terminal_state::{register_renderer, resume, suspend, RendererType};
use prompt::spinner;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Spinner 加载指示器演示");
    println!("======================");
    println!();

    // 演示 1：基本使用
    demo_basic_spinner();

    // 演示 2：不同完成状态
    demo_completion_states();

    // 演示 3：自定义帧动画
    demo_custom_frames();

    // 演示 4：with 便捷方法
    demo_with_method();

    // 演示 5：更新消息
    demo_update_message();

    // 演示 6：与日志协调
    demo_spinner_with_logs();

    // 演示 7：手动注册渲染器（高级用法）
    demo_manual_registration();

    // 演示 8：无渲染器时的日志输出（对比）
    demo_without_registration();

    println!("\n=== 所有演示完成 ===");
}

/// 演示 1：基本使用
fn demo_basic_spinner() {
    println!("\n=== Demo 1: 基本 Spinner ===\n");

    let sp = spinner("正在加载...").start();

    // 模拟工作
    thread::sleep(Duration::from_secs(2));

    sp.stop();
    println!("基本 spinner 完成");
}

/// 演示 2：不同完成状态
fn demo_completion_states() {
    println!("\n=== Demo 2: 不同完成状态 ===\n");

    // 成功状态
    let sp = spinner("正在执行任务 1...").start();
    thread::sleep(Duration::from_secs(1));
    sp.with_success("任务 1 成功完成！");

    // 失败状态 (使用 with_error)
    let sp = spinner("正在执行任务 2...").start();
    thread::sleep(Duration::from_secs(1));
    sp.with_error("任务 2 执行失败");

    // 信息状态 (使用 with_info)
    let sp = spinner("正在执行任务 3...").start();
    thread::sleep(Duration::from_secs(1));
    sp.with_info("任务 3 完成，有信息");

    // 自定义完成消息
    let sp = spinner("正在执行任务 4...").start();
    thread::sleep(Duration::from_secs(1));
    sp.finish_with_message("→ 任务 4 已处理");
}

/// 演示 3：自定义帧动画
fn demo_custom_frames() {
    println!("\n=== Demo 3: 自定义帧动画 ===\n");

    // 点动画
    let sp = spinner("Dots animation")
        .with_frames(vec![".", "..", "...", "...."])
        .with_interval(Duration::from_millis(200))
        .start();
    thread::sleep(Duration::from_secs(2));
    sp.with_success("完成");

    // 箭头动画
    let sp = spinner("Arrow animation")
        .with_frames(vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"])
        .with_interval(Duration::from_millis(100))
        .start();
    thread::sleep(Duration::from_secs(2));
    sp.with_success("完成");

    // 方块动画
    let sp = spinner("Block animation")
        .with_frames(vec!["▖", "▘", "▝", "▗"])
        .with_interval(Duration::from_millis(150))
        .start();
    thread::sleep(Duration::from_secs(2));
    sp.with_success("完成");
}

/// 演示 4：with 便捷方法
fn demo_with_method() {
    println!("\n=== Demo 4: with 便捷方法 ===\n");

    // 使用 with 方法包装一个操作
    let result: Result<i32, &str> = spinner("正在计算...")
        .with(|| {
            // 模拟计算
            thread::sleep(Duration::from_secs(1));
            Ok(42)
        });

    println!("计算结果: {:?}", result);

    // 使用 with_output 方法（适用于会产生输出的操作）
    let result: Result<String, &str> = spinner("正在处理...")
        .with_output(|| {
            // 模拟处理
            thread::sleep(Duration::from_millis(500));
            Ok("处理完成".to_string())
        });

    println!("处理结果: {:?}", result);
}

/// 演示 5：更新消息
fn demo_update_message() {
    println!("\n=== Demo 5: 更新消息 ===\n");

    let sp = spinner("步骤 1/3: 初始化...").start();
    thread::sleep(Duration::from_secs(1));

    sp.update_message("步骤 2/3: 处理数据...");
    thread::sleep(Duration::from_secs(1));

    sp.update_message("步骤 3/3: 完成...");
    thread::sleep(Duration::from_secs(1));

    sp.with_success("所有步骤完成！");
}

/// 模拟日志输出（在实际应用中由 tracing 触发）
fn simulate_log_output(message: &str) {
    // 暂停终端渲染
    suspend();

    // 输出日志
    let mut stderr = io::stderr();
    writeln!(stderr, "[LOG] {}", message).unwrap();
    stderr.flush().unwrap();

    // 恢复终端渲染
    resume();
}

/// 演示 6：Spinner 与日志协调
fn demo_spinner_with_logs() {
    println!("\n=== Demo 6: Spinner 与日志协调 ===\n");

    let sp = spinner("Processing...").start();

    // 模拟一些工作，期间产生日志
    for i in 1..=5 {
        thread::sleep(Duration::from_millis(500));
        simulate_log_output(&format!("Step {} completed", i));
    }

    sp.with_success("Processing complete!");
}

/// 演示 7：手动注册渲染器（高级用法）
fn demo_manual_registration() {
    println!("\n=== Demo 7: 手动注册渲染器（高级用法）===\n");

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = Arc::clone(&running);

    // 注册渲染器
    register_renderer(RendererType::Spinner, || {
        // 这里可以放重绘逻辑，但渲染线程会自动重绘
    });

    // 启动自定义渲染线程
    let handle = thread::spawn(move || {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut idx = 0;

        while running_clone.load(Ordering::SeqCst) {
            // 检查是否暂停
            if !prompt::output::terminal_state::is_suspended() {
                let mut stderr = io::stderr();
                write!(stderr, "\r{} Custom spinner...", frames[idx % frames.len()]).unwrap();
                stderr.flush().unwrap();
            }

            idx += 1;
            thread::sleep(Duration::from_millis(100));
        }

        // 清除行
        let mut stderr = io::stderr();
        write!(stderr, "\r                      \r").unwrap();
        stderr.flush().unwrap();
    });

    // 模拟日志输出
    for i in 1..=3 {
        thread::sleep(Duration::from_millis(800));
        simulate_log_output(&format!("Custom log {}", i));
    }

    // 停止渲染
    running.store(false, Ordering::SeqCst);
    handle.join().unwrap();

    // 注销渲染器
    prompt::output::terminal_state::unregister_renderer();

    println!("✓ Custom spinner finished!");
}

/// 演示 8：无渲染器时的日志输出（对比）
fn demo_without_registration() {
    println!("\n=== Demo 8: 无渲染器时的日志输出 ===\n");

    // 这种情况下 suspend/resume 不会有任何效果
    for i in 1..=3 {
        thread::sleep(Duration::from_millis(300));
        simulate_log_output(&format!("Normal log {} (no spinner)", i));
    }

    println!("✓ Logs output normally when no renderer is active");
}
