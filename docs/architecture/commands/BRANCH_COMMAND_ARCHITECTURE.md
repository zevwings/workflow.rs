# 分支管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的分支管理命令模块架构，包括：
- 本地分支清理功能
- 分支忽略列表管理功能

分支管理命令提供智能的分支清理功能，可以安全地删除已合并的分支，同时保留重要的分支（如 main/master、develop、当前分支和用户配置的忽略分支）。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/git/` 模块提供。

---

## 📁 相关文件

### CLI 入口层

分支管理命令现在作为 `workflow` 主命令的子命令，通过 `src/main.rs` 中的 `Commands::Branch` 枚举定义。

```
src/main.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow branch` 子命令分发到对应的命令处理函数

### 命令封装层

```
src/commands/branch/
├── mod.rs          # 分支命令模块声明（8 行）
├── clean.rs        # 分支清理命令（~195 行）
├── ignore.rs       # 分支忽略列表管理命令（~94 行）
└── helpers.rs      # 辅助函数（BranchConfig 管理，~98 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（确认、预览等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/git/`) 的功能
- 管理分支配置文件（`branch.toml`）

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/git/`**：Git 操作（`GitBranch`、`GitRepo`）
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::get_default_branch()` - 获取默认分支
  - `GitBranch::get_all_branches()` - 获取所有本地分支
  - `GitBranch::delete()` - 删除分支
  - `GitRepo::extract_repo_name()` - 提取仓库名
  - `GitRepo::prune_remote()` - 清理远端引用
- **`commands/check/`**：环境检查（`CheckCommand::run_all()`）
- **`lib/base/util/`**：工具函数（`confirm()`）
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）

