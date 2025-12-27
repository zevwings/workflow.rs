# Format 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Format 模块架构，包括：
- **显示格式化器**（DisplayFormatter）：路径、列表项、键值对、文件大小格式化
- **消息格式化器**（MessageFormatter）：错误消息、操作消息、进度信息格式化

该模块提供统一的格式化功能，确保整个项目的输出格式一致，提升用户体验和代码可维护性。

**注意**：本模块是基础设施模块，被整个项目广泛使用。所有需要格式化输出的命令都使用这些格式化器。

**模块统计：**
- 总代码行数：约 200 行
- 文件数量：2 个核心文件
- 主要组件：
  - DisplayFormatter（4 个方法）
  - MessageFormatter（3 个方法）
- 依赖：仅标准库，无外部依赖

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/format/
├── mod.rs          # 模块声明和导出 (16行)
├── display.rs      # 显示格式化器 (207行)
└── message.rs      # 消息格式化器 (129行)
```

### 依赖模块

- **标准库**：`std::path::Path`、`std::fmt`
- **无外部依赖**：Format 模块仅使用 Rust 标准库，无第三方依赖

### 模块集成

Format 模块被所有需要格式化输出的命令和模块广泛使用：

- **Lifecycle 命令**：使用 `DisplayFormatter::size()` 格式化文件大小
- **Jira 命令**：使用 `MessageFormatter::error()` 格式化错误消息
- **PR 命令**：使用 `MessageFormatter::operation()` 格式化操作消息
- **Branch 命令**：使用 `DisplayFormatter::path()` 格式化路径显示
- **Config 命令**：使用 `DisplayFormatter::key_value()` 格式化配置显示
- **所有命令**：使用 `MessageFormatter::progress()` 格式化进度信息

---

## 🔄 集成关系

Format 模块是 Workflow CLI 的基础设施模块，为所有需要格式化输出的命令和模块提供统一的格式化接口。该模块通过以下方式与其他模块集成：

1. **命令层集成**：所有命令层模块通过 Format 模块提供的接口进行输出格式化
2. **统一格式**：提供统一的格式化标准，确保所有输出格式一致
3. **易于维护**：集中管理格式化逻辑，修改时只需更新一处

### 主要集成场景

- **文件大小显示**：Lifecycle 命令使用 `DisplayFormatter::size()` 显示文件大小
- **错误消息**：所有命令使用 `MessageFormatter::error()` 格式化错误消息
- **操作提示**：所有命令使用 `MessageFormatter::operation()` 显示操作进度
- **路径显示**：所有命令使用 `DisplayFormatter::path()` 显示文件路径
- **配置显示**：Config 命令使用 `DisplayFormatter::key_value()` 显示配置项

---

## 🏗️ 架构设计

### 设计原则

1. **统一格式**：所有格式化功能统一管理，确保输出格式一致
2. **易于维护**：集中管理格式化逻辑，修改时只需更新一处
3. **类型安全**：使用 Rust 类型系统保证参数类型正确
4. **易于扩展**：可以轻松添加新的格式化方法
5. **无状态设计**：所有格式化器都是静态方法，无需实例化

### 核心组件

#### 1. DisplayFormatter 结构体 (`display.rs`)

**职责**：提供统一的显示格式化功能，包括路径、列表项、键值对和文件大小的格式化。

**主要方法**：

##### DisplayFormatter::path

```rust
pub fn path(path: &Path) -> String
```

**功能**：格式化路径显示，优先显示相对路径

**参数**：
- `path` - 要格式化的路径

**返回**：格式化后的路径字符串（相对路径或绝对路径）

**实现逻辑**：
- 尝试将路径转换为相对于当前工作目录的相对路径
- 如果转换失败，返回完整路径
- 使用 `Path::display()` 进行平台无关的路径格式化

**示例**：
```rust
use workflow::base::format::DisplayFormatter;
use std::path::Path;

let path = Path::new("/home/user/project/src/main.rs");
let formatted = DisplayFormatter::path(path);
// 如果路径在当前工作目录下，返回相对路径
// 否则返回完整路径
```

##### DisplayFormatter::list_item

```rust
pub fn list_item(prefix: &str, item: &str) -> String
```

**功能**：格式化列表项显示

**参数**：
- `prefix` - 前缀符号（如 "  -"、"  *"）
- `item` - 项目内容

**返回**：格式化后的列表项字符串

**格式**：`"{prefix} {item}"`

**示例**：
```rust
use workflow::base::format::DisplayFormatter;

