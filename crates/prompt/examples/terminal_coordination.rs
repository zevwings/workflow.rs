//! 终端协调示例
//!
//! 演示 spinner/progress 与日志输出的协调功能。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example terminal_coordination
//! ```

use prompt::output::terminal_state::{register_renderer, resume, suspend, RendererType};
use prompt::{spinner, progress_bar};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

/// 演示 1：Spinner 与日志协调
fn demo_spinner_with_logs() {
    println!("\n=== Demo 1: Spinner 与日志协调 ===\n");

    let spinner = spinner("Processing...").start();

    // 模拟一些工作，期间产生日志
    for i in 1..=5 {
        thread::sleep(Duration::from_millis(500));
        simulate_log_output(&format!("Step {} completed", i));
    }

    spinner.with_success("Processing complete!");
}

/// 演示 2：ProgressBar 与日志协调
fn demo_progress_with_logs() {
    println!("\n=== Demo 2: ProgressBar 与日志协调 ===\n");

    let total = 10u64;
    let pb = progress_bar("Downloading...")
        .with_total(total)
        .start();

    // 模拟下载，期间产生日志
    for i in 1..=total {
        thread::sleep(Duration::from_millis(300));
        pb.inc(1);

        if i % 3 == 0 {
            simulate_log_output(&format!("Downloaded chunk {}/{}", i, total));
        }
    }

    pb.with_success("Download complete!");
}

/// 演示 3：手动注册渲染器（用于自定义场景）
fn demo_manual_registration() {
    println!("\n=== Demo 3: 手动注册渲染器 ===\n");

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

/// 演示 4：不注册渲染器时的日志输出（对比）
fn demo_without_registration() {
    println!("\n=== Demo 4: 无渲染器时的日志输出 ===\n");

    // 这种情况下 suspend/resume 不会有任何效果
    for i in 1..=3 {
        thread::sleep(Duration::from_millis(300));
        simulate_log_output(&format!("Normal log {} (no spinner)", i));
    }

    println!("✓ Logs output normally when no renderer is active");
}

fn main() {
    println!("终端协调功能演示");
    println!("================");
    println!();
    println!("此示例演示 spinner/progress 与日志输出如何协调，");
    println!("确保它们不会互相覆盖。");

    // 演示 4 先运行（无渲染器的情况）
    demo_without_registration();

    // 演示 1：Spinner
    demo_spinner_with_logs();

    // 演示 2：ProgressBar
    demo_progress_with_logs();

    // 演示 3：手动注册
    demo_manual_registration();

    println!("\n=== 所有演示完成 ===");
}
