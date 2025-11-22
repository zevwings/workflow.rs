# 工具函数模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的工具函数模块架构，包括：
- 日志输出系统（带颜色的日志宏和日志级别管理）
- 字符串处理工具（敏感值隐藏）
- 浏览器和剪贴板操作
- 文件解压和校验和验证
- 用户确认对话框

这些工具函数为整个项目提供通用的基础设施支持，被所有模块广泛使用。

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/util/
├── mod.rs         # 模块入口，重新导出所有公共 API
├── logger.rs      # 日志输出系统（~566 行）
├── string.rs      # 字符串处理工具（~38 行）
├── browser.rs     # 浏览器操作（~29 行）
├── clipboard.rs   # 剪贴板操作（~35 行）
├── unzip.rs       # 文件解压工具（~61 行）
├── checksum.rs    # 校验和验证工具（~164 行）
└── confirm.rs     # 用户确认对话框（~45 行）
```

**总计：约 938 行代码，8 个文件，7 个主要组件**

### 依赖模块

- **`colored`**：终端颜色输出
- **`open`**：打开浏览器
- **`arboard`**：剪贴板操作
- **`zip`**：ZIP 文件解压
- **`sha2`**：SHA256 校验和
- **`dialoguer`**：用户确认对话框

### 模块集成

工具模块被所有其他模块广泛使用：

- **日志系统**：所有模块使用 `log_*!` 宏输出日志
- **剪贴板**：PR、QK 命令使用 `Clipboard::copy()`
- **浏览器**：PR 命令使用 `Browser::open()`
- **文件操作**：Lifecycle 命令使用 `Unzip::extract()` 和 `Checksum::verify()`
- **用户确认**：多个命令使用 `confirm()` 函数

---

## 🏗️ 架构设计

### 设计原则

1. **单一职责**：每个工具函数只负责一个明确的功能
2. **易用性**：提供简洁的 API 和宏
3. **可配置性**：日志级别可通过环境变量或运行时设置
4. **跨平台**：所有工具函数支持多平台

### 核心组件

#### 1. 日志输出系统 (`logger.rs`)

### 功能概述

提供带颜色的日志输出功能，支持日志级别控制，包括：
- 日志级别枚举（None, Error, Warn, Info, Debug）
- 带颜色的日志输出（成功、错误、警告、信息、调试）
- 日志级别管理（环境变量、运行时设置）
- 日志宏（`log_success!`, `log_error!`, `log_warning!`, `log_info!`, `log_debug!`, `log_message!`, `log_break!`）

### 核心组件

#### LogLevel 枚举

```rust
pub enum LogLevel {
    None = 0,   // 不输出任何日志
    Error = 1,  // 只输出错误
    Warn = 2,   // 输出警告和错误
    Info = 3,   // 输出信息、警告和错误（默认）
    Debug = 4,  // 输出所有日志（包括调试）
}
```

**特性**：
- 支持从字符串解析（`FromStr` 实现）
- 支持比较和排序（`PartialOrd`, `Ord` 实现）
- 默认级别根据编译模式自动决定：
  - Debug 模式：`Debug` 级别
  - Release 模式：`Info` 级别
- 支持通过环境变量 `WORKFLOW_LOG_LEVEL` 设置
- 支持运行时动态设置

#### Logger 结构体

提供静态方法用于格式化日志消息：

- `Logger::success(message)` - 成功消息（绿色 ✓）
- `Logger::error(message)` - 错误消息（红色 ✗）
- `Logger::warning(message)` - 警告消息（黄色 ⚠）
- `Logger::info(message)` - 信息消息（蓝色 ℹ）
- `Logger::debug(message)` - 调试消息（灰色 ⚙）

提供打印方法（受日志级别控制）：

- `Logger::print_success(message)` - 总是输出（不受日志级别限制）
- `Logger::print_error(message)` - 仅在 `>= Error` 时输出
- `Logger::print_warning(message)` - 仅在 `>= Warn` 时输出
- `Logger::print_info(message)` - 仅在 `>= Info` 时输出
- `Logger::print_debug(message)` - 仅在 `>= Debug` 时输出
- `Logger::print_message(message)` - 总是输出（不受日志级别限制）
- `Logger::print_separator(char, length)` - 打印分隔线
- `Logger::print_separator_with_text(char, length, text)` - 打印带文本的分隔线

### 日志宏

#### log_success!

```rust
log_success!("Operation completed");
log_success!("Found {} items", count);
```

- 总是输出，不受日志级别限制
- 用于显示命令执行成功的结果

#### log_error!

```rust
log_error!("Operation failed");
log_error!("Error: {} - {}", code, message);
```

- 仅在日志级别 `>= Error` 时输出
- 用于显示错误信息

#### log_warning!

```rust
log_warning!("This is a warning");
log_warning!("Warning: {} items missing", count);
```

- 仅在日志级别 `>= Warn` 时输出
- 用于显示警告信息

#### log_info!

```rust
log_info!("Processing data");
log_info!("Processing {} items", count);
```

- 仅在日志级别 `>= Info` 时输出
- 用于显示一般信息

#### log_debug!

```rust
log_debug!("Debug information");
log_debug!("Debug: {} = {}", key, value);
```

- 仅在日志级别 `>= Debug` 时输出
- 在 Debug 模式下自动启用，在 Release 模式下不输出
- 用于开发调试

#### log_message!

```rust
log_message!("Running environment checks...");
log_message!("[1/2] Checking Git repository status...");
```

- 总是输出，不受日志级别限制
- 用于输出 setup/check 等命令的说明信息
- 这些信息是指令性的，用户需要看到

#### log_break!

```rust
log_break!();                    // 输出换行符
log_break!('-');                 // 使用默认分隔符（80个 '-'）
log_break!('=');                 // 指定分隔符字符
log_break!('=', 100);            // 指定分隔符字符和长度
log_break!('=', 20, "text");     // 在分隔线中间插入文本
```

- 用于输出分隔线或换行
- 支持自定义字符和长度
- 支持在分隔线中间插入文本

### 日志级别管理

#### 初始化日志级别

```rust
// 从环境变量或使用默认值
LogLevel::init(None);

