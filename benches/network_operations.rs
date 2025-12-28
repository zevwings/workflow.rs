//! 网络操作性能基准测试
//!
//! 测试 HTTP 重试机制和网络操作的性能。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow::base::http::retry::HttpRetryConfig;

fn bench_retry_config_creation(c: &mut Criterion) {
    c.bench_function("retry_config_default", |b| {
        b.iter(|| {
            black_box(HttpRetryConfig::default());
        });
    });

    c.bench_function("retry_config_new", |b| {
        b.iter(|| {
            black_box(HttpRetryConfig::new());
        });
    });
}

criterion_group!(benches, bench_retry_config_creation);
criterion_main!(benches);
