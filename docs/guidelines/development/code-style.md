# 代码风格规范

> 本文档定义了 Workflow CLI 项目的代码风格规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [代码格式化](#-代码格式化)
- [Lint 检查](#-lint-检查)
- [Rust 命名约定](#-rust-命名约定)
- [代码组织](#-代码组织)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了代码风格规范，包括代码格式化、Lint 检查、Rust 命名约定和代码组织规范。

### 核心原则

- **一致性**：所有代码必须遵循统一的风格规范
- **自动化**：使用工具自动检查和格式化代码
- **可读性**：代码风格应提高代码可读性

### 使用场景

- 编写新代码时参考
- 代码审查时检查
- 代码格式化时使用

### 快速参考

| 操作 | 命令 | 说明 |
|------|------|------|
| **格式化代码** | `cargo fmt` | 自动格式化代码 |
| **检查格式** | `cargo fmt --check` | 检查代码格式（CI/CD） |
| **Lint 检查** | `cargo clippy -- -D warnings` | 运行 Clippy 检查 |
| **Lint 检查（Makefile）** | `make lint` | 使用 Makefile 运行 Lint |

---

## 代码格式化

所有代码必须使用 `rustfmt` 进行格式化：

```bash
# 自动格式化代码
cargo fmt

# 检查代码格式（CI/CD 中使用）
cargo fmt --check
```

**规则**：
- 提交前必须运行 `cargo fmt`
- CI/CD 会检查代码格式，格式不正确会导致构建失败
- 使用默认的 `rustfmt` 配置（项目根目录的 `rustfmt.toml` 如果存在）

---

## Lint 检查

使用 `clippy` 进行代码质量检查：

```bash
# 运行 Clippy 检查
cargo clippy -- -D warnings

# 或使用 Makefile
make lint
```

**规则**：
- 所有警告必须修复（`-D warnings` 会将警告视为错误）
- 禁止使用 `#[allow(clippy::xxx)]` 除非有充分理由，并添加注释说明
- 定期运行 `cargo clippy` 检查代码质量

---

## Rust 命名约定

遵循 Rust 官方命名约定：

- **模块名**：`snake_case`（如 `jira_logs`、`pr_helpers`）
- **函数名**：`snake_case`（如 `download_logs`、`create_pr`）
- **变量名**：`snake_case`（如 `api_token`、`response_data`）
- **常量名**：`SCREAMING_SNAKE_CASE`（如 `MAX_RETRIES`、`DEFAULT_TIMEOUT`）
- **类型名**：`PascalCase`（如 `HttpClient`、`JiraTicket`）
- **Trait 名**：`PascalCase`（如 `PlatformProvider`、`ResponseParser`）
- **枚举变体**：`PascalCase`（如 `GitHub`、`Codeup`）

---

## 代码组织

### 导入顺序

1. 标准库导入
2. 第三方库导入
3. 项目内部导入

```rust
// 标准库
use std::path::PathBuf;
use std::fs;

// 第三方库
use color_eyre::Result;
use serde::Deserialize;

// 项目内部
use crate::base::http::HttpClient;
use crate::jira::client::JiraClient;
```

### 模块声明

- 使用 `mod.rs` 文件管理模块声明
- 按功能分组组织模块
- 使用 `pub use` 重新导出常用的公共 API

```rust
// src/lib/jira/mod.rs
mod client;
mod config;
mod ticket;

pub use client::JiraClient;
pub use ticket::JiraTicket;
```

---

## 🔍 故障排除

### 问题 1：代码格式检查失败

**症状**：运行 `cargo fmt --check` 时提示格式不正确

**解决方案**：

1. 运行 `cargo fmt` 自动格式化代码
2. 检查是否有自定义的 `rustfmt.toml` 配置
3. 确保使用最新版本的 `rustfmt`

### 问题 2：Clippy 警告过多

**症状**：运行 `cargo clippy` 时出现大量警告

**解决方案**：

1. 逐个修复警告（优先修复高优先级警告）
2. 对于确实需要忽略的警告，使用 `#[allow(clippy::xxx)]` 并添加注释说明原因
3. 定期运行 `cargo clippy` 保持代码质量

---

## 📚 相关文档

### 开发规范

- [错误处理规范](./error-handling.md) - 错误处理规范
- [命名规范](./naming.md) - 命名规范
- [模块组织规范](./module-organization.md) - 模块组织规范

### 检查工作流

- [提交前检查](./workflows/pre-commit.md) - 代码质量检查流程

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 代码已格式化（`cargo fmt`）
- [ ] 通过 Clippy 检查（`cargo clippy -- -D warnings`）
- [ ] 遵循 Rust 命名约定
- [ ] 导入顺序正确（标准库 → 第三方库 → 项目内部）
- [ ] 模块组织清晰

---

**最后更新**: 2025-12-23