let item = DisplayFormatter::list_item("  -", "config.toml");
assert_eq!(item, "  - config.toml");
```

##### DisplayFormatter::key_value

```rust
pub fn key_value(key: &str, value: &str, separator: Option<&str>) -> String
```

**功能**：格式化键值对显示

**参数**：
- `key` - 键名
- `value` - 值
- `separator` - 分隔符（默认为 ": "）

**返回**：格式化后的键值对字符串

**格式**：`"{key}{separator}{value}"`

**示例**：
```rust
use workflow::base::format::DisplayFormatter;

let kv = DisplayFormatter::key_value("Version", "1.0.0", None);
assert_eq!(kv, "Version: 1.0.0");

let kv = DisplayFormatter::key_value("Status", "Active", Some(" = "));
assert_eq!(kv, "Status = Active");
```

##### DisplayFormatter::size

```rust
pub fn size(bytes: u64) -> String
```

**功能**：格式化文件大小显示

**参数**：
- `bytes` - 字节数

**返回**：格式化后的字符串，例如 "1.23 MB" 或 "1024 B"

**规则**：
- 自动选择合适的单位（B, KB, MB, GB, TB）
- 小于 1024 字节时显示为 "X B"（整数）
- 大于等于 1024 字节时显示为 "X.XX UNIT"（保留两位小数）
- 使用 1024 作为进制（二进制单位）

**示例**：
```rust
use workflow::base::format::DisplayFormatter;

assert_eq!(DisplayFormatter::size(0), "0 B");
assert_eq!(DisplayFormatter::size(1024), "1.00 KB");
assert_eq!(DisplayFormatter::size(1048576), "1.00 MB");
assert_eq!(DisplayFormatter::size(1073741824), "1.00 GB");
```

**关键特性**：
- 自动单位选择：根据字节数自动选择合适的单位
- 精度控制：小值显示整数，大值保留两位小数
- 二进制单位：使用 1024 作为进制，符合计算机存储习惯

**使用场景**：
- 文件大小显示：在下载、更新等命令中显示文件大小
- 进度提示：显示下载进度和文件大小
- 配置显示：显示配置项的文件大小限制

#### 2. MessageFormatter 结构体 (`message.rs`)

**职责**：提供统一的消息格式化功能，包括错误消息、操作消息和进度信息的格式化。

**主要方法**：

##### MessageFormatter::error

```rust
pub fn error(operation: &str, target: &str, error: &str) -> String
```

**功能**：格式化错误消息

**参数**：
- `operation` - 操作名称（如 "read"、"write"、"create"）
- `target` - 操作目标（文件、路径、资源等）
- `error` - 错误信息

**返回**：格式化后的错误消息字符串

**格式**：`"Failed to {operation} {target}: {error}"`

**示例**：
```rust
use workflow::base::format::MessageFormatter;

let msg = MessageFormatter::error("read", "config.toml", "Permission denied");
assert_eq!(msg, "Failed to read config.toml: Permission denied");

let msg = MessageFormatter::error("create", "new branch", "Branch already exists");
assert_eq!(msg, "Failed to create new branch: Branch already exists");
```

**关键特性**：
- 统一格式：所有错误消息使用相同的格式，提升一致性
- 易于理解：清晰的错误消息格式，便于用户理解问题
- 易于维护：集中管理错误消息格式，修改时只需更新一处

**使用场景**：
- 文件操作错误：读取、写入、创建文件时的错误消息
- 网络操作错误：HTTP 请求失败时的错误消息
- 配置错误：配置验证失败时的错误消息

##### MessageFormatter::operation

```rust
pub fn operation(action: &str, target: &str) -> String
```

**功能**：格式化操作消息

**参数**：
- `action` - 动作名称（如 "Creating"、"Updating"、"Deleting"）
- `target` - 操作目标

**返回**：格式化后的操作消息字符串

**格式**：`"{action} {target}..."`

**示例**：
```rust
use workflow::base::format::MessageFormatter;

