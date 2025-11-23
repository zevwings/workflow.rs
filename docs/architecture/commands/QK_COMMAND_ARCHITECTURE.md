# 日志和 Jira 命令模块架构文档

## 📋 概述

日志和 Jira 命令层是 Workflow CLI 的命令接口，提供从 Jira ticket 下载日志、查找请求 ID、搜索关键词、清理日志目录和显示 ticket 信息等功能。该层采用命令模式设计，通过调用 `lib/jira/logs/` 模块提供的 API 实现业务功能。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/jira/logs/` 模块提供。

**命令结构**：
- `workflow log` - 日志操作命令（download, find, search, clean）
- `workflow jira` - Jira 操作命令（info）

---

## 📁 相关文件

### CLI 入口层

日志和 Jira 命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Log` 和 `Commands::Jira` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow log` 和 `workflow jira` 子命令分发到对应的命令处理函数

### 命令封装层 (`commands/qk/`)

```
src/commands/qk/
├── mod.rs          # QK 命令模块声明（20 行）
├── download.rs     # 下载日志命令（33 行）
├── find.rs         # 查找请求 ID 命令（45 行）
├── search.rs       # 搜索关键词命令（97 行）
├── clean.rs        # 清理日志目录命令（58 行）
└── info.rs         # 显示 ticket 信息命令（103 行）
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
  - `JiraLogs::download_from_jira()` - 下载日志
  - `JiraLogs::extract_response_content()` - 提取响应内容
  - `JiraLogs::search_keyword()` - 搜索关键词
  - `JiraLogs::clean_dir()` - 清理目录
  - `JiraLogs::ensure_log_file_exists()` - 确保日志文件存在
- **`lib/jira/`**：Jira 集成
  - `Jira::get_ticket_info()` - 获取 ticket 信息
- **`lib/base/util/`**：工具函数
  - `Clipboard::copy()` - 复制到剪贴板
- **`lib/base/settings/`**：配置管理
  - `Settings::get()` - 获取配置（`log_output_folder_name`、`log_download_base_dir` 等）

