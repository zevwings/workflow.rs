# QK 命令调用流程分析

## 概述

`qk` 命令是一个独立的快速日志操作工具，提供三个核心功能：
- **下载日志**：从 Jira ticket 下载日志附件
- **查找请求 ID**：在日志文件中查找并提取响应内容
- **搜索关键词**：在日志文件中搜索关键词

## 架构层次

```
┌─────────────────────────────────────────────────────────┐
│  CLI 入口层 (bin/qk.rs)                                  │
│  - 命令行参数解析 (clap)                                 │
│  - 命令分发                                             │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│  命令封装层 (commands/qk/)                               │
│  - QuickCommand (统一接口，向后兼容)                     │
│  - DownloadCommand / FindCommand / SearchCommand         │
└──────────────────┬──────────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────────┐
│  核心逻辑层 (lib/log/logs.rs)                            │
│  - Logs::download_from_jira()                           │
│  - Logs::find_and_send_to_streamock()                   │
│  - Logs::search_keyword()                                │
│  - Logs::get_log_file_path()                             │
└─────────────────────────────────────────────────────────┘
```

## 详细调用流程

### 1. Download 命令流程

```
用户输入: qk PROJ-123 download [--all]
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ bin/qk.rs::main()                                        │
│  - Cli::parse() 解析命令行参数                           │
│  - 匹配 QkCommands::Download { all }                    │
│  - --all 选项控制是否下载所有附件                        │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/mod.rs::QuickCommand::download()             │
│  - 向后兼容的统一接口                                    │
│  - 传递 download_all 参数                                │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/download.rs::DownloadCommand::download()    │
│  1. 根据 download_all 参数显示不同的提示信息            │
│  2. 加载 Settings                                        │
│  3. 获取 log_output_folder_name 配置                    │
│  4. 调用 Logs::download_from_jira()                     │
│  5. 输出成功信息和文件路径                               │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ lib/log/logs.rs::Logs::download_from_jira()             │
│  1. 确定输出目录 (~/Downloads/logs_<JIRA_ID>)           │
│  2. 创建目录结构                                         │
│  3. Jira::get_attachments() 获取附件列表                │
│  4. 根据 download_all_attachments 参数决定：             │
│     - true: 下载所有附件到 downloads/ 目录              │
│     - false: 只下载日志附件 (log.zip, log.z01, etc.)    │
│  5. 如果存在 log.zip，处理日志附件：                     │
│     - 检查是否有分片文件                                 │
│     - 有: merge_split_zips() 合并分片                   │
│     - 无: 直接复制 log.zip 为 merged.zip                │
│     - extract_zip() 解压 merged.zip                     │
│  6. 返回基础目录路径                                     │
└─────────────────────────────────────────────────────────┘
```

**关键依赖**：
- `Jira::get_attachments()` - 从 Jira API 获取附件列表
- `Logs::merge_split_zips()` - 合并分片 zip 文件
- `Logs::extract_zip()` - 解压 zip 文件

**使用示例**：
```bash
# 只下载日志附件（默认行为）
qk WEW-763 download

# 下载所有附件
qk WEW-763 download --all
# 或使用短选项
qk WEW-763 download -a
```

---

### 2. Find 命令流程

```
用户输入: qk PROJ-123 find [REQUEST_ID]
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ bin/qk.rs::main()                                        │
│  - Cli::parse() 解析命令行参数                           │
│  - 匹配 QkCommands::Find { request_id }                 │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/mod.rs::QuickCommand::find_request_id()      │
│  - 向后兼容的统一接口                                    │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/find.rs::FindCommand::find_request_id()      │
│  1. Logs::get_log_file_path() 获取日志文件路径           │
│  2. 检查日志文件是否存在                                 │
│     - 不存在: 返回错误提示                               │
│  3. 获取请求 ID                                          │
│     - 参数提供: 直接使用                                 │
│     - 未提供: dialoguer::Input 交互式输入                │
│  4. 加载 Settings                                        │
│  5. 调用 Logs::find_and_send_to_streamock()             │
│  6. Clipboard::copy() 复制响应内容到剪贴板               │
│  7. 输出成功信息                                         │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ lib/log/logs.rs::Logs::find_and_send_to_streamock()     │
│  1. extract_response_content() 提取响应内容             │
│  2. find_request_id() 获取日志条目信息 (URL)            │
│  3. 生成 name (格式: #<REQUEST_ID> <URL路径>)           │
│  4. 生成 domain (格式: <JIRA_SERVICE>/browse/<JIRA_ID>) │
│  5. 生成时间戳                                           │
│  6. 创建 JSON payload                                    │
│  7. 发送 POST 请求到 Streamock 服务                      │
│     (默认: http://localhost:3001/api/submit)            │
│  8. 返回响应内容                                         │
└─────────────────────────────────────────────────────────┘
```

**关键依赖**：
- `Logs::get_log_file_path()` - 根据 JIRA ID 解析日志文件路径
- `Logs::extract_response_content()` - 从日志文件提取响应内容
- `Logs::find_request_id()` - 查找请求 ID 并获取 URL 信息
- `Clipboard::copy()` - 复制内容到剪贴板

---

### 3. Search 命令流程

