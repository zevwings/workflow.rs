# util 模块迁移规划

## 概述

将跨命令复用的工具逻辑从各命令模块抽离至 `crates/app/src/util/`，统一管理。

**状态**：⏳ 待实施

---

## 1. 迁移范围

### 1.1 适合迁移

| 模块 | 用途 | 使用方 |
|------|------|--------|
| **ssh_guard**（新建） | pull/push 前的 SSH 保障 | repo、commit、pr merge、pr create、pr update |
| **to_slug** | summary → URL 友好 slug（小写、连字符） | branch/create、pr/create、pr/reword |
| **branch_type_from_branch_name** | 从分支名解析 BranchType | branch/create、pr/create、pr/reword |
| **strip_branch_type_prefix** | 去掉分支名类型前缀（如 `feature/`） | generate_branch_name_by_summary 内部 |

### 1.2 不建议迁移

| 模块 | 原因 |
|------|------|
| pr/utils | PR 模板渲染、PR ID 验证、generate_pull_request_body 等，强依赖 PR 流程和配置 |
| jira/utils | Jira ID 交互、状态配置，强依赖 Jira 流程 |
| branch/utils 其余部分 | generate_branch_name_from_template、generate_branch_name_from_jira 等依赖 bootstrap 和模板配置 |
| commands/args.rs | CLI 参数定义，放在 commands 下更合适 |

---

## 2. 目录结构

```
crates/app/src/util/
├── mod.rs
├── ssh_guard.rs      # SSH 保障（新建）
└── branch.rs         # to_slug, branch_type_from_branch_name, strip_branch_type_prefix
```

---

## 3. 依赖关系

- **branch/create**、**pr/create**、**pr/reword**：`use crate::util::branch::{to_slug, branch_type_from_branch_name}`
- **branch/utils**：保留 `generate_branch_name_*` 等业务逻辑，内部使用 `crate::util::branch::*`
