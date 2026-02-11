# SSH 工具需求文档

## 概述

为 Workflow CLI 提供 SSH 密钥管理能力，封装 `ssh-add` 等系统命令，支持 Git push/pull 前的密钥检查与加载。

**状态**：⏳ 待实施
**优先级**：中

---

## 命令设计

| 命令 | 功能 |
|------|------|
| `workflow ssh status` | 列出 ssh-agent 中已加载的密钥及指纹 |
| `workflow ssh add` | 添加默认或指定密钥到 agent（支持有效期） |
| `workflow ssh remove` | 从 agent 移除指定密钥 |

---

## 功能说明

### ssh status

- 执行 `ssh-add -l` 获取密钥列表
- 解析输出，展示指纹与路径（如有）
- agent 为空时给出友好提示

### ssh add

- 参数：`[KEY_PATH]`（可选）、`--lifetime SECS`（可选）
- 默认：添加 `~/.ssh/id_ed25519` 或 `~/.ssh/id_rsa`
- 实现：调用 `ssh-add [path]` 或 `ssh-add -t SECS [path]`
- macOS：可选支持 `--apple-use-keychain`

### ssh remove

- 参数：`KEY_PATH`（必填）
- 实现：调用 `ssh-add -d [path]`
- 全部移除：`ssh-add -D`（可选子命令）

---

## 技术要点

1. **实现方式**：`std::process::Command` 调用系统 `ssh-add`
2. **分层**：domain（SshService trait）→ storage/services（实现）→ app（命令）
3. **错误**：解析 stderr，映射为 `ServiceError::SSH`
4. **跨平台**：`-K`/`--apple-use-keychain` 仅 macOS 生效

---

## 实施顺序

1. Phase 1：`ssh status` + domain/storage 骨架
2. Phase 2：`ssh add`
3. Phase 3：`ssh remove`
