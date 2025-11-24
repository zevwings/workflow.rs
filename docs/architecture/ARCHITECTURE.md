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
│   └── install.rs          # 安装命令入口（使用 commands::lifecycle::install）
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
│   │   ├── update.rs       # 更新 PR
│   │   └── integrate.rs    # 集成分支命令
│   ├── log/                # 日志操作命令
│   │   ├── mod.rs          # Log 命令模块声明
│   │   ├── download.rs     # 下载日志命令
│   │   ├── find.rs         # 查找请求 ID 命令
│   │   └── search.rs       # 搜索关键词命令
│   ├── jira/               # Jira 操作命令
│   │   ├── mod.rs          # Jira 命令模块声明
│   │   ├── info.rs         # 显示 ticket 信息命令
│   │   ├── attachments.rs  # 下载附件命令
│   │   └── clean.rs        # 清理日志目录命令
│   ├── branch/             # 分支管理命令
│   │   ├── mod.rs          # Branch 命令模块声明
│   │   ├── clean.rs        # 清理本地分支命令
│   │   └── ignore.rs      # 管理分支忽略列表命令
│   ├── github/             # GitHub 账号管理命令
│   │   ├── mod.rs          # GitHub 命令模块声明
│   │   ├── github.rs       # GitHub 账号管理实现
│   │   └── helpers.rs      # GitHub 辅助函数（账号信息收集等）
│   ├── check/              # 环境检查命令
│   │   ├── mod.rs          # Check 命令模块声明
│   │   └── check.rs        # 综合检查命令（git_status, network）
│   ├── proxy/              # 代理管理命令
│   │   ├── mod.rs          # Proxy 命令模块声明
│   │   └── proxy.rs        # 代理管理命令（on, off, check）
│   ├── config/             # 配置管理命令
│   │   ├── mod.rs          # Config 命令模块声明
│   │   ├── setup.rs        # 初始化设置命令（交互式配置）
│   │   ├── show.rs         # 配置查看命令（显示当前配置）
│   │   ├── log.rs           # 日志级别管理命令（set, check）
│   │   └── completion.rs   # Shell Completion 管理命令
│   └── lifecycle/          # 生命周期管理命令
│       ├── mod.rs          # Lifecycle 命令模块声明
│       ├── install.rs      # 安装命令实现（安装二进制和补全脚本）
│       ├── uninstall.rs    # 卸载命令实现（清理所有相关文件）
│       └── update.rs       # 更新命令实现（重新构建、更新二进制文件）
└── lib/                    # 核心功能库（业务逻辑层）
    ├── mod.rs              # 库模块声明
    ├── base/               # 基础设施模块
    │   ├── mod.rs          # Base 模块声明
    │   ├── http/           # HTTP 客户端模块
    │   │   ├── mod.rs      # HTTP 模块声明
    │   │   ├── client.rs   # HTTP 客户端（单例模式）
    │   │   ├── config.rs   # 请求配置
    │   │   ├── response.rs # HTTP 响应（延迟解析）
    │   │   ├── auth.rs     # Basic Authentication
    │   │   ├── method.rs   # HTTP 方法枚举
    │   │   ├── parser.rs   # 响应解析器
    │   │   └── retry.rs   # HTTP 重试工具
    │   ├── llm/            # LLM 集成（AI 功能）
    │   │   ├── mod.rs      # LLM 模块声明
    │   │   ├── client.rs   # LLM 统一客户端
    │   │   └── types.rs    # LLM 请求参数类型
    │   ├── settings/       # 配置管理
    │   │   ├── mod.rs      # Settings 模块声明
    │   │   ├── settings.rs # Settings 结构体和配置加载
    │   │   ├── paths.rs    # 路径管理（配置文件、安装路径、Shell 路径）
    │   │   └── defaults.rs # 默认值辅助函数
    │   ├── shell/          # Shell 检测与管理
    │   │   ├── mod.rs      # Shell 模块声明
    │   │   ├── detect.rs  # Shell 类型检测
    │   │   ├── reload.rs  # Shell 配置重新加载
    │   │   └── config.rs  # Shell 配置文件管理
    │   └── util/           # 工具函数
    │       ├── mod.rs      # Util 模块声明
    │       ├── logger.rs  # 日志输出系统
    │       ├── string.rs  # 字符串处理工具
    │       ├── browser.rs # 浏览器操作
    │       ├── clipboard.rs # 剪贴板操作
    │       ├── unzip.rs   # 文件解压工具
    │       ├── checksum.rs # 校验和验证工具
    │       └── confirm.rs # 用户确认对话框
    ├── git/                # Git 操作模块
    │   ├── mod.rs          # Git 模块声明和导出
    │   ├── branch.rs       # 分支管理操作
    │   ├── commit.rs       # 提交相关操作
    │   ├── repo.rs         # 仓库检测和类型识别
    │   ├── stash.rs        # 暂存管理
    │   ├── config.rs       # Git 配置管理
    │   ├── pre_commit.rs   # Pre-commit hooks 支持
    │   ├── helpers.rs      # Git 操作辅助函数
    │   └── types.rs        # Git 相关类型定义
    ├── jira/               # Jira API 集成
    │   ├── mod.rs          # Jira 模块声明
    │   ├── api/            # API 方法子模块
    │   │   ├── mod.rs      # API 模块声明
    │   │   ├── http_client.rs  # JiraHttpClient (HTTP 层)
    │   │   ├── issue.rs    # Issue/Ticket 相关 API
    │   │   ├── user.rs     # 用户相关 API
    │   │   └── project.rs  # 项目相关 API
    │   ├── config.rs       # ConfigManager (TOML 配置管理器)
    │   ├── client.rs       # JiraClient 包装器（向后兼容）
    │   ├── helpers.rs      # Jira 辅助函数（项目提取等）
    │   ├── types.rs        # 数据模型定义
    │   ├── users.rs        # 用户信息管理
    │   ├── ticket.rs       # Ticket/Issue 操作
    │   ├── status.rs       # 状态管理
    │   ├── history.rs      # 工作历史记录管理
    │   └── logs/           # 日志处理模块
    │       ├── mod.rs      # JiraLogs 结构体定义
    │       ├── constants.rs # 常量定义
    │       ├── helpers.rs  # 日志处理辅助函数
    │       ├── path.rs     # 路径管理功能
    │       ├── download.rs # 下载功能
    │       ├── search.rs   # 搜索和查找功能
    │       ├── zip.rs      # ZIP 处理功能
    │       └── clean.rs    # 清理功能
    ├── pr/                 # PR 相关功能
    │   ├── mod.rs          # PR 模块声明
    │   ├── platform.rs     # PlatformProvider trait 和工厂函数
    │   ├── helpers.rs      # PR 辅助函数
    │   ├── llm.rs          # LLM 功能（PR 标题生成）
    │   ├── github/         # GitHub 平台实现
    │   │   ├── mod.rs      # GitHub 模块导出
    │   │   ├── platform.rs # GitHub 平台实现
    │   │   ├── requests.rs # GitHub API 请求结构体
    │   │   ├── responses.rs # GitHub API 响应结构体
    │   │   └── errors.rs   # GitHub 错误处理
    │   └── codeup/         # Codeup 平台实现
    │       ├── mod.rs      # Codeup 模块导出
    │       ├── platform.rs # Codeup 平台实现
    │       ├── requests.rs # Codeup API 请求结构体
    │       ├── responses.rs # Codeup API 响应结构体
    │       └── errors.rs   # Codeup 错误处理
    ├── completion/         # Shell Completion 管理
    │   ├── mod.rs          # Completion 模块声明
    │   ├── completion.rs   # Completion 管理工具
    │   ├── generate.rs     # Completion 脚本生成器
    │   └── files.rs        # Completion 文件工具函数
    ├── proxy/              # 代理管理
    │   ├── mod.rs          # Proxy 模块声明
    │   ├── proxy.rs        # 类型定义（ProxyType, ProxyInfo, ProxyConfig）
    │   ├── system_reader.rs # 系统代理读取器
    │   ├── config_generator.rs # 代理配置生成器
    │   └── manager.rs      # 代理管理器
    └── rollback/           # 回滚管理
        ├── mod.rs          # Rollback 模块声明
        └── rollback.rs     # 回滚管理器（备份、恢复、清理）
