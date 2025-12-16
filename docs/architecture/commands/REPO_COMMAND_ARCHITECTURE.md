# Repo 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Repo 命令模块架构，包括：
- 仓库级配置初始化功能（交互式设置）
- 仓库级配置查看功能
- 仓库清理功能（清理本地分支和 tag）

Repo 命令模块提供仓库级配置管理功能，支持为每个项目独立配置分支前缀、提交模板、PR 模板等设置。配置存储在项目根目录的 `.workflow/config.toml` 文件中，可以提交到 Git 与团队成员共享。

**定位**：命令层专注于用户交互、参数解析和输出格式化，核心业务逻辑由 `lib/repo/` 模块提供。

**模块统计：**
- 命令数量：3 个（setup、show、clean）
- 总代码行数：约 646 行
- 文件数量：3 个
- 主要依赖：`lib/repo/`、`lib/template/`、`lib/git/`、`lib/base/dialog/`

---

## 📁 相关文件

### CLI 入口层

```
src/bin/workflow.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow repo` 子命令分发到对应的命令处理函数
- **命令枚举**：`RepoSubcommand` 定义了所有 Repo 相关的子命令（setup、show、clean）

### 命令封装层

```
src/commands/repo/
├── mod.rs          # Repo 命令模块声明（10 行）
├── setup.rs        # 仓库设置命令（308 行）
├── show.rs         # 仓库配置显示命令（79 行）
└── clean.rs        # 仓库清理命令（259 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择、确认等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/repo/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/repo/`**：仓库配置管理
  - `RepoConfig::exists()` - 检查配置是否存在
  - `RepoConfig::load()` - 加载配置
  - `RepoConfig::save()` - 保存配置
- **`lib/template/`**：模板配置管理
  - `TemplateConfig::load()` - 加载模板配置
- **`lib/git/`**：Git 操作
  - `GitRepo::extract_repo_name()` - 提取仓库名
  - `GitRepo::is_git_repo()` - 检查是否是 Git 仓库
- **`lib/base/dialog/`**：用户交互对话框
  - `ConfirmDialog` - 确认对话框
  - `InputDialog` - 输入对话框
- **`lib/base/settings/`**：路径管理
  - `Paths::project_config()` - 获取项目配置文件路径

详细架构文档：参见 [Repo 模块架构文档](../lib/REPO_ARCHITECTURE.md)、[Template 模块架构文档](../lib/TEMPLATE_ARCHITECTURE.md)

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
src/bin/workflow.rs (workflow 主命令，参数解析)
  ↓
