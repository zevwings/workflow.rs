# SSH 远程操作保障

## 概述

在执行 pull/push 等远程 Git 操作时，若使用 SSH 协议，可能因 ssh-agent 或密钥未就绪而失败。本需求定义**自动保障**的业务规则。

---

## 1. 适用范围

- **触发场景**：任何执行 pull/push 的命令（如 `repo pull`、`repo push`、`commit --push`、`pr create`、`pr merge` 等）
- **生效条件**：仅当 origin 远程 URL 为 **SSH 协议**（`git@...` 或 `ssh://...`）时
- **不适用**：HTTPS 远程无需处理

---

## 2. 问题场景与处理规则

| 场景 | 表现 | 处理规则 |
|------|------|----------|
| 密钥已生成但未加入 agent（单个） | agent 可用，但无已加载密钥；`~/.ssh` 下仅一个密钥 | **自动添加**该密钥到 agent，然后执行操作 |
| 密钥已生成但未加入 agent（多个） | agent 可用，但无已加载密钥；`~/.ssh` 下有多个密钥 | **交互式选择**要添加的密钥，添加后执行操作 |
| 未生成密钥 | `~/.ssh` 下无密钥文件 | **交互式生成**密钥，添加后执行操作 |
| ssh-agent 未运行 | `SSH_AUTH_SOCK` 未设置或 agent 不可用 | **无法自动修复**，提示用户执行 `eval $(ssh-agent)` |

---

## 3. 执行策略

### 3.1 执行前预检查（仅 SSH 远程）

1. agent 不可用 → 提示并中止
2. agent 可用且已有密钥 → 直接执行
3. agent 可用但无密钥，且 `~/.ssh` 下有**单个**密钥 → 自动添加该密钥，再执行
4. agent 可用但无密钥，且 `~/.ssh` 下有**多个**密钥 → 交互式选择要添加的密钥，添加后执行
5. agent 可用但无密钥，且 `~/.ssh` 下无密钥 → 交互式生成密钥并添加，再执行

### 3.2 执行失败后重试（仅 SSH 远程 + 疑似认证错误）

- 若错误信息表明 SSH 认证失败，尝试自动添加密钥并**重试一次**
- 重试仍失败 → 返回带引导信息的错误（提示 `workflow ssh add` 或 `workflow ssh generate`）

---

## 4. 边界约定

- **密钥选择**：单个密钥时自动添加；多个密钥时交互式选择（按常见优先级展示：ed25519 > rsa > ecdsa）
- **非交互环境**：自动添加失败时给出明确错误和操作指引，不阻塞
- **带 passphrase 的密钥**：`ssh-add` 会交互式询问；无 TTY 时可能失败，按失败处理

---

## 5. 与现有 SSH 能力的关系

- 依赖 `workflow ssh` 命令提供的密钥生成与 agent 管理能力
- 本保障为**透明增强**：用户无感知时自动修复；无法修复时引导至 `workflow ssh` 子命令

---

## 6. 实现要点

### 6.1 集成点

- **实现位置**：`app/src/util/ssh_guard.rs`
- 在 `repo pull`、`repo push`、`commit --push`、`pr create`、`pr merge` 等命令中，于执行 pull/push 前调用 `ensure_ssh_ready(origin_url)`
- `origin_url` 通过 `git_repo.get_repo_info().origin_url` 获取
- **交互逻辑**：若 agent 可用但无已加载密钥，则根据 `~/.ssh` 下密钥数量分支：
  - **多个密钥** → 交互式选择要添加的密钥，添加后执行
  - **无密钥** → 交互式生成密钥并添加，再执行

### 6.2 SSH 远程判定

- `url.starts_with("git@") || url.starts_with("ssh://")`（或等价规则）

### 6.3 认证错误识别

- 重试触发条件：错误信息包含 `Permission denied`、`Host key verification failed`、`no mutual signature algorithm` 等 SSH 认证相关关键词

### 6.4 错误提示文案

- agent 不可用：`ssh-agent 未运行，请执行 eval $(ssh-agent) 后重试`
- 无密钥：`未找到 SSH 密钥，请执行 workflow ssh generate 生成密钥`
- 有密钥未加载：`密钥未加载到 agent，请执行 workflow ssh add 添加密钥`
