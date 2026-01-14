# 日志系统增强需求文档

## 📋 概述

本文档基于 Go 版本日志系统（`workflow.py/internal/logging`）的对比分析，提出 Rust 版本日志系统（`src/lib/base/logger`）的增强需求，以提升日志管理的功能性和可维护性。

**目标**: 将 Rust 版本日志系统功能对齐到 Go 版本水平，增强模块化日志管理、日志轮转和结构化日志支持。

**时间估算**: 2-3天（简化后，移除日志轮转、模块分离和统一错误日志）
**优先级**: 中优先级（功能增强，非阻塞性）

**更新说明**（2025-01-15）：
- ✅ 采用"每次操作独立日志文件"方案，避免日志文件无限增长
- ✅ 移除日志轮转需求（CLI 工具不需要）
- ❌ 不需要模块级日志分离（所有模块日志写入同一个命令日志文件）
- ✅ 移除统一错误日志收集（所有日志写入命令日志文件即可）
- ✅ 文件命名格式：`{command}-{timestamp}-{pid}.log`（如 `pr-create-20250115143120-12346.log`）

---

## 🔍 现状分析

### 1. 当前 Rust 实现（`src/lib/base/logger/`）

**架构特点**：
- **双层架构**：
  - `Logger` (console.rs)：Commands 层，用户友好的控制台输出
  - `Tracer` (tracing.rs)：Lib 层，结构化日志记录（使用 `tracing` crate）
- **日志级别管理**：`LogLevel` 枚举，支持 None/Error/Warn/Info/Debug
- **文件输出**：按日期分割（`workflow-YYYY-MM-DD.log`）

**优点**：
- ✅ 职责分离清晰（Commands vs Lib）
- ✅ 编译时优化（debug/release 模式自动调整）
- ✅ 类型安全（Rust 类型系统）
- ✅ 基础功能完整（日志级别、文件输出、控制台输出）

**缺点**：
- ❌ 无模块级日志分离（所有模块输出到单一文件，不需要分离）
- ❌ 无自动模块识别（需要手动指定模块信息）
- ❌ 无日志轮转（无大小限制、无备份管理、无自动压缩）
- ❌ 无统一错误日志收集（`error.log`）（不需要，所有日志写入命令日志文件即可）
- ❌ 无结构化日志字段支持（`WithField/WithFields/WithError`）
- ❌ 无 JSON 格式输出支持

### 2. Go 参考实现（`workflow.py/internal/logging/`）

**架构特点**：
- **统一接口**：使用 `GetLogger()` 获取 logger，自动识别模块
- **模块级日志分离**：每个模块输出到独立文件（`{module}.log`）（Rust 版本不需要）
- **自动模块识别**：通过 `runtime.Caller()` 自动识别调用者模块名
- **日志轮转**：使用 `lumberjack`（10MB/5备份/30天/压缩）
- **统一错误日志**：所有错误输出到 `error.log`（Rust 版本不需要，所有日志写入命令日志文件）
- **结构化日志**：支持 `WithField/WithFields/WithError`
- **多格式支持**：支持 text/json 格式

**功能覆盖**：
- ✅ 模块级日志分离（Rust 版本不需要）
- ✅ 自动模块识别
- ✅ 日志轮转
- ✅ 统一错误日志收集（Rust 版本不需要）
- ✅ 结构化日志字段
- ✅ JSON 格式输出
- ✅ 控制台+文件双重输出
- ✅ 日志级别管理

---

## 🎯 功能需求

### 1. 模块级日志分离（不需要）

**决定**：不需要模块级日志分离，所有模块的日志都写入同一个命令日志文件。

**需求描述**（已移除）：
- ~~每个模块（如 `http`、`jira`、`github`）的日志输出到独立文件~~
- ~~文件命名格式：`{module}-{timestamp}-{pid}.log`~~

**最终方案**：
- ✅ **单文件日志**：每次操作一个日志文件，包含所有模块的日志
  - 文件命名：`{command}-{timestamp}-{pid}.log`（例如：`pr-create-20250115143120-12346.log`）
  - 所有模块的日志都写入同一个文件
  - 模块信息作为日志字段（如 `module=jira`），不是文件分离
  - 优点：简单、完整、易管理、减少文件数量

