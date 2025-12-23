# 日志命令模块架构文档

## 📋 概述

日志命令层是 Workflow CLI 的命令接口，提供从 Jira ticket 下载日志、查找请求 ID、搜索关键词等功能。该层采用命令模式设计，通过调用 `lib/jira/logs/` 模块提供的 API 实现业务功能。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/jira/logs/` 模块提供。

**命令结构**：
- `workflow log` - 日志操作命令（download, find, search）

---

## 📁 相关文件

### CLI 入口层

日志命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Log` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow log` 子命令分发到对应的命令处理函数

### 命令封装层 (`commands/log/`)

```
src/commands/log/
├── mod.rs          # Log 命令模块声明
├── download.rs     # 下载日志命令（29 行）
├── find.rs         # 查找请求 ID 命令（45 行）
└── search.rs       # 搜索关键词命令（97 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/jira/logs/`) 的 API

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/jira/logs/`**：Jira 日志处理模块（`JiraLogs`）
  - `JiraLogs::new()` - 创建日志管理器
  - `JiraLogs::download-_from-_jira()` - 下载日志
  - `JiraLogs::extract-_response-_content()` - 提取响应内容
  - `JiraLogs::search-_keyword-_both-_files()` - 同时搜索 api.log 和 flutter-api.log
  - `JiraLogs::ensure-_log-_file-_exists()` - 确保日志文件存在
  - `JiraLogs::get-_api-_log-_file-_path()` - 获取 api.log 文件路径
- **`lib/base/util/`**：工具函数
  - `Clipboard::copy()` - 复制到剪贴板
- **`lib/base/settings/`**：配置管理
  - `Settings::get()` - 获取配置（`log-_output-_folder-_name`、`log-_download-_base-_dir` 等）

详细架构文档：参见 [Jira 模块架构文档](../architecture/jira.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/log/*.rs (命令封装层，处理交互)
  ↓
lib/jira/logs/ (通过 JiraLogs API 调用，具体实现见相关模块文档)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand
  ├─ Download → DownloadCommand::download()
  ├─ Find → FindCommand::find-_request-_id()
  └─ Search → SearchCommand::search()
```

**注意**：`Clean` 命令已迁移到 `workflow jira` 子命令，请参考 [Jira 命令模块架构文档](./jira.md)。

---

## 1. 下载日志命令 (`download`)

### 相关文件

```
src/commands/log/download.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Download
  ↓
commands/log/download.rs::DownloadCommand::download(jira-_id)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 显示下载提示信息
  3. 创建 JiraLogs 实例：JiraLogs::new()
  4. 调用 JiraLogs::download-_from-_jira(jira-_id, None, false)
     └─ 内部处理：下载日志附件、合并分片、解压文件
  5. 输出成功信息和文件路径
```

### 功能说明

1. **参数处理**：
   - `jira-_id` - Jira ticket ID（可选，不提供时会交互式输入）

2. **用户交互**：
   - 如果未提供 `jira-_id`，使用 `dialoguer::Input` 交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 显示下载进度和结果

3. **核心功能**：
   - 通过 `JiraLogs::download-_from-_jira()` API 实现下载功能
   - 只下载日志附件（文件匹配 log.zip, *.log, *.txt 模式）
   - 自动处理附件下载、分片合并、文件解压等操作

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例（自动加载配置和初始化 HTTP 客户端）

2. **下载执行**：
   - 调用 `JiraLogs::download-_from-_jira()` 执行下载
   - 只下载日志附件（`download-_all-_attachments = false`）
   - 自动处理分片 ZIP 文件的合并和解压

3. **结果输出**：
   - 显示下载成功信息
   - 显示文件保存路径

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::download-_from-_jira(jira-_id, output-_folder, download-_all-_attachments)`** - 下载日志附件
  - 参数：
    - `jira-_id` - Jira ticket ID
    - `output-_folder` - 输出文件夹名称（可选，None 时使用配置的默认值）
    - `download-_all-_attachments` - 是否下载所有附件（false，只下载日志附件）
  - 返回：基础目录路径

---

## 2. 查找请求 ID 命令 (`find`)

### 相关文件

```
src/commands/log/find.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Find
  ↓
commands/log/find.rs::FindCommand::find-_request-_id(jira-_id, request-_id)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 创建 JiraLogs 实例：JiraLogs::new()
  3. 获取请求 ID（从参数或交互式输入）
  4. 调用 JiraLogs::extract-_response-_content(jira-_id, request-_id)
     └─ 内部处理：解析日志文件、查找请求 ID、提取响应内容
  5. Clipboard::copy() 复制响应内容到剪贴板
  6. 输出成功信息
```

### 功能说明

1. **参数处理**：
   - `jira-_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `request-_id` - 请求 ID（可选，不提供时交互式输入）

2. **用户交互**：
   - 如果未提供 `request-_id`，使用 `dialoguer::Input` 交互式输入
   - 显示查找进度和结果

