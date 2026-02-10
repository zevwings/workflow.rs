# 测试架构改进 - Rust 测试组织原则补充说明

## 📚 为什么不把测试工具放在 `tests/` 目录？

### 问题背景

在很多编程语言中（如 Python, JavaScript），测试代码通常放在 `tests/` 目录，可以在测试之间共享工具函数。但 **Rust 的模块系统不允许这样做**。

### Rust 的 `tests/` 目录特性

```
tests/
├── integration_test.rs      # 编译为独立的可执行文件
├── another_test.rs           # 编译为另一个独立的可执行文件
└── common/                   # ❌ 不会被编译，除非显式导入
    └── mod.rs
```

**关键点**：
1. `tests/` 目录中的每个 `.rs` 文件都会被编译为**独立的二进制可执行文件**
2. 这些文件**不能互相导入**
3. `tests/` 目录中的代码**不能被 `src/` 导入**
4. `tests/` 只能访问 crate 的**公共接口**

### 实际问题演示

#### ❌ 错误方案：将工具放在 `tests/common/`

```
项目结构：
tests/
├── common/
│   └── test_data_factory.rs   # 测试数据工厂
├── integration_test.rs
└── another_test.rs
```

**问题 1：单元测试无法使用**
```rust
// crates/http/src/client.rs

#[cfg(test)]
mod tests {
    // ❌ 编译错误！src/ 不能导入 tests/
    use tests::common::TestDataFactory;

    #[test]
    fn test_something() {
        let data = TestDataFactory::github_pr().build();
    }
}
```

**问题 2：其他 crate 无法使用**
```rust
// crates/services/src/github_service.rs

#[cfg(test)]
mod tests {
    // ❌ 编译错误！不能跨 crate 访问 tests/
    use workflow::tests::common::TestDataFactory;
}
```

**问题 3：基准测试无法使用**
```rust
// benches/http_bench.rs

// ❌ 编译错误！benches/ 不能导入 tests/
use tests::common::TestDataFactory;
```

### ✅ 正确方案：使用 `src/testing/` + feature flag

```
项目结构：
crates/http/
├── src/
│   ├── lib.rs
│   ├── client.rs
│   └── testing/              # ✅ 测试工具作为 crate 的一部分
│       ├── mod.rs
│       ├── data_factory.rs
│       └── mock_server.rs
└── Cargo.toml

tests/
├── fixtures/                  # ✅ 只放数据文件
│   └── github_pr.json
└── integration_test.rs
```

**配置文件**：
```toml
# crates/http/Cargo.toml

[features]
default = []
testing = ["dep:mockito", "dep:tempfile"]

[dependencies]
mockito = { workspace = true, optional = true }
tempfile = { workspace = true, optional = true }
```

```rust
// crates/http/src/lib.rs

pub mod client;

// 只在测试或 testing feature 启用时编译
#[cfg(any(test, feature = "testing"))]
pub mod testing;
```

**优势 1：单元测试可以使用**
```rust
// crates/http/src/client.rs

#[cfg(test)]
mod tests {
    // ✅ 可以导入！同一个 crate 内
    use crate::testing::TestDataFactory;

    #[test]
    fn test_something() {
        let data = TestDataFactory::github_pr().build();
    }
}
```

**优势 2：其他 crate 可以使用**
```rust
// crates/services/Cargo.toml
[dev-dependencies]
http = { workspace = true, features = ["testing"] }

// crates/services/src/github_service.rs
#[cfg(test)]
mod tests {
    // ✅ 可以导入！通过 features = ["testing"]
    use http::testing::TestDataFactory;

    #[test]
    fn test_something() {
        let data = TestDataFactory::github_pr().build();
    }
}
```

**优势 3：基准测试可以使用**
```rust
// benches/http_bench.rs

// ✅ 可以导入！
use http::testing::TestDataFactory;

fn bench_something(c: &mut Criterion) {
    let data = TestDataFactory::github_pr().build();
    // ...
}
```

**优势 4：生产代码不包含测试工具**
```bash
# 普通构建（不包含 testing feature）
$ cargo build --release
# testing 模块不会被编译进去

# 测试构建（自动启用 #[cfg(test)]）
$ cargo test
# testing 模块会被编译

# 其他 crate 显式启用 testing feature
$ cargo test -p services --features testing
```

---

## 🎯 项目现有的良好实践

你的项目在 `storage` crate 中已经正确使用了这种模式：

### 现有代码分析

```rust
// crates/storage/Cargo.toml
[features]
testing = ["dep:tempfile"]

[dependencies]
tempfile = { workspace = true, optional = true }
```

```rust
// crates/storage/src/git/mod.rs
#[cfg(any(test, feature = "testing"))]
pub mod testing;  // ✅ 测试工具模块
```

```rust
// crates/storage/src/git/testing.rs
//! 测试辅助模块
//!
//! 提供 Git 服务测试的通用辅助函数和性能监控工具。

pub fn setup_repo() -> (TempDir, GitContext) { }
pub fn setup_repo_with_file() -> (TempDir, GitContext) { }
// ... 更多测试辅助函数
```