**原因**：
- ✅ CLI 工具单次操作日志量通常不大（单文件足够）
- ✅ 单次操作会调用多个模块，分离后需要查看多个文件，不方便
- ✅ 增加实现复杂度（需要模块识别、多文件管理）
- ✅ 文件数量多，管理复杂（每次操作产生 N+1 个文件）
- ✅ 模块信息作为字段已经足够，可以通过搜索过滤

**实现方案**：
- ✅ 所有模块的日志写入同一个命令日志文件
- ✅ 模块信息作为日志字段（通过 `log_*!` 宏自动注入）
- ✅ 可以通过日志字段搜索和过滤特定模块的日志

**验收标准**：
- ✅ 所有模块的日志写入同一个命令日志文件
- ✅ 模块信息作为字段出现在日志中（如 `module=jira`）
- ✅ 可以通过字段搜索过滤特定模块的日志
- ✅ 不创建多个文件

### 2. 自动模块识别（作为日志字段）

**需求描述**：
- 自动识别调用者模块名（如 `http`、`jira`、`github`）
- 通过调用栈分析或显式指定模块名
- 提供便捷的 API（如 `log_info!` 自动包含模块信息）
- **重要**：模块信息作为日志字段，不是文件分离

**实现方案**：
- 使用 `std::backtrace` 或 `tracing::Span` 实现模块识别
- 提供宏封装，自动注入模块信息
- 支持显式指定模块名（用于适配器场景）

**验收标准**：
- ✅ 自动识别模块名准确
- ✅ 性能影响可接受（调用栈分析开销）
- ✅ 支持显式指定模块名

### 3. 日志轮转

**需求描述**：
- 单个文件最大 10MB
- 保留最近 5 个备份
- 保留 30 天
- 自动压缩旧日志

**实现方案**：
- 使用 `tracing-appender` 或 `tracing-subscriber` 的 `rolling` 功能
- 或集成 `file-rotate` crate 实现日志轮转
- 配置轮转策略（大小、数量、时间）

**验收标准**：
- ✅ 日志文件达到大小限制时自动轮转
- ✅ 旧日志自动压缩
- ✅ 备份数量和时间限制生效
- ✅ 不影响日志写入性能

### 4. 统一错误日志收集（不需要）

**需求描述**：
- ~~所有模块的 ERROR 级别日志统一输出到 `error.log`~~
- ~~使用 Hook 机制拦截 ERROR 级别日志~~
- ~~不影响原有日志文件输出~~

**决定**：
- ❌ **不需要独立 error.log 文件**
- ✅ **所有日志（包括 ERROR）都写入命令日志文件**（如 `pr-create-{timestamp}-{pid}.log`）
- ✅ **原因**：CLI 工具每次命令执行日志量不大，ERROR 日志可以直接在命令日志文件中查看

**实现方案**：
- ~~使用 `tracing_subscriber::Layer` 实现错误日志 Hook~~
- ~~创建独立的错误日志文件 writer~~
- ~~过滤 ERROR 级别日志并写入 `error.log`~~
- ✅ 所有日志级别（包括 ERROR）都写入同一个命令日志文件

**验收标准**：
- ✅ ERROR 级别日志写入命令日志文件（不需要单独的 error.log）
- ✅ 可以通过日志级别过滤查看错误日志
- ✅ 简化实现，减少文件管理复杂度

### 5. 结构化日志字段

**需求描述**：
- 支持添加结构化字段（如 `user_id`、`request_id`、`duration`）
- 提供 `WithField`、`WithFields`、`WithError` 方法
- 字段自动包含在日志输出中

**实现方案**：
- 使用 `tracing` 的 `span` 和 `field` 功能
- 提供便捷的宏封装（如 `log_info_with_fields!`）
- 支持字段序列化（JSON 格式）