let msg = MessageFormatter::operation("Creating", "new branch");
assert_eq!(msg, "Creating new branch...");

let msg = MessageFormatter::operation("Updating", "configuration");
assert_eq!(msg, "Updating configuration...");
```

**关键特性**：
- 统一格式：所有操作消息使用相同的格式
- 进度提示：使用 "..." 表示操作正在进行中
- 易于理解：清晰的动词+目标格式

**使用场景**：
- 操作提示：在命令执行过程中显示操作进度
- 进度信息：显示当前正在执行的操作
- 用户反馈：向用户展示命令的执行状态

##### MessageFormatter::progress

```rust
pub fn progress(current: usize, total: usize, item: &str) -> String
```

**功能**：格式化进度信息

**参数**：
- `current` - 当前进度（已完成的数量）
- `total` - 总进度（总数量）
- `item` - 进度项目名称（如 "files"、"items"、"tasks"）

**返回**：格式化后的进度字符串

**格式**：`"[{current}/{total}] Processing {item}"`

**示例**：
```rust
use workflow::base::format::MessageFormatter;

let msg = MessageFormatter::progress(3, 10, "files");
assert_eq!(msg, "[3/10] Processing files");

let msg = MessageFormatter::progress(5, 20, "items");
assert_eq!(msg, "[5/20] Processing items");
```

**关键特性**：
- 统一格式：所有进度信息使用相同的格式
- 清晰显示：使用 `[current/total]` 格式显示进度
- 易于理解：清晰的进度信息格式

**使用场景**：
- 批量操作：在批量处理文件、数据等操作中显示进度
- 循环处理：在循环处理数据时显示当前进度
- 用户反馈：向用户展示操作的执行进度

### 设计模式

#### 1. 静态方法模式

所有格式化器都使用静态方法设计，无需实例化：

```rust
// DisplayFormatter
let formatted = DisplayFormatter::path(path);
let size = DisplayFormatter::size(bytes);

// MessageFormatter
let error_msg = MessageFormatter::error("read", "file", "error");
let progress = MessageFormatter::progress(3, 10, "files");
```

**优势**：
- 无需管理实例状态
- 使用简单，直接调用静态方法
- 适合工具函数的场景
- 零开销抽象，编译时优化

#### 2. 统一接口模式

所有格式化器都提供统一的接口，使用相同的调用模式：

```rust
// 统一的调用模式
Formatter::method(param1, param2, ...) -> String
```

**优势**：
- 易于学习和使用
- 一致的 API 设计
- 降低学习成本

### 错误处理

Format 模块的所有方法都是纯函数，不涉及错误处理：

- **输入验证**：由调用者负责验证输入参数
- **返回值**：所有方法返回 `String`，不会失败
- **错误处理**：错误处理由调用者负责，Format 模块只负责格式化

---

## 📋 使用示例

### DisplayFormatter 使用示例

#### 路径格式化

```rust
use workflow::base::format::DisplayFormatter;
use std::path::Path;
use workflow::log_message;

let path = Path::new("/home/user/project/src/main.rs");
let formatted_path = DisplayFormatter::path(path);
log_message!("File: {}", formatted_path);
```

#### 列表项格式化

```rust
use workflow::base::format::DisplayFormatter;
use workflow::log_message;

let items = vec!["config.toml", "README.md", "Cargo.toml"];
for item in items {
    let formatted = DisplayFormatter::list_item("  -", item);
    log_message!("{}", formatted);
}
// 输出：
//   - config.toml
//   - README.md
//   - Cargo.toml
```

#### 键值对格式化

```rust
use workflow::base::format::DisplayFormatter;
use workflow::log_message;

let config = vec![
    ("Version", "1.0.0"),
    ("Author", "Workflow Team"),
    ("License", "MIT"),
];

for (key, value) in config {
    let kv = DisplayFormatter::key_value(key, value, None);
    log_message!("{}", kv);
}
// 输出：
// Version: 1.0.0
// Author: Workflow Team
// License: MIT
```

#### 文件大小格式化

```rust
use workflow::base::format::DisplayFormatter;
use workflow::log_message;

let file_size = 1048576; // 1 MB
let formatted_size = DisplayFormatter::size(file_size);
log_message!("File size: {}", formatted_size);
// 输出：File size: 1.00 MB
```

### MessageFormatter 使用示例

#### 错误消息格式化

```rust
use workflow::base::format::MessageFormatter;
use workflow::log_error;

