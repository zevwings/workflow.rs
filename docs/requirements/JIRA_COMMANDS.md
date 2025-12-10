# JIRA 命令需求文档

## 📋 需求概述

本文档描述 JIRA 模块中新增命令的需求，包括评论、列表、状态转换和搜索功能。

**状态**: 📋 需求分析中
**分类**: JIRA 模块
**优先级**: 高优先级（`jira comment`、`jira transition`）、中优先级（`jira list`、`jira search`）
**来源**: 从 `docs/todo/JIRA_TODO.md` 迁移

---

## 🎯 需求目标

扩展 JIRA 模块的命令集，提供更完整的 JIRA ticket 管理功能：
1. 封装已有 API 为 CLI 命令，提升易用性
2. 提供便捷的 ticket 查询和列表功能
3. 支持 JQL 搜索，增强查询能力
4. 支持状态转换，简化工作流操作

---

## 📝 详细需求

### 1. `jira comment` - 添加评论

#### 1.1 功能描述
为 JIRA ticket 添加评论。底层 API `JiraTicket::add_comment()` 已实现，需要封装为 CLI 命令。

#### 1.2 功能要求
- 支持命令行直接输入评论内容
- 支持使用编辑器输入多行评论
- 支持从文件读取评论内容
- 支持 Markdown 格式（如果 JIRA 支持）

#### 1.3 命令示例
```bash
workflow jira comment PROJ-123 "Fixed the bug"      # 添加评论
workflow jira comment PROJ-123 --editor              # 使用编辑器输入评论
workflow jira comment PROJ-123 --file comment.txt    # 从文件读取评论
```

#### 1.4 实现建议
- 在 `src/commands/jira/` 下创建 `comment.rs`
- 支持多行输入、编辑器输入、文件输入
- 支持 Markdown 格式（如果 JIRA 支持）
- 调用现有的 `JiraTicket::add_comment()` API

---

### 2. `jira list` - 列出 tickets

#### 2.1 功能描述
列出项目中的 tickets，支持按状态、指派人等条件过滤。

#### 2.2 功能要求
- 支持按项目过滤
- 支持按状态过滤
- 支持按指派人过滤
- 支持限制返回数量
- 支持多种显示格式（表格、列表、卡片）

#### 2.3 命令示例
```bash
workflow jira list --project PROJ                      # 列出项目所有 tickets
workflow jira list --project PROJ --status "In Progress"  # 按状态过滤
workflow jira list --project PROJ --assignee me        # 按指派人过滤
workflow jira list --project PROJ --limit 20           # 限制数量
```

#### 2.4 实现建议
- 基于 `jira search` 实现，提供更友好的过滤选项
- 支持表格、列表、卡片等多种显示格式
- 使用 JIRA API 的搜索或列表端点

---

### 3. `jira transition` - 状态转换

#### 3.1 功能描述
转换 JIRA ticket 的状态。底层 API `JiraTicket::transition()` 已实现，需要封装为 CLI 命令。

#### 3.2 功能要求
- 支持转换到指定状态
- 支持列出可用状态
- 支持自动转换到下一个状态（工作流感知）

#### 3.3 命令示例
```bash
workflow jira transition PROJ-123 "In Progress"     # 转换到指定状态
workflow jira transition PROJ-123 --list             # 列出可用状态
workflow jira transition PROJ-123 --auto            # 自动转换到下一个状态
```

#### 3.4 实现建议
- 在 `src/commands/jira/` 下创建 `transition.rs`
- 在 `src/lib/cli/mod.rs` 的 `JiraSubcommand` 中添加 `Transition` 子命令
- 调用 `JiraTicket::transition()` 或 `JiraTicket::get_transitions()`

---

### 4. `jira search` - JQL 搜索

#### 4.1 功能描述
使用 JQL（Jira Query Language）搜索 tickets，提供强大的查询能力。

#### 4.2 功能要求
- 支持 JQL 查询语法
- 支持保存常用查询
- 支持交互式查询构建器
- 支持动态补全功能（为其他命令提供 ticket key 补全）

