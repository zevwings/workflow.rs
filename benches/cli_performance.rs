//! CLI 性能基准测试
//!
//! 测试 CLI 命令解析和执行的性能。

use clap::Parser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow::cli::Cli;

fn bench_cli_parse_simple(c: &mut Criterion) {
    c.bench_function("cli_parse_version", |b| {
        b.iter(|| {
            let args = vec!["workflow".to_string(), "version".to_string()];
            black_box(Cli::parse_from(args));
        });
    });
}

fn bench_cli_parse_complex(c: &mut Criterion) {
    c.bench_function("cli_parse_branch_create", |b| {
        b.iter(|| {
            let args = vec![
                "workflow".to_string(),
                "branch".to_string(),
                "create".to_string(),
                "PROJ-123".to_string(),
            ];
            black_box(Cli::parse_from(args));
        });
    });
}

fn bench_cli_parse_with_alias(c: &mut Criterion) {
    c.bench_function("cli_parse_with_alias_expansion", |b| {
        b.iter(|| {
            // 模拟别名展开后的命令解析（别名 "br" 展开为 "branch"）
            // 注意：这里测试的是解析已展开命令的性能，而不是别名展开本身的性能
            let args = vec![
                "workflow".to_string(),
                "branch".to_string(),
                "create".to_string(),
            ];
            black_box(Cli::parse_from(args));
        });
    });
}

criterion_group!(
    benches,
    bench_cli_parse_simple,
    bench_cli_parse_complex,
    bench_cli_parse_with_alias
);
criterion_main!(benches);
