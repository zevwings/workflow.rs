# JIRA 模块待办事项

## 📋 概述

本文档列出 JIRA 模块的待办功能，包括命令增强、新增命令和集成功能。

---

## ✅ 已完成功能

- ✅ `jira info` - 显示 ticket 基本信息
- ✅ `jira attachments` - 下载附件
- ✅ `jira clean` - 清理本地数据
- ✅ JIRA API：`transition`、`assign`、`add_comment`（已实现，待封装为命令）

---

## ❌ 待实现功能

### 1. `jira info` 增强功能

#### 1.1 显示更多字段
- ❌ 优先级（Priority）
- ❌ 创建/更新时间（Created/Updated）
- ❌ 报告人/指派人（Reporter/Assignee）
- ❌ 标签（Labels）
- ❌ 组件（Components）
- ❌ 修复版本（Fix Versions）
- ❌ 关联的 Issues（Linked Issues）
- ❌ 子任务列表（Subtasks）
- ❌ 时间跟踪（Time Tracking）

**实现建议**：
```rust
// 在 src/lib/jira/types.rs 中扩展 JiraIssueFields
pub struct JiraIssueFields {
    // ... 现有字段
    pub priority: Option<JiraPriority>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub reporter: Option<JiraUser>,
    pub assignee: Option<JiraUser>,
    pub labels: Option<Vec<String>>,
    pub components: Option<Vec<JiraComponent>>,
    pub fix_versions: Option<Vec<JiraVersion>>,
    pub issuelinks: Option<Vec<JiraIssueLink>>,
    pub subtasks: Option<Vec<JiraSubtask>>,
    pub time_tracking: Option<JiraTimeTracking>,
}
```

#### 1.2 评论详情展示
- ❌ 显示评论列表（作者、时间、内容）
- ❌ 支持分页显示（`--limit`、`--offset`）
- ❌ 支持按时间排序（`--sort`）
- ❌ 支持过滤（`--author`、`--since`）

**命令示例**：
```bash
workflow jira info PROJ-123 --comments          # 显示所有评论
workflow jira info PROJ-123 --comments --limit 10  # 只显示最近 10 条
workflow jira info PROJ-123 --comments --author user@example.com  # 过滤作者
```

#### 1.3 变更历史（Changelog）
- ❌ 显示 ticket 的状态变更历史
- ❌ 显示字段变更记录

**命令示例**：
```bash
workflow jira info PROJ-123 --changelog        # 显示变更历史
workflow jira info PROJ-123 --changelog --field status  # 只显示状态变更
```

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}/changelog` 端点
- 解析 changelog 数据，格式化显示

#### 1.4 自定义字段支持
- ❌ 支持显示和查询自定义字段

**命令示例**：
```bash
workflow jira info PROJ-123 --custom-fields    # 显示所有自定义字段
workflow jira info PROJ-123 --field customfield_10001  # 显示特定自定义字段
```

#### 1.5 输出格式支持
- ❌ JSON 格式输出
- ❌ YAML 格式输出
- ❌ Markdown 格式输出

**命令示例**：
```bash
workflow jira info PROJ-123                    # 默认表格格式
workflow jira info PROJ-123 --json             # JSON 格式
workflow jira info PROJ-123 --yaml             # YAML 格式
workflow jira info PROJ-123 --markdown         # Markdown 格式
```

#### 1.6 关联信息展示
- ❌ 显示关联的 PR
- ❌ 显示关联的分支

**命令示例**：
```bash
workflow jira info PROJ-123 --related         # 显示关联的 PR、分支
```

---

### 2. 新增 JIRA 命令

#### 2.1 `jira assign` - 分配 ticket
- ❌ 封装为 CLI 命令（API 已实现）

**当前状态**：`JiraTicket::assign()` 已实现，但未封装为 CLI 命令。

**命令示例**：
```bash
workflow jira assign PROJ-123                        # 分配给自己
workflow jira assign PROJ-123 user@example.com       # 分配给指定用户
workflow jira assign PROJ-123 --unassign             # 取消分配
```

**实现建议**：
- 在 `src/commands/jira/` 下创建 `assign.rs`
- 支持用户名、邮箱、account_id 等多种输入方式
- 支持交互式选择用户（从项目成员列表）

#### 2.2 `jira comment` - 添加评论
- ❌ 封装为 CLI 命令（API 已实现）

**当前状态**：`JiraTicket::add_comment()` 已实现，但未封装为 CLI 命令。

**命令示例**：
```bash
workflow jira comment PROJ-123 "Fixed the bug"      # 添加评论
workflow jira comment PROJ-123 --editor              # 使用编辑器输入评论
workflow jira comment PROJ-123 --file comment.txt    # 从文件读取评论
```

**实现建议**：
- 在 `src/commands/jira/` 下创建 `comment.rs`
- 支持多行输入、编辑器输入、文件输入
- 支持 Markdown 格式（如果 JIRA 支持）

#### 2.3 `jira create` - 创建 ticket
- ❌ 创建新的 JIRA ticket

**命令示例**：
```bash
workflow jira create --project PROJ --type Bug --summary "Bug description"  # 创建 Bug
workflow jira create --project PROJ --type Task --summary "Task" --description "Details"  # 创建 Task
workflow jira create --interactive                    # 交互式创建
```

**实现建议**：
- 使用 JIRA API `/issue` POST 端点
- 支持必填字段验证
- 支持模板（从现有 ticket 复制字段）

#### 2.4 `jira list` - 列出 tickets
- ❌ 列出项目中的 tickets（按状态、指派人等过滤）

**命令示例**：
```bash
workflow jira list --project PROJ                      # 列出项目所有 tickets
workflow jira list --project PROJ --status "In Progress"  # 按状态过滤
workflow jira list --project PROJ --assignee me        # 按指派人过滤
workflow jira list --project PROJ --limit 20           # 限制数量
```

**实现建议**：
- 基于 `jira search` 实现，提供更友好的过滤选项
- 支持表格、列表、卡片等多种显示格式

#### 2.5 `jira watch` - 关注/取消关注
- ❌ 关注或取消关注 ticket

**命令示例**：
```bash
workflow jira watch PROJ-123                          # 关注 ticket
workflow jira watch PROJ-123 --unwatch                # 取消关注
workflow jira watch --list                             # 列出关注的 tickets
```

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}/watchers` 端点

