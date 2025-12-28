# 性能优化规范

> 本文档定义了 Workflow CLI 项目的性能优化规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [性能测试要求](#-性能测试要求)
- [内存使用优化规则](#-内存使用优化规则)
- [异步操作使用规则](#-异步操作使用规则)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了性能优化规范，包括性能测试要求、内存使用优化规则和异步操作使用规则。

### 核心原则

- **关键路径测试**：关键路径必须进行性能测试
- **内存优化**：避免不必要的内存分配
- **异步操作**：网络请求应使用异步操作
- **流式处理**：大文件处理应使用流式处理

### 使用场景

- 编写性能关键代码时参考
- 性能优化时使用
- 性能测试时参考

---

## 性能测试要求

### 关键路径性能测试

1. **识别关键路径**：
   - 识别应用中的性能关键路径（如频繁调用的函数、主循环、数据处理流程）
   - 识别用户感知明显的操作（如命令执行、文件处理、网络请求）

2. **性能测试工具**：
   - 使用 `criterion` 进行基准测试（推荐）
   - 使用 `cargo bench` 运行基准测试
   - 使用 `cargo test --bench` 运行基准测试

3. **性能测试实现**：

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 测试代码
            my_function(black_box(input))
        })
    });
}

criterion_group!(benches, benchmark_function);
criterion_main!(benches);
```

4. **性能测试要求**：
   - 关键路径必须进行性能测试
   - 性能测试应作为 CI/CD 的一部分
   - 性能测试结果应记录和跟踪

### 性能回归测试要求

1. **回归测试时机**：
   - 性能关键代码变更后必须运行性能测试
   - 重构性能关键代码后必须运行性能测试
   - 添加新依赖后评估性能影响

2. **性能阈值**：
   - 建立性能基准线
   - 设置性能回归阈值（如性能下降不超过 5%）
   - 如果性能下降超过阈值，需要优化或回滚

3. **性能监控**：
   - 使用 `criterion` 的统计功能跟踪性能趋势
   - 记录性能测试历史数据
   - 识别性能回归趋势

### 性能基准测试要求

1. **建立基准线**：
   - 为关键路径建立性能基准线
   - 记录基准测试结果（平均值、中位数、P95、P99）
   - 在文档中记录性能基准

2. **定期运行基准测试**：
   - 定期运行基准测试（如每次发布前）
   - 记录性能趋势
   - 识别性能退化

3. **基准测试工具**：

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench my_benchmark

# 显示详细输出
cargo bench -- --nocapture
```

4. **基准测试最佳实践**：
   - 使用 `black_box` 防止编译器过度优化
   - 多次运行取平均值
   - 在稳定的环境中运行（避免 CPU 频率变化影响）

---

## 内存使用优化规则

### 避免不必要的内存分配

1. **优先使用栈分配**：
   - 优先使用栈分配，避免堆分配
   - 小数据结构应使用栈分配
   - 大数据结构才使用堆分配

2. **使用引用而非拥有所有权**：
   - 使用 `&str` 而不是 `String`（如果不需要拥有所有权）
   - 使用 `&[T]` 而不是 `Vec<T>`（如果不需要拥有所有权）
   - 使用 `Cow<'_, str>` 避免不必要的克隆

```rust
// ✅ 好的做法：使用引用
fn process_data(data: &str) {
    // 不需要拥有所有权，使用引用
}

// ❌ 不好的做法：不必要的所有权转移
fn process_data(data: String) {
    // 如果不需要拥有所有权，使用 &str
}
```

3. **使用智能指针减少复制**：
   - 使用 `Box<T>` 减少大结构体的复制
   - 使用 `Rc<T>` 或 `Arc<T>` 共享数据
   - 使用 `Cow<'_, T>` 延迟克隆

