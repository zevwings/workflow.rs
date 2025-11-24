# QK 命令迁移到 Log/Jira 命令分析文档

## 📋 概述

本文档分析从 `qk` 命令模块迁移到 `log` 和 `jira` 命令模块后的修改需求。

## ✅ 迁移完成状态

**迁移已完成！** 所有核心迁移任务已完成，代码可以正常编译和运行。

### 完成情况总结

- ✅ **阶段 1：模块结构修复** - 100% 完成
- ✅ **阶段 2：命令功能调整** - 100% 完成
- ✅ **阶段 3：命令注册** - 100% 完成
- ⚠️ **阶段 4：清理和验证** - 90% 完成（代码验证完成，功能测试需要手动验证）

### 迁移状态

- ✅ `qk` 文件夹内容已迁移到 `log` 和 `jira` 文件夹
- ✅ `main.rs` 中已更新为使用 `commands::log::` 和 `commands::jira::` 导入
- ✅ `commands/mod.rs` 中已移除 `pub mod qk;`，添加了 `pub mod log;` 和 `pub mod jira;`
- ✅ `log` 和 `jira` 文件夹已创建 `mod.rs` 文件
- ✅ 命令功能已调整（`log download` 只下载日志，`jira attachments` 下载所有附件）

### 文件迁移映射

| 原文件 (qk/) | 目标文件 | 说明 |
|-------------|---------|------|
| `download.rs` | `log/download.rs` | 下载日志命令（应只下载日志） |
| `find.rs` | `log/find.rs` | 查找请求 ID 命令 |
| `search.rs` | `log/search.rs` | 搜索关键词命令 |
| `clean.rs` | `jira/clean.rs` | 清理命令（已迁移） |
| `info.rs` | `jira/info.rs` | 显示 ticket 信息命令（已迁移） |
| `download.rs` | `jira/attachments.rs` | 下载附件命令（已重命名为 `attachments.rs`，下载所有附件） |

---

## 🔍 当前问题分析

### 1. 模块声明问题

**状态**：✅ 已解决
- `src/commands/mod.rs` 中已移除 `pub mod qk;`，添加了 `pub mod log;` 和 `pub mod jira;`
- `src/main.rs` 中已更新为使用 `use commands::log::` 和 `use commands::jira::` 导入
- `log` 和 `jira` 文件夹已创建 `mod.rs` 文件

### 2. 命令功能问题

**状态**：✅ 已解决

**问题 1：`log/download.rs`**
- ✅ 已移除 `download_all` 参数，方法内部固定为 `false`（只下载日志文件）
- ✅ CLI 中保留 `--all` 参数但忽略（向后兼容）

**问题 2：`jira/attachments.rs`**
- ✅ 已重命名为 `attachments.rs`
- ✅ 结构体已改为 `AttachmentsCommand`
- ✅ 已移除 `download_all` 参数，方法内部固定为 `true`（下载所有附件）
- ✅ 已在 `main.rs` 中添加 `JiraSubcommand::Attachments` 子命令

### 3. 命令注册问题

**状态**：✅ 已解决
- ✅ `main.rs` 中 `LogSubcommand::Download` 已更新导入路径
- ✅ `main.rs` 中 `JiraSubcommand` 已添加 `Attachments` 和 `Clean` 子命令
- ✅ `main.rs` 中命令分发逻辑已更新为使用 `log::` 和 `jira::` 模块

---

## 📝 TODO 清单

### 阶段 1：模块结构修复

- [x] **1.1** 创建 `src/commands/log/mod.rs`
  - 导出 `download`, `find`, `search` 模块
  - 导出对应的命令结构体：`DownloadCommand`, `FindCommand`, `SearchCommand`

- [x] **1.2** 创建 `src/commands/jira/mod.rs`
  - 导出 `info`, `clean`, `attachments` 模块（注意：`download` 应重命名为 `attachments`）
  - 导出对应的命令结构体：`InfoCommand`, `CleanCommand`, `AttachmentsCommand`

- [x] **1.3** 更新 `src/commands/mod.rs`
  - 移除 `pub mod qk;`
  - 添加 `pub mod log;`
  - 添加 `pub mod jira;`

- [x] **1.4** 更新 `src/main.rs` 导入
  - 移除 `use commands::qk::` 导入
  - 添加 `use commands::log::` 导入
  - 添加 `use commands::jira::` 导入

### 阶段 2：命令功能调整