// 显式设置日志级别
LogLevel::init(Some(LogLevel::Debug));
```

**优先级**：
1. 如果提供了 `level` 参数，使用参数值
2. 如果设置了 `WORKFLOW_LOG_LEVEL` 环境变量，使用环境变量值
3. 否则使用默认级别（根据编译模式决定）

#### 运行时设置日志级别

```rust
LogLevel::set_level(LogLevel::Debug);
let current_level = LogLevel::get_level();
```

### 设计决策

1. **成功消息总是显示**：成功消息是命令执行结果的重要反馈，应该始终显示给用户
2. **说明信息不受日志级别限制**：setup/check 等命令的说明信息是指令性的，用户需要看到
3. **调试信息自动控制**：根据编译模式自动决定是否输出调试信息，避免在 Release 版本中泄露调试信息
4. **线程安全**：使用 `Mutex` 保证日志级别的线程安全访问

### 使用场景

- **CLI 命令输出**：所有命令使用日志宏输出执行结果
- **错误处理**：使用 `log_error!` 显示错误信息
- **进度提示**：使用 `log_info!` 显示处理进度
- **调试开发**：使用 `log_debug!` 输出调试信息（仅在 Debug 模式）

#### 2. 字符串处理工具 (`string.rs`)

### 功能概述

提供字符串处理相关的工具函数，主要用于敏感信息的隐藏。

### 核心函数

#### mask_sensitive_value

```rust
pub fn mask_sensitive_value(value: &str) -> String
```

**功能**：隐藏敏感值（用于显示）

**规则**：
- 短值（长度 ≤ 12）：完全隐藏，显示为 `***`
- 长值（长度 > 12）：显示前 4 个字符和后 4 个字符，中间用 `***` 代替

**示例**：

```rust
use workflow::mask_sensitive_value;

