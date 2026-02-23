//! CLI 启动性能基准测试

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn get_cli_binary() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .join("target/release/workflow")
}

fn bench_cli_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_startup");
    group.sample_size(30);

    group.bench_function("help", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary().as_os_str())
                .arg("--help")
                .output()
                .expect("Failed to execute workflow --help");
            black_box(output);
        });
    });

    group.bench_function("version", |b| {
        b.iter(|| {
            let output = Command::new(get_cli_binary().as_os_str())
                .arg("--version")
                .output()
                .expect("Failed to execute workflow --version");
            black_box(output);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_cli_startup);
criterion_main!(benches);
