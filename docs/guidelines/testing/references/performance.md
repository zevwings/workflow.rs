# 性能测试指南

> 本文档介绍性能测试和基准测试的方法。

---

## 性能测试要求

**单元测试**：
- 单个测试 < 100ms
- 模块测试套件 < 1s

**集成测试**：
- 单个测试 < 1s  
- 模块测试套件 < 10s

## 基准测试（Benchmark）

### 使用 Criterion

```bash
# 运行所有基准测试
make bench

# 运行特定基准测试
make bench-cli
make bench-core
make bench-network
```

### 创建基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_example(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| {
            black_box(some_operation());
        });
    });
}

criterion_group!(benches, bench_example);
criterion_main!(benches);
```

更多详细配置和使用方法，请参考实际测试代码。

---

**最后更新**: 2025-12-25