```

---

## 🏛️ 架构层次

### 三层架构设计

```
┌─────────────────────────────────────────┐
│         CLI 入口层                      │
│  - main.rs (workflow 主命令)            │
│  - bin/install.rs (独立安装命令)        │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      命令封装层 (commands/)              │
│  - commands/log/  (日志操作)              │
│  - commands/jira/ (Jira 操作)             │
│  - commands/pr/  (PR 操作)               │
│  - commands/branch/ (分支管理)           │
│  - commands/github/ (GitHub 账号管理)   │
│  - commands/check/ (环境检查)            │
│  - commands/proxy/ (代理管理)            │
│  - commands/config/ (配置管理)           │
│  - commands/lifecycle/ (生命周期管理)    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      核心业务逻辑层 (lib/)               │
│  - lib/base/     (基础设施：HTTP、LLM、Settings、Shell、Util) │
│  - lib/pr/       (PR 功能)               │
│  - lib/jira/     (Jira 集成，包含日志处理) │
│  - lib/git/      (Git 操作)              │
│  - lib/completion/ (Completion 管理)    │
│  - lib/proxy/    (代理管理)              │
│  - lib/rollback/ (回滚管理)              │
└─────────────────────────────────────────┘
```

### 模块职责

- **CLI 入口层** (`main.rs` 和 `bin/`): `main.rs` 是 `workflow` 主命令的入口，负责命令行参数解析和命令分发；`bin/install.rs` 是独立的安装命令入口
- **命令封装层** (`commands/`): 提供 CLI 命令封装，处理用户交互和日志输出，所有命令都通过 `workflow` 主命令调用
- **核心业务逻辑层** (`lib/`): 包含所有业务逻辑，可被其他模块复用

### 数据流向

```
用户输入 → main.rs → commands/pr/*.rs → lib/pr/*.rs → 执行操作
用户输入 → main.rs → commands/log/*.rs → lib/jira/logs/*.rs → 执行操作
用户输入 → main.rs → commands/jira/*.rs → lib/jira/*.rs → 执行操作
用户输入 → main.rs → commands/branch/*.rs → lib/git/branch.rs → 执行操作
用户输入 → main.rs → commands/github/*.rs → lib/git/config.rs → 执行操作
用户输入 → main.rs → commands/check/*.rs → lib/git/*.rs → 执行操作
用户输入 → main.rs → commands/proxy/*.rs → lib/proxy/*.rs → 执行操作
用户输入 → main.rs → commands/config/*.rs → lib/base/settings/*.rs → 执行操作
用户输入 → main.rs → commands/lifecycle/*.rs → lib/completion/*.rs → 执行操作
```

---

## 🧠 核心模块设计

核心模块位于 `lib/` 目录下，提供所有业务逻辑实现。各模块简要说明如下：

### 基础设施模块 (`lib::base`)

- **HTTP 模块** (`lib::base::http`) - 统一 HTTP 客户端，支持认证、重试、延迟解析等特性
  - 详细架构请参考 [HTTP_ARCHITECTURE.md](./lib/HTTP_ARCHITECTURE.md)

- **LLM 模块** (`lib::base::llm`) - 统一配置驱动的 LLM 客户端，支持 OpenAI、DeepSeek 和代理 API
  - 详细架构请参考 [LLM_ARCHITECTURE.md](./lib/LLM_ARCHITECTURE.md)

- **Settings 模块** (`lib::base::settings`) - 配置管理，提供统一的配置加载和路径管理
  - 详细架构请参考 [SETTINGS_ARCHITECTURE.md](./lib/SETTINGS_ARCHITECTURE.md)

- **Shell 模块** (`lib::base::shell`) - Shell 检测与管理，支持配置自动加载
  - 详细架构请参考 [SHELL_ARCHITECTURE.md](./lib/SHELL_ARCHITECTURE.md)

- **工具函数模块** (`lib::base::util`) - 通用工具函数（日志、字符串、浏览器、剪贴板等）
  - 详细架构请参考 [TOOLS_ARCHITECTURE.md](./lib/TOOLS_ARCHITECTURE.md)

### Git 模块 (`lib::git`)

提供 Git 仓库操作功能，包括分支管理、提交、暂存、配置管理等。
- 详细架构请参考 [GIT_ARCHITECTURE.md](./lib/GIT_ARCHITECTURE.md)

### Jira 模块 (`lib::jira`)

提供 Jira API 集成功能，包括 Issue 管理、用户管理、状态管理、工作历史记录和日志处理等。
- 详细架构请参考 [JIRA_ARCHITECTURE.md](./lib/JIRA_ARCHITECTURE.md)

### PR 模块 (`lib::pr`)

提供跨平台 PR 管理功能，支持 GitHub 和 Codeup，包括创建、合并、关闭、更新等操作。
- 详细架构请参考 [PR_ARCHITECTURE.md](./lib/PR_ARCHITECTURE.md)

### Completion 模块 (`lib::completion`)

提供 Shell 补全脚本生成和管理功能。
- 详细架构请参考 [COMPLETION_ARCHITECTURE.md](./lib/COMPLETION_ARCHITECTURE.md)

### Proxy 模块 (`lib::proxy`)

提供代理管理功能，包括系统代理读取、配置生成和管理。
- 详细架构请参考 [PROXY_ARCHITECTURE.md](./lib/PROXY_ARCHITECTURE.md)

### Rollback 模块 (`lib::rollback`)

提供回滚管理功能，支持备份、恢复和清理操作。
- 详细架构请参考 [ROLLBACK_ARCHITECTURE.md](./lib/ROLLBACK_ARCHITECTURE.md)

---

## 💾 数据存储

### 配置文件位置

配置文件存储在以下位置：

- `~/.workflow/config/workflow.toml` - 主配置文件（Jira、GitHub、日志、LLM、Codeup 配置）
- `~/.workflow/config/llm.toml` - LLM 配置文件（可选，如果配置了 LLM）
- `~/.workflow/config/jira-status.toml` - Jira 项目状态映射配置
- `~/.workflow/config/jira-users.toml` - Jira 用户缓存配置
- `~/.workflow/config/branch.toml` - 分支清理忽略列表配置（按仓库分组）
- `~/.workflow/work-history/` - PR 和 Jira ticket 的关联历史（按仓库存储）

### Jira Status 配置 (`jira-status.toml`)

存储每个 Jira 项目在创建 PR 和合并 PR 时的状态映射关系。

**格式**：
```toml
[WEW]
created-pr = "In Progress"
merged-pr = "Done"

[PROJ]
created-pr = "In Review"
merged-pr = "Done"
```

**使用场景**：
- 创建 PR 时自动更新 Jira ticket 状态为 `created-pr` 配置的状态
- 合并 PR 时自动更新 Jira ticket 状态为 `merged-pr` 配置的状态

### Branch 配置 (`branch.toml`)

存储分支清理时的忽略列表，按仓库名分组。

**格式**：
```toml
[zevwings/workflow.rs]
ignore = [
    "zw/important-feature",
    "zw/refactor-code-base",
    "release/v1.0",
]

[company/project-name]
ignore = [
    "important-branch-name",
    "hotfix/critical",
]
```

**使用场景**：
- `workflow branch clean` 命令会自动排除配置文件中列出的分支
- 通过 `workflow branch ignore` 命令管理忽略列表（add/remove/list）

**相关文件**：
- `src/commands/branch/helpers.rs` - 分支配置管理逻辑（BranchConfig）
- `src/commands/branch/` - 分支管理命令实现

### Work History (`~/.workflow/work-history/`)

存储 PR ID 到 Jira ticket 的映射关系，用于在合并 PR 时自动查找对应的 Jira ticket。按仓库分别存储在不同的 JSON 文件中。

**文件位置**：
- `~/.workflow/work-history/{repo_id}.json` - 每个仓库对应一个 JSON 文件

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
- `src/lib/jira/history.rs` - 工作历史的读写逻辑

---

## 📝 开发规范

详细的开发规范请参考 [开发规范文档](../guidelines/DEVELOPMENT_GUIDELINES.md)。

该文档包含：
- 代码风格规范（格式化、Lint、命名约定）
- 错误处理规范（anyhow、上下文信息）
- 文档规范（公共 API 文档、注释格式）
- 命名规范（文件、函数、结构体、常量）
- 模块组织规范（目录结构、模块职责）
- Git 工作流（分支策略、工作流程）
- 提交规范（Conventional Commits）
- 测试规范（单元测试、集成测试）
- 代码审查（审查清单、审查重点）
- 依赖管理（添加依赖、版本管理）
- 开发工具（必需工具、常用命令）

---
