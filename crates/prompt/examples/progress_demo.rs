//! 进度条示例
//!
//! 演示 ProgressBar 的各种使用方式。
//!
//! 运行方式：
//! ```bash
//! cargo run -p prompt --example progress_demo
//! ```

use prompt::output::terminal_state::{resume, suspend};
use prompt::progress_bar;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn main() {
    println!("进度条功能演示");
    println!("==============");
    println!();

    // 演示 1：基本进度条
    demo_basic_progress();

    // 演示 2：下载模式
    demo_download_mode();

    // 演示 3：自定义样式
    demo_custom_style();

    // 演示 4：不同完成状态
    demo_completion_states();

    // 演示 5：未知总量（不确定进度）
    demo_indeterminate_progress();

    // 演示 6：与日志协调
    demo_progress_with_logs();

    println!("\n=== 所有演示完成 ===");
}

/// 演示 1：基本进度条
fn demo_basic_progress() {
    println!("\n=== Demo 1: 基本进度条 ===\n");

    let total = 50u64;
    let pb = progress_bar("Processing items...").with_total(total).start();

    for _ in 0..total {
        thread::sleep(Duration::from_millis(50));
        pb.inc(1);
    }

    pb.with_success("处理完成！");
}

/// 演示 2：下载模式
fn demo_download_mode() {
    println!("\n=== Demo 2: 下载模式 ===\n");

    // 模拟下载 10MB 文件
    let total_bytes = 10 * 1024 * 1024u64;
    let pb = progress_bar("Downloading file...")
        .with_total(total_bytes)
        .with_download_mode()
        .start();

    // 模拟下载过程
    let chunk_size = 512 * 1024u64; // 512KB per chunk
    let chunks = total_bytes / chunk_size;

    for _ in 0..chunks {
        thread::sleep(Duration::from_millis(100));
        pb.inc(chunk_size);
    }

    // 处理剩余字节
    let remaining = total_bytes % chunk_size;
    if remaining > 0 {
        pb.inc(remaining);
    }

    pb.with_success("下载完成！");
}

/// 演示 3：自定义样式
fn demo_custom_style() {
    println!("\n=== Demo 3: 自定义样式 ===\n");

    // 样式 1：使用 #>-
    println!("样式 1: #>-");
    let pb = progress_bar("Loading...")
        .with_total(30)
        .with_progress_chars("#>-")
        .with_bar_width(40)
        .start();

    for _ in 0..30 {
        thread::sleep(Duration::from_millis(50));
        pb.inc(1);
    }
    pb.with_success("完成");

    // 样式 2：使用 =>
    println!("\n样式 2: =>");
    let pb = progress_bar("Processing...")
        .with_total(30)
        .with_progress_chars("=>")
        .with_bar_width(40)
        .start();

    for _ in 0..30 {
        thread::sleep(Duration::from_millis(50));
        pb.inc(1);
    }
    pb.with_success("完成");

    // 样式 3：使用 ▓░
    println!("\n样式 3: ▓░");
    let pb = progress_bar("Rendering...")
        .with_total(30)
        .with_progress_chars("▓░")
        .with_bar_width(40)
        .start();

    for _ in 0..30 {
        thread::sleep(Duration::from_millis(50));
        pb.inc(1);
    }
    pb.with_success("完成");
}

/// 演示 4：不同完成状态
fn demo_completion_states() {
    println!("\n=== Demo 4: 不同完成状态 ===\n");

    // 成功完成
    let pb = progress_bar("Task 1...").with_total(20).start();

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(30));
        pb.inc(1);
    }
    pb.with_success("任务 1 成功！");

    // 失败 (使用 with_error)
    let pb = progress_bar("Task 2...").with_total(20).start();

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(30));
        pb.inc(1);
    }
    pb.with_error("任务 2 失败");

    // 信息 (使用 with_info)
    let pb = progress_bar("Task 3...").with_total(20).start();

    for _ in 0..20 {
        thread::sleep(Duration::from_millis(30));
        pb.inc(1);
    }
    pb.with_info("任务 3 完成，有信息");
}

/// 演示 5：未知总量（不确定进度）
fn demo_indeterminate_progress() {
    println!("\n=== Demo 5: 未知总量 ===\n");

    // 不设置 total，显示不确定进度
    let pb = progress_bar("Scanning files...").start();

    // 模拟扫描过程
    for i in 0..50 {
        thread::sleep(Duration::from_millis(50));
        pb.inc(1);

        // 每 10 个更新消息
        if (i + 1) % 10 == 0 {
            pb.update_message(format!("Scanned {} files...", (i + 1) * 10));
        }
    }

    pb.with_success("扫描完成，共找到 500 个文件");
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

/// 演示 6：ProgressBar 与日志协调
fn demo_progress_with_logs() {
    println!("\n=== Demo 6: ProgressBar 与日志协调 ===\n");

    let total = 10u64;
    let pb = progress_bar("Downloading...").with_total(total).start();

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