#### 2.6 `jira transition` - 状态转换
- ❌ 封装为 CLI 命令（API 已实现）

**当前状态**：`JiraTicket::transition()` 已实现，但未封装为 CLI 命令。

**命令示例**：
```bash
workflow jira transition PROJ-123 "In Progress"     # 转换到指定状态
workflow jira transition PROJ-123 --list             # 列出可用状态
workflow jira transition PROJ-123 --auto             # 自动转换到下一个状态
```

**实现建议**：
- 在 `src/commands/jira/` 下创建 `transition.rs`
- 在 `src/lib/cli/mod.rs` 的 `JiraSubcommand` 中添加 `Transition` 子命令
- 调用 `JiraTicket::transition()` 或 `JiraTicket::get_transitions()`

#### 2.7 `jira update` - 更新 ticket
- ❌ 更新 ticket 的字段（summary、description、priority 等）

**命令示例**：
```bash
workflow jira update PROJ-123 --summary "New summary"  # 更新摘要
workflow jira update PROJ-123 --description "New description"  # 更新描述
workflow jira update PROJ-123 --priority High         # 更新优先级
workflow jira update PROJ-123 --labels "bug,urgent"    # 更新标签
```

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}` PUT 端点
- 支持批量更新多个字段

#### 2.8 `jira search` - JQL 搜索
- ❌ 使用 JQL（Jira Query Language）搜索 tickets

**命令示例**：
```bash
workflow jira search "project = PROJ AND status = Open"  # JQL 搜索
workflow jira search "assignee = currentUser()"         # 搜索分配给自己的
workflow jira search --saved "my-open-tickets"          # 使用保存的查询
workflow jira search --interactive                       # 交互式构建查询
```

**实现建议**：
- 使用 JIRA API `/search` GET 端点
- 支持保存常用查询
- 支持交互式查询构建器

#### 2.9 `jira link` - 关联 tickets
- ❌ 关联或取消关联 tickets

**命令示例**：
```bash
workflow jira link PROJ-123 PROJ-124 --type "Blocks"  # 关联 tickets
workflow jira link PROJ-123 PROJ-124 --unlink          # 取消关联
workflow jira link PROJ-123 --list                     # 列出关联的 tickets
```

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}/links` 端点
- 支持多种关联类型（blocks、relates to、duplicates 等）

#### 2.10 `jira worklog` - 工作时间记录
- ❌ 记录或查看工作时间

