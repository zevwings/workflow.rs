# 跨平台测试方案

> 本文档定义跨平台测试的策略和方法，确保代码在所有支持的平台上都能正确运行。

---

## 📋 目录

- [概述](#-概述)
- [支持的平台](#-支持的平台)
- [本地测试](#-本地测试)
- [CI/CD 跨平台测试](#-cicd-跨平台测试)

---

## 🎯 概述

Workflow CLI 项目支持多个平台和架构，需要确保代码在所有目标平台上都能正确运行。

### 测试目标

- ✅ **功能一致性**：确保所有平台上的功能行为一致
- ✅ **构建验证**：验证代码可以在所有目标平台上正确构建
- ✅ **平台兼容性**：发现和修复平台特定的问题

---

## 🖥️ 支持的平台

| 平台 | Target | 说明 |
|------|--------|------|
| macOS Intel | `x86_64-apple-darwin` | 原生支持 |
| macOS Apple Silicon | `aarch64-apple-darwin` | 原生支持 |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | glibc 动态链接 |
| Linux x86_64 (静态) | `x86_64-unknown-linux-musl` | musl 静态链接 |
| Windows x86_64 | `x86_64-pc-windows-msvc` | MSVC 工具链 |
| Windows x86_64 (MinGW) | `x86_64-pc-windows-gnu` | MinGW 工具链 |

**注意**：项目支持原生平台构建。musl 版本提供静态链接，可在任何 Linux 发行版上运行。

---

## 🔧 本地测试

### 前置要求

#### 安装 Rust toolchain

确保已安装 Rust 和所需的 target：

```bash
# macOS Intel
rustup target add x86_64-apple-darwin

# macOS Apple Silicon
rustup target add aarch64-apple-darwin

# Linux x86_64
rustup target add x86_64-unknown-linux-gnu

# Linux x86_64 (静态链接)
rustup target add x86_64-unknown-linux-musl

# Windows x86_64 (MSVC)
rustup target add x86_64-pc-windows-msvc

# Windows x86_64 (MinGW)
rustup target add x86_64-pc-windows-gnu
```

### 构建和测试命令

#### 构建可执行文件

```bash
# 在当前平台构建
cargo build --release

# 指定目标平台（仅限原生平台）
cargo build --release --target x86_64-unknown-linux-gnu
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行文档测试
cargo test --doc

# 运行集成测试
cargo test --test integration_test
```

---

## 🚀 CI/CD 跨平台测试

项目在 CI/CD 中使用矩阵策略并行运行所有原生平台的测试：

- **单元测试**：在所有平台上运行
- **文档测试**：在所有平台上运行
- **构建验证**：验证所有目标平台可以成功构建

详细配置请参考 [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml)。

### 支持的 CI 平台

- ✅ `ubuntu-latest` - Linux x86_64
- ✅ `macos-latest` - macOS (Intel 和 Apple Silicon)
- ✅ `windows-latest` - Windows x86_64

---

## 📚 相关文档

- [Windows 测试指南](./windows.md) - Windows 环境下的测试详细指南
- [Parallels Desktop Windows 测试指南](./parallels-windows.md) - 在 macOS 上使用 Parallels Desktop 测试 Windows 版本
- [测试组织规范](./organization.md) - 测试组织结构和命名约定
- [测试编写规范](./writing.md) - 测试编写的具体规范
- [CI/CD 工作流](../ci-workflow.md) - CI/CD 工作流说明

---

**最后更新**: 2025-01-28
