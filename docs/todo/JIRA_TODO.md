# JIRA 模块待办事项

## 📋 概述

本文档列出 JIRA 模块的待办功能，包括命令增强、新增命令和集成功能。

---


## ❌ 待实现功能

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
- ❌ PR 关闭时触发

**当前状态**：PR 创建和合并时已支持自动更新 JIRA 状态。

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
   - `jira assign` - 分配 ticket
   - `jira create` - 创建 ticket

### 中优先级
1. **JIRA 搜索和列表**
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
   - `jira assign` - 分配 ticket
   - `jira create` - 创建 ticket

2. **第二阶段**：增强现有功能
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

- [JIRA 命令需求文档](../requirements/JIRA_COMMANDS.md) - 已转换为需求文档
- [Git 工作流待办事项](./GIT_TODO.md)
- [工作流自动化待办事项](./WORKFLOW_TODO.md)
- [JIRA 模块架构文档](../architecture/lib/JIRA_ARCHITECTURE.md)

---

**最后更新**: 2025-12-09
