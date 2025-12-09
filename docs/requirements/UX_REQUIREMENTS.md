# 用户体验优化需求文档

## 📋 概述

本文档详细描述用户体验优化相关的功能需求，包括交互式界面、快捷命令和错误处理与恢复功能。

**文档状态**: 待实现
**创建日期**: 2025-12-09
**优先级**: 高/中/低（见各功能说明）

---

## 🎯 需求列表

### 1. 交互式选择功能

#### 1.1 Fuzzy Finder 选择器

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 使用 fuzzy finder 选择 tickets/PRs
- 支持多选功能
- 提供交互式选择体验

**功能要求**：
1. **Fuzzy Finder 集成**
   - 使用 `skim` 或 `fzf` 集成
   - 支持模糊搜索
   - 支持键盘导航

2. **多选支持**
   - 支持选择多个项目
   - 显示已选择项目数量
   - 支持取消选择

3. **使用场景**
   - 选择要操作的 PR
   - 选择要查看的 JIRA ticket
   - 选择要合并的分支

**命令接口**：
```bash
workflow pr merge --interactive                     # 交互式选择 PR
workflow jira info --interactive                    # 交互式选择 ticket
```

**验收标准**：
- [ ] 能够通过 fuzzy finder 搜索和选择 PRs
- [ ] 能够通过 fuzzy finder 搜索和选择 JIRA tickets
- [ ] 支持多选功能
- [ ] 交互体验流畅，响应迅速

---

### 2. 进度显示功能

#### 2.1 长时间操作进度条

**优先级**: 高
**状态**: ✅ 已实现

**需求描述**：
- 为长时间操作显示进度条
- 显示预计完成时间
- 提供操作状态反馈

**功能要求**：
1. **进度条显示**
   - 使用 `indicatif` 或类似库 ✅
   - 显示操作进度百分比 ✅
   - 显示已处理/总数 ✅
   - 显示预计剩余时间（ETA）✅

2. **适用操作**
   - 下载多个附件 ✅（已在 `jira attachments` 和 `log download` 命令中使用）
   - 批量操作
   - 同步多个 PR
   - 导出大量数据

**实现位置**：
- 核心实现：`src/lib/base/indicator/progress.rs`
- 使用示例：
  - `src/commands/jira/attachments.rs` - JIRA 附件下载
  - `src/commands/log/download.rs` - 日志文件下载

**实现特性**：
- ✅ `Progress::new()` - 已知总数的进度条，显示百分比和 ETA
- ✅ `Progress::new_download()` - 下载专用进度条，显示字节数和下载速度
- ✅ `Progress::new_unknown()` - 未知总数的 spinner 模式
- ✅ 支持进度更新、消息更新、完成处理

**验收标准**：
- [x] 长时间操作显示进度条
- [x] 进度条显示准确的进度百分比
- [x] 显示预计完成时间
- [x] 进度条样式美观，信息清晰

---

### 3. 快捷命令功能

#### 3.1 别名系统

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 支持自定义命令别名
- 简化常用命令输入
- 提高命令输入效率

**功能要求**：
1. **别名配置**
   - 在配置文件中定义别名
   - 支持命令参数传递
   - 支持别名嵌套（别名引用别名）

2. **配置格式**：
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

3. **使用示例**：
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search
```

**实现建议**：
```rust
pub struct AliasManager {
    aliases: HashMap<String, String>,
}

impl AliasManager {
    pub fn expand_alias(&self, command: &str) -> Result<String> {
        // 展开别名
    }
}
```

**验收标准**：
- [ ] 能够在配置文件中定义别名
- [ ] 别名能够正确展开为完整命令
- [ ] 支持命令参数传递
- [ ] 支持别名嵌套

---

#### 3.2 命令历史

**优先级**: 中
**状态**: ❌ 未实现

**需求描述**：
- 记录常用命令历史
- 支持快速重放历史命令
- 提高重复操作效率

**功能要求**：
1. **历史记录**
   - 保存命令历史到文件
   - 记录命令参数
   - 记录执行时间

2. **历史文件位置**：
   - `~/.workflow/history` 或类似位置

3. **命令接口**：
```bash
workflow history                                   # 查看命令历史
workflow history --replay 5                        # 重放第 5 条命令
workflow history --search "pr create"              # 搜索历史命令
```

**验收标准**：
- [ ] 能够记录命令历史
- [ ] 能够查看命令历史列表
- [ ] 能够重放历史命令
- [ ] 支持搜索历史命令

---

#### 3.3 智能补全

**优先级**: 低
**状态**: ❌ 未实现

**需求描述**：
- 增强 shell completion 功能
- 支持动态补全（从 API 获取数据）
- 提供基于上下文的补全建议

**功能要求**：
1. **补全类型**
   - 命令补全
   - 参数补全
   - 值补全（JIRA ticket keys、PR numbers、分支名等）

2. **动态补全**
   - 从 API 获取数据用于补全
   - 支持 JIRA ticket keys 补全
   - 支持 PR numbers 补全
   - 支持分支名补全

3. **Shell 支持**
   - bash completion
   - zsh completion
   - fish completion

4. **使用示例**：
```bash
# 自动补全 JIRA ticket keys
workflow jira info PROJ-<TAB>

