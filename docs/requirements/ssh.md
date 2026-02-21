# SSH 工具需求

## 概述

为 Workflow CLI 提供 SSH 密钥管理能力：**密钥生成**和 **ssh-agent 管理**。

**状态**：✅ 已实施

---

## 1. 命令与参数

| 命令 | 功能 |
|------|------|
| `workflow ssh generate` | 生成 Ed25519/RSA 密钥对 |
| `workflow ssh status` | 列出 ssh-agent 中已加载的密钥 |
| `workflow ssh add` | 添加密钥到 agent |
| `workflow ssh remove` | 从 agent 移除密钥 |

| 命令 | 参数 | 说明 |
|------|------|------|
| `generate` | `-o, --output <path>` | 输出路径，默认 `~/.ssh/id_ed25519` |
| `generate` | `-a, --algorithm <ed25519\|rsa>` | 算法，默认 ed25519 |
| `generate` | `-C, --comment <comment>` | 密钥注释 |
| `generate` | `--force` | 覆盖已存在文件 |
| `generate` | `--no-passphrase` | 不加密私钥（脚本/CI 用） |
| `add` | `[path]` 或 `-k, --key <path>` | 密钥路径 |
| `add` | `-t, --lifetime <seconds>` | 有效期（秒） |
| `remove` | `-f, --fingerprint <fp>` | 按指纹移除 |
| `remove` | `--all` | 清空 agent |

---

## 2. 业务规则

### generate

- 默认输出 `~/.ssh/id_ed25519`；RSA 默认 4096 位
- 文件已存在时拒绝覆盖（`--force` 可覆盖）
- 私钥权限 600；`~/.ssh` 不存在时自动创建

### status

- 表格展示：指纹 | 路径 | 算法 | 有效期
- agent 为空时友好提示

### add

- 无参数时扫描 `~/.ssh/` 下常见私钥（`id_ed25519`、`id_ed25519_sk`、`id_rsa`、`id_ecdsa`），交互选择
- 无 `--lifetime` 时交互选择有效期（1h / 8h / 永久）
- 加密密钥时交互输入 passphrase

### remove

- 无参数时展示已加载密钥列表，多选移除

---

## 3. 交互式设计

**原则**：无参数时走交互式流程；显式参数时直接执行，便于脚本化。

| 命令 | 交互式行为 |
|------|------------|
| `generate` | 选择算法 → 确认输出路径 → 询问是否设置 passphrase |
| `add` | 扫描并选择密钥 → 选择有效期 → 输入 passphrase（若需要） |
| `remove` | 多选要移除的密钥 |

---

## 4. 与 Workflow 集成

- 新增 `SSHStage`，Stage 顺序：**Jira → SSH → GitHub → LLM → Log**
- **is_configured**：ssh-agent 可用且至少有一个已加载密钥
- **check**：检查 agent 状态和可用密钥，无密钥时提示但不阻塞
- **setup**：无密钥时交互引导 →「生成新密钥」或「添加已有密钥」

---

## 5. 错误场景

| 错误 | 场景 |
|------|------|
| `AgentUnavailable` | ssh-agent 未运行或 `SSH_AUTH_SOCK` 未设置 |
| `KeyNotFound` | 指定路径的密钥不存在 |
| `KeyAlreadyExists` | generate 目标路径已存在且未使用 `--force` |
| `GenerationFailed` | 密钥生成失败 |
| `AddFailed` | 添加密钥失败 |
| `RemoveFailed` | 移除密钥失败 |
