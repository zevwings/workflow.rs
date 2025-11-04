# Workflow 架构设计

## 📋 项目概述

Workflow 是一个 Rust 编写的 CLI 工具，用于自动化开发工作流，提供 PR 管理、Jira 集成、日志处理等功能。

---

## 🏗️ 模块划分

```
src/
├── main.rs                 # CLI 入口和命令分发
├── commands/               # 命令实现模块
│   ├── pr/                 # PR 相关命令
│   │   ├── create.rs
│   │   ├── merge.rs
│   │   ├── show.rs
│   │   └── list.rs
│   ├── logs/               # 日志相关命令
│   │   ├── download.rs
│   │   ├── find.rs
│   │   └── search.rs
│   ├── jira/               # Jira 相关命令
│   │   ├── status.rs
│   │   └── show.rs
│   ├── update.rs           # 快速更新
│   ├── proxy.rs            # 代理检查
│   └── check.rs            # 综合检查
├── lib/                    # 核心功能库
│   ├── git.rs              # Git 操作
│   ├── github.rs           # GitHub API 集成
│   ├── codeup.rs           # Codeup API 集成
│   ├── jira.rs             # Jira API 集成
│   ├── logs.rs             # 日志处理
│   ├── repo.rs             # 仓库类型检测
│   ├── config.rs           # 配置管理
│   ├── ai.rs               # AI 功能模块（PR 标题生成）
│   └── utils.rs            # 工具函数
└── types/                  # 类型定义
    ├── pr.rs               # PR 相关类型
    ├── jira.rs             # Jira 相关类型
    └── config.rs           # 配置类型
```

---

## 🧠 核心模块设计

### AI 模块 (`lib::ai`)

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
- `src/lib/ai.rs` - Rust 原生实现
- `src/commands/pr/create.rs` - PR 创建命令（已集成 AI 功能）

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