**命令示例**：
```bash
workflow jira worklog PROJ-123 add 2h "Fixed bug"     # 记录 2 小时
workflow jira worklog PROJ-123 list                    # 查看工作记录
workflow jira worklog PROJ-123 --today                 # 查看今天的工作记录
```

**实现建议**：
- 使用 JIRA API `/issue/{issueIdOrKey}/worklog` 端点
- 支持时间格式解析（2h、30m、1d 等）

#### 2.11 `jira sprint` - Sprint 相关操作
- ❌ Sprint 相关操作（查看、移动 ticket 等）

**命令示例**：
```bash
workflow jira sprint list --board 1                   # 列出 Sprint
workflow jira sprint move PROJ-123 --sprint "Sprint 2"  # 移动 ticket 到 Sprint
workflow jira sprint info --sprint "Sprint 2"         # 查看 Sprint 信息
```

**实现建议**：
- 需要 JIRA Agile/Scrum 插件支持
- 使用 JIRA API `/sprint` 相关端点

---

### 3. JIRA 集成增强

#### 3.1 批量操作
- ❌ 批量更新状态
- ❌ 批量分配

**命令示例**：
```bash
workflow jira batch transition "PROJ-123,PROJ-124,PROJ-125" "Done"  # 批量转换状态
workflow jira batch assign "PROJ-123,PROJ-124" user@example.com      # 批量分配
```

**实现建议**：
- 支持从文件读取 ticket 列表
- 支持并行处理以提高效率
- 提供进度显示和错误处理

#### 3.2 自定义工作流规则
- ❌ 配置文件支持自定义工作流规则

**实现建议**：
- 在配置文件中定义工作流规则
- 支持多种触发条件（PR 创建、合并、关闭等）
- 支持自定义状态转换规则

#### 3.3 多种触发条件
- ❌ PR 创建时触发
- ❌ PR 合并时触发
- ❌ PR 关闭时触发

**当前状态**：PR 创建和合并时已支持自动更新 JIRA 状态。✅ 已实现

**拓展**：
- 支持更多触发条件
- 支持自定义触发规则

#### 3.4 自定义评论模板
- ❌ 支持自定义评论模板

**实现建议**：
- 在配置文件中定义评论模板
- 支持模板变量（如 `{{pr_url}}`、`{{branch_name}}` 等）

---

## 📊 优先级

### 高优先级
1. **JIRA 命令封装**（已有 API，封装即可）
   - `jira transition` - 状态转换（API 已实现，待封装为命令）
   - `jira assign` - 分配 ticket
   - `jira comment` - 添加评论
   - `jira create` - 创建 ticket

2. **JIRA info 增强**
   - 显示更多字段（优先级、创建时间、指派人等）
   - 评论详情展示

### 中优先级
1. **JIRA 搜索和列表**
   - `jira search` - JQL 搜索
   - `jira list` - 列出 tickets
   - `jira watch` - 关注/取消关注

2. **JIRA 更新和关联**
   - `jira update` - 更新 ticket 字段
   - `jira link` - 关联 tickets
   - `jira worklog` - 工作时间记录

3. **JIRA 集成增强**
   - 批量操作
   - 自定义工作流规则
   - 自定义评论模板

### 低优先级
1. **JIRA Sprint 支持**
   - `jira sprint` - Sprint 相关操作（需要 Agile/Scrum 插件）

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：封装已有 API 为命令
   - `jira transition` - 状态转换
   - `jira assign` - 分配 ticket
   - `jira comment` - 添加评论
   - `jira create` - 创建 ticket

2. **第二阶段**：增强现有功能
   - `jira info` 显示更多字段
   - `jira info` 评论详情展示
   - `jira search` - JQL 搜索
   - `jira update` - 更新 ticket

3. **第三阶段**：集成增强和高级功能
   - 批量操作
   - 自定义工作流规则
   - `jira link` - 关联 tickets
   - `jira worklog` - 工作时间记录
   - `jira sprint` - Sprint 相关操作

### 技术考虑
1. **API 设计**：保持与现有命令风格一致
2. **错误处理**：使用 `anyhow` 提供详细错误信息
3. **测试**：为新功能添加单元测试和集成测试
4. **文档**：及时更新文档和示例
5. **向后兼容**：确保新功能不影响现有功能

---

## 📚 相关文档

- [Git 工作流待办事项](./GIT.md)
- [工作流自动化待办事项](./WORKFLOW.md)
- [JIRA 模块架构文档](../architecture/lib/JIRA_ARCHITECTURE.md)

---

**最后更新**: 2024-12-19