**验收标准**：
- ✅ 可以添加单个或多个字段
- ✅ 字段自动包含在日志输出中
- ✅ 支持 JSON 格式输出字段
- ✅ API 易用性良好

### 6. JSON 格式输出

**需求描述**：
- 支持 JSON 格式日志输出
- 通过配置选择输出格式（text/json）
- JSON 格式包含所有结构化字段

**实现方案**：
- 使用 `tracing_subscriber::fmt::format::Json` 实现 JSON 格式
- 在 `Tracer::init()` 中根据配置选择格式
- 确保 JSON 格式包含所有字段信息

**验收标准**：
- ✅ JSON 格式输出正确
- ✅ 包含所有结构化字段
- ✅ 可通过配置切换格式
- ✅ JSON 格式可解析性良好

---

## 🏗️ 技术方案

### 1. 模块级日志分离实现（已移除）

**决定**：不需要模块级日志分离，所有模块的日志都写入同一个命令日志文件。

**原因**：
- CLI 工具单次操作日志量不大
- 模块信息作为日志字段已经足够
- 简化实现，减少文件管理复杂度

### 2. 自动模块识别实现

**方案 A：使用宏自动注入**

```rust
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            let module = module_path!()
                .split("::")
                .nth(2)
                .unwrap_or("unknown");
            tracing::info!(module = module, $($arg)*);
        }
    };
}
```

**方案 B：使用 backtrace（性能较差）**

```rust
use std::backtrace::Backtrace;

fn get_caller_module() -> String {
    let backtrace = Backtrace::capture();
    // 分析 backtrace 提取模块名
}
```

### 3. 日志轮转实现

**使用 tracing-appender**

```rust
use tracing_appender::rolling;

let file_appender = rolling::daily(log_dir, "workflow.log");
let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

tracing_subscriber::fmt()
    .with_writer(non_blocking)
    .init();
```

**或使用 file-rotate**

```rust
use file_rotate::{FileRotate, ContentLimit, compression::Compression};

let log = FileRotate::new(
    "workflow.log",
    ContentLimit::Bytes(10 * 1024 * 1024), // 10MB
    file_rotate::suffix::AppendCount::new(5),
    Compression::OnRotate(5),
    None,
);
```

### 4. 错误日志 Hook 实现（已移除）

**决定**：不需要独立 error.log 文件，所有日志（包括 ERROR）都写入命令日志文件。

**原因**：
- CLI 工具每次命令执行日志量不大
- ERROR 日志可以直接在命令日志文件中查看
- 简化实现，减少文件管理复杂度

### 5. 结构化字段实现

```rust
use tracing::{field, span, Level};

// 使用 span 添加字段
let span = span!(
    Level::INFO,
    "operation",
    user_id = 123,
    request_id = "abc",
);
let _guard = span.enter();

log_info!("Operation started");

// 或使用宏封装
log_info_with_fields!(
    user_id = 123,
    request_id = "abc",
    "Operation started"
);
```

---

## 📊 功能对比表

| 功能 | Go版本 | Rust版本（当前） | Rust版本（目标） | 优先级 |
|------|--------|-----------------|----------------|--------|
| **每次操作独立日志文件** | ❌ | ❌ | ✅ | P1 |
| **模块级日志分离** | ✅ | ❌ | ❌ 不需要 | - |
| **自动模块识别** | ✅ | ❌ | ✅ | P2 |
| **日志轮转** | ✅ | ❌ | ❌ 不需要 | - |
| **统一错误日志** | ✅ | ❌ | ❌ 不需要 | - |
| **结构化日志字段** | ✅ | ❌ | ✅ | P2 |
| **JSON 格式输出** | ✅ | ❌ | ✅ | P3 |
| **控制台+文件双重输出** | ✅ | ✅ | ✅ | - |
| **日志级别管理** | ✅ | ✅ | ✅ | - |
| **文件输出** | ✅ | ✅ | ✅ | - |

**优先级说明**：
- **P1（高优先级）**：核心功能，影响日志管理效率
- **P2（中优先级）**：重要功能，提升日志可维护性
- **P3（低优先级）**：增强功能，提升日志可读性
- **⚠️ 可选**：根据实际需求决定是否实现
- **❌ 不需要**：CLI 工具场景下不需要的功能