详细架构文档：参见 [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/qk/*.rs (命令封装层，处理交互)
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
  ├─ Find → FindCommand::find_request_id()
  ├─ Search → SearchCommand::search()
  ├─ Clean → CleanCommand::clean()
  └─ None → InfoCommand::show() (默认)
```

---

## 1. 下载日志命令 (`download`)

### 相关文件

```
src/commands/qk/download.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Download
  ↓
commands/qk/download.rs::DownloadCommand::download(jira_id, download_all)
  ↓
  1. 根据 download_all 参数显示不同的提示信息
  2. 创建 JiraLogs 实例：JiraLogs::new()
  3. 调用 JiraLogs::download_from_jira(jira_id, None, download_all)
     └─ 内部处理：下载附件、合并分片、解压文件
  4. 输出成功信息和文件路径
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（必需）
   - `download_all` - 是否下载所有附件（默认：只下载日志附件）

2. **用户交互**：
   - 根据 `download_all` 参数显示不同的提示信息
   - 显示下载进度和结果

3. **核心功能**：
   - 通过 `JiraLogs::download_from_jira()` API 实现下载功能
   - 自动处理附件下载、分片合并、文件解压等操作

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例（自动加载配置和初始化 HTTP 客户端）

2. **下载执行**：
   - 调用 `JiraLogs::download_from_jira()` 执行下载
   - 支持下载所有附件或仅下载日志附件
   - 自动处理分片 ZIP 文件的合并和解压

3. **结果输出**：
   - 显示下载成功信息
   - 显示文件保存路径

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::download_from_jira(jira_id, output_folder, download_all_attachments)`** - 下载日志附件
  - 参数：
    - `jira_id` - Jira ticket ID
    - `output_folder` - 输出文件夹名称（可选，None 时使用配置的默认值）
    - `download_all_attachments` - 是否下载所有附件
  - 返回：基础目录路径

---

## 2. 查找请求 ID 命令 (`find`)

### 相关文件

```
src/commands/qk/find.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Find
  ↓
commands/qk/find.rs::FindCommand::find_request_id(jira_id, request_id)
  ↓
  1. 创建 JiraLogs 实例：JiraLogs::new()
  2. 获取请求 ID（参数提供或交互式输入）
  3. 调用 JiraLogs::extract_response_content(jira_id, request_id)
     └─ 内部处理：解析日志文件、查找请求 ID、提取响应内容
  4. Clipboard::copy() 复制响应内容到剪贴板
  5. 输出成功信息
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（必需）
   - `request_id` - 请求 ID（可选，不提供时交互式输入）

2. **用户交互**：
   - 如果未提供 `request_id`，使用 `dialoguer::Input` 交互式输入
   - 显示查找进度和结果

3. **核心功能**：
   - 通过 `JiraLogs::extract_response_content()` API 提取响应内容
   - 自动复制响应内容到剪贴板

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例

2. **请求 ID 获取**：
   - 优先使用命令行参数
   - 如果未提供，交互式输入

3. **内容提取**：
   - 调用 `JiraLogs::extract_response_content()` 提取响应内容
   - 自动处理日志文件解析和内容提取

4. **剪贴板操作**：
   - 使用 `Clipboard::copy()` 复制响应内容到剪贴板

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::extract_response_content(jira_id, request_id)`** - 提取响应内容
  - 参数：
    - `jira_id` - Jira ticket ID
    - `request_id` - 请求 ID
  - 返回：响应内容字符串

---

## 3. 搜索关键词命令 (`search`)

### 相关文件

```
src/commands/qk/search.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Search
  ↓
commands/qk/search.rs::SearchCommand::search(jira_id, search_term)
  ↓
  1. 创建 JiraLogs 实例：JiraLogs::new()
  2. 确保日志文件存在：JiraLogs::ensure_log_file_exists(jira_id)
  3. 获取搜索词（参数提供或交互式输入）
  4. 调用 JiraLogs::search_keyword(jira_id, search_term)
     └─ 内部处理：解析日志文件、搜索关键词、收集匹配结果
  5. 格式化输出结果（显示匹配的 URL 和 ID）
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（必需）
   - `search_term` - 搜索关键词（可选，不提供时交互式输入）

2. **用户交互**：
   - 如果未提供 `search_term`，使用 `dialoguer::Input` 交互式输入
   - 格式化输出匹配结果

3. **核心功能**：
   - 通过 `JiraLogs::search_keyword()` API 搜索关键词
   - 支持搜索多个日志文件（flutter-api.log 和 api.log）
   - 自动去重和格式化输出

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例
   - 确保日志文件存在

2. **搜索词获取**：
   - 优先使用命令行参数
   - 如果未提供，交互式输入

3. **搜索执行**：
   - 调用 `JiraLogs::search_keyword()` 执行搜索
   - 支持搜索多个日志文件

4. **结果展示**：
   - 格式化输出匹配的日志条目
   - 显示请求 ID 和 URL 信息
   - 按日志文件分组显示

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::ensure_log_file_exists(jira_id)`** - 确保日志文件存在
  - 参数：`jira_id` - Jira ticket ID
  - 返回：日志文件路径
- **`JiraLogs::search_keyword(jira_id, search_term)`** - 搜索关键词
  - 参数：
    - `jira_id` - Jira ticket ID
    - `search_term` - 搜索关键词
  - 返回：匹配的日志条目列表（`Vec<LogEntry>`）

---

## 4. 清理日志目录命令 (`clean`)

### 相关文件

```
src/commands/qk/clean.rs
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::LogSubcommand::Clean
  ↓
commands/qk/clean.rs::CleanCommand::clean(jira_id, dry_run, list_only)
  ↓
  1. 根据参数显示不同的提示信息
  2. 创建 JiraLogs 实例：JiraLogs::new()
  3. 调用 JiraLogs::clean_dir(jira_id, dry_run, list_only)
     └─ 内部处理：计算目录信息、列出内容、预览或删除
  4. 输出操作结果
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（可为空字符串，表示清理整个基础目录）
   - `dry_run` - 预览模式，不实际删除
   - `list_only` - 只列出目录内容

2. **用户交互**：
   - 根据参数显示不同的提示信息
   - 显示操作结果

3. **核心功能**：
   - 通过 `JiraLogs::clean_dir()` API 清理日志目录
   - 支持预览模式和列表模式

### 关键步骤说明

1. **初始化**：
   - 创建 `JiraLogs` 实例

2. **操作执行**：
   - 调用 `JiraLogs::clean_dir()` 执行清理操作
   - 根据参数决定操作模式（预览、列表、删除）

3. **结果输出**：
   - 显示操作结果
   - 如果删除成功，显示成功信息

### JiraLogs API 调用

- **`JiraLogs::new()`** - 创建 JiraLogs 实例
- **`JiraLogs::clean_dir(jira_id, dry_run, list_only)`** - 清理日志目录
  - 参数：
    - `jira_id` - Jira ticket ID（空字符串表示清理整个基础目录）
    - `dry_run` - 预览模式
    - `list_only` - 只列出目录内容
  - 返回：是否成功删除（bool）

---

## 5. 显示 Ticket 信息命令 (`info`)

### 相关文件

```
src/commands/qk/info.rs
src/main.rs (命令入口，默认命令)
```

### 调用流程

```
src/main.rs::JiraSubcommand::Info
  ↓
commands/qk/info.rs::InfoCommand::show(jira_id)
  ↓
  1. 调用 Jira::get_ticket_info(jira_id) 获取 ticket 信息
  2. 显示基本信息（Key, ID, Summary, Status）
  3. 显示描述（如果有）
  4. 显示附件列表（如果有）
  5. 显示评论数量（如果有）
  6. 显示 Jira URL
```

### 功能说明

1. **参数处理**：
   - `jira_id` - Jira ticket ID（必需）

2. **用户交互**：
   - 格式化显示 ticket 信息
   - 使用分隔线和图标美化输出

3. **核心功能**：
   - 通过 `Jira::get_ticket_info()` API 获取 ticket 信息
   - 格式化显示所有相关信息

### 关键步骤说明

1. **信息获取**：
   - 调用 `Jira::get_ticket_info()` 获取 ticket 信息

2. **信息展示**：
   - 显示基本信息（Key, ID, Summary, Status）
   - 显示描述（如果有）
   - 显示附件列表（格式化文件大小）
   - 显示评论数量
   - 显示 Jira URL

### Jira API 调用

- **`Jira::get_ticket_info(jira_id)`** - 获取 ticket 信息
  - 参数：`jira_id` - Jira ticket ID
  - 返回：Issue 结构体（包含所有 ticket 信息）

### 数据流

#### Download 命令数据流

```
命令行参数 (JIRA_ID, --all)
  ↓
DownloadCommand::download()
  ↓
JiraLogs::download_from_jira()
  ↓
Jira API (获取附件列表)
  ↓
下载到本地
  ↓
合并分片、解压文件
  ↓
输出文件路径
```

#### Find 命令数据流

```
命令行参数 (JIRA_ID, REQUEST_ID)
  ↓
FindCommand::find_request_id()
  ↓
JiraLogs::extract_response_content()
  ↓
解析日志文件、提取响应内容
  ↓
Clipboard::copy() 复制到剪贴板
  ↓
输出成功信息
```

#### Search 命令数据流

```
命令行参数 (JIRA_ID, SEARCH_TERM)
  ↓
SearchCommand::search()
  ↓
JiraLogs::search_keyword()
  ↓
解析日志文件、搜索关键词
  ↓
收集匹配的 LogEntry
  ↓
格式化输出到终端
```

#### Clean 命令数据流

```
命令行参数 (JIRA_ID, --dry-run, --list)
  ↓
CleanCommand::clean()
  ↓
JiraLogs::clean_dir()
  ↓
计算目录信息、列出内容
  ↓
预览或删除目录
  ↓
输出操作结果
```

#### Info 命令数据流

```
命令行参数 (JIRA_ID)
  ↓
InfoCommand::show()
  ↓
Jira::get_ticket_info()
  ↓
Jira API (获取 ticket 信息)
  ↓
格式化显示 ticket 信息
```

---

## 🏗️ 架构设计

### 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `DownloadCommand::download()` - 下载日志
- `FindCommand::find_request_id()` - 查找请求 ID
- `SearchCommand::search()` - 搜索关键词
- `CleanCommand::clean()` - 清理日志目录
- `InfoCommand::show()` - 显示 ticket 信息

### 2. 分层调用模式

**命令层（CLI → Commands）**：
所有命令通过 `src/main.rs` 直接调用对应的命令结构体：
```
src/main.rs::main()
  ↓
match cli.subcommand
  ├─ Download → DownloadCommand::download()
  ├─ Find → FindCommand::find_request_id()
  ├─ Search → SearchCommand::search()
  ├─ Clean → CleanCommand::clean()
  └─ None → InfoCommand::show()
```

**库层调用（Commands → JiraLogs）**：
命令层通过 `JiraLogs` API 调用核心业务逻辑：
```
DownloadCommand::download()
  ↓
JiraLogs::new()
  ↓
JiraLogs::download_from_jira()
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
  - Info 命令：Jira API 调用失败会返回错误信息

- **交互式输入错误**：
  - Find/Search 命令：如果用户取消输入或输入无效，`dialoguer::Input` 会返回错误

---

## 📝 扩展性

### 添加新命令

1. 在 `commands/qk/` 下创建新的命令文件（如 `new_command.rs`）
2. 实现命令结构体和处理方法（如 `NewCommand::execute()`）
3. 在 `commands/qk/mod.rs` 中导出命令结构体
4. 在 `src/main.rs` 中添加命令枚举（`LogSubcommand` 或 `JiraSubcommand`）
5. 在 `src/main.rs` 的 `main()` 函数中添加命令分发逻辑

### 添加新的用户交互

1. 使用 `dialoguer` 库添加交互式输入
2. 在命令方法中处理用户输入
3. 调用 `JiraLogs` API 执行操作

### 添加新的输出格式

1. 在命令方法中格式化输出
2. 使用 `log_*!` 宏输出信息
3. 使用 `log_break!` 宏添加分隔线

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Jira 模块架构文档](../lib/JIRA_ARCHITECTURE.md)
- [PR 命令模块架构文档](./PR_COMMAND_ARCHITECTURE.md)

---

## 📋 使用示例

### Download 命令

```bash
# 只下载日志附件（默认行为）
workflow log download WEW-763

# 下载所有附件
workflow log download WEW-763 --all
# 或使用短选项
workflow log download WEW-763 -a
```

### Find 命令

```bash
# 提供请求 ID
workflow log find WEW-763 abc123

# 交互式输入请求 ID
workflow log find WEW-763
```

### Search 命令

```bash
# 提供搜索词
workflow log search WEW-763 "error"

# 交互式输入搜索词
workflow log search WEW-763
```

### Clean 命令

```bash
# 清理整个日志基础目录
workflow log clean

# 清理指定 JIRA ID 的日志目录
workflow log clean WEW-763

# 预览清理操作（dry-run）
workflow log clean --dry-run
# 或使用短选项
workflow log clean -n

# 列出目录内容
workflow log clean --list
# 或使用短选项
workflow log clean -l
```

### Info 命令

```bash
# 显示 ticket 信息
workflow jira info WEW-763
```

---

## ✅ 总结

QK 命令层采用清晰的命令模式设计：

1. **CLI 层**：参数解析和命令分发
2. **命令层**：用户交互和格式化输出
3. **库层调用**：通过 `JiraLogs` API 调用核心业务逻辑

**设计优势**：
- ✅ **职责分离**：命令层专注于用户交互和输出格式化
- ✅ **易于扩展**：添加新命令只需实现命令结构体和处理方法
- ✅ **交互友好**：支持交互式输入和参数传递两种方式
- ✅ **错误处理**：完整的错误处理和容错机制