```rust
// crates/storage/benches/git_services_bench.rs
use storage::git::testing::*;  // ✅ 基准测试可以使用

fn bench_something(c: &mut Criterion) {
    let (_tmp, ctx) = setup_repo_with_file();
    // ...
}
```

**这正是我们推荐的方案！**

---

## 📋 完整对比表

| 特性 | `tests/common/` 方案 | `src/testing/` + feature 方案 |
|------|---------------------|------------------------------|
| 单元测试能用 | ❌ 不能 | ✅ 可以 |
| 集成测试能用 | ⚠️ 可以（需要特殊处理） | ✅ 可以 |
| 其他 crate 能用 | ❌ 不能 | ✅ 可以（通过 feature） |
| 基准测试能用 | ❌ 不能 | ✅ 可以 |
| 生产代码包含 | N/A | ❌ 不包含（通过 feature 排除） |
| 符合 Rust 惯例 | ❌ 不符合 | ✅ 符合 |
| 依赖方向正确 | ❌ 违背（src 不应该依赖 tests） | ✅ 正确 |

---

## 🔧 迁移指南

如果你已经有代码放在 `tests/common/`，应该如何迁移？

### 步骤 1：识别代码类型

```
tests/common/
├── test_data_factory.rs      # → 应该移到 src/testing/
├── mock_server.rs             # → 应该移到 src/testing/
└── fixtures.json              # → 保留在 tests/fixtures/
```

**规则**：
- 如果是**可执行代码**（函数、结构体） → 移到 `src/testing/`
- 如果是**数据文件**（JSON, YAML） → 保留在 `tests/fixtures/`

### 步骤 2：移动文件

```bash
# 创建测试工具目录
mkdir -p crates/http/src/testing

# 移动工具代码
mv tests/common/test_data_factory.rs crates/http/src/testing/data_factory.rs
mv tests/common/mock_server.rs crates/http/src/testing/mock_server.rs

# 数据文件保留
# tests/fixtures/ 保持不变
```

### 步骤 3：更新 Cargo.toml

```toml
# crates/http/Cargo.toml

[features]
testing = ["dep:mockito", "dep:tempfile"]

[dependencies]
mockito = { workspace = true, optional = true }
tempfile = { workspace = true, optional = true }
```

### 步骤 4：更新导出

```rust
// crates/http/src/lib.rs

#[cfg(any(test, feature = "testing"))]
pub mod testing;
```

```rust
// crates/http/src/testing/mod.rs

pub mod data_factory;
pub mod mock_server;

pub use data_factory::TestDataFactory;
pub use mock_server::MockServerManager;
```

### 步骤 5：更新引用

```rust
// 之前
use tests::common::TestDataFactory;  // ❌

// 之后
use http::testing::TestDataFactory;   // ✅
```

---

## 🤔 常见问题

### Q1: 为什么不创建单独的测试工具 crate（如 `test-utils`）？

**A**: 可以，但不推荐，因为：
1. 增加了项目复杂度（多一个 crate）
2. 测试工具通常与业务代码紧密耦合
3. 使用 feature flag 已经足够灵活

**何时使用独立 crate**：
- 测试工具非常复杂（>1000 行）
- 多个项目共享测试工具
- 测试工具需要独立版本管理

### Q2: `#[cfg(test)]` 和 `feature = "testing"` 有什么区别？

**A**:
- `#[cfg(test)]`: Rust 自动启用，运行 `cargo test` 时
- `feature = "testing"`: 手动启用，通过 `features = ["testing"]`

**使用场景**：
```rust
// 只在本 crate 的测试中使用
#[cfg(test)]
mod tests { }

// 在其他 crate 的测试中也要使用
#[cfg(any(test, feature = "testing"))]
pub mod testing { }
```

### Q3: 集成测试如何共享代码？

**A**: 使用特殊的模块结构：

```
tests/
├── common/
│   └── mod.rs           # 不是 lib.rs，不会被独立编译
├── test1.rs
└── test2.rs
```

```rust
// tests/common/mod.rs
pub fn setup() { }

// tests/test1.rs
mod common;  // 显式导入

#[test]
fn test1() {
    common::setup();
}

// tests/test2.rs
mod common;  // 显式导入

#[test]
fn test2() {
    common::setup();
}
```

**但这只适用于集成测试内部共享，不能被单元测试或其他 crate 使用！**

---

## 📖 参考资料

- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/interoperability.html#cargotoml-includes-custom-build-script-c-build)
- [Cargo Book - Features](https://doc.rust-lang.org/cargo/reference/features.html)

---

## ✅ 总结

**核心原则**：
1. ✅ 测试工具放在 `src/testing/`，使用 feature flag
2. ✅ 测试数据文件放在 `tests/fixtures/`
3. ✅ 单元测试放在源文件的 `#[cfg(test)] mod tests`
4. ✅ 集成测试放在 `tests/*.rs`
5. ❌ 不要把可复用的测试工具代码放在 `tests/common/`

**你的项目已经在正确的道路上**（`storage` crate 的 `testing` 模块）。
测试架构改进方案将扩展这个模式到其他 crate（`http`, `services` 等）。

---

**文档版本**: 1.0
**创建日期**: 2024-02-10
**相关文档**: [测试架构改进主文档](./06-test-architecture-improvement.md)
