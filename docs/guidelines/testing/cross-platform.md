# 跨平台测试方案

> 本文档定义跨平台测试的策略和方法，确保代码在所有支持的平台上都能正确运行。

---

## 📋 目录

- [概述](#-概述)
- [支持的平台](#-支持的平台)
- [本地交叉编译测试](#-本地交叉编译测试)
- [CI/CD 跨平台测试](#-cicd-跨平台测试)
- [常见问题](#-常见问题)

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
| Windows x86_64 (GNU) | `x86_64-pc-windows-gnu` | MinGW 工具链 |

---

## 🔧 本地交叉编译测试

### 前置要求

#### 1. 安装 Rust target

```bash
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-musl
```

#### 2. 安装交叉编译工具链

**macOS 上交叉编译到 Windows：**

```bash
brew install mingw-w64
```

**macOS 上交叉编译到 Linux：**

**方案 1：使用 cross 工具（推荐，需要 Docker）**

```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --target x86_64-unknown-linux-musl
```

**方案 2：使用 musl-cross（不需要 Docker）**

```bash
brew install filosottile/musl-cross/musl-cross
echo 'export PATH="/usr/local/opt/musl-cross/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

**注意**：项目已配置 `git2` 使用 `vendored-openssl` feature，会自动编译 OpenSSL，无需手动配置。

### 交叉编译命令

#### 构建可执行文件

```bash
# Windows
cargo build --target x86_64-pc-windows-gnu

# Linux
cargo build --target x86_64-unknown-linux-musl
# 或使用 cross
cross build --target x86_64-unknown-linux-musl
```

#### 编译测试代码

```bash
# ⚠️ 注意：交叉编译的测试无法直接运行！
# 使用 --no-run 标志只编译不运行

# Windows
cargo test --target x86_64-pc-windows-gnu --no-run

# Linux
cargo test --target x86_64-unknown-linux-musl --no-run
# 或使用 cross
cross test --target x86_64-unknown-linux-musl --no-run
```

**重要说明**：
- ✅ `Finished test profile` 表示编译成功
- ❌ `cannot execute binary file` 是正常现象（无法在当前平台运行）
- ✅ 使用 `--no-run` 标志只编译不运行

#### 验证可执行文件

**Windows（使用 Wine）：**

```bash
# macOS: 安装 Wine
brew install --cask wine-stable

# 运行 Windows 可执行文件（抑制警告）
WINEDEBUG=-all wine target/x86_64-pc-windows-gnu/debug/workflow.exe --version
```

**Linux（使用 Docker）：**

```bash
docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/debug:/app -w /app alpine:latest ./workflow --version
```

### 配置文件

交叉编译配置位于 `.cargo/config.toml`，详细说明请参考 [`.cargo/config.toml`](../../../.cargo/config.toml)。

---

## 🚀 CI/CD 跨平台测试

项目在 CI/CD 中使用矩阵策略并行运行所有平台的测试：

- **单元测试**：在所有平台上运行
- **集成测试**：在所有平台上运行
- **文档测试**：在所有平台上运行
- **构建验证**：验证所有目标平台可以成功构建

详细配置请参考 [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml)。

---

## ❓ 常见问题

### Q: 为什么交叉编译的测试无法运行？

A: 交叉编译的二进制文件是为目标平台编译的，无法在当前平台直接运行。

**解决方案**：

1. **只编译不运行（推荐）**：
   ```bash
   cargo test --target x86_64-pc-windows-gnu --no-run
   ```

2. **使用 Wine/Docker 验证基本功能**：
   ```bash
   # Windows
   WINEDEBUG=-all wine target/x86_64-pc-windows-gnu/debug/workflow.exe --version

   # Linux
   docker run --rm -v $(pwd)/target/x86_64-unknown-linux-musl/debug:/app -w /app alpine:latest ./workflow --version
   ```

3. **在 CI/CD 中运行完整测试**（最佳方案）

### Q: 交叉编译到 Linux musl 时遇到 OpenSSL 错误怎么办？

A: 项目已配置 `git2` 使用 `vendored-openssl` feature，会自动编译 OpenSSL。如果仍有问题，推荐使用 `cross` 工具：

```bash
cross build --target x86_64-unknown-linux-musl
```

### Q: 如何选择交叉编译工具？

A:
- **使用 cross**：推荐，自动处理所有工具链配置（需要 Docker）
- **使用 musl-cross**：不需要 Docker，但需要手动配置 PATH

---

## 📚 相关文档

- [测试组织规范](./organization.md) - 测试组织结构和命名约定
- [测试编写规范](./writing.md) - 测试编写的具体规范
- [CI/CD 工作流](../ci-workflow.md) - CI/CD 工作流说明
- [`.cargo/config.toml`](../../../.cargo/config.toml) - 交叉编译配置

---

**最后更新**: 2025-01-28