match read_file("config.toml") {
    Ok(content) => {
        // 处理内容
    }
    Err(e) => {
        let error_msg = MessageFormatter::error("read", "config.toml", &e.to_string());
        log_error!("{}", error_msg);
    }
}
```

#### 操作消息格式化

```rust
use workflow::base::format::MessageFormatter;
use workflow::log_message;

let operation_msg = MessageFormatter::operation("Creating", "new branch");
log_message!("{}", operation_msg);
// 输出：Creating new branch...
```

#### 进度信息格式化

```rust
use workflow::base::format::MessageFormatter;
use workflow::log_message;

let files = vec!["file1.txt", "file2.txt", "file3.txt"];
let total = files.len();

for (index, file) in files.iter().enumerate() {
    let current = index + 1;
    let progress_msg = MessageFormatter::progress(current, total, "files");
    log_message!("{} {}", progress_msg, file);
    // 处理文件
}
// 输出：
// [1/3] Processing files file1.txt
// [2/3] Processing files file2.txt
// [3/3] Processing files file3.txt
```

### 组合使用示例

```rust
use workflow::base::format::{DisplayFormatter, MessageFormatter};
use workflow::{log_message, log_error};
use std::path::Path;

fn process_file(path: &Path) -> Result<(), String> {
    let formatted_path = DisplayFormatter::path(path);
    let operation_msg = MessageFormatter::operation("Processing", &formatted_path);
    log_message!("{}", operation_msg);

    // 处理文件...

    match process() {
        Ok(_) => {
            let size = DisplayFormatter::size(file_size);
            log_message!("File size: {}", size);
            Ok(())
        }
        Err(e) => {
            let error_msg = MessageFormatter::error("process", &formatted_path, &e);
            log_error!("{}", error_msg);
            Err(e)
        }
    }
}
```

---

## 📝 扩展性

### 添加新的格式化方法

1. **确定格式化需求**：分析需要格式化的数据类型和格式要求
2. **选择格式化器**：根据数据类型选择合适的格式化器（DisplayFormatter 或 MessageFormatter）
3. **实现格式化方法**：在对应的格式化器中添加新的静态方法
4. **添加文档注释**：为新方法添加完整的文档注释，包括参数、返回值、示例
5. **添加测试**：为新方法添加单元测试

**示例**：添加日期时间格式化

```rust
impl DisplayFormatter {
    /// 格式化日期时间显示
    ///
    /// # 参数
    ///
    /// * `datetime` - 日期时间
    ///
    /// # 返回
    ///
    /// 格式化后的日期时间字符串
    pub fn datetime(datetime: &DateTime<Utc>) -> String {
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
```

### 添加新的格式化器类型

如果现有格式化器无法满足需求，可以创建新的格式化器类型：

1. **创建新的格式化器文件**：在 `src/lib/base/format/` 目录下创建新文件
2. **实现格式化器结构体**：定义格式化器结构体和静态方法
3. **导出格式化器**：在 `mod.rs` 中导出新的格式化器
4. **添加文档**：更新架构文档，添加新格式化器的说明

---

## 📚 相关文档

- [主架构文档](./architecture.md) - 项目总体架构
- [Dialog 模块架构文档](./dialog.md) - 用户交互相关模块
- [Logger 模块架构文档](./logger.md) - 日志输出相关模块

---

## ✅ 总结

Format 模块采用清晰的静态方法设计：

1. **统一格式**：所有格式化功能统一管理，确保输出格式一致
2. **易于维护**：集中管理格式化逻辑，修改时只需更新一处
3. **类型安全**：使用 Rust 类型系统保证参数类型正确
4. **易于扩展**：可以轻松添加新的格式化方法
5. **无状态设计**：所有格式化器都是静态方法，无需实例化

**设计优势**：
- ✅ **统一性**：所有格式化功能使用统一的接口和格式
- ✅ **可维护性**：集中管理，易于维护和更新
- ✅ **可扩展性**：易于添加新的格式化方法
- ✅ **性能**：零开销抽象，编译时优化
- ✅ **易用性**：简单的 API，易于学习和使用

---

**最后更新**: 2025-12-23

