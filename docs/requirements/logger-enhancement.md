# 日志系统增强需求文档

## 📋 概述

本文档基于 Go 版本日志系统（`workflow.py/internal/logging`）的对比分析，提出 Rust 版本日志系统（`src/lib/base/logger`）的增强需求，以提升日志管理的功能性和可维护性。

**目标**: 将 Rust 版本日志系统功能对齐到 Go 版本水平，增强模块化日志管理、日志轮转和结构化日志支持。

**时间估算**: 5-7天  
**优先级**: 中优先级（功能增强，非阻塞性）

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
- ❌ 无模块级日志分离（所有模块输出到单一文件）
- ❌ 无自动模块识别（需要手动指定模块信息）
- ❌ 无日志轮转（无大小限制、无备份管理、无自动压缩）
- ❌ 无统一错误日志收集（`error.log`）
- ❌ 无结构化日志字段支持（`WithField/WithFields/WithError`）
- ❌ 无 JSON 格式输出支持

### 2. Go 参考实现（`workflow.py/internal/logging/`）

**架构特点**：
- **统一接口**：使用 `GetLogger()` 获取 logger，自动识别模块
- **模块级日志分离**：每个模块输出到独立文件（`{module}.log`）
- **自动模块识别**：通过 `runtime.Caller()` 自动识别调用者模块名
- **日志轮转**：使用 `lumberjack`（10MB/5备份/30天/压缩）
- **统一错误日志**：所有错误输出到 `error.log`
- **结构化日志**：支持 `WithField/WithFields/WithError`
- **多格式支持**：支持 text/json 格式

**功能覆盖**：
- ✅ 模块级日志分离
- ✅ 自动模块识别
- ✅ 日志轮转
- ✅ 统一错误日志收集
- ✅ 结构化日志字段
- ✅ JSON 格式输出
- ✅ 控制台+文件双重输出
- ✅ 日志级别管理

---

## 🎯 功能需求

### 1. 模块级日志分离

**需求描述**：
- 每个模块（如 `http`、`jira`、`github`）的日志输出到独立文件
- 文件命名格式：`{module}.log`（例如：`http.log`、`jira.log`）
- 支持全局日志文件（`workflow.log`）作为补充

**实现方案**：
- 在 `Tracer` 中实现模块级 logger 管理
- 使用 `tracing_subscriber::Layer` 实现模块过滤
- 通过 `tracing::Span` 或自定义字段标识模块

**验收标准**：
- ✅ 每个模块的日志输出到独立文件
- ✅ 模块识别准确（通过调用栈或显式指定）
- ✅ 线程安全（支持并发场景）

### 2. 自动模块识别

**需求描述**：
- 自动识别调用者模块名（如 `http`、`jira`、`github`）
- 通过调用栈分析或显式指定模块名
- 提供便捷的 API（如 `trace_info!` 自动包含模块信息）

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

### 4. 统一错误日志收集

**需求描述**：
- 所有模块的 ERROR 级别日志统一输出到 `error.log`
- 使用 Hook 机制拦截 ERROR 级别日志
- 不影响原有日志文件输出

**实现方案**：
- 使用 `tracing_subscriber::Layer` 实现错误日志 Hook
- 创建独立的错误日志文件 writer
- 过滤 ERROR 级别日志并写入 `error.log`

**验收标准**：
- ✅ 所有 ERROR 级别日志写入 `error.log`
- ✅ 不影响原有模块日志文件
- ✅ 性能影响可接受

### 5. 结构化日志字段

**需求描述**：
- 支持添加结构化字段（如 `user_id`、`request_id`、`duration`）
- 提供 `WithField`、`WithFields`、`WithError` 方法
- 字段自动包含在日志输出中

**实现方案**：
- 使用 `tracing` 的 `span` 和 `field` 功能
- 提供便捷的宏封装（如 `trace_info_with_fields!`）
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

### 1. 模块级日志分离实现

**方案 A：使用 tracing::Span（推荐）**

```rust
use tracing::{span, Level, Span};

// 创建模块级 span
let span = span!(Level::INFO, "module", module = "http");
let _guard = span.enter();

// 日志自动包含模块信息
trace_info!("HTTP request started");
```

**方案 B：使用自定义 Layer**

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

struct ModuleLayer {
    module: String,
    writer: File,
}