assert_eq!(mask_sensitive_value("short"), "***");
assert_eq!(mask_sensitive_value("verylongapikey123456"), "very***3456");
```

### 使用场景

- **日志输出**：在日志中隐藏 API Key、密码等敏感信息
- **配置显示**：在显示配置时隐藏敏感值
- **错误信息**：在错误信息中隐藏敏感数据

#### 3. 浏览器操作 (`browser.rs`)

### 功能概述

提供在系统默认浏览器中打开 URL 的功能。

### 核心组件

#### Browser 结构体

```rust
pub struct Browser;
```

#### Browser::open

```rust
pub fn open(url: &str) -> Result<()>
```

**功能**：在浏览器中打开 URL

**参数**：
- `url` - 要打开的 URL

**错误**：如果无法打开浏览器，返回相应的错误信息

**实现**：使用 `open` crate 提供的跨平台功能

### 使用场景

- **打开文档**：在用户需要查看文档时自动打开浏览器
- **打开 GitHub 链接**：在 PR 相关命令中打开 GitHub 链接
- **打开 Jira 链接**：在 Jira 相关命令中打开 Jira 链接

#### 4. 剪贴板操作 (`clipboard.rs`)

### 功能概述

提供剪贴板的读写功能。

### 核心组件

#### Clipboard 结构体

```rust
pub struct Clipboard;
```

#### Clipboard::copy

```rust
pub fn copy(text: &str) -> Result<()>
```

**功能**：复制文本到剪贴板

**参数**：
- `text` - 要复制的文本

**错误**：如果复制失败，返回相应的错误信息

**实现**：使用 `clipboard` crate 提供的跨平台功能

### 使用场景

- **复制命令**：复制生成的命令到剪贴板，方便用户使用
- **复制 URL**：复制生成的 URL 到剪贴板
- **复制配置**：复制配置信息到剪贴板

#### 5. 文件解压工具 (`unzip.rs`)

### 功能概述

提供 tar.gz 文件解压功能。

### 核心组件

#### Unzip 结构体

```rust
pub struct Unzip;
```

#### Unzip::extract_tar_gz

```rust
pub fn extract_tar_gz(tar_gz_path: &Path, output_dir: &Path) -> Result<()>
```

**功能**：解压 tar.gz 文件到指定目录

**参数**：
- `tar_gz_path` - tar.gz 文件路径
- `output_dir` - 解压目标目录

**错误**：如果解压失败，返回相应的错误信息

**实现**：
- 使用 `flate2` crate 进行 Gzip 解压
- 使用 `tar` crate 进行 tar 归档解压

**示例**：

```rust
use workflow::Unzip;
use std::path::Path;

Unzip::extract_tar_gz(
    Path::new("archive.tar.gz"),
    Path::new("./output")
)?;
```

### 使用场景

- **更新功能**：解压下载的更新包
- **安装功能**：解压安装包
- **补全脚本安装**：解压补全脚本包

#### 6. 校验和验证工具 (`checksum.rs`)

### 功能概述

提供文件校验和计算和验证功能，包括：
- 计算文件的 SHA256 哈希值
- 解析校验和文件内容
- 验证文件完整性
- 构建校验和 URL（纯字符串操作）

### 核心组件

#### Checksum 结构体

```rust
pub struct Checksum;
```

#### Checksum::calculate_file_sha256

```rust
pub fn calculate_file_sha256(file_path: &Path) -> Result<String>
```

**功能**：计算文件的 SHA256 哈希值

**参数**：
- `file_path` - 要计算哈希值的文件路径

**返回**：返回文件的 SHA256 哈希值（十六进制字符串）

**实现**：
- 使用 `sha2` crate 进行 SHA256 计算
- 使用 8KB 缓冲区逐块读取文件，避免内存占用过大

#### Checksum::parse_hash_from_content

```rust
pub fn parse_hash_from_content(content: &str) -> Result<String>
```

**功能**：从校验和文件内容中提取哈希值

**参数**：
- `content` - 校验和文件的文本内容

**返回**：返回提取的哈希值

**格式支持**：
- `"hash  filename"` 格式（标准格式）
- `"hash"` 格式（只有哈希值）

**示例**：

```rust
use workflow::Checksum;

