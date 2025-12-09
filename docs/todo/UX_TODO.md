# 用户体验优化待办事项

## 📋 概述

本文档列出用户体验优化相关的待办功能，包括交互式界面、快捷命令和错误处理与恢复。

---

## ❌ 待实现功能

### 1. 交互式界面

#### 1.1 交互式选择
- ❌ Fuzzy finder 选择 tickets/PRs
- ❌ 多选支持

**功能**：使用 fuzzy finder 选择 tickets/PRs。

**实现建议**：
- 使用 `skim` 或 `fzf` 集成
- 支持多选

**使用场景**：
- 选择要操作的 PR
- 选择要查看的 JIRA ticket
- 选择要合并的分支

**命令示例**：
```bash
workflow pr merge --interactive                     # 交互式选择 PR
workflow jira info --interactive                    # 交互式选择 ticket
```

#### 1.3 进度显示
- ❌ 长时间操作的进度条
- ❌ 预计时间显示

**功能**：长时间操作的进度条。

**实现建议**：
- 使用 `indicatif` 或类似库
- 显示操作进度和预计时间

**适用操作**：
- 下载多个附件
- 批量操作
- 同步多个 PR
- 导出大量数据

---

### 2. 快捷命令

#### 2.1 别名系统
- ❌ 自定义命令别名

**功能**：自定义命令别名。

**配置示例**：
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

**命令示例**：
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search
```

**实现建议**：
- 在配置文件中定义别名
- 支持命令参数传递
- 支持别名嵌套（别名引用别名）

#### 2.2 命令历史
- ❌ 记录常用命令
- ❌ 快速重放

**功能**：记录常用命令。

**实现建议**：
- 保存命令历史到文件
- 支持快速重放

**命令示例**：
```bash
workflow history                                   # 查看命令历史
workflow history --replay 5                        # 重放第 5 条命令
workflow history --search "pr create"              # 搜索历史命令
```

**历史文件位置**：
- `~/.workflow/history` 或类似位置

#### 2.3 智能补全
- ❌ 增强 shell completion
- ❌ 动态补全（从 API 获取数据）

**功能**：基于上下文的补全建议。

**实现建议**：
- 增强 shell completion（bash、zsh、fish）
- 支持动态补全（从 API 获取数据）

**补全类型**：
- 命令补全
- 参数补全
- 值补全（JIRA ticket keys、PR numbers、分支名等）

**命令示例**：
```bash
# 自动补全 JIRA ticket keys
workflow jira info PROJ-<TAB>

# 自动补全 PR numbers
workflow pr merge <TAB>
```

---

### 3. 错误处理与恢复

#### 3.1 配置重试策略
- ❌ 配置重试策略
- ❌ 不同错误类型的重试策略

**功能**：网络请求失败自动重试。

**当前状态**：HTTP 客户端已实现重试机制。✅ 已实现

**拓展**：
- 支持配置重试策略
- 支持不同错误类型的重试策略

**配置示例**：
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

#### 3.2 操作撤销
- ❌ 记录操作历史
- ❌ 撤销最近的操作

**功能**：支持撤销某些操作。

**实现建议**：
- 记录操作历史
- 支持撤销最近的操作

**可撤销操作**：
- JIRA 状态转换
- JIRA 分配
- PR 合并（如果支持）
- 分支删除（如果支持）

**命令示例**：
```bash
workflow undo                                      # 撤销最近的操作
workflow undo --list                                # 列出可撤销的操作
workflow undo --count 3                             # 撤销最近 3 个操作
```

**实现建议**：
- 使用操作日志记录所有可撤销操作
- 支持操作回滚（如果 API 支持）

#### 3.3 详细错误信息
- ❌ 友好的错误提示
- ❌ 解决建议
- ❌ 错误代码和解决方案链接

**功能**：提供更友好的错误提示和解决建议。

**实现建议**：
- 使用 `anyhow` 的上下文信息
- 提供错误代码和解决方案链接

**错误信息格式**：
```
Error: Failed to create PR

Reason: Branch 'feature/new-feature' not found

Possible solutions:
  1. Create the branch first: workflow branch create feature/new-feature
  2. Check branch name: workflow branch list
  3. See documentation: https://docs.example.com/pr-create

Error code: PR_CREATE_BRANCH_NOT_FOUND
```

---

## 📊 优先级

### 高优先级
1. **进度显示**
   - 长时间操作的进度条

2. **错误处理增强**
   - 详细错误信息
   - 解决建议

### 中优先级
1. **交互式选择**
   - Fuzzy finder 选择 tickets/PRs

2. **别名系统**
   - 自定义命令别名

3. **命令历史**
   - 记录常用命令

### 低优先级
1. **智能补全**
   - 增强 shell completion
   - 动态补全

3. **操作撤销**
   - 记录操作历史
   - 撤销最近的操作

---

## 📝 实现建议

### 开发顺序
1. **第一阶段**：基础用户体验优化
   - 进度显示
   - 详细错误信息

2. **第二阶段**：交互式功能
   - 交互式选择（fuzzy finder）
   - 别名系统
   - 命令历史

3. **第三阶段**：高级功能
   - 智能补全
   - 操作撤销

### 技术考虑
1. **Fuzzy Finder**：使用 `skim` 或集成 `fzf`
3. **进度条**：使用 `indicatif` 显示进度
4. **Shell Completion**：使用 `clap_complete` 生成补全脚本
5. **错误处理**：使用 `anyhow` 提供详细错误信息
6. **操作日志**：记录操作历史以支持撤销
7. **测试**：为新功能添加单元测试和集成测试
8. **文档**：及时更新文档和示例

### 实现细节

#### 进度显示实现
```rust
use indicatif::{ProgressBar, ProgressStyle};

pub fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}
```

#### 别名系统实现
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

#### 错误信息实现
```rust
use anyhow::{Context, Result};

pub fn create_pr_with_error_context(params: CreatePrParams) -> Result<PullRequest> {
    create_pr(&params)
        .context("Failed to create PR")
        .with_context(|| format!("Branch: {}", params.branch))
}
```

---

## 📚 相关文档

- [JIRA 模块待办事项](./JIRA_TODO.md)
- [Git 工作流待办事项](./GIT_TODO.md)
- [工作流自动化待办事项](./WORKFLOW_TODO.md)

---

**最后更新**: 2025-12-09
