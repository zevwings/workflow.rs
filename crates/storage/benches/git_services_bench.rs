//! Git 服务性能基准测试
//!
//! 使用 Criterion 进行性能基准测试，评估各个 Git 服务的性能表现。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use git2::IndexAddOption;
use storage::{git::services::*, testing::*};

/// 创建 CommitServiceImpl 的辅助函数
fn create_commit_service(ctx: GitContext) -> CommitServiceImpl {
    CommitServiceImpl::new(ctx, noop_hook_service())
}

// ============================================================
// Commit Service 基准测试
// ============================================================

/// 测试获取工作树状态的性能
fn bench_commit_get_working_tree_status(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit_get_working_tree_status");

    // 测试不同数量的更改文件
    for &(modified, untracked) in &[(10, 10), (50, 50), (100, 100)] {
        let (_tmp, ctx) = setup_repo_with_changes(modified, untracked);
        let service = create_commit_service(ctx);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}m_{}u", modified, untracked)),
            &(modified, untracked),
            |b, _| {
                b.iter(|| {
                    service.get_working_tree_status().unwrap();
                });
            },
        );
    }

    group.finish();
}

/// 测试获取提交信息的性能
fn bench_commit_get_commit_info(c: &mut Criterion) {
    let (_tmp, ctx) = setup_repo_with_commits(100);
    let service = create_commit_service(ctx);

    c.bench_function("commit_get_commit_info", |b| {
        b.iter(|| {
            service.get_commit_info(black_box("HEAD")).unwrap();
        });
    });
}

