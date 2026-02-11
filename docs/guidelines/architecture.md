# Workflow 架构设计

## 项目概述

Rust CLI 工具，自动化开发工作流（PR、Jira、日志等）。采用 Cargo workspace 多 crate 结构。

---

## Crate 结构

| Crate | 职责 |
|-------|------|
| **app** | CLI 入口、命令分发、bootstrap、interactive |
| **domain** | 领域实体、配置、仓储 trait（config、git、github、jira、pr、path、summary 等） |
| **storage** | 仓储实现（git、github、jira、config） |
| **services** | 应用服务（PR、分支、提交、摘要等） |
| **http** | HTTP 客户端、重试 |
| **llm** | LLM 客户端与对话 |
| **toolkit** | 日志、路径、模板、shell、rollback、util |
| **prompt** | 对话框、表单、进度、输出样式 |
| **di** | 依赖注入、容器绑定 |

**依赖方向**：app → (commands) → domain/storage/services → http/llm/toolkit/prompt

---

## 常用命令

| 操作 | 命令 |
|------|------|
| 构建 | `cargo build -p app` 或 `cargo build --release -p app` |
| 运行 | `cargo run -p app`（默认 `workflow` bin）或 `cargo run -p app --bin install` |
| 测试 | `cargo test` 或 `make test` |
| 性能 | `make bench` / `make bench-cli` / `make bench-storage` |
| 文档 | `cargo doc --open` |

---

## 配置

- 全局：`~/.workflow/`（由 `toolkit::paths` 定义）
- 项目：`.workflow/config.toml`

---

## 相关文档

- [开发规范](./development.md) - 代码风格、错误处理、命名
- [测试规范](./testing.md) - 测试组织与命令

**最后更新**: 2025-02-11