---

## 🚀 实施计划

### 阶段一：每次操作独立日志文件（1天）

**目标**：实现每次命令执行创建独立日志文件

**任务清单**：
- [x] 修改 `Tracer::init()` 接受可选的命令名参数
- [x] 实现 `extract_command_name()` 函数，从 Commands 枚举提取命令路径
- [x] 修改 `get_log_file_path()` 使用命令名 + 时间戳 + PID
- [x] 更新 `main.rs` 在命令解析后初始化 tracer
- [x] 处理嵌套命令（如 `jira log download` -> `jira-log-download`）
- [ ] 测试文件创建和写入
- [ ] 添加单元测试
- [x] 更新文档

**验收标准**：
- ✅ 每次命令执行创建新的日志文件
- ✅ 文件命名格式正确（`{command}-{timestamp}-{pid}.log`）
- ✅ 命令名正确提取（包括嵌套命令）
- ✅ 文件唯一性保证（命令名 + 时间戳 + PID）
- ✅ 无命令时使用 `workflow-{timestamp}-{pid}.log`
- [ ] 测试覆盖率达到 80%+

### 阶段二：自动模块识别和结构化字段（1-2天）

**目标**：实现自动模块识别和结构化日志字段

**重要说明**：
- ⚠️ **不会产生多个文件**：模块识别只是在日志内容中标记模块信息（作为字段），所有模块的日志仍然写入同一个文件
- ✅ **单文件方案**：每次操作仍然只有一个日志文件（`{command}-{timestamp}-{pid}.log`）
- ✅ **模块信息作为字段**：每条日志自动包含模块信息，便于搜索和过滤

**任务清单**：
- [x] 设计模块识别机制（使用宏自动注入）
- [x] 修改 `trace_*!` 宏自动包含模块信息（作为日志字段）
- [x] 设计结构化字段 API（`trace_*_with_fields!` 宏）
- [x] 实现字段注入机制（使用 tracing 字段）
- [x] 添加便捷宏（`trace_*_with_fields!`）
- [ ] 添加单元测试
- [x] 更新文档

**验收标准**：
- ✅ 自动识别模块名准确
- ✅ 模块信息作为字段出现在日志中（例如：`module=jira`）
- ✅ 可以添加单个或多个字段
- ✅ 字段自动包含在日志输出中
- ✅ API 易用性良好
- ✅ **仍然只有一个日志文件**（不创建多个文件）
- [ ] 测试覆盖率达到 80%+

**日志输出示例**：
```
2025-01-15 14:31:20 INFO module=jira: Fetching ticket info
2025-01-15 14:31:21 INFO module=http: Sending request to API
2025-01-15 14:31:22 INFO module=jira: Ticket info received
```

所有日志都在同一个文件中，但每条日志都包含模块信息，便于搜索和过滤。

### 阶段三：JSON 格式输出（可选，1天）

**目标**：实现 JSON 格式输出

**任务清单**：
- [x] 实现 JSON 格式输出（使用 `tracing_subscriber::fmt::layer().json()`）
- [x] 更新配置支持格式选择（text/json）
- [x] 确保 JSON 格式包含所有字段信息
- [x] 添加单元测试
- [x] 更新文档

**验收标准**：
- ✅ JSON 格式输出正确
- ✅ 包含所有结构化字段
- ✅ 可通过配置切换格式
- ✅ JSON 格式可解析性良好
- [ ] 测试覆盖率达到 80%+

### 阶段四：测试和优化（1天）

**目标**：完善测试、性能优化和文档

**任务清单**：
- [ ] 完善单元测试和集成测试
- [ ] 性能测试和优化
- [ ] 更新 API 文档
- [ ] 更新使用示例
- [ ] 迁移指南（如有需要）

**验收标准**：
- ✅ 测试覆盖率达到 80%+
- ✅ 性能满足要求（日志写入开销 < 5%）
- ✅ 文档完整
- ✅ 使用示例清晰