commands/repo/*.rs (命令封装层，处理交互)
  ↓
lib/repo/* (业务逻辑层，配置管理)
  ↓
.workflow/config.toml (项目级配置文件)
```

### 命令分发流程

```
src/bin/workflow.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.command {
  Commands::Repo { subcommand } => match subcommand {
    RepoSubcommand::Setup => RepoSetupCommand::run()
    RepoSubcommand::Show => RepoShowCommand::show()
    RepoSubcommand::Clean { dry_run } => RepoCleanCommand::clean(dry_run.dry_run)
  }
}
```

### 数据流

```
用户输入参数
  ↓
命令参数解析（clap）
  ↓
检查 Git 仓库状态
  ↓
加载现有配置（如果存在）
  ↓
用户交互（输入配置项、选择选项等）
  ↓
保存配置到 .workflow/config.toml
  ↓
显示完成提示
```

---

## 1. Repo Setup 命令 (`setup.rs`)

### 相关文件

```
src/commands/repo/setup.rs (308 行)
```

### 调用流程

```
RepoSetupCommand::run()
  ↓
GitRepo::extract_repo_name() (获取仓库名)
  ↓
Paths::project_config() (获取配置文件路径)
  ↓
RepoConfig::load() (加载现有配置，如果存在)
  ↓
RepoSetupCommand::collect_config() (交互式收集配置)
  ├─ 分支配置（分支前缀）
  ├─ 模板配置（提交模板、分支模板、PR 模板）
  └─ PR 配置（自动接受变更类型选择）
  ↓
RepoConfig::save() (保存配置)
  ↓
显示完成提示
```

### 功能说明

Repo Setup 命令用于交互式初始化仓库级配置，支持以下功能：

1. **分支配置**：
   - 设置分支前缀（如 `feature`、`fix` 等）
   - 用于生成分支名时自动添加前缀

2. **模板配置**：
   - **提交模板**：配置提交消息模板格式
     - 是否使用 scope（Conventional Commits 格式）
     - 自定义提交模板（可选）
   - **分支模板**：配置分支命名模板（可选）
   - **PR 模板**：配置 PR body 模板（可选）

3. **PR 配置**：
   - 自动接受变更类型选择（跳过确认提示）

4. **配置管理**：
   - 自动检测并加载现有配置
   - 支持更新现有配置
   - 配置保存到 `.workflow/config.toml`（项目根目录）

### 关键步骤说明

1. **仓库检测**：
   - 检查是否在 Git 仓库中
   - 提取仓库名用于显示

2. **配置加载**：
   - 如果配置文件存在，加载现有配置
   - 在交互式输入时显示当前值作为默认值

3. **交互式配置收集**：
   - **分支前缀**：可选输入，支持保持现有值
   - **提交模板**：询问是否配置，支持使用默认模板或自定义
   - **分支模板**：询问是否配置，支持使用默认模板或自定义
   - **PR 模板**：询问是否配置，支持使用默认模板或自定义
   - **自动接受变更类型**：确认是否自动接受 PR 创建时的变更类型选择

4. **配置保存**：
   - 合并新配置与现有配置
   - 保存到 `.workflow/config.toml`
   - 自动创建 `.workflow/` 目录（如果不存在）

5. **完成提示**：
   - 显示配置文件路径
   - 提示可以提交到 Git 与团队共享

### 数据流

```
用户输入 (workflow repo setup)
  ↓
检查 Git 仓库
  ↓
加载现有配置（如果存在）
  ↓
交互式收集配置
  ├─ 分支前缀
  ├─ 提交模板配置
  ├─ 分支模板配置
  ├─ PR 模板配置
  └─ PR 自动接受配置
  ↓
保存配置到 .workflow/config.toml
  ↓
显示完成提示
```

### 依赖模块

- **`lib/repo/`**：仓库配置管理
  - `RepoConfig::load()` - 加载配置
  - `RepoConfig::save()` - 保存配置
- **`lib/git/`**：Git 操作
  - `GitRepo::extract_repo_name()` - 提取仓库名
- **`lib/base/dialog/`**：对话框
  - `ConfirmDialog` - 确认对话框
  - `InputDialog` - 输入对话框
- **`lib/base/settings/`**：路径管理
  - `Paths::project_config()` - 获取项目配置文件路径

### RepoSetupCommand::ensure() 方法

`RepoSetupCommand` 还提供了一个 `ensure()` 方法，用于在其他命令中自动检查并提示用户配置仓库设置。

**使用场景**：
- 在 `workflow branch create`、`workflow commit`、`workflow pr create` 等命令开始时调用
- 如果配置不存在，自动提示用户运行 `workflow repo setup`

**特点**：
- 仅在交互式环境（TTY）中提示
- 如果用户取消，不影响命令继续执行
- 如果配置已存在，立即返回

---

## 2. Repo Clean 命令 (`clean.rs`)

### 相关文件

```
src/commands/repo/clean.rs (259 行)
```

### 调用流程

```
RepoCleanCommand::clean(dry_run)
  ↓
CheckCommand::run_all() (运行环境检查)
  ↓
GitBranch::current_branch() (获取当前分支)
  ↓
GitBranch::get_default_branch() (获取默认分支)
  ↓
GitRepo::extract_repo_name() (获取仓库名)
  ↓
GitRepo::prune_remote() (清理远端引用)
  ↓
RepoConfig::get_ignore_branches() (读取忽略分支列表)
  ↓
GitBranch::get_local_branches() (获取所有本地分支)
  ↓
分类分支（已合并 vs 未合并）
  ↓
显示预览（已合并分支、未合并分支）
  ↓
[如果是 dry-run 模式，直接返回]
  ↓
确认删除
  ↓
删除已合并分支
  ↓
[如果有未合并分支，询问是否强制删除]
  ↓
清理本地 tag（只存在于本地但不在远程的 tag）
  ↓
显示清理结果
```

### 功能说明

Repo Clean 命令用于清理本地分支和本地 tag，支持以下功能：

1. **分支清理**：
   - 清理已合并的分支（相对于默认分支）
   - 支持强制删除未合并的分支（需要用户确认）
   - 自动排除以下分支：
     - 当前分支
     - 默认分支（main/master）
     - develop 分支
     - 忽略列表中的分支（从仓库配置读取）

2. **Tag 清理**：
   - 清理只存在于本地但不在远程的 tag
   - 自动检测本地和远程 tag 的差异

3. **安全机制**：
   - 运行前进行环境检查（Git 状态、网络连接）
   - 显示预览信息（将要删除的分支和 tag）
   - 支持 dry-run 模式（预览模式，不实际删除）
   - 删除前需要用户确认
   - 已合并分支和未合并分支分别处理

### 关键步骤说明

1. **环境检查**：
   - 检查 Git 仓库状态
   - 检查网络连接（GitHub）

2. **初始化信息收集**：
   - 获取当前分支（不能删除）
   - 获取默认分支（不能删除）
   - 获取仓库名（用于显示）

3. **清理远端引用**：
   - 使用 `git remote prune origin` 清理已删除的远程分支引用

4. **读取配置**：
   - 从仓库配置读取忽略分支列表
   - 构建排除分支列表（当前分支、默认分支、develop、忽略列表）

5. **分支分类**：
   - 检查每个分支是否已合并到默认分支
   - 分类为已合并分支和未合并分支

6. **预览显示**：
   - 显示将要删除的已合并分支列表
   - 显示将要删除的未合并分支列表（警告标记）

7. **Dry-run 模式**：
   - 如果启用 dry-run，只显示预览，不实际删除

8. **删除确认**：
   - 显示删除统计信息（总数、已合并数、未合并数）
   - 用户确认后才执行删除

9. **删除已合并分支**：
   - 逐个删除已合并分支
   - 记录删除成功和失败的数量

10. **处理未合并分支**：
    - 询问用户是否强制删除未合并分支
    - 如果确认，逐个强制删除
    - 如果取消，跳过这些分支

11. **清理本地 tag**：
    - 获取所有 tag 信息（本地和远程）
    - 筛选出只存在于本地但不在远程的 tag
    - 显示预览
    - 用户确认后删除

12. **显示结果**：
    - 显示分支清理结果（删除数量、跳过数量）
    - 显示 tag 清理结果（删除数量、跳过数量）

### 数据流

```
用户输入 (workflow repo clean [--dry-run])
  ↓
环境检查（Git 状态、网络连接）
  ↓
获取当前分支、默认分支、仓库名
  ↓
清理远端引用
  ↓
读取忽略分支列表（仓库配置）
  ↓
获取所有本地分支
  ↓
过滤排除分支
  ↓
分类分支（已合并 vs 未合并）
  ↓
显示预览
  ↓
[如果是 dry-run，直接返回]
  ↓
确认删除
  ↓
删除已合并分支
  ↓
[如果有未合并分支，询问是否强制删除]
  ↓
清理本地 tag（只存在于本地但不在远程）
  ↓
显示清理结果
```

### 依赖模块

- **`lib/git/`**：Git 操作
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::get_default_branch()` - 获取默认分支
  - `GitBranch::get_local_branches()` - 获取本地分支列表
  - `GitBranch::is_branch_merged()` - 检查分支是否已合并
  - `GitBranch::delete()` - 删除分支
  - `GitRepo::extract_repo_name()` - 提取仓库名
  - `GitRepo::prune_remote()` - 清理远端引用
  - `GitTag::list_all_tags()` - 获取所有 tag 信息
  - `GitTag::delete_local()` - 删除本地 tag
- **`lib/repo/`**：仓库配置管理
  - `RepoConfig::get_ignore_branches()` - 获取忽略分支列表
- **`lib/base/dialog/`**：对话框
  - `ConfirmDialog` - 确认对话框
- **`commands/check/`**：环境检查
  - `CheckCommand::run_all()` - 运行所有检查

---

## 3. Repo Show 命令 (`show.rs`)

### 相关文件

```
src/commands/repo/show.rs (79 行)
```

### 调用流程

```
RepoShowCommand::show()
  ↓
GitRepo::extract_repo_name() (获取仓库名)
  ↓
RepoConfig::load() (加载分支配置)
  ↓
TemplateConfig::load() (加载模板配置)
  ↓
格式化并显示配置信息
  ├─ 分支配置（前缀）
  ├─ 提交模板配置
  ├─ 分支模板配置
  └─ PR 模板配置
```

### 功能说明

Repo Show 命令用于显示当前仓库的配置信息，包括：

1. **仓库信息**：
   - 显示当前仓库名

2. **分支配置**：
   - 显示分支前缀（如果已配置）
   - 如果未配置，提示运行 `workflow repo setup`

3. **模板配置**：
   - **提交模板**：显示是否使用 scope、默认模板内容
   - **分支模板**：显示默认模板和类型特定模板（feature、bugfix、hotfix、refactoring、chore）
   - **PR 模板**：显示默认 PR 模板内容

### 关键步骤说明

1. **仓库检测**：
   - 检查是否在 Git 仓库中
   - 提取仓库名用于显示

2. **配置加载**：
   - 从项目级配置加载分支配置
   - 从全局和项目级配置加载模板配置

3. **格式化显示**：
   - 按配置类型分组显示
   - 使用分隔线区分不同配置节
   - 对于未配置的项，显示提示信息

### 数据流

```
用户输入 (workflow repo show)
  ↓
检查 Git 仓库
  ↓
加载配置（分支配置、模板配置）
  ↓
格式化并显示配置信息
```

### 依赖模块

- **`lib/repo/`**：仓库配置管理
  - `RepoConfig::load()` - 加载配置
- **`lib/template/`**：模板配置管理
  - `TemplateConfig::load()` - 加载模板配置
- **`lib/git/`**：Git 操作
  - `GitRepo::extract_repo_name()` - 提取仓库名

---

## 🏗️ 架构设计

### 设计模式

#### 1. 分层架构

命令层专注于用户交互和参数解析，业务逻辑层处理核心功能。

**优势**：
- 职责清晰，易于维护
- 业务逻辑可复用
- 便于测试

#### 2. 交互式工作流

提供完整的交互式流程，包括输入、选择、确认等步骤。

**优势**：
- 用户体验友好
- 减少误操作
- 提供清晰的配置选项

#### 3. 项目级配置

配置存储在项目根目录，可以提交到 Git 与团队共享。

**优势**：
- 团队协作友好
- 不同项目可以有不同的配置
- 配置版本控制

### 错误处理

#### 分层错误处理

1. **CLI 层**：参数解析错误、命令不存在等
2. **命令层**：用户取消操作、输入验证失败等
3. **库层**：配置读写错误、文件系统错误等

#### 容错机制

- **配置文件不存在**：自动创建空配置
- **Git 仓库检测失败**：返回错误，提示用户检查
- **配置解析失败**：返回错误，提示用户检查配置文件格式

---

## 📋 使用示例

### Repo Setup 命令

```bash
# 交互式初始化仓库配置
workflow repo setup
```

### Repo Clean 命令

```bash
# 清理本地分支和本地 tag
workflow repo clean

# 预览模式（不实际删除）
workflow repo clean --dry-run
```

### Repo Show 命令

```bash
# 显示当前仓库配置
workflow repo show
```

---

## 📝 扩展性

### 添加新配置项

1. 在 `lib/repo/config.rs` 中的 `ProjectConfig` 结构体中添加新字段
2. 在 `commands/repo/setup.rs` 的 `collect_config()` 方法中添加交互式收集逻辑
3. 在 `commands/repo/show.rs` 的 `show()` 方法中添加显示逻辑
4. 更新配置文件的保存和加载逻辑

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Repo 模块架构文档](../lib/REPO_ARCHITECTURE.md) - Lib 层模块
- [Template 模块架构文档](../lib/TEMPLATE_ARCHITECTURE.md) - 模板系统
- [Git 模块架构文档](../lib/GIT_ARCHITECTURE.md) - Git 操作相关

---

## ✅ 总结

Repo 命令模块采用清晰的分层架构设计：

1. **交互式配置**：提供完整的交互式配置流程
2. **项目级配置**：配置存储在项目根目录，可提交到 Git
3. **团队协作**：支持团队共享配置
4. **自动检测**：在其他命令中自动检测并提示配置

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 用户体验友好，交互式配置
- ✅ 支持团队协作，配置可版本控制
- ✅ 自动检测和提示，减少配置遗漏

**当前实现状态**：
- ✅ Repo Setup 功能完整实现
- ✅ Repo Show 功能完整实现
- ✅ Repo Clean 功能完整实现
- ✅ 交互式工作流完整
- ✅ 自动检测机制完整
- ✅ 分支和 tag 清理功能完整