# 自动补全 PR numbers
workflow pr merge <TAB>
```

**技术实现**：
- 使用 `clap_complete` 生成补全脚本
- 实现动态补全逻辑

**验收标准**：
- [ ] 支持 bash/zsh/fish 补全
- [ ] 能够动态补全 JIRA ticket keys
- [ ] 能够动态补全 PR numbers
- [ ] 补全响应迅速

---

### 4. 错误处理与恢复

#### 4.1 配置重试策略

**优先级**: 中
**状态**: ❌ 未实现（基础重试已实现）

**需求描述**：
- 支持配置重试策略
- 支持不同错误类型的重试策略
- 提供灵活的重试配置

**功能要求**：
1. **当前状态**：HTTP 客户端已实现基础重试机制 ✅

2. **扩展需求**：
   - 支持配置重试策略
   - 支持不同错误类型的重试策略
   - 支持自定义重试参数

3. **配置示例**：
```toml
[retry]
enabled = true
max_retries = 3
initial_delay = "1s"
max_delay = "10s"
backoff_multiplier = 2.0

[retry.strategies]
network_error = { max_retries = 5, initial_delay = "500ms" }
rate_limit = { max_retries = 3, initial_delay = "5s" }
server_error = { max_retries = 2, initial_delay = "2s" }
```

**验收标准**：
- [ ] 能够在配置文件中设置重试策略
- [ ] 支持不同错误类型的重试策略
- [ ] 重试策略能够正确应用

---

#### 4.2 操作撤销

**优先级**: 低
**状态**: ❌ 未实现

**需求描述**：
- 记录操作历史
- 支持撤销最近的操作
- 提供操作回滚能力

**功能要求**：
1. **操作记录**
   - 记录所有可撤销操作
   - 记录操作参数和结果
   - 使用操作日志持久化

2. **可撤销操作**：
   - JIRA 状态转换
   - JIRA 分配
   - PR 合并（如果 API 支持）
   - 分支删除（如果 API 支持）

3. **命令接口**：
```bash
workflow undo                                      # 撤销最近的操作
workflow undo --list                                # 列出可撤销的操作
workflow undo --count 3                             # 撤销最近 3 个操作
```

4. **实现建议**：
   - 使用操作日志记录所有可撤销操作
   - 支持操作回滚（如果 API 支持）

**验收标准**：
- [ ] 能够记录可撤销操作
- [ ] 能够列出可撤销操作
- [ ] 能够撤销最近的操作
- [ ] 撤销操作能够正确回滚

---

#### 4.3 详细错误信息

**优先级**: 高
**状态**: ❌ 未实现

**需求描述**：
- 提供友好的错误提示
- 提供解决建议
- 提供错误代码和解决方案链接

**功能要求**：
1. **错误信息格式**：
```
Error: Failed to create PR

Reason: Branch 'feature/new-feature' not found

Possible solutions:
  1. Create the branch first: workflow branch create feature/new-feature
  2. Check branch name: workflow branch list
  3. See documentation: https://docs.example.com/pr-create

Error code: PR_CREATE_BRANCH_NOT_FOUND
```

2. **实现建议**：
   - 使用 `anyhow` 的上下文信息
   - 提供错误代码和解决方案链接
   - 为常见错误提供解决建议

3. **实现示例**：
```rust
use anyhow::{Context, Result};

pub fn create_pr_with_error_context(params: CreatePrParams) -> Result<PullRequest> {
    create_pr(&params)
        .context("Failed to create PR")
        .with_context(|| format!("Branch: {}", params.branch))
}
```

**验收标准**：
- [ ] 错误信息友好易懂
- [ ] 提供具体的解决建议
- [ ] 包含错误代码
- [ ] 错误信息格式统一

---

## 📊 优先级总结

### 高优先级
1. ~~**进度显示** - 长时间操作的进度条~~ ✅ 已实现
2. **详细错误信息** - 友好的错误提示和解决建议

### 中优先级
1. **交互式选择** - Fuzzy finder 选择 tickets/PRs
2. **别名系统** - 自定义命令别名
3. **命令历史** - 记录常用命令
4. **配置重试策略** - 配置重试策略

### 低优先级
1. **智能补全** - 增强 shell completion 和动态补全
2. **操作撤销** - 记录操作历史和撤销功能

---

## 🚀 实施计划

### 第一阶段：基础用户体验优化
- [x] 进度显示功能 ✅
- [ ] 详细错误信息功能

### 第二阶段：交互式功能
- [ ] 交互式选择（fuzzy finder）
- [ ] 别名系统
- [ ] 命令历史

### 第三阶段：高级功能
- [ ] 智能补全
- [ ] 操作撤销
- [ ] 配置重试策略扩展

---

## 🛠️ 技术栈

1. **Fuzzy Finder**：使用 `skim` 或集成 `fzf`
2. **进度条**：使用 `indicatif` 显示进度
3. **Shell Completion**：使用 `clap_complete` 生成补全脚本
4. **错误处理**：使用 `anyhow` 提供详细错误信息
5. **操作日志**：记录操作历史以支持撤销

---

## ✅ 验收检查清单

### 交互式选择
- [ ] Fuzzy finder 集成完成
- [ ] 多选功能实现
- [ ] 交互体验流畅

### 进度显示 ✅
- [x] 进度条显示准确
- [x] 预计时间计算正确
- [x] 样式美观

### 别名系统
- [ ] 别名配置功能
- [ ] 别名展开正确
- [ ] 支持参数传递

### 命令历史
- [ ] 历史记录功能
- [ ] 重放功能
- [ ] 搜索功能

### 智能补全
- [ ] Shell 补全脚本生成
- [ ] 动态补全实现
- [ ] 补全响应迅速

### 错误处理
- [ ] 重试策略配置
- [ ] 详细错误信息
- [ ] 操作撤销功能

---

## 📚 相关文档

- [UX TODO 文档](../todo/UX_TODO.md)
- [JIRA 模块待办事项](../todo/JIRA_TODO.md)
- [Git 工作流待办事项](../todo/GIT_TODO.md)
- [工作流自动化待办事项](../todo/WORKFLOW_TODO.md)

---

**最后更新**: 2025-12-09