详细架构文档：参见 [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/main.rs (workflow 主命令，参数解析)
  ↓
commands/branch/*.rs (命令封装层，处理交互)
  ↓
lib/git/* (通过 Git API 调用，具体实现见相关模块文档)
  ↓
~/.workflow/config/branch.toml (配置文件)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  BranchSubcommand::Clean { dry_run } => BranchCleanCommand::clean()
  BranchSubcommand::Ignore { subcommand } => match subcommand {
    IgnoreSubcommand::Add { branch_name } => BranchIgnoreCommand::add()
    IgnoreSubcommand::Remove { branch_name } => BranchIgnoreCommand::remove()
    IgnoreSubcommand::List => BranchIgnoreCommand::list()
  }
}
```

---

## 1. 分支清理命令 (`clean.rs`)

### 相关文件

```
src/commands/branch/clean.rs (~195 行)
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::BranchSubcommand::Clean { dry_run }
  ↓
commands/branch/clean.rs::BranchCleanCommand::clean(dry_run)
  ↓
  1. 运行检查（check::CheckCommand::run_all()）
  2. 获取当前分支、默认分支、仓库名
  3. 清理远端引用（GitRepo::prune_remote()）
  4. 读取配置文件（BranchConfig::load()）
  5. 构建排除分支列表（当前分支、默认分支、develop、忽略列表）
  6. 获取所有本地分支（GitBranch::get_all_branches()）
  7. 过滤出需要删除的分支
  8. 分类分支（已合并 vs 未合并）
  9. 显示预览
  10. Dry-run 模式（如果启用，直接返回）
  11. 确认删除
  12. 删除已合并分支（GitBranch::delete()）
  13. 处理未合并分支（需要用户确认强制删除）
  14. 显示结果
```

### 功能说明

分支清理命令提供智能的分支清理功能：

1. **前置检查**：运行所有环境检查（git status、network 等）
2. **分支分类**：
   - 自动排除：当前分支、默认分支（main/master）、develop 分支
   - 配置文件排除：从 `branch.toml` 读取忽略列表
   - 合并状态分类：区分已合并和未合并的分支
3. **安全机制**：
   - 预览模式：显示将要删除的分支列表
   - Dry-run 模式：只预览，不实际删除
   - 确认机制：删除前需要用户确认
   - 未合并分支：需要额外确认才能强制删除
4. **清理远端引用**：自动清理已删除的远端分支引用

### 关键步骤说明

1. **排除分支列表构建**：
   - 当前分支（始终保留）
   - 默认分支（main 或 master）
   - develop 分支（始终保留）
   - 配置文件中的忽略分支（按仓库分组）

2. **分支合并状态检查**：
   - 使用 `git branch --merged <base_branch>` 检查分支是否已合并
   - 已合并分支：安全删除
   - 未合并分支：需要用户确认强制删除

3. **删除策略**：
   - 已合并分支：直接删除（`GitBranch::delete(branch, false)`）
   - 未合并分支：需要用户确认后强制删除（`GitBranch::delete(branch, true)`）

4. **配置文件管理**：
   - 配置文件路径：`~/.workflow/config/branch.toml`
   - 按仓库名分组存储忽略分支列表
   - 使用 `ConfigManager` 进行配置读写

### 数据流

```
用户输入 (workflow branch clean [--dry-run])
  ↓
环境检查 (CheckCommand::run_all())
  ↓
获取分支信息 (GitBranch, GitRepo)
  ↓
读取配置文件 (BranchConfig::load())
  ↓
过滤和分类分支
  ↓
预览显示
  ↓
用户确认
  ↓
删除分支 (GitBranch::delete())
  ↓
显示结果
```

---

## 2. 分支忽略列表管理命令 (`ignore.rs`)

### 相关文件

```
src/commands/branch/ignore.rs (~94 行)
src/main.rs (命令入口)
```

### 调用流程

#### Add 命令

```
src/main.rs::IgnoreSubcommand::Add { branch_name }
  ↓
commands/branch/ignore.rs::BranchIgnoreCommand::add(branch_name)
  ↓
  1. 获取仓库名（GitRepo::extract_repo_name()）
  2. 读取配置文件（BranchConfig::load()）
  3. 添加分支到忽略列表（add_ignore_branch()）
  4. 保存配置文件（save()）
  5. 显示结果
```

#### Remove 命令

```
src/main.rs::IgnoreSubcommand::Remove { branch_name }
  ↓
commands/branch/ignore.rs::BranchIgnoreCommand::remove(branch_name)
  ↓
  1. 获取仓库名（GitRepo::extract_repo_name()）
  2. 读取配置文件（BranchConfig::load()）
  3. 从忽略列表移除分支（remove_ignore_branch()）
  4. 保存配置文件（save()）
  5. 显示结果
```

#### List 命令

```
src/main.rs::IgnoreSubcommand::List
  ↓
commands/branch/ignore.rs::BranchIgnoreCommand::list()
  ↓
  1. 获取仓库名（GitRepo::extract_repo_name()）
  2. 读取配置文件（BranchConfig::load()）
  3. 获取忽略分支列表（get_ignore_branches()）
  4. 格式化显示
```

### 功能说明

分支忽略列表管理命令提供分支忽略列表的完整管理功能：

1. **添加分支到忽略列表**：
   - 自动检测当前仓库名
   - 检查分支是否已在列表中
   - 按仓库分组存储

2. **从忽略列表移除分支**：
   - 自动检测当前仓库名
   - 如果列表为空，自动清理仓库配置

3. **列出忽略分支**：
   - 显示当前仓库的所有忽略分支
   - 格式化输出，显示总数

### 关键步骤说明

1. **配置文件结构**：
   ```toml
   [repositories."owner/repo"]
   ignore = ["branch1", "branch2"]
   ```

2. **仓库名提取**：
   - 使用 `GitRepo::extract_repo_name()` 提取仓库名
   - 格式：`owner/repo`（如 `github.com/owner/repo` → `owner/repo`）

3. **配置管理**：
   - 使用 `ConfigManager<BranchConfig>` 进行配置读写
   - 自动创建配置文件（如果不存在）
   - 按仓库分组管理，支持多仓库配置

---

## 3. 辅助函数 (`helpers.rs`)

### 相关文件

```
src/commands/branch/helpers.rs (~98 行)
```

### 功能说明

辅助函数模块提供分支配置文件的完整管理功能：

1. **配置结构体**：
   - `BranchConfig` - 分支配置根结构
   - `RepositoryIgnore` - 仓库忽略配置

2. **配置管理函数**：
   - `BranchConfig::load()` - 读取配置文件
   - `save()` - 保存配置文件
   - `get_ignore_branches()` - 获取指定仓库的忽略分支列表
   - `add_ignore_branch()` - 添加分支到忽略列表
   - `remove_ignore_branch()` - 从忽略列表移除分支

### 配置文件结构

```toml
[repositories."owner/repo1"]
ignore = ["feature-branch", "hotfix-branch"]

[repositories."owner/repo2"]
ignore = ["staging", "production"]
```

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `BranchCleanCommand::clean()` - 清理分支
- `BranchIgnoreCommand::add()` - 添加忽略分支
- `BranchIgnoreCommand::remove()` - 移除忽略分支
- `BranchIgnoreCommand::list()` - 列出忽略分支

#### 2. 配置管理模式

使用 `ConfigManager` 统一管理配置文件：
- 配置文件路径：`~/.workflow/config/branch.toml`
- 按仓库分组存储，支持多仓库配置
- 自动创建配置文件（如果不存在）

#### 3. 安全机制

- **预览模式**：显示将要删除的分支列表
- **Dry-run 模式**：只预览，不实际删除
- **确认机制**：删除前需要用户确认
- **分类处理**：区分已合并和未合并的分支

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
   - `clap` 自动处理参数验证和错误提示

2. **命令层**：用户交互错误、业务逻辑错误
   - 配置文件不存在：自动创建
   - 分支删除失败：记录警告，继续处理其他分支

3. **库层**：Git 操作错误、文件操作错误
   - 通过 `GitBranch` 和 `GitRepo` API 返回的错误信息
   - Git 操作失败、文件读写错误等

### 容错机制

- **配置文件不存在**：自动创建空配置
- **分支删除失败**：记录警告，继续处理其他分支
- **仓库名提取失败**：返回错误，提示用户检查 Git 仓库状态

---

## 📝 扩展性

### 添加新的分支清理规则

1. 在 `clean.rs` 的 `exclude_branches` 构建逻辑中添加新规则
2. 更新排除分支列表的构建逻辑

### 添加新的忽略列表操作

1. 在 `helpers.rs` 中添加新的辅助函数
2. 在 `ignore.rs` 中添加新的命令方法
3. 在 `src/main.rs` 中添加新的子命令枚举

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 操作相关
- [配置管理命令模块架构文档](./CONFIG_COMMAND_ARCHITECTURE.md) - 配置管理相关

---

## 📋 使用示例

### Clean 命令

```bash
# 预览将要删除的分支（dry-run）
workflow branch clean --dry-run

# 清理分支（需要确认）
workflow branch clean
```

### Ignore 命令

```bash
# 添加分支到忽略列表
workflow branch ignore add feature-branch

# 从忽略列表移除分支
workflow branch ignore remove feature-branch

# 列出忽略的分支
workflow branch ignore list
```

---

## ✅ 总结

分支管理命令层采用清晰的分层架构设计：

1. **智能清理**：自动识别已合并分支，安全删除
2. **灵活配置**：支持按仓库配置忽略列表
3. **安全机制**：预览、确认、分类处理，确保操作安全

**设计优势**：
- ✅ **安全性**：多重确认机制，防止误删重要分支
- ✅ **智能性**：自动识别已合并分支，分类处理
- ✅ **灵活性**：支持按仓库配置忽略列表
- ✅ **用户友好**：清晰的预览和确认提示
