# System 系统交互模块架构文档

## 📋 概述

System 模块是 Workflow CLI 的基础设施模块之一，提供平台检测、浏览器操作和剪贴板操作相关的工具函数。该模块分为三个子模块：平台检测（`platform.rs`）、浏览器操作（`browser.rs`）和剪贴板操作（`clipboard.rs`），为整个项目提供统一的系统交互接口。

**模块统计：**
- 总代码行数：约 200+ 行
- 文件数量：4 个（`mod.rs`、`platform.rs`、`browser.rs`、`clipboard.rs`）
- 主要组件：3 个（`Platform`、`Browser`、`Clipboard`）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/system/
├── mod.rs          # 模块导出和公共 API (12行)
├── platform.rs     # 平台检测工具 (196行)
├── browser.rs      # 浏览器操作 (41行)
└── clipboard.rs    # 剪贴板操作 (78行)
```

### 依赖模块

- **`std::env`**：环境变量和常量（`env::consts::OS`、`env::consts::ARCH`）
- **`std::process::Command`**：执行系统命令（`ldd` 命令检测静态链接）
- **`open`**：打开浏览器（跨平台）
- **`clipboard`**：剪贴板操作（跨平台，条件编译）
- **`lib/base/fs/file`**：文件读取（`FileReader` 读取 `/etc/os-release`）

### 模块集成

System 模块被整个项目广泛使用：

- **生命周期管理**：
  - `Lifecycle` 模块使用 `Platform::detect()` 和 `Platform::release_identifier()` 检测平台
  - 用于匹配 GitHub Releases 中的资源文件

- **PR 模块**：
  - `PR` 命令使用 `Browser::open()` 在浏览器中打开 PR 链接
  - `PR` 命令使用 `Clipboard::copy()` 复制 PR URL 到剪贴板

- **其他模块**：
  - `Log` 命令使用 `Clipboard::copy()` 复制日志内容
  - `Jira` 命令使用 `Clipboard::copy()` 复制 Jira ticket 信息

---

## 🏗️ 架构设计

### 设计原则

1. **跨平台支持**：所有功能支持 macOS、Linux、Windows
2. **条件编译**：剪贴板功能在特定平台不可用时静默失败
3. **工具类设计**：使用静态方法或零大小结构体，无需实例化
4. **错误处理**：统一的错误处理，提供清晰的错误消息

### 核心组件

#### 1. Platform 结构体（平台检测）

**位置**：`platform.rs`

**职责**：封装操作系统和架构信息，提供平台检测和标识符生成功能

**字段**：
- `os: String` - 操作系统类型
- `arch: String` - 系统架构

**主要方法**：

##### `new(os: impl Into<String>, arch: impl Into<String>) -> Self`

创建新的平台实例。

##### `detect() -> Self`

检测当前运行平台，自动检测操作系统和架构信息。

**示例**：
```rust
use workflow::base::system::Platform;

let platform = Platform::detect();
println!("OS: {}, Arch: {}", platform.os(), platform.arch());
```

##### `os() -> &str`

获取操作系统类型（如 "macos", "linux", "windows"）。

##### `arch() -> &str`

获取系统架构（如 "x86_64", "aarch64"）。

##### `is_macos() -> bool`

检查是否为 macOS 平台。

##### `is_linux() -> bool`

检查是否为 Linux 平台。

##### `is_windows() -> bool`

检查是否为 Windows 平台。

##### `is_x86_64() -> bool`

检查是否为 x86_64 架构。

##### `is_aarch64() -> bool`

检查是否为 ARM64/aarch64 架构。

##### `release_identifier() -> Result<String>`

生成 GitHub Releases 格式的平台标识符。

**支持的平台格式**：
- macOS: `macOS-Intel`, `macOS-AppleSilicon`
- Linux: `Linux-x86_64`, `Linux-x86_64-static`, `Linux-ARM64`
- Windows: `Windows-x86_64`, `Windows-ARM64`

**检测逻辑**：
- 对于 Linux x86_64，自动检测是否需要静态链接版本：
  1. 检查是否是 Alpine Linux（通常使用 musl）
  2. 检测当前二进制是否静态链接（使用 `ldd` 命令）

**示例**：
```rust
use workflow::base::system::Platform;

let platform = Platform::detect();
let identifier = platform.release_identifier()?;
println!("Release identifier: {}", identifier);
// 输出：macOS-AppleSilicon 或 Linux-x86_64-static 等
```

#### 2. Browser 结构体（浏览器操作）

**位置**：`browser.rs`

**职责**：提供在系统默认浏览器中打开 URL 的功能

**主要方法**：

##### `open(url: &str) -> Result<()>`

在浏览器中打开 URL。

**实现**：使用 `open` crate 提供的跨平台功能。

**示例**：
```rust
use workflow::base::system::Browser;

