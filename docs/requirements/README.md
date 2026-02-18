# 需求分析文档

本目录存放 Workflow CLI 的需求分析、功能说明、设计方案和实施计划。

## 文档规范

### 文档类型

- **需求分析**：功能需求、非功能需求、约束条件
- **设计方案**：技术方案、架构设计、实现思路
- **实施计划**：分阶段计划、任务清单、时间估算
- **设计提案**：未实现的架构/功能提案
- **待办事项**：功能待办、改进计划

### 文档性质

本目录下的文档为**临时分析文档**，可随时删除。实施完成后 1 个月内可清理或转为参考文档。重要设计提案应移动到 `docs/guidelines/`。

---

## 当前需求文档

| 文档 | 状态 | 实现度 | 优先级 |
|-----|------|--------|--------|
| [jira.md](./jira.md) | 🚧 部分完成 | ~40% | 高 |
| [integration.md](./integration.md) | ⏳ 待实施 | 0% | 中 |
| [i18n.md](./i18n.md) | ⏳ 待实施 | 0% | 高 |
| [ssh.md](./ssh.md) | ⏳ 待实施 | 0% | 中 |
| [chinese-content.md](./chinese-content.md) | 📋 参考 | - | 中 |

### 文档详情

#### [jira.md](./jira.md)
- **已完成**：`jira info`、`changelog`、`comments`、`attachments`、`clean`；JIRA API 部分封装；PR 自动更新状态
- **待实现**：info 增强、新命令（assign、comment、create、list、watch）、批量操作

#### [integration.md](./integration.md)
- **内容**：更多平台支持（GitLab、Bitbucket）、通知系统（桌面、邮件）

#### [i18n.md](./i18n.md)
- **内容**：rust-i18n 框架、CLI/错误/日志国际化、LLM Prompt 语言支持、中英文双语

#### [ssh.md](./ssh.md)
- **内容**：SSH 密钥管理工具，`ssh status` / `add` / `remove` 子命令

#### [chinese-content.md](./chinese-content.md)
- **内容**：中文内容统一规范（适用范围、术语、句式）

---

## 使用指南

- **创建**：使用 `kebab-case` 命名，明确标注状态和实现度
- **更新**：及时更新进度、已完成任务、待办事项
- **归档**：实施完成后归档或删除，重要提案移至 `docs/guidelines/`

---

## 相关文档

- [开发规范](../guidelines/development.md)
- [架构设计](../guidelines/architecture.md)
- [测试规范](../guidelines/testing.md)