- [x] **2.1** 修改 `src/commands/log/download.rs`
  - 固定 `download_all = false`（已移除参数，方法内部固定为 `false`）
  - 更新日志提示，明确只下载日志文件
  - 移除或更新 `--all` 参数相关的条件判断（已移除参数）
  - 更新注释，说明只下载日志文件

- [x] **2.2** 重命名 `src/commands/jira/download.rs` → `src/commands/jira/attachments.rs`
  - 重命名文件（已完成）
  - 将 `DownloadCommand` 重命名为 `AttachmentsCommand`（已完成）
  - 固定 `download_all = true`（已移除参数，方法内部固定为 `true`）
  - 更新日志提示，明确下载所有附件
  - 更新注释，说明下载所有附件

### 阶段 3：命令注册

- [x] **3.1** 更新 `src/main.rs` 中的 `LogSubcommand` 枚举
  - 保留 `all` 参数但在命令实现中忽略（已实现，使用 `all: _` 忽略参数）

- [x] **3.2** 更新 `src/main.rs` 中的 `JiraSubcommand` 枚举
  - 添加 `Attachments` 子命令（已完成）
  - 添加 `Clean` 子命令（已完成）

- [x] **3.3** 更新 `src/main.rs` 中的命令分发逻辑
  - 修改 `LogSubcommand::Download` 的处理（已完成，调用 `DownloadCommand::download(&jira_id)?`）
  - 添加 `JiraSubcommand::Attachments` 的处理（已完成，调用 `AttachmentsCommand::download(&jira_id)?`）
  - 添加 `JiraSubcommand::Clean` 的处理（已完成）
  - 更新其他命令的导入路径（`Find`, `Search` 等，已完成）

### 阶段 4：清理和验证

- [x] **4.1** 检查并修复所有编译错误
  - 运行 `cargo check` 验证编译（✅ 编译通过）
  - 修复所有导入路径错误（✅ 已完成）

- [x] **4.2** 更新帮助信息
  - 检查 `main.rs` 中的帮助文本是否需要更新（✅ 已更新）
  - 确保命令描述准确反映新功能（✅ 已完成）

- [ ] **4.3** 验证命令功能
  - 测试 `workflow log download PROJ-123` - 应只下载日志（需要手动测试）
  - 测试 `workflow log find PROJ-123` - 应正常工作（需要手动测试）
  - 测试 `workflow log search PROJ-123` - 应正常工作（需要手动测试）
  - 测试 `workflow jira attachments PROJ-123` - 应下载所有附件（需要手动测试）
  - 测试 `workflow jira info PROJ-123` - 应正常工作（需要手动测试）
  - 测试 `workflow jira clean PROJ-123` - 应正常工作（需要手动测试）

- [ ] **4.4** 更新文档（可选）
  - 更新架构文档中的文件路径引用（可选）
  - 更新 README 中的命令示例（可选）
  - 更新命令架构文档（可选）

---

## 🎯 实施建议

### 优先级

1. **高优先级**：阶段 1（模块结构修复）- 必须完成才能编译
2. **高优先级**：阶段 3（命令注册）- 必须完成才能使用命令
3. **中优先级**：阶段 2（命令功能调整）- 实现需求功能
4. **低优先级**：阶段 4（清理和验证）- 确保质量

### 实施顺序

1. 先完成阶段 1，确保代码可以编译
2. 然后完成阶段 3，确保命令可以注册和分发
3. 最后完成阶段 2，实现具体的功能需求
4. 最后进行阶段 4 的验证和清理

---

## 📌 注意事项

1. **向后兼容性**：
   - `log download` 命令的 `--all` 参数可以保留，但在实现中忽略
   - 这样可以保持 CLI 接口的向后兼容性

2. **命名一致性**：
   - `jira/download.rs` 必须重命名为 `attachments.rs` 以符合文档和语义
   - 结构体名称也要相应修改

3. **模块导出**：
   - 确保 `mod.rs` 文件正确导出所有需要的模块和结构体
   - 注意 Rust 的模块可见性规则

4. **错误处理**：
   - 确保所有错误信息仍然准确
   - 更新错误消息中的模块路径引用

---

## 🔗 相关文件

- `src/commands/log/` - 日志相关命令
- `src/commands/jira/` - Jira 相关命令
- `src/main.rs` - 主命令入口
- `src/commands/mod.rs` - 命令模块声明
- `docs/architecture/commands/LOG_COMMAND_ARCHITECTURE.md` - 日志命令架构文档
- `docs/architecture/commands/JIRA_COMMAND_ARCHITECTURE.md` - Jira 命令架构文档

