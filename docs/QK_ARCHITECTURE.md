# QK 模块架构文档

## 📋 概述

QK 模块是 Workflow CLI 的快速日志操作工具，提供从 Jira ticket 下载日志、查找请求 ID、搜索关键词、清理日志目录和显示 ticket 信息等功能。该模块采用三层架构设计，核心业务逻辑集中在 `lib/log/logs.rs`，命令层提供便捷的用户接口。

---

## 📁 相关文件

### CLI 入口层

```
src/bin/qk.rs (103 行)
```
- **职责**：独立的 QK 命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将请求分发到对应的命令处理函数

### 命令封装层 (`commands/qk/`)

```
src/commands/qk/
├── mod.rs          # QK 命令模块声明和统一接口（51 行）
├── download.rs     # 下载日志命令（38 行）
├── find.rs         # 查找请求 ID 命令（60 行）
├── search.rs       # 搜索关键词命令（59 行）
├── clean.rs        # 清理日志目录命令（33 行）
└── info.rs         # 显示 ticket 信息命令（92 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/log/`) 的功能

### 核心业务逻辑层 (`lib/log/`)

```
src/lib/log/
├── mod.rs          # 日志模块声明
└── logs.rs         # 日志处理核心逻辑（1195 行）
```

**职责**：
- 从 Jira 下载日志附件
- 合并分片 zip 文件
- 解压日志文件
- 查找请求 ID 并提取响应内容
- 搜索关键词
- 清理日志目录
- 解析日志文件路径

### 依赖模块

- **`lib/jira/`**：Jira 集成（获取附件、获取 ticket 信息等）
- **`lib/http/`**：HTTP 客户端（发送请求到 Streamock 服务）
- **`lib/utils/clipboard.rs`**：剪贴板操作（复制响应内容）
- **`lib/settings/`**：配置管理（环境变量读取）

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
bin/qk.rs (CLI 入口，参数解析)
  ↓
