# Tag 管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Tag 管理命令模块架构，包括：
- Tag 删除功能（支持本地和远程 tag 删除，支持模式匹配和交互式选择）

Tag 管理命令提供安全的 tag 删除功能，可以删除本地和/或远程 tag，支持模式匹配批量删除，并提供完整的预览和确认机制。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/git/` 模块提供。

---

## 📁 相关文件

### CLI 入口层

Tag 管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Tag` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow tag` 子命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/tag/
├── mod.rs          # Tag 命令模块声明
└── delete.rs       # Tag 删除命令（~248 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（确认、预览等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/git/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/git/`**：Git 操作（`GitTag`）
  - `GitTag::list_all_tags()` - 获取所有 tag（本地和远程）
  - `GitTag::get_tag_info()` - 获取 tag 信息
  - `GitTag::delete_local()` - 删除本地 tag
  - `GitTag::delete_remote()` - 删除远程 tag
- **`lib/base/dialog/`**：对话框（`ConfirmDialog`、`MultiSelectDialog`）

详细架构文档：参见 [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/tag/*.rs (命令封装层，处理交互)
  ↓
lib/git/tag.rs (通过 Git API 调用，具体实现见相关模块文档)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  TagSubcommand::Delete { tag_name, local, remote, pattern, dry_run, force } => TagDeleteCommand::execute()
}
```

---

## 1. Tag 删除命令 (`delete.rs`)

### 相关文件

```
src/commands/tag/delete.rs (~248 行)
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::TagSubcommand::Delete { tag_name, local, remote, pattern, dry_run, force }
  ↓
commands/tag/delete.rs::TagDeleteCommand::execute(tag_name, local, remote, pattern, dry_run, force)
  ↓
  1. 获取所有 tag（GitTag::list_all_tags()）
  2. 确定要删除的 tag 列表：
     - 如果提供了 pattern：使用模式匹配过滤
     - 如果提供了 tag_name：使用指定 tag
     - 否则：交互式选择（MultiSelectDialog）
  3. 获取 tag 信息（GitTag::get_tag_info()）
  4. 显示预览（tag 名称、commit hash、存在位置）
  5. Dry-run 模式（如果启用，只预览不执行）
  6. 确认删除（除非使用 force）
  7. 执行删除：
     - 删除本地 tag（如果存在且未指定 --remote-only）
     - 删除远程 tag（如果存在且未指定 --local-only）
  8. 显示结果
```

### 功能说明

Tag 删除命令提供安全的 tag 删除功能：

1. **多种选择方式**：
   - 直接指定 tag 名称
   - 使用模式匹配（`--pattern`，支持 shell 通配符：`*`、`?`）
   - 交互式选择（不提供参数时）

2. **删除范围控制**：
   - `--local`：只删除本地 tag
   - `--remote`：只删除远程 tag
   - 默认：删除本地和远程 tag（如果存在）

3. **安全机制**：
   - 预览模式：显示将要删除的 tag 列表（tag 名称、commit hash、存在位置）
   - Dry-run 模式：只预览，不实际删除
   - 确认机制：删除前需要用户确认（除非使用 `--force`）

4. **交互式选择**：
   - 使用 `MultiSelectDialog` 支持多选
   - 显示 tag 信息（名称、commit hash、存在位置）
   - 格式：`<tag_name> (commit: <hash>, local/remote/both)`

5. **模式匹配**：
   - 支持 shell 通配符：`*`（匹配任意字符）、`?`（匹配单个字符）、`.`（转义为字面量）
   - 自动转换为正则表达式进行匹配
   - 示例：`--pattern "v1.*"` 匹配所有以 `v1.` 开头的 tag

### 关键步骤说明

1. **Tag 列表获取**：
   - 使用 `GitTag::list_all_tags()` 获取所有 tag（本地和远程）
   - 返回 `TagInfo` 列表，包含 tag 名称、commit hash、存在位置

2. **Tag 选择**：
   - 模式匹配：使用 `filter_tags_by_pattern()` 将 shell 通配符转换为正则表达式
   - 交互式选择：使用 `MultiSelectDialog` 显示所有 tag，支持多选

3. **Tag 信息获取**：
   - 使用 `GitTag::get_tag_info()` 获取每个 tag 的详细信息
   - 包括 tag 名称、commit hash、本地/远程存在状态

4. **删除执行**：
   - 根据 `--local` 和 `--remote` 参数确定删除范围
   - 使用 `GitTag::delete_local()` 删除本地 tag
   - 使用 `GitTag::delete_remote()` 删除远程 tag
   - 记录删除结果（成功/失败）

### 数据流

```
用户输入 (workflow tag delete [TAG_NAME] [--local] [--remote] [--pattern PATTERN] [--dry-run] [--force])
  ↓
获取所有 tag（GitTag::list_all_tags()）
  ↓
确定要删除的 tag 列表（模式匹配/指定名称/交互式选择）
  ↓
获取 tag 信息（GitTag::get_tag_info()）
  ↓
显示预览
  ↓
Dry-run 模式（如果启用）
  ↓
确认删除（除非使用 force）
  ↓
执行删除（GitTag::delete_local() / GitTag::delete_remote()）
  ↓
显示结果
```

### 依赖模块

- **`lib/git/tag.rs`**：Git Tag 操作（`GitTag`）
  - `GitTag::list_all_tags()` - 获取所有 tag（本地和远程）
  - `GitTag::get_tag_info()` - 获取 tag 信息
  - `GitTag::delete_local()` - 删除本地 tag
  - `GitTag::delete_remote()` - 删除远程 tag
- **`lib/base/dialog/`**：对话框（`ConfirmDialog`、`MultiSelectDialog`）

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `TagDeleteCommand::execute()` - 删除 tag

#### 2. 安全机制

- **预览模式**：显示将要删除的 tag 列表
- **Dry-run 模式**：只预览，不实际删除
- **确认机制**：删除前需要用户确认
- **强制模式**：使用 `--force` 跳过确认

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - Tag 不存在：记录警告，继续处理其他 tag
   - 删除失败：记录警告，继续处理其他 tag

3. **库层**：Git 操作错误、文件操作错误
   - 通过 `GitTag` API 返回的错误信息
   - Git 操作失败、文件读写错误等

### 容错机制

- **Tag 不存在**：记录警告，继续处理其他 tag
- **删除失败**：记录警告，继续处理其他 tag
- **模式匹配失败**：返回错误，提示用户检查模式格式

---

## 📝 扩展性

### 添加新的 Tag 操作

1. 在 `tag/` 目录下创建新的命令文件
2. 在 `src/lib/cli/tag.rs` 中添加新的子命令枚举
3. 在 `src/main.rs` 中添加命令分发逻辑
4. 在 `lib/git/tag.rs` 中添加对应的 Git 操作方法（如需要）

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git Tag 操作相关
- [Tag Lib 层架构文档](../lib/TAG_ARCHITECTURE.md) - Tag 模块详细说明

---

## 📋 使用示例

### Delete 命令

```bash
# 删除指定 tag（本地和远程）
workflow tag delete v1.0.0

# 只删除本地 tag
workflow tag delete v1.0.0 --local

# 只删除远程 tag
workflow tag delete v1.0.0 --remote

# 使用模式匹配删除多个 tag
workflow tag delete --pattern "v1.*"

# 预览模式（不实际删除）
workflow tag delete v1.0.0 --dry-run

# 强制删除（跳过确认）
workflow tag delete v1.0.0 --force

# 交互式选择（不提供参数）
workflow tag delete
```

---

## ✅ 总结

Tag 管理命令层采用清晰的分层架构设计：

1. **安全删除**：支持本地和远程 tag 删除，提供完整的预览和确认机制
2. **灵活选择**：支持直接指定、模式匹配和交互式选择
3. **安全机制**：预览、确认、强制模式，确保操作安全
4. **用户友好**：清晰的预览和确认提示，详细的错误信息

**设计优势**：
- ✅ **安全性**：多重确认机制，防止误删重要 tag
- ✅ **灵活性**：支持多种选择方式和删除范围控制
- ✅ **用户友好**：清晰的预览和确认提示，详细的错误信息
- ✅ **可扩展性**：模块化设计便于添加新功能