#### 4.3 命令示例
```bash
workflow jira search "project = PROJ AND status = Open"  # JQL 搜索
workflow jira search "assignee = currentUser()"         # 搜索分配给自己的
workflow jira search --saved "my-open-tickets"          # 使用保存的查询
workflow jira search --interactive                       # 交互式构建查询
```

#### 4.4 实现建议
- 使用 JIRA API `/search` GET 端点
- 支持保存常用查询到配置文件
- 支持交互式查询构建器（逐步构建查询条件）
- 实现 `JiraIssueApi::search_issues()` 方法（在 `src/lib/jira/api/issue.rs` 中）

#### 4.5 关联功能
- **动态补全支持**：`jira_ticket_keys()` 方法需要此 API 支持
  - 位置：`src/lib/completion/dynamic.rs`
  - 用途：为 `jira info` 等命令提供 ticket key 的自动补全
  - 依赖：`JiraIssueApi::search_issues()` 方法

---

## 🔧 技术实现

### API 封装
- 对于已有 API（`comment`、`transition`），直接封装为 CLI 命令
- 对于新功能（`list`、`search`），需要实现新的 API 方法

### 命令结构
- 所有命令放在 `src/commands/jira/` 目录下
- 在 `src/lib/cli/mod.rs` 的 `JiraSubcommand` 枚举中添加新的子命令
- 保持与现有命令风格一致

### 输入处理
- 支持多种输入方式：命令行参数、编辑器、文件
- 使用 `dialoguer` 或类似库实现交互式输入
- 支持 Markdown 格式（如果 JIRA API 支持）

### 显示格式
- 使用 `tabled` 或类似库实现表格显示
- 支持多种输出格式：表格、列表、JSON、卡片视图
- 提供可配置的列显示选项

---

## ✅ 验收标准

### `jira comment`
- [ ] 能够通过命令行添加评论
- [ ] 支持使用编辑器输入多行评论
- [ ] 支持从文件读取评论内容
- [ ] 评论成功添加到 JIRA ticket
- [ ] 支持 Markdown 格式（如果 JIRA 支持）

### `jira list`
- [ ] 能够列出项目中的所有 tickets
- [ ] 支持按状态过滤
- [ ] 支持按指派人过滤
- [ ] 支持限制返回数量
- [ ] 支持多种显示格式（表格、列表、卡片）
- [ ] 输出格式清晰易读

### `jira transition`
- [ ] 能够转换 ticket 状态
- [ ] 支持列出可用状态
- [ ] 支持自动转换到下一个状态
- [ ] 状态转换成功应用到 JIRA ticket
- [ ] 提供清晰的错误提示（如状态不可用）

### `jira search`
- [ ] 能够使用 JQL 语法搜索 tickets
- [ ] 支持保存常用查询
- [ ] 支持交互式查询构建器
- [ ] 搜索结果准确
- [ ] 支持动态补全功能（为其他命令提供 ticket key 补全）
- [ ] 实现 `JiraIssueApi::search_issues()` 方法

---

## 📊 优先级说明

### 高优先级
1. **`jira comment`** - API 已实现，封装即可使用
2. **`jira transition`** - API 已实现，封装即可使用

### 中优先级
1. **`jira list`** - 提供便捷的列表功能，基于 search 实现
2. **`jira search`** - 提供强大的查询能力，支持动态补全功能

---

## 🔗 依赖关系

### 实现顺序建议
1. **第一阶段**：`jira comment` 和 `jira transition`（API 已存在，只需封装）
2. **第二阶段**：`jira search`（需要实现 API 方法，支持动态补全）
3. **第三阶段**：`jira list`（基于 search 实现，提供友好界面）

### 技术依赖
- `jira search` 的实现是 `jira list` 的基础
- `jira search` 的 API 实现是动态补全功能的前提
- 所有命令都需要 JIRA API 客户端支持

---

## 📚 相关文档

- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- [JIRA 模块架构文档](../architecture/lib/JIRA_ARCHITECTURE.md)
- [Git 工作流需求文档](./GIT_WORKFLOW.md)

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27