/// 测试创建提交的性能（不带 --all）
fn bench_commit_create_without_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit_create_without_all");

    for &file_count in &[10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &file_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        // 每次迭代都创建新的仓库
                        let (tmp, ctx) = setup_repo_with_changes(count, 0);
                        // 暂存所有文件
                        {
                            let repo = ctx.repository();
                            let mut index = repo.index().unwrap();
                            index.add_all(["."].iter(), IndexAddOption::DEFAULT, None).unwrap();
                            index.write().unwrap();
                        }
                        (tmp, ctx)
                    },
                    |(_tmp, ctx)| {
                        let service = create_commit_service(ctx);
                        service.commit(black_box("test commit"), false).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

/// 测试创建提交的性能（带 --all）
fn bench_commit_create_with_all(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit_create_with_all");
    group.sample_size(10); // 减少样本数量，因为这个操作较慢

    for &file_count in &[10, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(file_count),
            &file_count,
            |b, &count| {
                b.iter_batched(
                    || {
                        // 每次迭代都创建新的仓库
                        setup_repo_with_changes(count, count)
                    },
                    |(_tmp, ctx)| {
                        let service = create_commit_service(ctx);
                        service.commit(black_box("test commit"), true).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================
// Branch Service 基准测试
// ============================================================

/// 测试列出分支的性能
fn bench_branch_list_branches(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_list_branches");

    for &branch_count in &[10, 50, 100] {
        let (_tmp, ctx) = setup_repo_with_branches(branch_count);
        let service = BranchServiceImpl::new(ctx);

        group.bench_with_input(
            BenchmarkId::from_parameter(branch_count),
            &branch_count,
            |b, _| {
                b.iter(|| {
                    service.list_branches(false, false).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// 测试创建分支的性能
fn bench_branch_create(c: &mut Criterion) {
    c.bench_function("branch_create", |b| {
        b.iter_batched(
            || {
                // 每次都创建新仓库
                setup_repo_with_file()
            },
            |(_tmp, ctx)| {
                let service = BranchServiceImpl::new(ctx);
                service.create_branch(black_box("new-branch")).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// 测试切换分支的性能
fn bench_branch_checkout(c: &mut Criterion) {
    c.bench_function("branch_checkout", |b| {
        b.iter_batched(
            || {
                let (tmp, ctx) = setup_repo_with_file();
                // 创建一个分支
                {
                    let repo = ctx.repository();
                    let head = repo.head().unwrap();
                    let commit = head.peel_to_commit().unwrap();
                    repo.branch("test-branch", &commit, false).unwrap();
                }
                (tmp, ctx)
            },
            |(_tmp, ctx)| {
                let service = BranchServiceImpl::new(ctx);
                service.checkout_branch(black_box("test-branch")).unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// 测试检查分支是否存在的性能
fn bench_branch_has_branch(c: &mut Criterion) {
    let (_tmp, ctx) = setup_repo_with_branches(50);
    let service = BranchServiceImpl::new(ctx);

    c.bench_function("branch_has_branch", |b| {
        b.iter(|| {
            service.has_branch(black_box("branch_25")).unwrap();
        });
    });
}

// ============================================================
// Blame Service 基准测试
// ============================================================

/// 测试获取文件 blame 信息的性能
fn bench_blame_get_file_blame(c: &mut Criterion) {
    let mut group = c.benchmark_group("blame_get_file_blame");
    group.sample_size(10); // Blame 操作较慢

    for &line_count in &[100, 500, 1000] {
        let (_tmp, ctx) = setup_repo_with_large_file(line_count);
        let service = BlameServiceImpl::new(ctx);

        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &line_count,
            |b, _| {
                b.iter(|| {
                    service.get_file_blame(black_box("large_file.txt"), None).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// 测试获取文件内容的性能
fn bench_blame_get_file_content(c: &mut Criterion) {
    let mut group = c.benchmark_group("blame_get_file_content");

    for &line_count in &[100, 1000, 5000] {
        let (_tmp, ctx) = setup_repo_with_large_file(line_count);
        let service = BlameServiceImpl::new(ctx);

        group.bench_with_input(
            BenchmarkId::from_parameter(line_count),
            &line_count,
            |b, _| {
                b.iter(|| {
                    service.get_file_content(black_box("large_file.txt"), None).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// 测试获取文件范围 blame 信息的性能
fn bench_blame_get_file_blame_range(c: &mut Criterion) {
    let (_tmp, ctx) = setup_repo_with_large_file(1000);
    let service = BlameServiceImpl::new(ctx);

    c.bench_function("blame_get_file_blame_range", |b| {
        b.iter(|| {
            service
                .get_file_blame_range(black_box("large_file.txt"), 100, 200, None)
                .unwrap();
        });
    });
}

// ============================================================
// Tag Service 基准测试
// ============================================================

/// 测试创建 tag 的性能
fn bench_tag_create(c: &mut Criterion) {
    use domain::TagCreateScope;

    c.bench_function("tag_create", |b| {
        b.iter_batched(
            setup_repo_with_file,
            |(_tmp, ctx)| {
                let service = TagServiceImpl::new(ctx);
                service
                    .create_tag(
                        black_box("v1.0.0"),
                        None,
                        None,
                        TagCreateScope::Local,
                        false,
                    )
                    .unwrap();
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// 测试列出 tag 的性能
fn bench_tag_list_tags(c: &mut Criterion) {
    // 创建包含多个 tags 的仓库
    let (_tmp, ctx) = {
        let (tmp, ctx) = setup_repo_with_file();
        {
            let repo = ctx.repository();
            let obj = repo.revparse_single("HEAD").unwrap();
            // 创建 50 个 tags
            for i in 0..50 {
                let tag_name = format!("v{}.0.0", i);
                repo.tag_lightweight(&tag_name, &obj, false).unwrap();
            }
        }
        (tmp, ctx)
    };

    let service = TagServiceImpl::new(ctx);

    c.bench_function("tag_list_tags", |b| {
        b.iter(|| {
            service.list_tags(false).unwrap();
        });
    });
}

/// 测试检查 tag 是否存在的性能
fn bench_tag_has_tag(c: &mut Criterion) {
    let (_tmp, ctx) = {
        let (tmp, ctx) = setup_repo_with_file();
        {
            let repo = ctx.repository();
            let obj = repo.revparse_single("HEAD").unwrap();
            repo.tag_lightweight("v1.0.0", &obj, false).unwrap();
        }
        (tmp, ctx)
    };

    let service = TagServiceImpl::new(ctx);

    c.bench_function("tag_has_tag", |b| {
        b.iter(|| {
            service.has_tag(black_box("v1.0.0")).unwrap();
        });
    });
}

// ============================================================
// Context 基准测试
// ============================================================

/// 测试仓库上下文并发访问性能
fn bench_context_concurrent_access(c: &mut Criterion) {
    use std::{sync::Arc, thread};

    let (_tmp, ctx) = setup_repo_with_file();
    let ctx: Arc<GitContext> = Arc::new(ctx);

    c.bench_function("context_concurrent_access", |b| {
        b.iter(|| {
            let mut handles = vec![];

            // 创建 10 个线程并发访问
            for _ in 0..10 {
                let ctx_clone = Arc::clone(&ctx);
                let handle = thread::spawn(move || {
                    let service = create_commit_service((*ctx_clone).clone());
                    service.get_commit_info("HEAD").unwrap();
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });
}

// ============================================================
// Criterion 配置
// ============================================================

criterion_group!(
    commit_benches,
    bench_commit_get_working_tree_status,
    bench_commit_get_commit_info,
    bench_commit_create_without_all,
    bench_commit_create_with_all,
);

criterion_group!(
    branch_benches,
    bench_branch_list_branches,
    bench_branch_create,
    bench_branch_checkout,
    bench_branch_has_branch,
);

criterion_group!(
    blame_benches,
    bench_blame_get_file_blame,
    bench_blame_get_file_content,
    bench_blame_get_file_blame_range,
);

criterion_group!(
    tag_benches,
    bench_tag_create,
    bench_tag_list_tags,
    bench_tag_has_tag,
);

criterion_group!(context_benches, bench_context_concurrent_access,);

criterion_main!(
    commit_benches,
    branch_benches,
    blame_benches,
    tag_benches,
    context_benches,
);
