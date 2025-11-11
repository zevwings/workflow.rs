# Workflow 架构设计

## 📋 项目概述

Workflow 是一个 Rust 编写的 CLI 工具，用于自动化开发工作流，提供 PR 管理、Jira 集成、日志处理等功能。

---

## 🏗️ 模块划分

```
src/
├── main.rs                 # CLI 入口和命令分发
├── lib.rs                  # 库入口和模块声明（重新导出所有公共 API）
├── bin/                    # 独立可执行文件（独立的 CLI 工具）
│   ├── pr.rs               # PR 命令入口（使用 commands::pr）
│   ├── qk.rs               # 快速日志操作入口（使用 commands::qk）
│   └── install.rs          # 安装命令入口（使用 commands::install）
├── commands/               # 命令实现模块（CLI 命令封装层）
│   ├── mod.rs              # 命令模块声明
│   ├── pr/                 # PR 相关命令
│   │   ├── mod.rs          # PR 命令模块声明
│   │   ├── helpers.rs      # PR 辅助函数（PR ID 解析等）
│   │   ├── create.rs       # 创建 PR
│   │   ├── merge.rs        # 合并 PR
│   │   ├── close.rs        # 关闭 PR
│   │   ├── status.rs       # PR 状态查询
│   │   ├── list.rs         # 列出 PR
│   │   └── update.rs       # 更新 PR
│   ├── qk/                 # 快速日志操作命令
│   │   ├── mod.rs          # QK 命令模块声明
│   │   ├── download.rs     # 下载日志命令
│   │   ├── find.rs         # 查找请求 ID 命令
│   │   ├── search.rs       # 搜索关键词命令
│   │   ├── clean.rs        # 清理日志目录命令
│   │   └── info.rs         # 显示 ticket 信息命令
│   ├── check.rs            # 综合检查命令（git_status, network）
│   ├── proxy.rs            # 代理管理命令（on, off, check）
│   ├── config.rs           # 配置查看命令（显示当前配置）
│   ├── setup.rs            # 初始化设置命令（交互式配置）
│   ├── install.rs          # 安装命令实现（安装二进制和补全脚本）
│   └── uninstall.rs        # 卸载命令实现（清理所有相关文件）
└── lib/                    # 核心功能库（业务逻辑层）
    ├── mod.rs              # 库模块声明
    ├── git/                # Git 操作模块
    │   ├── mod.rs          # Git 模块声明
    │   ├── commands.rs     # Git 命令封装
    │   ├── repo.rs         # 仓库操作和类型检测
    │   └── types.rs        # Git 相关类型定义
    ├── http/               # HTTP 客户端模块
    │   ├── mod.rs          # HTTP 模块声明
    │   ├── client.rs       # HTTP 客户端实现（支持认证和代理）
    │   └── response.rs     # HTTP 响应类型定义
    ├── jira/               # Jira API 集成
    │   ├── mod.rs          # Jira 模块声明
    │   ├── client.rs       # Jira REST API 客户端
    │   ├── helpers.rs      # Jira 辅助函数（项目提取等）
    │   └── status.rs       # 状态配置和工作历史管理
    ├── pr/                 # PR 相关功能
    │   ├── mod.rs          # PR 模块声明
    │   ├── github.rs       # GitHub PR 实现
    │   ├── codeup.rs       # Codeup PR 实现
    │   ├── provider.rs     # PR 提供商抽象
    │   ├── helpers.rs      # PR 辅助函数
    │   └── constants.rs    # PR 相关常量
    ├── llm/                # LLM 集成（AI 功能）
    │   ├── mod.rs          # LLM 模块声明
    │   ├── llm.rs          # LLM 客户端实现（支持多提供商）
    │   └── translator.rs   # 翻译功能（标题生成和翻译判断）
    ├── log/                # 日志处理
    │   ├── mod.rs          # 日志模块声明
    │   └── logs.rs         # 日志处理核心逻辑（包含路径解析）
    ├── settings/           # 配置管理
    │   ├── mod.rs          # Settings 模块声明
    │   └── settings.rs     # 环境变量单例配置
    └── utils/              # 工具函数
        ├── mod.rs          # Utils 模块声明
        ├── browser.rs      # 浏览器操作（打开 URL）
        ├── clipboard.rs    # 剪贴板操作（复制/读取）
        ├── completion.rs   # Shell 补全脚本生成
        ├── env.rs          # 环境变量工具（读取和写入）
        ├── logger.rs       # 日志输出工具（格式化日志）
        ├── proxy.rs        # 代理工具（代理信息管理）
        ├── shell.rs        # Shell 信息检测（检测当前 shell）
        ├── string.rs       # 字符串处理工具（敏感信息掩码等）
        └── uninstall.rs    # 卸载工具（清理配置和文件）
```