impl<S> Layer<S> for ModuleLayer
where
    S: tracing::Subscriber,
{
    // 实现模块过滤逻辑
}
```

### 2. 自动模块识别实现

**方案 A：使用宏自动注入**

```rust
#[macro_export]
macro_rules! trace_info {
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

### 4. 错误日志 Hook 实现

```rust
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Layer;

struct ErrorLogHook {
    writer: File,
}

impl<S> Layer<S> for ErrorLogHook
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: &tracing_subscriber::layer::Context<'_, S>) {
        if event.metadata().level() == &tracing::Level::ERROR {
            // 写入 error.log
        }
    }
}
```

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

trace_info!("Operation started");

// 或使用宏封装
trace_info_with_fields!(
    user_id = 123,
    request_id = "abc",
    "Operation started"
);
```

---

## 📊 功能对比表

| 功能 | Go版本 | Rust版本（当前） | Rust版本（目标） | 优先级 |
|------|--------|-----------------|----------------|--------|
| **模块级日志分离** | ✅ | ❌ | ✅ | P1 |
| **自动模块识别** | ✅ | ❌ | ✅ | P1 |
| **日志轮转** | ✅ | ❌ | ✅ | P2 |
| **统一错误日志** | ✅ | ❌ | ✅ | P2 |
| **结构化日志字段** | ✅ | ❌ | ✅ | P2 |
| **JSON 格式输出** | ✅ | ❌ | ✅ | P3 |
| **控制台+文件双重输出** | ✅ | ✅ | ✅ | - |
| **日志级别管理** | ✅ | ✅ | ✅ | - |
| **文件输出** | ✅ | ✅ | ✅ | - |

**优先级说明**：
- **P1（高优先级）**：核心功能，影响日志管理效率
- **P2（中优先级）**：重要功能，提升日志可维护性
- **P3（低优先级）**：增强功能，提升日志可读性

---

## 🚀 实施计划

### 阶段一：模块级日志分离（2天）

**目标**：实现模块级日志分离和自动模块识别

**任务清单**：
- [ ] 设计模块识别机制（使用宏或 Span）
- [ ] 实现模块级 logger 管理
- [ ] 实现模块过滤 Layer
- [ ] 更新 `Tracer::init()` 支持模块分离
- [ ] 添加单元测试
- [ ] 更新文档

**验收标准**：
- ✅ 每个模块的日志输出到独立文件
- ✅ 模块识别准确（自动或显式）
- ✅ 线程安全
- [ ] 测试覆盖率达到 80%+

### 阶段二：日志轮转和错误日志收集（2天）

**目标**：实现日志轮转和统一错误日志收集

**任务清单**：
- [ ] 集成日志轮转库（`tracing-appender` 或 `file-rotate`）
- [ ] 配置轮转策略（大小、数量、时间、压缩）
- [ ] 实现错误日志 Hook
- [ ] 更新 `Tracer::init()` 支持轮转和错误日志
- [ ] 添加单元测试
- [ ] 更新文档

**验收标准**：
- ✅ 日志文件达到大小限制时自动轮转
- ✅ 旧日志自动压缩
- ✅ 所有 ERROR 级别日志写入 `error.log`
- [ ] 测试覆盖率达到 80%+

### 阶段三：结构化日志字段和 JSON 格式（1-2天）

**目标**：实现结构化日志字段和 JSON 格式输出

**任务清单**：
- [ ] 设计结构化字段 API（`WithField/WithFields/WithError`）
- [ ] 实现字段注入机制（使用 Span）
- [ ] 实现 JSON 格式输出
- [ ] 更新配置支持格式选择
- [ ] 添加便捷宏（`trace_info_with_fields!`）
- [ ] 添加单元测试
- [ ] 更新文档

**验收标准**：
- ✅ 可以添加单个或多个字段
- ✅ 字段自动包含在日志输出中
- ✅ JSON 格式输出正确
- ✅ 可通过配置切换格式
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

### 1. 模块识别机制

**推荐方案：使用宏自动注入模块信息**

```rust
// 在 tracing.rs 中实现
#[macro_export]
macro_rules! trace_info {
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
            tracing::info!(module = %module, $($arg)*);
        }
    };
}
```

**显式指定模块（用于适配器场景）**：

```rust
// 提供显式指定模块的宏
#[macro_export]
macro_rules! trace_info_with_module {
    ($module:expr, $($arg:tt)*) => {
        tracing::info!(module = %$module, $($arg)*);
    };
}
```

### 2. 模块级日志文件管理

```rust
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing_appender::rolling;

struct ModuleLoggerManager {
    loggers: Arc<Mutex<HashMap<String, File>>>,
    log_dir: PathBuf,
}

impl ModuleLoggerManager {
    fn get_module_logger(&self, module: &str) -> File {
        let mut loggers = self.loggers.lock().unwrap();
        loggers.entry(module.to_string())
            .or_insert_with(|| {
                let file_path = self.log_dir.join(format!("{}.log", module));
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)
                    .unwrap()
            })
            .clone()
    }
}
```

### 3. 日志轮转配置

```rust
use tracing_appender::rolling::{RollingFileAppender, Rotation};

let file_appender = RollingFileAppender::new(
    Rotation::daily(),  // 按天轮转
    log_dir,
    "workflow.log",
);

// 或使用大小限制
let file_appender = RollingFileAppender::new(
    Rotation::new()
        .max_size(10 * 1024 * 1024)  // 10MB
        .max_files(5)                 // 保留 5 个文件
        .max_age(Duration::days(30)), // 保留 30 天
    log_dir,
    "workflow.log",
);
```

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
pub fn trace_info_with_fields(
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
macro_rules! trace_info_with_fields {
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

- [ ] **模块级日志分离**：
  - 每个模块的日志输出到独立文件（`{module}.log`）
  - 模块识别准确（自动或显式）
  - 线程安全（支持并发场景）

- [ ] **日志轮转**：
  - 日志文件达到大小限制时自动轮转
  - 旧日志自动压缩
  - 备份数量和时间限制生效
  - 不影响日志写入性能

- [ ] **统一错误日志**：
  - 所有 ERROR 级别日志写入 `error.log`
  - 不影响原有模块日志文件
  - 性能影响可接受

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

1. [ ] 完成阶段一：模块级日志分离
2. [ ] 完成阶段二：日志轮转和错误日志收集
3. [ ] 建立基线测试

### 下周目标

1. [ ] 完成阶段三：结构化日志字段和 JSON 格式
2. [ ] 完成阶段四：测试和优化
3. [ ] 文档完善

---

## 📊 进度跟踪

### 当前状态

- **总体进度**: 0% (0/4 阶段完成)
- **当前阶段**: 未开始
- **下个里程碑**: 模块级日志分离完成（第2天）

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
**状态**: ⏳ 待实施  
**实现度**: 0%