```rust
// ✅ 好的做法：使用 Box 避免大结构体复制
struct LargeStruct {
    data: [u8; 1024],
}

fn process_large(large: Box<LargeStruct>) {
    // Box 只复制指针，不复制数据
}

// ✅ 好的做法：使用 Cow 延迟克隆
use std::borrow::Cow;

fn process_string(s: Cow<'_, str>) {
    // 如果不需要修改，不克隆；如果需要修改，才克隆
}
```

### 预分配内存

1. **预分配 Vec**：
   - 使用 `Vec::with_capacity` 预分配内存
   - 如果知道大小，预分配可以避免多次重新分配

```rust
// ✅ 好的做法：预分配内存
let mut vec = Vec::with_capacity(1000);
for i in 0..1000 {
    vec.push(i);
}

// ❌ 不好的做法：多次重新分配
let mut vec = Vec::new();
for i in 0..1000 {
    vec.push(i);  // 可能多次重新分配
}
```

2. **预分配 String**：
   - 使用 `String::with_capacity` 预分配字符串
   - 如果知道字符串长度，预分配可以提高性能

```rust
// ✅ 好的做法：预分配字符串
let mut s = String::with_capacity(100);
for i in 0..100 {
    s.push_str(&i.to_string());
}
```

3. **预分配 HashMap**：
   - 使用 `HashMap::with_capacity` 预分配哈希表
   - 如果知道元素数量，预分配可以避免多次重新哈希

```rust
use std::collections::HashMap;

// ✅ 好的做法：预分配哈希表
let mut map = HashMap::with_capacity(100);
for i in 0..100 {
    map.insert(i, i * 2);
}
```

### 大文件处理

1. **使用流式处理**：
   - 大文件处理时使用 `BufReader`、`BufWriter`
   - 避免一次性将整个文件加载到内存

```rust
use std::fs::File;
use std::io::{BufReader, BufRead};

// ✅ 好的做法：流式处理
let file = File::open("large_file.txt")?;
let reader = BufReader::new(file);
for line in reader.lines() {
    process_line(&line?);
}

// ❌ 不好的做法：一次性加载整个文件
let content = fs::read_to_string("large_file.txt")?;  // 可能内存不足
```

---

## 异步操作使用规则

### 网络请求应使用异步操作

1. **使用异步 HTTP 客户端**：
   - 网络请求应使用异步操作（如 `reqwest` 的异步 API）
   - 避免阻塞主线程

```rust
// ✅ 好的做法：使用异步操作
use reqwest;

async fn fetch_data(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    response.text().await
}

// ❌ 不好的做法：阻塞操作
fn fetch_data(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)?;  // 阻塞主线程
    response.text()
}
```

2. **并发处理**：
   - 多个网络请求应并发处理
   - 使用 `futures::future::join_all` 或 `tokio::try_join!` 并发执行

```rust
use futures::future::join_all;

// ✅ 好的做法：并发处理
async fn fetch_multiple(urls: Vec<String>) -> Result<Vec<String>> {
    let futures = urls.into_iter().map(|url| fetch_data(&url));
    join_all(futures).await
        .into_iter()
        .collect::<Result<Vec<_>>>()
}
```

---

## 🔍 故障排除

### 问题 1：性能测试结果不稳定

**症状**：性能测试结果波动较大

**解决方案**：

1. 使用 `black_box` 防止编译器过度优化
2. 多次运行取平均值
3. 在稳定的环境中运行（避免 CPU 频率变化影响）

### 问题 2：内存使用过高

**症状**：程序内存使用过高

**解决方案**：

1. 检查是否有不必要的内存分配
2. 使用引用而非拥有所有权
3. 使用流式处理大文件

---

## 📚 相关文档

### 开发规范

- [代码风格规范](../code-style.md) - 代码风格规范
- [模块组织规范](../module-organization.md) - 模块组织规范

### 工具文档

- [criterion](https://docs.rs/criterion/) - 基准测试工具文档

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 关键路径已进行性能测试
- [ ] 避免不必要的内存分配
- [ ] 网络请求使用异步操作
- [ ] 大文件处理使用流式处理

---

**最后更新**: 2025-12-23