---

## 🏛️ 架构层次

### 三层架构设计

```
┌─────────────────────────────────────────┐
│         CLI 入口层 (bin/)               │
│  - bin/qk.rs                            │
│  - bin/pr.rs                            │
│  - bin/install.rs                       │
│  - main.rs (workflow 主命令)            │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      命令封装层 (commands/)              │
│  - commands/qk/  (日志操作)              │
│  - commands/pr/  (PR 操作)               │
│  - commands/check, proxy, config, etc.  │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      核心业务逻辑层 (lib/)               │
│  - lib/log/      (日志处理)              │
│  - lib/pr/       (PR 功能)               │
│  - lib/jira/     (Jira 集成)             │
│  - lib/git/      (Git 操作)              │
│  - lib/llm/      (AI 功能)               │
│  - lib/utils/    (工具函数)             │
└─────────────────────────────────────────┘
```

### 模块职责

- **CLI 入口层** (`bin/`): 独立的可执行文件，负责命令行参数解析和命令分发
- **命令封装层** (`commands/`): 提供 CLI 命令封装，处理用户交互和日志输出
- **核心业务逻辑层** (`lib/`): 包含所有业务逻辑，可被其他模块复用

### 数据流向

```
用户输入 → bin/qk.rs → commands/qk/*.rs → lib/log/logs.rs → 执行操作
用户输入 → bin/pr.rs → commands/pr/*.rs → lib/pr/*.rs → 执行操作
```

---

## 🧠 核心模块设计

### AI 模块 (`lib::llm`)

#### 概述
AI 模块负责从 Jira ticket 获取描述并生成 PR 标题，采用 Rust 原生实现，无需 Python 依赖。

#### 功能特性
- 从 Jira ticket 获取描述并生成 PR 标题
- 自动判断是否需要翻译（非英文或描述太长）
- 使用 LLM（OpenAI/DeepSeek/Proxy）翻译为简洁的英文 PR 标题
- 支持多 LLM 提供商（OpenAI、DeepSeek、Proxy）

#### 配置

**环境变量**：
- `LLM_PROVIDER` - LLM 提供商（可选，默认 `openai`）
- `LLM_OPENAI_KEY` - OpenAI API Key
- `LLM_DEEPSEEK_KEY` - DeepSeek API Key
- `LLM_PROXY_URL` - Proxy API URL
- `LLM_PROXY_KEY` - Proxy API Key

支持的值：`openai`、`deepseek`、`proxy`

#### 错误处理
PR 创建命令的错误处理：尝试使用 AI 生成标题，如果失败则回退到手动输入。

**使用示例**：
```bash
# 创建 PR（自动生成标题）
workflow pr create PROJ-123

# 手动指定标题（跳过 AI 生成）
workflow pr create PROJ-123 --title "Fix bug"
```

#### 相关文件
- `src/lib/llm/` - LLM 集成模块
- `src/commands/pr/create.rs` - PR 创建命令（已集成 AI 功能）

---

### 日志处理模块 (`lib::log`)

#### 概述
日志处理模块提供从 Jira ticket 下载日志、查找请求 ID、搜索关键词等功能。所有核心业务逻辑都在 `lib/log/logs.rs` 中，`commands/qk/` 提供便捷的命令包装器。

#### 功能特性
- **下载日志**：从 Jira ticket 下载日志附件（支持分片文件合并）
- **查找请求 ID**：在日志文件中查找请求 ID 并提取响应内容
- **搜索关键词**：在日志文件中搜索关键词并返回匹配的请求信息
- **自动路径解析**：根据 JIRA ID 自动解析日志文件路径

