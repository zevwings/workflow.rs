//! 核心操作性能基准测试
//!
//! 测试核心业务逻辑的性能，包括字符串处理、分支操作、配置操作等。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow::base::util::string::mask_sensitive_value;
use workflow::branch::naming::BranchNaming;

fn bench_string_mask(c: &mut Criterion) {
    c.bench_function("string_mask_short", |b| {
        b.iter(|| {
            black_box(mask_sensitive_value("test@example.com"));
        });
    });

    c.bench_function("string_mask_long", |b| {
        b.iter(|| {
            black_box(mask_sensitive_value(
                "very-long-string-that-needs-masking@example.com",
            ));
        });
    });
}

fn bench_branch_slugify(c: &mut Criterion) {
    c.bench_function("branch_slugify_simple", |b| {
        b.iter(|| {
            black_box(BranchNaming::slugify("Simple Branch Name"));
        });
    });

    c.bench_function("branch_slugify_complex", |b| {
        b.iter(|| {
            black_box(BranchNaming::slugify(
                "Complex Branch Name with Special Characters!@#$%",
            ));
        });
    });

    c.bench_function("branch_slugify_long", |b| {
        b.iter(|| {
            black_box(BranchNaming::slugify(
                "Very Long Branch Name That Needs To Be Slugified Properly With Many Words",
            ));
        });
    });
}

fn bench_branch_sanitize(c: &mut Criterion) {
    c.bench_function("branch_sanitize_basic", |b| {
        b.iter(|| {
            black_box(BranchNaming::sanitize("feature/test-branch"));
        });
    });

    c.bench_function("branch_sanitize_complex", |b| {
        b.iter(|| {
            black_box(BranchNaming::sanitize(
                "feature/test--branch---with---multiple---dashes",
            ));
        });
    });
}

fn bench_config_load(c: &mut Criterion) {
    c.bench_function("config_load_settings", |b| {
        b.iter(|| {
            black_box(workflow::base::settings::Settings::get());
        });
    });
}

criterion_group!(
    benches,
    bench_string_mask,
    bench_branch_slugify,
    bench_branch_sanitize,
    bench_config_load
);
criterion_main!(benches);