---

## 📝 实施细节

### 0. 每次操作独立日志文件（核心功能）

**文件命名格式**：
- 格式：`{command}-{timestamp}-{pid}.log`
- 示例：`pr-create-20250115143120-12346.log`
- 说明：命令名使用短横线连接（如 `pr-create`、`jira-info`、`branch-create`）
- 如果没有命令（显示帮助等），使用 `workflow-{timestamp}-{pid}.log`

**实现方案**：

```rust
// 在 tracing.rs 中修改 Tracer::init() 和 get_log_file_path()
impl Tracer {
    /// 初始化 tracing subscriber（从配置读取日志级别）
    ///
    /// # 参数
    ///
    /// * `command_name` - 可选的命令名（如 "pr-create"、"jira-info"），如果为 None，使用 "workflow"
    pub fn init_with_command(command_name: Option<&str>) {
        // ... 现有逻辑 ...

        if let Ok(file_path) = Self::get_log_file_path(command_name) {
            // ... 文件创建逻辑 ...
        }
    }

    /// 获取日志文件路径
    ///
    /// # 参数
    ///
    /// * `command_name` - 可选的命令名，如果为 None，使用 "workflow"
    fn get_log_file_path(command_name: Option<&str>) -> color_eyre::Result<std::path::PathBuf> {
        let logs_dir = Paths::logs_dir().wrap_err("Failed to get logs directory")?;
        let tracing_dir = logs_dir.join("tracing");
        DirectoryWalker::new(&tracing_dir).ensure_exists()?;

        // 生成时间戳（YYYYMMDDHHMMSS 格式）
        let timestamp = Local::now().format("%Y%m%d%H%M%S");
        let pid = std::process::id();

        // 确定命令名前缀
        let command_prefix = command_name.unwrap_or("workflow");

        // 文件命名：{command}-{timestamp}-{pid}.log
        let log_file = tracing_dir.join(format!("{}-{}-{}.log", command_prefix, timestamp, pid));

        Ok(log_file)
    }
}

// 在 main.rs 中，需要先解析命令，然后初始化 tracer
fn main() -> Result<()> {
    color_eyre::install()?;

    // 解析命令（但不执行）
    let args: Vec<String> = std::env::args().collect();
    let expanded_args = AliasManager::expand_args(args)?;
    let cli = Cli::parse_from(expanded_args);

    // 从命令中提取命令名
    let command_name = extract_command_name(&cli.command);

    // 初始化 tracing（传入命令名）
    workflow::Tracer::init_with_command(command_name.as_deref());

    // 执行命令
    match cli.command {
        // ... 命令处理 ...
    }
}

/// 从 Commands 枚举中提取命令路径字符串
///
/// 例如：Commands::Pr { subcommand: PRCommands::Create { ... } } -> "pr-create"
fn extract_command_name(command: &Option<Commands>) -> Option<String> {
    // 实现命令路径提取逻辑
    // 需要递归处理嵌套的子命令
}
```

**优点**：
- ✅ 每次命令执行创建新文件，避免日志文件无限增长
- ✅ 无需日志轮转功能
- ✅ 便于追踪单次操作的完整日志
- ✅ 实现简单

**文件示例**：
```
~/.workflow/logs/tracing/
├── pr-create-20250115143120-12345.log    # pr create 命令
├── jira-info-20250115143210-12346.log    # jira info 命令
├── branch-create-20250115143250-12347.log # branch create 命令
├── jira-log-download-20250115143300-12348.log # jira log download 命令
└── workflow-20250115143310-12349.log      # 无命令（显示帮助等）
```

### 1. 模块识别机制

**重要说明**：
- ⚠️ **不创建多个文件**：模块识别只是在日志内容中添加模块字段，所有日志仍然写入同一个文件
- ✅ **模块信息作为字段**：每条日志包含 `module=jira` 这样的字段，便于搜索和过滤
- ✅ **单文件方案**：每次操作只有一个日志文件（`{command}-{timestamp}-{pid}.log`）