let content = "abc123def456  file.tar.gz";
let hash = Checksum::parse_hash_from_content(content)?;
assert_eq!(hash, "abc123def456");
```

#### Checksum::verify

```rust
pub fn verify(file_path: &Path, expected_hash: &str) -> Result<()>
```

**功能**：验证文件完整性（通过比较哈希值）

**参数**：
- `file_path` - 要验证的文件路径
- `expected_hash` - 期望的 SHA256 哈希值

**返回**：如果哈希值匹配，返回 `Ok(())`；否则返回错误

**实现**：
- 计算文件的实际哈希值
- 与期望哈希值进行比较
- 输出验证结果日志

#### Checksum::build_url

```rust
pub fn build_url(url: &str) -> String
```

**功能**：从下载 URL 构建校验和 URL

**参数**：
- `download_url` - 下载文件的 URL

**返回**：返回校验和文件的 URL

**规则**：在下载 URL 后添加 `.sha256` 后缀

**示例**：

```rust
use workflow::Checksum;

let url = "https://example.com/file.tar.gz";
assert_eq!(Checksum::build_url(url), "https://example.com/file.tar.gz.sha256");
```

### 使用场景

- **更新功能**：验证下载的更新包完整性
- **安装功能**：验证安装包完整性
- **安全验证**：确保下载的文件未被篡改

#### 7. 用户确认对话框 (`confirm.rs`)

### 功能概述

提供统一的用户确认接口，简化 `Confirm::new()` 的使用。

### 核心函数

#### confirm

```rust
pub fn confirm(prompt: &str, default: bool, message: Option<&str>) -> Result<bool>
```

**功能**：显示确认对话框并获取用户选择

**参数**：
- `prompt` - 提示信息
- `default` - 默认选择（true 表示默认确认，false 表示默认取消）
- `cancel_message` - 取消时的错误消息（可选）
  - 如果为 `Some(msg)`，取消时返回错误
  - 如果为 `None`，取消时返回 `Ok(false)`

**返回**：
- 用户确认：返回 `Ok(true)`
- 用户取消且 `cancel_message` 为 `Some`：返回错误
- 用户取消且 `cancel_message` 为 `None`：返回 `Ok(false)`

**实现**：使用 `dialoguer` crate 提供交互式确认对话框

### 使用场景

#### 场景 1：需要根据用户选择执行不同逻辑

```rust
use workflow::confirm;

if confirm("Do you want to continue?", true, None)? {
    // 用户确认，执行操作
} else {
    // 用户取消，执行其他逻辑
}
```

#### 场景 2：需要用户确认才能继续，取消则终止

```rust
use workflow::confirm;

confirm(
    "This operation cannot be undone. Continue?",
    false,
    Some("Operation cancelled.")
)?;
// 如果用户取消，函数会返回错误，不会继续执行
```

### 设计决策

1. **统一的确认接口**：简化 `Confirm::new()` 的使用，提供更简洁的 API
2. **灵活的取消处理**：支持两种取消处理方式：
   - 返回 `Ok(false)`：允许调用者根据返回值决定后续逻辑
   - 返回错误：强制要求用户确认，取消则终止操作

### 设计模式

#### 工具类设计

所有工具函数采用静态方法设计，无需实例化：

```rust
// 日志系统
Logger::print_info("message");

// 浏览器操作
Browser::open("https://example.com");

// 剪贴板操作
Clipboard::copy("text");

// 文件操作
Unzip::extract_tar_gz(path, dir)?;
Checksum::verify(file, hash)?;
```

**优势**：
- 无需管理实例状态
- 使用简单，直接调用静态方法
- 适合工具函数的场景

#### 日志宏设计

使用 Rust 宏系统提供便捷的日志输出接口：

```rust
log_info!("Processing {} items", count);
```

**优势**：
- 支持格式化字符串（类似 `println!`）
- 编译时展开，性能开销小
- 统一的日志输出接口

#### 错误处理策略

所有可能失败的操作都返回 `Result<T>`：

```rust
Browser::open(url)?;
Clipboard::copy(text)?;
Unzip::extract_tar_gz(path, dir)?;
Checksum::verify(file, hash)?;
```

**优势**：
- 统一的错误处理方式
- 使用 `?` 操作符简化错误传播
- 使用 `anyhow` 提供详细的错误上下文

---

## 🔄 调用流程与数据流

### 整体架构流程

```
应用层（命令、模块）
  ↓
工具函数 API
  ├─ 日志系统 → 终端输出
  ├─ 浏览器操作 → 系统浏览器
  ├─ 剪贴板操作 → 系统剪贴板
  ├─ 文件操作 → 文件系统
  └─ 用户确认 → 终端交互
