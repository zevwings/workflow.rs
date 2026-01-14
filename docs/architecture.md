# Workflow 架构设计

## 📋 项目概述

Workflow 是一个 Rust 编写的 CLI 工具，用于自动化开发工作流，提供 PR 管理、Jira 集成、日志处理等功能。

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
│  - commands/commit/ (Commit 管理)        │
│  - commands/github/ (GitHub 账号管理)   │
│  - commands/check/ (环境检查)            │
│  - commands/proxy/ (代理管理)            │
│  - commands/config/ (配置管理)           │
│  - commands/repo/ (仓库配置管理)          │
│  - commands/lifecycle/ (生命周期管理)    │
└─────────────────┬───────────────────────┘
                  │
┌─────────────────▼───────────────────────┐
│      核心业务逻辑层 (lib/)               │
│  - lib/base/     (基础设施：HTTP、LLM、Settings、Shell、Util) │
│  - lib/pr/       (PR 功能)               │
│  - lib/jira/     (Jira 集成，包含日志处理) │
│  - lib/git/      (Git 操作)              │
│  - lib/commit/   (Commit 业务逻辑)       │
│  - lib/completion/ (Completion 管理)    │
│  - lib/proxy/    (代理管理)              │
│  - lib/repo/     (仓库配置管理)          │
│  - lib/template/ (模板系统)              │
│  - lib/rollback/ (回滚管理)              │
└─────────────────────────────────────────┘
```

### 模块职责

- **CLI 入口层** (`main.rs` 和 `bin/`): `main.rs` 是 `workflow` 主命令的入口，负责命令行参数解析和命令分发；`bin/install.rs` 是独立的安装命令入口
- **命令封装层** (`commands/`): 提供 CLI 命令封装，处理用户交互和日志输出，所有命令都通过 `workflow` 主命令调用
- **核心业务逻辑层** (`lib/`): 包含所有业务逻辑，可被其他模块复用

### 数据流向

```
用户输入 → main.rs → commands/*.rs → lib/*.rs → 执行操作
```

---

## 🧠 核心模块设计

核心模块位于 `lib/` 目录下，提供所有业务逻辑实现。各模块简要说明如下：

### 基础设施模块 (`lib::base`)

- **HTTP 模块** (`lib::base::http`) - 统一 HTTP 客户端，支持认证、重试、延迟解析等特性
- **LLM 模块** (`lib::base::llm`) - 统一配置驱动的 LLM 客户端，支持 OpenAI、DeepSeek 和代理 API
- **Settings 模块** (`lib::base::settings`) - 配置管理，提供统一的配置加载和路径管理，支持 iCloud 存储（macOS）
- **Shell 模块** (`lib::base::shell`) - Shell 检测与管理，支持配置自动加载
- **工具函数模块** (`lib::base::util`) - 通用工具函数（日志、字符串、浏览器、剪贴板等）

### 业务模块

- **Git 模块** (`lib::git`) - Git 仓库操作功能，包括分支管理、提交、暂存、配置管理等
- **Commit 模块** (`lib::commit`) - Commit 相关的业务逻辑，包括 amend 和 reword 操作
- **Jira 模块** (`lib::jira`) - Jira API 集成功能，包括 Issue 管理、用户管理、状态管理、工作历史记录和日志处理等
- **PR 模块** (`lib::pr`) - 跨平台 PR 管理功能，支持 GitHub 和 Codeup，包括创建、合并、关闭、更新等操作
- **Completion 模块** (`lib::completion`) - Shell 补全脚本生成和管理功能
- **Proxy 模块** (`lib::proxy`) - 代理管理功能，包括系统代理读取、配置生成和管理
- **Repo 模块** (`lib::repo`) - 仓库级配置管理功能，配置存储在项目根目录的 `.workflow/config.toml` 文件中
- **Template 模块** (`lib::template`) - 模板渲染功能，支持分支命名模板、PR body 模板、Commit 消息模板等
- **Rollback 模块** (`lib::rollback`) - 回滚管理功能，支持备份、恢复和清理操作

**详细 API 文档**：运行 `cargo doc --open` 查看完整的 API 文档。

---

## 💾 数据存储

### 配置文件位置

配置文件存储在以下位置：

- `~/.workflow/config/workflow.toml` - 主配置文件（Jira、GitHub、日志、LLM、Codeup 配置）
- `~/.workflow/config/llm.toml` - LLM 配置文件（可选）
- `~/.workflow/config/jira-status.toml` - Jira 项目状态映射配置
- `~/.workflow/config/jira-users.toml` - Jira 用户缓存配置
- `~/.workflow/work-history/` - PR 和 Jira ticket 的关联历史（按仓库存储）
- `.workflow/config.toml` - 项目级配置（分支前缀、忽略列表等）

### 配置说明

#### Jira Status 配置

存储每个 Jira 项目在创建 PR 和合并 PR 时的状态映射关系。

```toml
[WEW]
created-pr = "In Progress"
merged-pr = "Done"
```

#### Branch 配置（项目级配置）

分支配置存储在项目根目录的 `.workflow/config.toml` 文件中。

```toml
[branch]
prefix = "feature"
ignore = [
    "main",
    "master",
    "develop",
]
```

#### Work History

存储 PR ID 到 Jira ticket 的映射关系，用于在合并 PR 时自动查找对应的 Jira ticket。

---

## 📚 相关文档

- [开发规范](./development.md) - 代码风格、错误处理、命名、模块组织等开发规范
- [测试规范](./testing.md) - 测试组织、编写、命令参考
- [迁移文档](./migration/README.md) - 版本迁移指南

**API 文档**：运行 `cargo doc --open` 查看完整的 API 文档。

---

**最后更新**: 2025-01-27