3. **核心功能**：
   - 通过 `JiraLogs::extract-_response-_content()` API 提取响应内容
   - 自动复制响应内容到剪贴板

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例

2. **JIRA ID 和请求 ID 获取**：
   - 优先使用命令行参数
   - 如果未提供 `jira-_id`，交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 如果未提供 `request-_id`，交互式输入（提示："Enter request ID to find"）

3. **内容提取**：
   - 调用 `JiraLogs::extract-_response-_content()` 提取响应内容
   - 自动处理日志文件解析和内容提取

4. **剪贴板操作**：
   - 使用 `Clipboard::copy()` 复制响应内容到剪贴板
   - 注意：Linux ARM64 和 musl 静态链接版本不支持剪贴板功能（详见 [工具函数模块架构文档](../architecture/tools.md)）

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::extract-_response-_content(jira-_id, request-_id)`** - 提取响应内容
  - 参数：
    - `jira-_id` - Jira ticket ID
    - `request-_id` - 请求 ID
  - 返回：响应内容字符串

---

## 3. 搜索关键词命令 (`search`)

### 相关文件

```
src/commands/log/search.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Search
  ↓
commands/log/search.rs::SearchCommand::search(jira-_id, search-_term)
  ↓
  1. 获取 JIRA ID（从参数或交互式输入）
  2. 创建 JiraLogs 实例：JiraLogs::new()
  3. 确保日志文件存在：JiraLogs::ensure-_log-_file-_exists(jira-_id)
  4. 获取搜索词（从参数或交互式输入）
  5. 调用 JiraLogs::search-_keyword-_both-_files(jira-_id, search-_term)
     └─ 内部处理：同时搜索 api.log 和 flutter-api.log，解析日志文件、搜索关键词、收集匹配结果
  6. 格式化输出结果（按文件分组显示匹配的 URL 和 ID）
```

### 功能说明

1. **参数处理**：
   - `jira-_id` - Jira ticket ID（可选，不提供时会交互式输入）
   - `search-_term` - 搜索关键词（可选，不提供时交互式输入）

2. **用户交互**：
   - 如果未提供 `jira-_id`，使用 `dialoguer::Input` 交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 如果未提供 `search-_term`，使用 `dialoguer::Input` 交互式输入（提示："Enter search term"）
   - 格式化输出匹配结果（按 api.log 和 flutter-api.log 分组显示）

3. **核心功能**：
   - 通过 `JiraLogs::search-_keyword-_both-_files()` API 同时搜索两个日志文件
   - 同时搜索 api.log 和 flutter-api.log 两个文件
   - 自动去重和格式化输出
   - 按文件分组显示搜索结果

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例
   - 确保日志文件存在

2. **JIRA ID 和搜索词获取**：
   - 优先使用命令行参数
   - 如果未提供 `jira-_id`，交互式输入（提示："Enter Jira ticket ID (e.g., PROJ-123)"）
   - 如果未提供 `search-_term`，交互式输入（提示："Enter search term"）

3. **搜索执行**：
   - 调用 `JiraLogs::search-_keyword-_both-_files()` 执行搜索
   - 同时搜索 api.log 和 flutter-api.log 两个文件

4. **结果展示**：
   - 格式化输出匹配的日志条目
   - 显示请求 ID 和 URL 信息
   - 按日志文件分组显示

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::ensure-_log-_file-_exists(jira-_id)`** - 确保日志文件存在
  - 参数：`jira-_id` - Jira ticket ID
  - 返回：日志文件路径
- **`JiraLogs::get-_api-_log-_file-_path(jira-_id)`** - 获取 api.log 文件路径
  - 参数：`jira-_id` - Jira ticket ID
  - 返回：api.log 文件路径（基于 flutter-api.log 的路径在同一目录下查找）
- **`JiraLogs::search-_keyword-_both-_files(jira-_id, search-_term)`** - 同时搜索两个日志文件
  - 参数：
    - `jira-_id` - Jira ticket ID
    - `search-_term` - 搜索关键词
  - 返回：`(api-_results, flutter-_api-_results)` - 两个文件的结果元组，每个都是 `Vec<LogEntry>`
  - 说明：同时搜索 api.log 和 flutter-api.log，如果文件不存在则返回空结果（不报错）

---

### 数据流

#### Download 命令数据流

```
命令行参数 (JIRA_ID)
  ↓
DownloadCommand::download()
  ↓
JiraLogs::download-_from-_jira()
  ↓
Jira API (获取附件列表)
  ↓
下载日志附件到本地
  ↓
合并分片、解压文件
  ↓
输出文件路径
```

#### Find 命令数据流

```
命令行参数或交互式输入 (JIRA_ID, REQUEST_ID)
  ↓
FindCommand::find-_request-_id()
  ↓
JiraLogs::extract-_response-_content()
  ↓
解析日志文件、提取响应内容
  ↓
Clipboard::copy() 复制到剪贴板
  ↓
输出成功信息
```

