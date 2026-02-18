//! Git 服务压力测试和性能测试
//!
//! 这些测试默认被忽略（标记为 #[ignore]），只在需要时运行：
//! ```bash
//! cargo test -p storage --test stress_tests --features testing -- --ignored --test-threads=1
//! ```

#[cfg(feature = "testing")]
use std::sync::{Arc, Barrier};
#[cfg(feature = "testing")]
use std::thread;
#[cfg(feature = "testing")]
use std::time::{Duration, Instant};

#[cfg(feature = "testing")]
use storage::git::services::*;
use storage::testing::*;

/// 创建 CommitServiceImpl 的辅助函数
#[cfg(feature = "testing")]
fn create_commit_service(ctx: GitContext) -> CommitServiceImpl {
    CommitServiceImpl::new(ctx, noop_hook_service())
}

// ============================================================
// 并发访问测试
// ============================================================

/// 测试多线程并发访问同一个 GitContext
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_concurrent_read_access() {
    let (_tmp, ctx) = setup_repo_with_file();
    let ctx = Arc::new(ctx);
    let thread_count = 100;
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = vec![];

    let start = Instant::now();

    // 创建 100 个线程同时读取
    for i in 0..thread_count {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            // 等待所有线程准备就绪
            barrier_clone.wait();

            let service = create_commit_service((*ctx_clone).clone());

            // 执行多次读取操作
            for _ in 0..10 {
                service.get_commit_info("HEAD").unwrap();
                service.get_working_tree_status().unwrap();
            }

            println!("Thread {} completed", i);
        });

        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!(
        "Concurrent read test completed: {} threads, duration: {:?}",
        thread_count, duration
    );

    // 断言性能在合理范围内（100 线程应该在 30 秒内完成）
    assert!(
        duration < Duration::from_secs(30),
        "Concurrent read test duration too long: {:?}",
        duration
    );
}

