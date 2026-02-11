//! 性能监控示例
//!
//! 演示如何在代码中使用性能监控工具
//!
//! 运行示例:
//! ```bash
//! cargo run -p storage --example performance_monitoring --features testing
//! ```

#[cfg(feature = "testing")]
use std::time::Duration;
#[cfg(feature = "testing")]
use storage::git::services::*;
#[cfg(feature = "testing")]
use storage::testing::performance::{measure, PerformanceCollector, PerformanceTimer};
#[cfg(feature = "testing")]
use storage::testing::*;

/// 创建 CommitServiceImpl 的辅助函数
#[cfg(feature = "testing")]
fn create_commit_service(ctx: GitContext) -> CommitServiceImpl {
    CommitServiceImpl::new(ctx, noop_hook_service())
}

#[cfg(feature = "testing")]
fn main() {
    println!("=== 性能监控示例 ===\n");

    // 1. 使用 PerformanceTimer
    println!("1. 使用 PerformanceTimer:");
    {
        let (_tmp, ctx) = setup_repo_with_files(10);

        let timer = PerformanceTimer::new("setup_and_query").with_threshold(Duration::from_secs(2));

        let service = create_commit_service(ctx);
        let status = service.get_working_tree_status().unwrap();

        println!("   文件数: {}", status.staged.len());
        let duration = timer.stop();
        println!("   耗时: {:?}\n", duration);
    }

    // 2. 使用 measure 函数
    println!("2. 使用 measure 函数:");
    {
        let (_tmp, ctx) = setup_repo_with_file();
        let service = create_commit_service(ctx);

        let info = measure("get_commit_info", || {
            service.get_commit_info("HEAD").unwrap()
        });

        println!("   提交: {}", info.sha);
        println!("   作者: {}\n", info.author_name);
    }

    // 3. 使用 PerformanceCollector 进行批量测试
    println!("3. 使用 PerformanceCollector:");
    {
        let (_tmp, ctx) = setup_repo_with_commits(5);
        let service = create_commit_service(ctx);

        let mut collector = PerformanceCollector::new("commit_info_queries");

        // 执行多次测试
        for _ in 0..5 {
            collector.measure(|| {
                service.get_commit_info("HEAD").unwrap();
            });
        }

        // 打印统计信息
        collector.print_stats();
    }

    println!("\n=== 示例完成 ===");
}

#[cfg(not(feature = "testing"))]
fn main() {
    eprintln!("此示例需要 testing feature");
    eprintln!("运行: cargo run -p storage --example performance_monitoring --features testing");
    std::process::exit(1);
}