```
用户输入: qk PROJ-123 search [SEARCH_TERM]
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ bin/qk.rs::main()                                        │
│  - Cli::parse() 解析命令行参数                           │
│  - 匹配 QkCommands::Search { search_term }              │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/mod.rs::QuickCommand::search()               │
│  - 向后兼容的统一接口                                    │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ commands/qk/search.rs::SearchCommand::search()           │
│  1. Logs::get_log_file_path() 获取日志文件路径          │
│  2. 检查日志文件是否存在                                 │
│     - 不存在: 返回错误提示                               │
│  3. 获取搜索词                                           │
│     - 参数提供: 直接使用                                 │
│     - 未提供: dialoguer::Input 交互式输入                │
│  4. 调用 Logs::search_keyword()                         │
│  5. 格式化输出结果                                       │
│     - 无结果: 输出警告信息                               │
│     - 有结果: 输出匹配的 URL 和 ID                       │
└──────────────────┬──────────────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────────────┐
│ lib/log/logs.rs::Logs::search_keyword()                 │
│  1. 打开日志文件                                         │
│  2. 逐行读取日志文件                                     │
│  3. 识别日志条目 (以 "💡" 开头的行)                     │
│  4. 解析日志条目 (提取 ID 和 URL)                        │
│  5. 在当前条目块中搜索关键词 (不区分大小写)              │
│  6. 如果匹配，保存到结果列表                             │
│  7. 返回所有匹配的 LogEntry 列表                         │
└─────────────────────────────────────────────────────────┘
```

**关键依赖**：
- `Logs::get_log_file_path()` - 根据 JIRA ID 解析日志文件路径
- `Logs::parse_log_entry()` - 解析日志条目格式
- `Logs::extract_url_from_line()` - 从日志行提取 URL

---

## 数据流向

### Download 命令数据流

```
Jira API
    │
    ▼ (附件列表)
Jira::get_attachments()
    │
    ▼ (过滤日志附件)
下载到本地 downloads/ 目录
    │
    ▼ (合并分片)
merged.zip
    │
    ▼ (解压)
~/Downloads/logs_<JIRA_ID>/merged/
    │
    ▼ (输出路径)
用户终端
```

### Find 命令数据流

```
命令行参数 (JIRA_ID, REQUEST_ID)
    │
    ▼
解析日志文件路径 (~/Downloads/logs_<JIRA_ID>/merged/flutter-api.log)
    │
    ▼
读取日志文件
    │
    ▼
提取响应内容 + 查找 URL 信息
    │
    ▼
生成 JSON payload
    │
    ▼
发送到 Streamock 服务
    │
    ▼
复制响应内容到剪贴板
    │
    ▼
输出成功信息
```

### Search 命令数据流

```
命令行参数 (JIRA_ID, SEARCH_TERM)
    │
    ▼
解析日志文件路径
    │
    ▼
逐行读取日志文件
    │
    ▼
匹配关键词 (不区分大小写)
    │
    ▼
收集匹配的 LogEntry (ID + URL)
    │
    ▼
格式化输出到终端
```

---

## 关键函数调用链

### 1. Download 调用链

```
main()
  └─> QuickCommand::download(jira_id, download_all)
      └─> DownloadCommand::download(jira_id, download_all)
          └─> Logs::download_from_jira(jira_id, log_output_folder_name, download_all_attachments)
              ├─> Jira::get_attachments()
              ├─> Logs::download_file() [下载所有附件或仅日志附件]
              ├─> Logs::merge_split_zips() [可选，如果存在 log.zip]
              └─> Logs::extract_zip() [可选，如果存在 log.zip]
```

### 2. Find 调用链

```
main()
  └─> QuickCommand::find_request_id()
      └─> FindCommand::find_request_id()
          ├─> Logs::get_log_file_path()
          │   └─> Logs::find_log_file()
          ├─> dialoguer::Input::interact() [可选]
          └─> Logs::find_and_send_to_streamock()
              ├─> Logs::extract_response_content()
              ├─> Logs::find_request_id()
              └─> reqwest::Client::post() [发送到 Streamock]
          └─> Clipboard::copy()
```

### 3. Search 调用链

```
main()
  └─> QuickCommand::search()
      └─> SearchCommand::search()
          ├─> Logs::get_log_file_path()
          │   └─> Logs::find_log_file()
          ├─> dialoguer::Input::interact() [可选]
          └─> Logs::search_keyword()
              ├─> Logs::parse_log_entry()
              └─> Logs::extract_url_from_line()
```

---

## 配置依赖

所有命令都依赖 `Settings` 配置：

- **Download 命令**：
  - `log_output_folder_name` - 日志输出文件夹名称
  - `--all` / `-a` 选项 - 控制是否下载所有附件（默认只下载日志附件）

- **Find 命令**：
  - `jira_service_address` - Jira 服务地址（用于生成 domain）

- **Search 命令**：
  - 无特定配置依赖

---

## 错误处理

### 文件不存在错误
- **Find/Search 命令**：如果日志文件不存在，会提示用户先执行 download 命令

### API 调用错误
- **Download 命令**：Jira API 调用失败会返回错误信息
- **Find 命令**：Streamock 服务调用失败会记录错误并返回

### 交互式输入错误
- **Find/Search 命令**：如果用户取消输入或输入无效，会返回错误

---

## 向后兼容性

`QuickCommand` 结构体作为统一接口，保持向后兼容：
- 所有方法都标记为 `#[allow(dead_code)]`
- 内部直接调用拆分的命令模块（`DownloadCommand`、`FindCommand`、`SearchCommand`）
- 允许未来逐步迁移到新的命令结构

---

## 总结

`qk` 命令采用清晰的三层架构：
1. **CLI 层**：负责参数解析和命令分发
2. **命令层**：提供用户友好的接口，处理交互式输入和输出格式化
3. **核心层**：包含所有业务逻辑，可被其他模块复用

这种设计实现了关注点分离，使得代码易于维护和扩展。
