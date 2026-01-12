# 测试规范

> Workflow CLI 项目的测试规范和最佳实践

---

## 📋 目录

- [测试类型](#-测试类型)
- [测试组织结构](#-测试组织结构)
- [测试编写规范](#-测试编写规范)
- [测试命令参考](#-测试命令参考)
- [测试工具](#-测试工具)

---

## 🎯 测试类型

### 1. 单元测试 (Unit Tests)

- **位置**：与源代码在同一文件中
- **特点**：测试私有函数，快速执行
- **组织方式**：使用 `#[cfg(test)]` 模块

```rust
// src/lib/base/http.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        // 测试私有函数
    }
}
```

### 2. 集成测试 (Integration Tests)

- **位置**：`tests/` 目录
- **特点**：测试公共 API，独立编译
- **组织方式**：使用目录结构组织

---

## 📁 测试组织结构

### 测试目录结构

```
tests/
├── base/              # Base 模块测试
├── cli/                # CLI 命令层测试
├── completion/         # Completion 模块测试
├── git/                # Git 模块测试
├── jira/               # Jira 模块测试
├── pr/                 # PR 模块测试
├── common/             # 共享测试工具
├── fixtures/           # 测试数据
└── integration/        # 集成测试
```

### 结构说明

- **模块对应**：测试目录结构与源代码模块结构对应
- **每个目录**：包含 `mod.rs` 文件用于模块声明
- **共享工具**：`common/` 目录存放共享的测试工具函数
- **测试数据**：`fixtures/` 目录存放测试数据文件

---

## ✅ 测试编写规范

### 测试结构（AAA 模式）

每个测试应包含：

- **Arrange**：准备测试数据和环境
- **Act**：执行被测试的功能
- **Assert**：验证结果

```rust
#[test]
fn test_parse_ticket_id() {
    // Arrange
    let input = "PROJ-123";

    // Act
    let result = parse_ticket_id(input);

    // Assert
    assert_eq!(result, Some("PROJ-123"));
}
```

### 错误处理测试

为错误情况编写测试：

```rust
#[test]
fn test_parse_ticket_id_invalid() {
    assert_eq!(parse_ticket_id("invalid"), None);
    assert_eq!(parse_ticket_id(""), None);
}
```

### 边界条件测试

测试边界条件和极端情况：

```rust
#[test]
fn test_parse_ticket_id_boundary() {
    // 最小长度
    assert_eq!(parse_ticket_id("A-1"), Some("A-1"));
    // 最大长度
    assert_eq!(parse_ticket_id("VERY-LONG-PROJECT-NAME-123"), Some("VERY-LONG-PROJECT-NAME-123"));
}
```

### 测试命名

- 测试函数名：`test_*`（如 `test_parse_ticket_id`）
- 测试用例命名清晰，描述测试内容

### 测试独立性

- 每个测试应该独立运行，不依赖其他测试
- 避免共享可变状态
- 使用 `#[serial_test::serial]` 标记需要串行执行的测试

---

## 🚀 测试命令参考

### 运行所有测试

```bash
# 运行所有测试
cargo test

# 运行测试并显示输出（即使通过）
cargo test -- --nocapture

# 运行测试并显示所有输出
cargo test -- --show-output
```

### 运行特定测试

```bash
# 运行特定测试函数
cargo test test_function_name

# 运行匹配模式的测试
cargo test test_parse

# 运行特定模块的测试
cargo test base::

# 运行集成测试
cargo test --test integration_test
```

### 测试过滤

```bash
# 运行包含特定字符串的测试
cargo test -- --test-threads=1 parse

# 排除特定测试
cargo test -- --skip test_slow
```

### 测试覆盖率

使用 `cargo-tarpaulin` 检查测试覆盖率：

```bash
# 安装
cargo install cargo-tarpaulin

# 运行覆盖率检查
cargo tarpaulin

# 生成 HTML 报告
cargo tarpaulin --out Html

# 指定覆盖率目标
cargo tarpaulin --out Html --fail-under 80
```

---

## 🛠️ 测试工具

### pretty_assertions

提供更清晰的断言输出，显示彩色 diff。

```rust
use pretty_assertions::assert_eq;

#[test]
fn test_example() {
    let actual = "Hello";
    let expected = "World";
    assert_eq!(actual, expected);  // 会显示清晰的彩色 diff
}
```

### rstest

支持参数化测试和 fixtures，减少代码重复。

**参数化测试**：

```rust
use rstest::rstest;

#[rstest]
#[case("input1", "output1")]
#[case("input2", "output2")]
fn test_multiple_cases(#[case] input: &str, #[case] expected: &str) {
    let result = process(input);
    assert_eq!(result, expected);
}
```

### insta

快照测试，用于测试输出格式。

```rust
use insta::assert_json_snapshot;

#[test]
fn test_output() {
    let output = generate_output();
    assert_json_snapshot!(output);
}
```

### mockito

HTTP API Mock 测试。

```rust
use mockito::mock;

#[test]
fn test_api_call() {
    let _m = mock("GET", "/api/endpoint")
        .with_status(200)
        .with_body(r#"{"key": "value"}"#)
        .create();

    // 测试代码
}
```

---

## 📚 相关文档

- [开发规范](./development.md) - 代码风格、错误处理、命名、模块组织等开发规范
- [架构设计](./architecture.md) - 项目整体架构设计

---

**最后更新**: 2025-01-27