#### 架构设计

**三层架构**：
- **核心逻辑层**：`lib/log/logs.rs` - 包含所有业务逻辑
  - `download_from_jira()` - 从 Jira 下载日志
  - `find_request_id()` - 查找请求 ID
  - `extract_response_content()` - 提取响应内容
  - `search_keyword()` - 搜索关键词
  - `find_and_send_to_streamock()` - 查找并发送到 Streamock
  - `get_log_file_path()` - 根据 JIRA ID 解析日志文件路径
  - `find_log_file()` - 查找日志文件
  - `merge_split_zips()` - 合并分片文件
  - `extract_zip()` - 解压文件

- **命令封装层**：`commands/qk/` - 提供便捷的命令接口
  - `mod.rs` - 模块导出
  - `download.rs` - 下载命令（调用 `Logs::download_from_jira()`）
  - `find.rs` - 查找命令（调用 `Logs::find_and_send_to_streamock()`，添加剪贴板操作）
  - `search.rs` - 搜索命令（调用 `Logs::search_keyword()`，格式化输出）
  - `clean.rs` - 清理命令（调用 `Logs::clean_jira_dir()`）
  - `info.rs` - 信息命令（显示 ticket 信息）

- **CLI 入口层**：`bin/qk.rs` - 独立的可执行文件（命令行参数解析和命令分发）

#### 使用示例
```bash
# 下载日志
qk PROJ-123 download

# 查找请求 ID（自动解析日志文件路径）
qk PROJ-123 find [REQUEST_ID]

# 搜索关键词（自动解析日志文件路径）
qk PROJ-123 search [SEARCH_TERM]
```

#### 相关文件
- `src/lib/log/logs.rs` - 核心业务逻辑
- `src/commands/qk/` - 命令包装器
- `src/bin/qk.rs` - CLI 入口

---

## 💾 数据存储

### 配置文件位置

所有配置文件存储在 `${HOME}/.workflow/` 目录下：

- `jira-status.json` - Jira 项目状态映射配置
- `work-history.json` - PR 和 Jira ticket 的关联历史

### Jira Status 配置 (`jira-status.json`)

存储每个 Jira 项目在创建 PR 和合并 PR 时的状态映射关系。

**格式**：
```json
{
  "WEW": {
    "created-pr": "In Progress",
    "merged-pr": "Done"
  },
  "PROJ": {
    "created-pr": "In Review",
    "merged-pr": "Done"
  }
}
```

**使用场景**：
- 创建 PR 时自动更新 Jira ticket 状态为 `created-pr` 配置的状态
- 合并 PR 时自动更新 Jira ticket 状态为 `merged-pr` 配置的状态

### Work History (`work-history.json`)

存储 PR ID 到 Jira ticket 的映射关系，用于在合并 PR 时自动查找对应的 Jira ticket。

**格式**：
```json
{
  "456": {
    "jira_ticket": "PROJ-123",
    "pr_url": "https://github.com/xxx/pull/456",
    "created_at": "2024-01-15T10:30:00Z",
    "merged_at": null,
    "repository": "github.com/xxx/yyy",
    "branch": "feature/PROJ-123-add-feature"
  }
}
```

**使用流程**：
1. **写入时机**：创建 PR 时自动记录映射关系
2. **读取时机**：合并 PR 时查找对应的 Jira ticket，自动更新状态
3. **容错机制**：如果历史记录中没有，会尝试从 PR 标题中提取 Jira ticket ID

**相关文件**：
- `src/lib/jira/status.rs` - 工作历史的读写逻辑

---

## 📝 开发规范

### 代码风格
- 使用 `rustfmt` 格式化代码
- 使用 `clippy` 进行 lint 检查
- 遵循 Rust 命名约定

### 错误处理
- 使用 `anyhow::Result` 作为返回类型
- 提供清晰的错误信息
- 使用 `Context` 添加上下文信息

### 文档
- 所有公共函数添加文档注释
- 使用 `///` 编写文档
- 包含使用示例

### 提交规范
- 使用 Conventional Commits 格式
- 保持提交历史清晰

---