commands/qk/*.rs (命令封装层，处理交互)
  ↓
lib/log/logs.rs (核心业务逻辑层)
  ↓
lib/jira/, lib/http/, lib/utils/ 等 (依赖模块)
```

---

## 1. 下载日志命令 (`download`)

### 调用流程

```
bin/qk.rs::QkCommands::Download
  ↓
commands/qk/mod.rs::QuickCommand::download()
  ↓
commands/qk/download.rs::DownloadCommand::download()
  ↓
  1. 根据 download_all 参数显示不同的提示信息
  2. 加载 Settings
  3. 获取 log_output_folder_name 配置
  4. 调用 Logs::download_from_jira()
     ├─ 确定输出目录 (~/Downloads/logs_<JIRA_ID>)
     ├─ 创建目录结构
     ├─ Jira::get_attachments() 获取附件列表
     ├─ 根据 download_all_attachments 参数决定：
     │   - true: 下载所有附件到 downloads/ 目录
     │   - false: 只下载日志附件 (log.zip, log.z01, etc.)
     └─ 如果存在 log.zip，处理日志附件：
         - 检查是否有分片文件
         - 有: merge_split_zips() 合并分片
         - 无: 直接复制 log.zip 为 merged.zip
         - extract_zip() 解压 merged.zip
  5. 输出成功信息和文件路径
```

### 功能说明

1. **附件下载**：
   - 支持下载所有附件或仅下载日志附件
   - 自动识别日志附件（log.zip, log.z01, log.z02 等）

2. **分片文件处理**：
   - 自动检测分片 zip 文件
   - 合并分片文件为 merged.zip
   - 解压合并后的文件

3. **目录结构**：
   - 创建 `~/Downloads/logs_<JIRA_ID>/` 目录
   - 所有附件下载到 `downloads/` 子目录
   - 解压后的日志文件在 `merged/` 子目录

### 关键步骤说明

1. **附件获取**：
   - 使用 `Jira::get_attachments()` 从 Jira API 获取附件列表
   - 根据 `download_all_attachments` 参数过滤附件

2. **分片合并**：
   - 使用 `Logs::merge_split_zips()` 合并分片文件
   - 支持标准 zip 分片格式（.z01, .z02 等）

3. **文件解压**：
   - 使用 `Logs::extract_zip()` 解压合并后的 zip 文件
   - 解压到 `merged/` 子目录

---

## 2. 查找请求 ID 命令 (`find`)

### 调用流程

```
bin/qk.rs::QkCommands::Find
  ↓
commands/qk/mod.rs::QuickCommand::find_request_id()
  ↓
commands/qk/find.rs::FindCommand::find_request_id()
  ↓
  1. Logs::get_log_file_path() 获取日志文件路径
  2. 检查日志文件是否存在
  3. 获取请求 ID（参数提供或交互式输入）
  4. 调用 Logs::find_and_send_to_streamock()
     ├─ extract_response_content() 提取响应内容
     ├─ find_request_id() 获取日志条目信息 (URL)
     ├─ 生成 name (格式: #<REQUEST_ID> <URL路径>)
     ├─ 生成 domain (格式: <JIRA_SERVICE>/browse/<JIRA_ID>)
     ├─ 生成时间戳
     ├─ 创建 JSON payload
     └─ 发送 POST 请求到 Streamock 服务
  5. Clipboard::copy() 复制响应内容到剪贴板
  6. 输出成功信息
```

### 功能说明

1. **日志文件解析**：
   - 根据 JIRA ID 自动解析日志文件路径
   - 支持多种日志文件格式

2. **响应内容提取**：
   - 从日志文件中提取指定请求 ID 的响应内容
   - 自动识别响应块

3. **Streamock 集成**：
   - 自动发送请求信息到 Streamock 服务
   - 生成包含请求 ID、URL、时间戳的 JSON payload

4. **剪贴板操作**：
   - 自动复制响应内容到剪贴板
   - 方便用户直接粘贴使用

### 关键步骤说明

1. **路径解析**：
   - 使用 `Logs::get_log_file_path()` 根据 JIRA ID 解析日志文件路径
   - 支持默认路径格式：`~/Downloads/logs_<JIRA_ID>/merged/flutter-api.log`

2. **内容提取**：
   - 使用 `Logs::extract_response_content()` 提取响应内容
   - 使用 `Logs::find_request_id()` 查找请求 ID 并获取 URL 信息

3. **Streamock 集成**：
   - 默认发送到 `http://localhost:3001/api/submit`
   - 生成包含完整请求信息的 JSON payload

---

## 3. 搜索关键词命令 (`search`)

### 调用流程

```
bin/qk.rs::QkCommands::Search
  ↓
commands/qk/mod.rs::QuickCommand::search()
  ↓
commands/qk/search.rs::SearchCommand::search()
  ↓
  1. Logs::get_log_file_path() 获取日志文件路径
  2. 检查日志文件是否存在
  3. 获取搜索词（参数提供或交互式输入）
  4. 调用 Logs::search_keyword()
     ├─ 打开日志文件
     ├─ 逐行读取日志文件
     ├─ 识别日志条目 (以 "💡" 开头的行)
     ├─ 解析日志条目 (提取 ID 和 URL)
     ├─ 在当前条目块中搜索关键词 (不区分大小写)
     └─ 如果匹配，保存到结果列表
  5. 格式化输出结果
     - 无结果: 输出警告信息
     - 有结果: 输出匹配的 URL 和 ID
```

### 功能说明

1. **关键词搜索**：
   - 支持不区分大小写的关键词搜索
   - 在日志条目块中搜索匹配内容

2. **日志条目识别**：
   - 自动识别日志条目（以 "💡" 开头的行）
   - 解析日志条目格式，提取 ID 和 URL

3. **结果展示**：
   - 格式化输出匹配的日志条目
   - 显示请求 ID 和 URL 信息

### 关键步骤说明

1. **日志解析**：
   - 使用 `Logs::parse_log_entry()` 解析日志条目格式
   - 使用 `Logs::extract_url_from_line()` 从日志行提取 URL

2. **关键词匹配**：
   - 在当前日志条目块中搜索关键词
   - 不区分大小写匹配

---

## 4. 清理日志目录命令 (`clean`)

### 调用流程

```
bin/qk.rs::QkCommands::Clean
  ↓
commands/qk/mod.rs::QuickCommand::clean()
  ↓
commands/qk/clean.rs::CleanCommand::clean()
  ↓
  1. 根据参数显示不同的提示信息
     - list_only: 列出目录内容
     - dry_run: 预览清理操作
     - 正常: 执行清理操作
  2. 调用 Logs::clean_jira_dir()
     ├─ 解析日志目录路径
     ├─ 检查目录是否存在
     ├─ list_only: 列出目录内容
     ├─ dry_run: 预览将要删除的文件
     └─ 正常: 删除整个日志目录
  3. 输出操作结果
```

### 功能说明

1. **目录清理**：
   - 删除指定 JIRA ID 的整个日志目录
   - 支持预览模式（dry-run）和列表模式（list-only）

2. **安全机制**：
   - 提供预览模式，避免误删
   - 提供列表模式，查看目录内容

### 关键步骤说明

1. **路径解析**：
   - 根据 JIRA ID 解析日志目录路径
   - 使用 `Logs::clean_jira_dir()` 执行清理操作

2. **操作模式**：
   - `list_only`: 只列出目录内容，不删除
   - `dry_run`: 预览将要删除的文件，不实际删除
   - 正常模式: 实际删除整个目录

---

## 5. 显示 Ticket 信息命令 (`info`)

### 调用流程

```
bin/qk.rs::QkCommands::Info (或默认无子命令)
  ↓
commands/qk/mod.rs::QuickCommand::show()
  ↓
commands/qk/info.rs::InfoCommand::show()
  ↓
  1. Jira::get_ticket_info() 获取 ticket 信息
  2. 显示基本信息
     - Key, ID, Summary, Status
  3. 显示描述（如果有）
  4. 显示附件列表（如果有）
  5. 显示评论数量（如果有）
  6. 显示 Jira URL
```

### 功能说明

1. **Ticket 信息获取**：
   - 从 Jira API 获取完整的 ticket 信息
   - 显示基本信息、描述、附件、评论等

2. **格式化显示**：
   - 格式化文件大小显示
   - 清晰的分类展示

### 关键步骤说明

1. **信息获取**：
   - 使用 `Jira::get_ticket_info()` 获取 ticket 信息
   - 解析并格式化显示

2. **附件列表**：
   - 显示所有附件的文件名和大小
   - 格式化文件大小（B, KB, MB, GB, TB）

---

## 📊 数据流

### Download 命令数据流

```
Jira API
  ↓ (附件列表)
Jira::get_attachments()
  ↓ (过滤日志附件)
下载到本地 downloads/ 目录
  ↓ (合并分片)
merged.zip
  ↓ (解压)
~/Downloads/logs_<JIRA_ID>/merged/
  ↓ (输出路径)
用户终端
```

### Find 命令数据流

```
命令行参数 (JIRA_ID, REQUEST_ID)
  ↓
解析日志文件路径
  ↓
读取日志文件
  ↓
提取响应内容 + 查找 URL 信息
  ↓
生成 JSON payload
  ↓
发送到 Streamock 服务
  ↓
复制响应内容到剪贴板
  ↓
输出成功信息
```

### Search 命令数据流

```
命令行参数 (JIRA_ID, SEARCH_TERM)
  ↓
解析日志文件路径
  ↓
逐行读取日志文件
  ↓
匹配关键词 (不区分大小写)
  ↓
收集匹配的 LogEntry (ID + URL)
  ↓
格式化输出到终端
```

---

## 🔗 与其他模块的集成

### Jira 模块集成

- **`lib/jira/`**：Jira 集成
  - `Jira::get_attachments()` - 获取附件列表
  - `Jira::get_ticket_info()` - 获取 ticket 信息

### HTTP 模块集成

- **`lib/http/`**：HTTP 客户端
  - 发送 POST 请求到 Streamock 服务
  - 支持代理和认证

### 工具模块集成

- **`lib/utils/clipboard.rs`**：剪贴板操作
  - `Clipboard::copy()` - 复制响应内容到剪贴板

### 配置模块集成

- **`lib/settings/`**：配置管理
  - `log_output_folder_name` - 日志输出文件夹名称
  - `jira_service_address` - Jira 服务地址

---

## 🎯 设计模式

### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `DownloadCommand::download()`
- `FindCommand::find_request_id()`
- `SearchCommand::search()`
- `CleanCommand::clean()`
- `InfoCommand::show()`

### 2. 统一接口模式

`QuickCommand` 结构体作为统一接口，保持向后兼容：
- 所有方法都标记为 `#[allow(dead_code)]`
- 内部直接调用拆分的命令模块
- 允许未来逐步迁移到新的命令结构

### 3. 工具函数模式

将复杂的操作封装到 `lib/log/logs.rs` 中的工具函数，命令层只负责调用和交互：
- `Logs::download_from_jira()` - 下载日志
- `Logs::find_and_send_to_streamock()` - 查找并发送到 Streamock
- `Logs::search_keyword()` - 搜索关键词
- `Logs::clean_jira_dir()` - 清理日志目录

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **核心层**：文件操作错误、API 调用错误

### 容错机制

- **文件不存在错误**：
  - Find/Search 命令：如果日志文件不存在，会提示用户先执行 download 命令

- **API 调用错误**：
  - Download 命令：Jira API 调用失败会返回错误信息
  - Find 命令：Streamock 服务调用失败会记录错误并返回
  - Info 命令：Jira API 调用失败会返回错误信息

- **交互式输入错误**：
  - Find/Search 命令：如果用户取消输入或输入无效，会返回错误

---

## 📝 扩展性

### 添加新命令

1. 在 `commands/qk/` 下创建新的命令文件
2. 实现命令结构体和处理方法
3. 在 `commands/qk/mod.rs` 中导出
4. 在 `bin/qk.rs` 中添加命令枚举和处理逻辑
5. 在 `QuickCommand` 中添加统一接口方法（可选）

### 添加新的日志格式支持

1. 在 `lib/log/logs.rs` 中添加新的日志格式解析逻辑
2. 更新 `parse_log_entry()` 和 `extract_url_from_line()` 方法

### 添加新的搜索功能

1. 在 `lib/log/logs.rs` 中添加新的搜索方法
2. 在命令层添加对应的命令实现

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [PR 模块架构文档](./PR_ARCHITECTURE.md)
- [日志处理模块文档](./ARCHITECTURE.md#日志处理模块-liblog)

---

## 使用示例

### Download 命令

```bash
# 只下载日志附件（默认行为）
qk WEW-763 download

# 下载所有附件
qk WEW-763 download --all
# 或使用短选项
qk WEW-763 download -a
```

### Find 命令

```bash
# 提供请求 ID
qk WEW-763 find abc123

# 交互式输入请求 ID
qk WEW-763 find
```

### Search 命令

```bash
# 提供搜索词
qk WEW-763 search "error"

# 交互式输入搜索词
qk WEW-763 search
```

### Clean 命令

```bash
# 清理日志目录
qk WEW-763 clean

# 预览清理操作（dry-run）
qk WEW-763 clean --dry-run

# 列出目录内容
qk WEW-763 clean --list
```

### Info 命令

```bash
# 显示 ticket 信息
qk WEW-763 info

# 或直接使用（无子命令时默认显示 info）
qk WEW-763
```