```

### 数据流

#### 日志输出流程

```
log_info!("message")
  ↓
Logger::print_info()
  ↓
检查日志级别
  ↓
格式化消息（添加颜色和图标）
  ↓
输出到终端
```

#### 文件操作流程

```
Unzip::extract_tar_gz(path, dir)
  ↓
读取文件
  ↓
解压到目标目录
  ↓
返回 Result

Checksum::verify(file, hash)
  ↓
读取文件内容
  ↓
计算 SHA256
  ↓
比较哈希值
  ↓
返回 Result
```

### 与其他模块的集成

util 模块是基础设施模块，被整个项目广泛使用：

- **CLI 命令层**：所有命令使用日志宏输出结果
- **配置管理**：使用日志宏和确认对话框
- **更新功能**：使用解压和校验和工具
- **Git 操作**：使用日志宏输出状态
- **Jira 操作**：使用日志宏和浏览器工具
- **PR 操作**：使用日志宏、浏览器和剪贴板工具

**依赖关系**：

```
util (基础设施)
  ↓
所有业务模块（commands, lib/*）
```

---

## 📝 扩展性

### 添加新的日志级别

1. 在 `LogLevel` 枚举中添加新级别
2. 在 `FromStr` 实现中添加字符串解析
3. 在 `as_str` 方法中添加字符串转换
4. 添加对应的日志宏（如果需要）

### 添加新的工具函数

1. 创建新的模块文件（如 `new_tool.rs`）
2. 在 `mod.rs` 中声明模块并重新导出
3. 在 `src/lib.rs` 中添加到全局导出（如果需要）

### 添加新的日志宏

1. 在 `logger.rs` 中添加宏定义
2. 使用 `#[macro_export]` 导出宏
3. 在 `src/lib.rs` 中重新导出（如果需要）

---

## 📚 相关文档

- [总体架构文档](../ARCHITECTURE.md)
- [Settings 模块架构文档](./SETTINGS_ARCHITECTURE.md)
- [HTTP 架构文档](./HTTP_ARCHITECTURE.md)

---

## 📋 使用示例

### 日志输出

```rust
use workflow::{log_success, log_error, log_warning, log_info, log_debug};

// 成功消息
log_success!("Operation completed successfully");

// 错误消息
log_error!("Failed to process request");

// 警告消息
log_warning!("This operation is deprecated");

// 信息消息
log_info!("Processing {} items", count);

// 调试消息
log_debug!("Debug info: {:?}", data);

// 分隔线
log_break!();
```

### 字符串处理

```rust
use workflow::mask_sensitive_value;

let api_key = "verylongapikey123456";
println!("API Key: {}", mask_sensitive_value(api_key));
// 输出：API Key: very***3456
```

### 浏览器操作

```rust
use workflow::Browser;

Browser::open("https://github.com")?;
```

### 剪贴板操作

```rust
use workflow::Clipboard;

Clipboard::copy("Hello, World!")?;
log_success!("Copied to clipboard");
```

### 文件操作

```rust
use workflow::{Unzip, Checksum};

// 解压文件
Unzip::extract_tar_gz("archive.tar.gz", "output_dir")?;

// 验证校验和
Checksum::verify("file.bin", "expected_sha256_hash")?;
```

### 用户确认

```rust
use workflow::confirm;

if confirm("Do you want to continue?")? {
    log_info!("User confirmed");
} else {
    log_info!("User cancelled");
}
```

---

## ✅ 总结

工具函数模块为整个项目提供通用的基础设施支持：

1. **日志系统**：带颜色的日志输出，支持日志级别控制
2. **字符串处理**：敏感值隐藏
3. **浏览器和剪贴板**：系统集成操作
4. **文件操作**：解压和校验和验证
5. **用户确认**：交互式对话框

**设计优势**：
- ✅ **易用性**：简洁的 API 和宏
- ✅ **一致性**：统一的错误处理方式
- ✅ **可配置性**：日志级别可通过环境变量或运行时设置
- ✅ **跨平台**：所有工具函数支持多平台
- ✅ **高性能**：宏在编译时展开，流式处理文件