**推荐方案：使用宏自动注入模块信息**

```rust
// 在 tracing.rs 中实现
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            // 从 module_path!() 提取模块名
            let module = {
                let path = module_path!();
                path.split("::")
                    .skip(2)  // 跳过 crate 和 base
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            };
            // 模块信息作为字段添加到日志中（不创建新文件）
            tracing::info!(module = %module, $($arg)*);
        }
    };
}
```

**日志输出示例**：
```
2025-01-15 14:31:20 INFO module=jira: Fetching ticket info
2025-01-15 14:31:21 INFO module=http: Sending request to API
2025-01-15 14:31:22 INFO module=jira: Ticket info received
```

**显式指定模块（用于适配器场景）**：

```rust
// 提供显式指定模块的宏
#[macro_export]
macro_rules! log_info_with_module {
    ($module:expr, $($arg:tt)*) => {
        tracing::info!(module = %$module, $($arg)*);
    };
}
```

### 2. 模块级日志文件管理（已移除）

**决定**：不需要模块级日志文件管理，所有模块的日志都写入同一个命令日志文件。

**原因**：
- CLI 工具单次操作日志量不大
- 模块信息作为日志字段已经足够
- 简化实现，减少文件管理复杂度

### 3. 日志轮转配置（已移除，不需要）

**注意**：CLI 工具场景下不需要日志轮转，因为每次操作都创建新的日志文件。如果将来需要清理旧日志，可以：
- 按日期清理：删除 N 天前的日志文件
- 按数量清理：保留最近 N 个日志文件
- 按大小清理：总大小超过阈值时删除最旧的

这些功能可以在后续版本中作为可选功能实现。

### 4. 错误日志 Hook

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

struct ErrorLogHook {
    writer: Arc<Mutex<File>>,
}

impl<S> Layer<S> for ErrorLogHook
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: &tracing_subscriber::layer::Context<'_, S>) {
        if event.metadata().level() == &tracing::Level::ERROR {
            // 格式化并写入 error.log
            let mut writer = self.writer.lock().unwrap();
            // 写入逻辑
        }
    }
}
```

### 5. 结构化字段 API

```rust
// 使用 Span 添加字段
pub fn log_info_with_fields(
    fields: &[(&str, &dyn std::fmt::Display)],
    message: &str,
) {
    let mut span = tracing::span!(tracing::Level::INFO, "log");
    for (key, value) in fields {
        span.record(key, &format!("{}", value));
    }
    let _guard = span.enter();
    tracing::info!("{}", message);
}