#### Search 命令数据流

```
命令行参数或交互式输入 (JIRA_ID, SEARCH_TERM)
  ↓
SearchCommand::search()
  ↓
JiraLogs::search-_keyword-_both-_files()
  ↓
解析日志文件、搜索关键词
  ↓
收集匹配的 LogEntry
  ↓
格式化输出到终端
```

---

## 🏗️ 架构设计

### 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `DownloadCommand::download()` - 下载日志
- `FindCommand::find-_request-_id()` - 查找请求 ID
- `SearchCommand::search()` - 搜索关键词

**注意**：`Clean` 命令已迁移到 `workflow jira` 子命令，请参考 [Jira 命令模块架构文档](./jira.md)。

### 2. 分层调用模式

**命令层（CLI → Commands）**：
所有命令通过 `src/main.rs` 直接调用对应的命令结构体：
```
src/main.rs::main()
  ↓
match cli.subcommand
  ├─ Download → DownloadCommand::download()
  ├─ Find → FindCommand::find-_request-_id()
  └─ Search → SearchCommand::search()
```

**注意**：`Clean` 命令已迁移到 `workflow jira` 子命令，请参考 [Jira 命令模块架构文档](./jira.md)。

**库层调用（Commands → JiraLogs）**：
命令层通过 `JiraLogs` API 调用核心业务逻辑：
```
DownloadCommand::download()
  ↓
JiraLogs::new()
  ↓
JiraLogs::download-_from-_jira()
```

### 3. 依赖注入模式

- 命令层不直接创建依赖，而是通过 `JiraLogs::new()` 创建实例
- `JiraLogs` 内部自动加载配置和初始化依赖

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - 交互式输入错误（用户取消输入）
   - 参数验证错误

3. **库层**：文件操作错误、API 调用错误
   - 通过 `JiraLogs` API 返回的错误信息
   - 文件不存在、API 调用失败等

#### 容错机制

- **文件不存在错误**：
  - Find/Search 命令：如果日志文件不存在，`JiraLogs` API 会返回错误，命令层会提示用户先执行 download 命令

- **API 调用错误**：
  - Download 命令：Jira API 调用失败会通过 `JiraLogs` API 返回错误信息

- **交互式输入错误**：
  - Find/Search 命令：如果用户取消输入或输入无效，`dialoguer::Input` 会返回错误

---

## 📝 扩展性

### 添加新命令

1. 在 `commands/log/` 下创建新的命令文件（如 `new-_command.rs`）
2. 实现命令结构体和处理方法（如 `NewCommand::execute()`）
3. 在 `commands/log/mod.rs` 中导出命令结构体
4. 在 `src/main.rs` 中添加命令枚举（`LogSubcommand`）
5. 在 `src/main.rs` 的 `main()` 函数中添加命令分发逻辑

### 添加新的用户交互

1. 使用 `dialoguer` 库添加交互式输入
2. 在命令方法中处理用户输入
3. 调用 `JiraLogs` API 执行操作

### 添加新的输出格式

1. 在命令方法中格式化输出
2. 使用 `log_*!` 宏输出信息
3. 使用 `log-_break!` 宏添加分隔线

---

## 📚 相关文档

- [主架构文档](../architecture.md)
- [Jira 模块架构文档](../architecture/jira.md)
- [Jira 命令模块架构文档](./jira.md)
- [PR 命令模块架构文档](./pr.md)

---

## 📋 使用示例

### Download 命令

```bash
# 提供 JIRA ID
workflow log download PROJ-123

# 交互式输入 JIRA ID
workflow log download
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
```

### Find 命令

```bash
# 提供所有参数
workflow log find PROJ-123 abc123

# 提供 JIRA ID，交互式输入请求 ID
workflow log find PROJ-123
# 提示: Enter request ID to find

# 交互式输入 JIRA ID 和请求 ID
workflow log find
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
# 提示: Enter request ID to find
```

### Search 命令

```bash
# 提供所有参数
workflow log search PROJ-123 "error"

# 提供 JIRA ID，交互式输入搜索词
workflow log search PROJ-123
# 提示: Enter search term

# 交互式输入 JIRA ID 和搜索词
workflow log search
# 提示: Enter Jira ticket ID (e.g., PROJ-123)
# 提示: Enter search term
```

---

## ✅ 总结

日志命令层采用清晰的命令模式设计：

1. **CLI 层**：参数解析和命令分发
2. **命令层**：用户交互和格式化输出
3. **库层调用**：通过 `JiraLogs` API 调用核心业务逻辑

**设计优势**：
- ✅ **职责分离**：命令层专注于用户交互和输出格式化
- ✅ **易于扩展**：添加新命令只需实现命令结构体和处理方法
- ✅ **交互友好**：支持交互式输入和参数传递两种方式
- ✅ **错误处理**：完整的错误处理和容错机制

---

**最后更新**: 2025-12-16