/// 测试混合读写操作的并发性能
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_concurrent_mixed_operations() {
    let (_tmp, ctx) = setup_repo_with_changes(10, 10);
    let ctx = Arc::new(ctx);
    let thread_count = 20;
    let mut handles = vec![];

    let start = Instant::now();

    // 10 个读线程
    for i in 0..thread_count / 2 {
        let ctx_clone = Arc::clone(&ctx);
        let handle = thread::spawn(move || {
            let service = create_commit_service((*ctx_clone).clone());
            for _ in 0..5 {
                service.get_commit_info("HEAD").unwrap();
                thread::sleep(Duration::from_millis(10));
            }
            println!("Read thread {} completed", i);
        });
        handles.push(handle);
    }

    // 10 个状态查询线程
    for i in 0..thread_count / 2 {
        let ctx_clone = Arc::clone(&ctx);
        let handle = thread::spawn(move || {
            let service = create_commit_service((*ctx_clone).clone());
            for _ in 0..5 {
                service.get_working_tree_status().unwrap();
                thread::sleep(Duration::from_millis(10));
            }
            println!("Status thread {} completed", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();
    println!("Concurrent mixed operations test completed, duration: {:?}", duration);

    assert!(
        duration < Duration::from_secs(20),
        "Concurrent mixed operations test duration too long: {:?}",
        duration
    );
}

// ============================================================
// 大规模数据测试
// ============================================================

/// 测试处理大量文件的性能
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_large_file_count_performance() {
    let file_counts = [100, 500, 1000, 2000];

    for &count in &file_counts {
        println!("\nTesting {} files...", count);
        let start = Instant::now();

        let (_tmp, ctx) = setup_repo_with_files(count);
        let setup_time = start.elapsed();

        let service = create_commit_service(ctx);

        // 测试获取状态的性能
        let status_start = Instant::now();
        let status = service.get_working_tree_status().unwrap();
        let status_time = status_start.elapsed();

        println!("  - Setup duration: {:?}", setup_time);
        println!("  - Status query duration: {:?}", status_time);
        println!("  - 文件总数: {}", status.staged.len());

        // 断言性能合理
        // 2000 个文件的状态查询应该在 5 秒内完成
        if count <= 2000 {
            assert!(
                status_time < Duration::from_secs(5),
                "{} file status query duration too long: {:?}",
                count,
                status_time
            );
        }
    }
}

/// 测试处理大文件 blame 的性能
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_large_file_blame_performance() {
    let line_counts = [1000, 5000, 10000];

    for &count in &line_counts {
        println!("\nTesting {} line file blame...", count);

        let (_tmp, ctx) = setup_repo_with_large_file(count);
        let service = BlameServiceImpl::new(ctx);

        let start = Instant::now();
        let blame = service.get_file_blame("large_file.txt", None).unwrap();
        let duration = start.elapsed();

        println!("  - Blame duration: {:?}", duration);
        println!("  - Line count: {}", blame.len());

        assert_eq!(blame.len(), count);

        // 10000 行应该在 30 秒内完成
        if count <= 10000 {
            assert!(
                duration < Duration::from_secs(30),
                "{} line blame duration too long: {:?}",
                count,
                duration
            );
        }
    }
}

/// 测试处理大量分支的性能
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_large_branch_count_performance() {
    let branch_counts = [50, 100, 200];

    for &count in &branch_counts {
        println!("\nTesting {} branches...", count);

        let (_tmp, ctx) = setup_repo_with_branches(count);
        let service = BranchServiceImpl::new(ctx);

        let start = Instant::now();
        let branches = service.list_branches(false, false).unwrap();
        let duration = start.elapsed();

        println!("  - List branches duration: {:?}", duration);
        println!("  - Branch count: {}", branches.len());

        // 200 个分支应该在 2 秒内完成
        assert!(
            duration < Duration::from_secs(2),
            "{} branch list duration too long: {:?}",
            count,
            duration
        );
    }
}

// ============================================================
// 提交性能测试（带 --all）
// ============================================================

/// 测试 commit --all 在不同文件数量下的性能
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_commit_all_performance() {
    let file_counts = [10, 50, 100];

    for &count in &file_counts {
        println!("\nTesting commit {} files (--all)...", count);

        let (_tmp, ctx) = setup_repo_with_changes(count, count);
        let service = create_commit_service(ctx);

        let start = Instant::now();
        service.commit("test commit", true).unwrap();
        let duration = start.elapsed();

        println!("  - Commit duration: {:?}", duration);

        // 100 个文件应该在 10 秒内完成
        assert!(
            duration < Duration::from_secs(10),
            "{} file commit duration too long: {:?}",
            count,
            duration
        );
    }
}

// ============================================================
// 锁竞争测试
// ============================================================

/// 测试 GitContext 锁竞争情况
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_lock_contention() {
    let (_tmp, ctx) = setup_repo_with_file();
    let ctx = Arc::new(ctx);
    let thread_count = 50;
    let barrier = Arc::new(Barrier::new(thread_count));
    let mut handles = vec![];
    let lock_wait_times = Arc::new(std::sync::Mutex::new(Vec::new()));

    for i in 0..thread_count {
        let ctx_clone = Arc::clone(&ctx);
        let barrier_clone = Arc::clone(&barrier);
        let times_clone = Arc::clone(&lock_wait_times);

        let handle = thread::spawn(move || {
            barrier_clone.wait();

            let start = Instant::now();
            let service = create_commit_service((*ctx_clone).clone());
            service.get_commit_info("HEAD").unwrap();
            let duration = start.elapsed();

            times_clone.lock().unwrap().push(duration);

            if i % 10 == 0 {
                println!("Thread {} - Duration: {:?}", i, duration);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 分析锁等待时间
    let times = lock_wait_times.lock().unwrap();
    let total: Duration = times.iter().sum();
    let avg = total / times.len() as u32;
    let max = times.iter().max().unwrap();

    println!("\nLock contention statistics:");
    println!("  - Thread count: {}", thread_count);
    println!("  - Average duration: {:?}", avg);
    println!("  - Maximum duration: {:?}", max);

    // 平均耗时应该小于 500ms
    assert!(
        avg < Duration::from_millis(500),
        "Lock contention caused average duration too long: {:?}",
        avg
    );
}

// ============================================================
// 内存使用测试
// ============================================================

/// 测试大量操作的内存稳定性
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_memory_stability() {
    println!("Starting memory stability test...");

    let (_tmp, ctx) = setup_repo_with_files(100);

    // 执行 1000 次操作，确保没有内存泄漏
    for i in 0..1000 {
        let service = create_commit_service(ctx.clone());
        service.get_commit_info("HEAD").unwrap();
        service.get_working_tree_status().unwrap();

        if i % 100 == 0 {
            println!("  - Completed {} operations", i);
        }
    }

    println!("Memory stability test completed");
}

// ============================================================
// 端到端性能测试
// ============================================================

/// 模拟真实工作流的端到端测试
#[test]
#[ignore]
#[cfg(feature = "testing")]
fn test_realistic_workflow() {
    println!("Simulating realistic workflow...");

    // 1. 创建仓库
    let (_tmp, ctx) = setup_repo_with_files(50);

    let commit_service = create_commit_service(ctx.clone());
    let branch_service = BranchServiceImpl::new(ctx.clone());

    let start = Instant::now();

    // 2. 检查状态
    let status = commit_service.get_working_tree_status().unwrap();
    println!("  - Status: {} files", status.staged.len());

    // 3. 创建分支
    branch_service.create_branch("feature-branch").unwrap();
    println!("  - Create branch completed");

    // 4. 切换分支
    branch_service.checkout_branch("feature-branch").unwrap();
    println!("  - Checkout branch completed");

    // 5. 列出分支
    let branches = branch_service.list_branches(false, false).unwrap();
    println!("  - Branch count: {}", branches.len());

    // 6. 获取提交信息
    let info = commit_service.get_commit_info("HEAD").unwrap();
    println!("  - Current commit: {}", info.sha);

    let duration = start.elapsed();
    println!("Workflow completed, total duration: {:?}", duration);

    // 整个工作流应该在 5 秒内完成
    assert!(
        duration < Duration::from_secs(5),
        "Workflow duration too long: {:?}",
        duration
    );
}