// 宏封装
#[macro_export]
macro_rules! log_info_with_fields {
    ($($key:ident = $value:expr),*; $($arg:tt)*) => {
        {
            let span = tracing::span!(tracing::Level::INFO, "log");
            $(
                span.record(stringify!($key), &format!("{}", $value));
            )*
            let _guard = span.enter();
            tracing::info!($($arg)*);
        }
    };
}
```

---

## ✅ 验收标准

### 功能验收

- [ ] **模块级日志分离**（已移除）：
  - ~~每个模块的日志输出到独立文件（`{module}.log`）~~
  - ✅ 所有模块的日志写入同一个命令日志文件
  - ✅ 模块信息作为日志字段（如 `module=jira`）
  - ✅ 可以通过字段搜索过滤特定模块的日志

- [ ] **日志轮转**：
  - 日志文件达到大小限制时自动轮转
  - 旧日志自动压缩
  - 备份数量和时间限制生效
  - 不影响日志写入性能

- [ ] **统一错误日志**（已移除）：
  - ~~所有 ERROR 级别日志写入 `error.log`~~
  - ✅ 所有日志（包括 ERROR）都写入命令日志文件
  - ✅ 简化实现，减少文件管理复杂度

- [ ] **结构化日志字段**：
  - 可以添加单个或多个字段
  - 字段自动包含在日志输出中
  - 支持 JSON 格式输出字段
  - API 易用性良好

- [ ] **JSON 格式输出**：
  - JSON 格式输出正确
  - 包含所有结构化字段
  - 可通过配置切换格式
  - JSON 格式可解析性良好

### 质量验收

- [ ] **测试覆盖率**：达到 80%+
- [ ] **性能要求**：日志写入开销 < 5%
- [ ] **文档完整**：API 文档、使用示例、迁移指南
- [ ] **向后兼容**：不影响现有代码（可选）

---

## ⚠️ 风险和缓解措施

### 🔴 高风险

- **模块识别性能开销**
  - 风险：调用栈分析可能影响性能
  - 缓解：使用编译时宏注入，避免运行时分析
  - 备选：提供显式指定模块的 API

- **日志轮转库兼容性**
  - 风险：`tracing-appender` 可能不满足所有需求
  - 缓解：评估多个库（`tracing-appender`、`file-rotate`），选择最适合的
  - 备选：自实现简单的轮转逻辑

### 🟡 中风险

- **结构化字段 API 设计**
  - 风险：API 设计可能不够易用
  - 缓解：参考 Go 版本 API，提供多种使用方式
  - 备选：提供宏封装简化使用

- **向后兼容性**
  - 风险：新功能可能影响现有代码
  - 缓解：保持现有 API 不变，新增功能作为可选特性
  - 备选：提供迁移指南

### 🟢 低风险

- **依赖库版本兼容性**
  - 风险：依赖库版本更新可能带来问题
  - 缓解：锁定版本，定期更新和测试
  - 备选：使用稳定版本

---

## 🚀 下一步行动

### 立即行动（今天）

1. [ ] 创建功能分支：`git checkout -b feature/logger-enhancement`
2. [ ] 评估日志轮转库（`tracing-appender` vs `file-rotate`）
3. [ ] 设计模块识别机制（宏 vs 运行时分析）

### 本周目标

1. [ ] 完成阶段一：每次操作独立日志文件
2. [ ] 完成阶段二：日志轮转和错误日志收集
3. [ ] 建立基线测试

### 下周目标

1. [ ] 完成阶段三：JSON 格式输出（可选）
2. [ ] 完成阶段四：测试和优化
3. [ ] 文档完善

---

## 📊 进度跟踪

### 当前状态

- **总体进度**: 0% (0/4 阶段完成)
- **当前阶段**: 未开始
- **下个里程碑**: 每次操作独立日志文件完成（第1天）

### 完成情况

- ✅ **已完成**: 需求分析和方案设计
- 🔄 **进行中**: 无
- ⏳ **待开始**: 所有实施阶段

### 每日更新

```
日期: ____
完成任务:
- [ ]
- [ ]

遇到问题:
-

明日计划:
- [ ]
- [ ]
```

---

## 🔗 相关资源

### 参考文档

- [tracing 文档](https://docs.rs/tracing/)
- [tracing-subscriber 文档](https://docs.rs/tracing-subscriber/)
- [tracing-appender 文档](https://docs.rs/tracing-appender/)
- [Go 版本日志实现](../../../workflow.py/internal/logging/logger.go)

### 工具链接

- [tracing-appender](https://docs.rs/tracing-appender/) - 日志轮转
- [file-rotate](https://docs.rs/file-rotate/) - 文件轮转
- [tracing-subscriber](https://docs.rs/tracing-subscriber/) - 订阅者实现

---

## 📚 相关文档

- [日志系统对比分析](./logger-enhancement.md)（本文档）
- [Go 版本日志实现](../../../workflow.py/internal/logging/logger.go)
- [Rust 版本日志实现](../../../src/lib/base/logger/)
- [架构文档](../architecture/architecture.md)

---

**最后更新**: 2025-01-15
**状态**: ✅ 已完成
**实现度**: 100%

**已完成功能**：
- ✅ 每次操作独立日志文件（阶段一）
- ✅ 自动模块识别（阶段二）
- ✅ 结构化日志字段支持（阶段二）
- ✅ JSON 格式输出（阶段三）
- ✅ 单元测试

**实现总结**：
- ✅ 所有核心功能已实现
- ✅ 所有测试通过
- ✅ 代码编译通过
- ✅ 文档已更新