Browser::open("https://github.com")?;
```

#### 3. Clipboard 结构体（剪贴板操作）

**位置**：`clipboard.rs`

**职责**：提供剪贴板的读写功能

**主要方法**：

##### `copy(text: &str) -> Result<()>`

复制文本到剪贴板。

**平台限制**：

**重要说明**：剪贴板功能在以下平台不可用（静默失败，不影响其他功能）：

- **Linux ARM64** (`aarch64-unknown-linux-gnu`)：由于 XCB 库在 Ubuntu 源中不可用，交叉编译时无法链接 XCB 库
- **musl 静态链接版本** (`*-unknown-linux-musl`)：musl 不支持 XCB 库

**支持平台**：
- ✅ macOS (Intel 和 Apple Silicon)
- ✅ Linux x86_64 (glibc)
- ✅ Windows (x86_64 和 ARM64)

**设计说明**：
- 在不受支持的平台上，`Clipboard::copy()` 会静默成功（返回 `Ok(())`），但不会实际复制内容
- 这样设计是为了确保其他功能不受影响，用户仍可以正常使用其他命令

**实现**：使用 `clipboard` crate 提供的跨平台功能（条件编译）。

**示例**：
```rust
use workflow::base::system::Clipboard;

Clipboard::copy("Hello, World!")?;
```

---

## 🔄 调用流程与数据流

### 典型调用流程（平台检测）

```
系统环境
  ↓
Platform::detect()
  ├─ 读取 env::consts::OS
  ├─ 读取 env::consts::ARCH
  └─ 返回 Platform 实例
  ↓
Platform::release_identifier()
  ├─ 检查操作系统和架构
  ├─ Linux x86_64: 检测是否需要静态链接
  │  ├─ 检查 /etc/os-release（Alpine Linux）
  │  └─ 执行 ldd 命令检测静态链接
  └─ 返回平台标识符字符串
```

### 典型调用流程（浏览器操作）

```
URL
  ↓
Browser::open(url)
  ├─ 调用 open::that(url)
  └─ 系统默认浏览器打开 URL
```

### 典型调用流程（剪贴板操作）

```
文本内容
  ↓
Clipboard::copy(text)
  ├─ 检查平台支持（条件编译）
  ├─ 初始化剪贴板上下文
  ├─ 设置剪贴板内容
  └─ 返回结果（不支持平台静默成功）
```

---

## 📋 使用示例

### 平台检测

```rust
use workflow::base::system::Platform;

// 检测当前平台
let platform = Platform::detect();
println!("OS: {}, Arch: {}", platform.os(), platform.arch());

// 检查平台类型
if platform.is_macos() {
    println!("Running on macOS");
}

if platform.is_x86_64() {
    println!("Running on x86_64 architecture");
}

// 生成 GitHub Releases 平台标识符
let identifier = platform.release_identifier()?;
println!("Release identifier: {}", identifier);
```

### 浏览器操作

```rust
use workflow::base::system::Browser;

// 在浏览器中打开 URL
Browser::open("https://github.com/user/repo")?;

// 打开 PR 链接
let pr_url = "https://github.com/user/repo/pull/123";
Browser::open(&pr_url)?;
```

### 剪贴板操作

```rust
use workflow::base::system::Clipboard;

// 复制文本到剪贴板
Clipboard::copy("Hello, World!")?;

// 复制 PR URL
let pr_url = "https://github.com/user/repo/pull/123";
Clipboard::copy(&pr_url)?;

// 注意：在不受支持的平台上会静默成功，但不会实际复制
```

---

## 🔍 错误处理

### 错误类型

1. **平台检测错误**：
   - 不支持的平台组合

2. **浏览器操作错误**：
   - 无法打开浏览器
   - URL 格式无效

3. **剪贴板操作错误**：
   - 剪贴板初始化失败（仅在支持的平台上）
   - 复制操作失败（仅在支持的平台上）

### 容错机制

- **平台不支持**：返回错误，提示用户平台不支持
- **浏览器打开失败**：返回错误，提示用户检查 URL 或浏览器设置
- **剪贴板不支持**：在不受支持的平台上静默成功，不影响其他功能

---

## 📝 扩展性

### 添加新的平台支持

1. 在 `Platform::release_identifier()` 中添加新的平台匹配
2. 添加相应的平台检测方法（如 `is_freebsd()`）

### 添加新的系统交互功能

1. 创建新的模块文件（如 `notification.rs`）
2. 在 `mod.rs` 中声明模块并重新导出
3. 在 `src/lib.rs` 中添加到全局导出（如果需要）

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Lifecycle 模块架构文档](./lifecycle.md) - 使用平台检测匹配 GitHub Releases
- [PR 模块架构文档](./pr.md) - 使用浏览器和剪贴板操作
- [Tools 模块架构文档](./tools.md) - 工具函数总览

---

## ✅ 总结

System 模块采用清晰的模块化设计：

1. **跨平台支持**：所有功能支持 macOS、Linux、Windows
2. **条件编译**：剪贴板功能在特定平台不可用时静默失败
3. **工具类设计**：使用静态方法或零大小结构体，无需实例化
4. **错误处理**：统一的错误处理，提供清晰的错误消息

**设计优势**：
- ✅ 跨平台支持，统一接口
- ✅ 容错性好，不支持平台静默失败
- ✅ 易于使用，简洁的 API
- ✅ 类型安全，使用 Rust 类型系统保证安全性

**当前实现状态**：
- ✅ 平台检测功能完整实现
- ✅ 浏览器操作功能完整实现
- ✅ 剪贴板操作功能完整实现（条件编译）
- ✅ 已在整个项目中广泛使用

---

**最后更新**: 2025-12-27

