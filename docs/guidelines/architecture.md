# Workflow 架构设计

## 项目概述

Rust CLI 工具，自动化开发工作流（PR、Jira、日志等）。Cargo workspace 多 crate 结构。

---

## Crate 职责

| Crate | 职责 |
|-------|------|
| **app** | CLI 入口、命令分发、bootstrap、interactive |
| **domain** | 领域实体、配置、仓储 trait |
| **storage** | 仓储实现（git、github、jira、config） |
| **services** | 应用服务（PR、分支、提交、摘要等） |
| **client** | 客户端 trait 与类型定义（Http、LLM、GitHub、Jira） |
| **infra** | 基础设施实现（HTTP 客户端、重试、LLM 客户端、bootstrap） |
| **toolkit** | 日志、路径、模板、shell、rollback、util |
| **prompt** | 对话框、表单、进度、输出样式 |
| **di** | 依赖注入、容器绑定 |

**依赖方向**：app → (commands) → domain/storage/services → client/toolkit/prompt；app → infra；infra → client

---

## 配置

- 全局：`~/.workflow/`
- 项目：`.workflow/config.toml`

---

**最后更新**: 2025-02-20
