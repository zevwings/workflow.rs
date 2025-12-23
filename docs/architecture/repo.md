# Repo 模块架构文档

## 📋 概述

Repo 模块是 Workflow CLI 的核心模块，提供仓库级配置管理功能。该模块采用分层架构设计，包括：

- **Lib 层**（`lib/repo/`）：提供仓库级配置管理的核心业务逻辑，包括配置检查、加载、保存等功能。配置存储在项目根目录的 `.workflow/config.toml` 文件中，支持分支配置、模板配置等
- **Commands 层**（`commands/repo/`）：提供 CLI 命令封装，处理用户交互，包括仓库级配置初始化、查看和仓库清理功能

Repo 模块支持为每个项目独立配置分支前缀、提交模板、PR 模板等设置，配置可以提交到 Git 与团队成员共享。

**模块统计：**
- Lib 层代码行数：约 358 行
- Commands 层代码行数：约 646 行
- 命令数量：3 个（setup、show、clean）
- 文件数量：Lib 层 2 个，Commands 层 3 个
- 主要组件：`RepoConfig`、`ProjectConfig`、`ProjectBranchConfig`

---

## 📁 Lib 层架构（核心业务逻辑）

Repo 模块（`lib/repo/`）是 Workflow CLI 的核心库模块，提供仓库级配置管理功能，包括配置检查、加载、保存等功能。

### 模块结构

```
src/lib/repo/
├── mod.rs          # Repo 模块声明和导出 (8行)
└── config.rs       # 仓库配置管理 (358行)
```

### 依赖模块

- **`lib/base/settings/`**：路径管理
  - `Paths::project_config()` - 获取项目配置文件路径
- **`lib/git/`**：Git 操作
  - `GitRepo::is_git_repo()` - 检查是否是 Git 仓库

### 模块集成

- **`commands/repo/`**：Repo 命令层
  - `setup.rs` - 使用 `RepoConfig` 进行配置加载和保存
  - `show.rs` - 使用 `RepoConfig` 进行配置加载和显示
- **`commands/branch/`**：分支管理命令
  - 使用 `RepoConfig::get_branch_prefix()` 获取分支前缀
  - 使用 `RepoConfig::get_ignore_branches()` 获取忽略分支列表
- **`commands/commit/`**、**`commands/pr/`**：其他命令
  - 使用 `RepoSetupCommand::ensure()` 确保配置存在

---

## 🏗️ Lib 层架构设计

### 设计原则

1. **职责单一**：专注于仓库级配置管理
2. **无状态设计**：所有方法都是静态方法，不维护状态
3. **错误传播**：使用 `Result` 类型传播错误，由调用层处理
4. **可测试性**：业务逻辑与文件操作分离，便于单元测试

### 核心组件

#### 1. RepoConfig 结构体

**职责**：提供仓库配置管理的静态方法

**主要方法**：
- `exists()` - 检查配置是否存在且完整
- `load()` - 加载项目配置
- `save()` - 保存项目配置
- `get_branch_prefix()` - 获取分支前缀
- `get_ignore_branches()` - 获取忽略分支列表

**关键特性**：
- 配置文件路径：`.workflow/config.toml`（项目根目录）
- 自动创建目录结构（如果不存在）
- 支持配置合并（保存时合并现有配置）
- 向后兼容（支持旧格式配置）

**使用场景**：
- `repo setup` 命令：加载和保存配置
- `repo show` 命令：加载配置用于显示
- `branch` 命令：获取分支前缀和忽略列表
- 其他命令：检查配置是否存在

#### 2. ProjectConfig 结构体

**职责**：表示项目级配置结构

**字段**：
- `template_commit: Map<String, Value>` - 提交模板配置
- `template_branch: Map<String, Value>` - 分支模板配置
- `template_pull_requests: Map<String, Value>` - PR 模板配置
- `branch: ProjectBranchConfig` - 分支配置
- `auto_accept_change_type: Option<bool>` - 自动接受变更类型选择

**关键特性**：
- 使用 `toml::Value` 存储模板配置，支持灵活配置
- 分支配置使用结构化类型 `ProjectBranchConfig`
- 支持可选字段（使用 `Option`）

#### 3. ProjectBranchConfig 结构体

**职责**：表示项目级分支配置

**字段**：
- `prefix: Option<String>` - 分支前缀（可选）
- `ignore: Vec<String>` - 忽略分支列表

**关键特性**：
- 使用 `serde` 进行序列化/反序列化
- 支持 TOML 格式
- 可选字段使用 `skip_serializing_if` 控制序列化

---

## 🔄 Lib 层核心功能

### 1. 配置检查 (`exists()`)

**功能**：检查仓库配置是否存在且完整

**流程**：
1. 检查是否在 Git 仓库中（如果不是，返回 `true`，跳过检查）
2. 检查配置文件是否存在
3. 检查配置是否有必需的节（`[template.commit]` 或 `[branch]`）

**返回值**：
- `Ok(true)` - 配置存在且完整
- `Ok(false)` - 配置不存在或不完整
- `Err(_)` - 检查过程中出错

### 2. 配置加载 (`load()`)

**功能**：从文件加载项目配置

**流程**：
1. 获取配置文件路径（`.workflow/config.toml`）
2. 如果文件不存在，返回默认配置
3. 读取并解析 TOML 文件
4. 解析各个配置节：
   - `[template.commit]` - 提交模板配置
   - `[template.branch]` - 分支模板配置
   - `[template.pull_requests]` - PR 模板配置
   - `[branch]` - 分支配置
   - `[pr]` 或顶层 `auto_accept_change_type` - PR 配置

**关键特性**：
- 文件不存在时返回默认配置（不报错）
- 支持向后兼容（手动解析旧格式）
- 使用 `toml::Value` 灵活处理模板配置

### 3. 配置保存 (`save()`)

**功能**：保存项目配置到文件

**流程**：
1. 获取配置文件路径
2. 确保目录存在（创建 `.workflow/` 目录）
3. 读取现有配置（如果存在）
4. 合并新配置与现有配置
5. 写入文件

**关键特性**：
- 自动创建目录结构
- 配置合并（保留现有配置，只更新提供的字段）
- 使用 `toml::to_string_pretty()` 格式化输出

### 4. 分支前缀获取 (`get_branch_prefix()`)

**功能**：获取当前仓库的分支前缀

**流程**：
1. 加载项目配置
2. 返回 `config.branch.prefix`

**返回值**：
- `Some(String)` - 如果配置了分支前缀
- `None` - 如果未配置或加载失败

### 5. 忽略分支列表获取 (`get_ignore_branches()`)

**功能**：获取当前仓库的忽略分支列表

**流程**：
1. 加载项目配置
2. 返回 `config.branch.ignore`

**返回值**：
- `Vec<String>` - 忽略分支列表（如果未配置或加载失败，返回空列表）

---

## 📁 Commands 层架构（命令封装）

Repo 命令层是 Workflow CLI 的命令接口，提供仓库级配置初始化、查看和仓库清理功能。该层采用命令模式设计，通过调用 `lib/repo/` 模块提供的 API 实现业务功能。

### 相关文件

#### CLI 入口层

```
src/bin/workflow.rs
```
- **职责**：`workflow` 主命令入口，负责命令行参数解析和命令分发
- **功能**：使用 `clap` 解析命令行参数，将 `workflow repo` 子命令分发到对应的命令处理函数
- **命令枚举**：`RepoSubcommand` 定义了所有 Repo 相关的子命令（setup、show、clean）

#### 命令封装层

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

---

## 🔄 集成关系

### Lib 层和 Commands 层的协作

Repo 模块采用清晰的分层架构，Lib 层和 Commands 层通过以下方式协作：

1. **配置管理**：Commands 层调用 `RepoConfig` 的方法进行配置加载和保存
2. **配置检查**：Commands 层使用 `RepoConfig::exists()` 检查配置是否存在
3. **配置获取**：Commands 层使用 `RepoConfig` 的 getter 方法获取配置值
4. **用户交互**：Commands 层负责格式化输出和用户提示

### 调用流程

#### 整体架构流程

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

#### 命令分发流程

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

---

## 📋 Commands 层命令详情

### 1. Repo Setup 命令 (`setup.rs`)

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

**关键步骤**：
1. 检查是否在 Git 仓库中
2. 如果配置文件存在，加载现有配置
3. 交互式收集配置（分支前缀、模板配置、PR 配置）
4. 保存配置到 `.workflow/config.toml`
5. 显示完成提示

**RepoSetupCommand::ensure() 方法**：

`RepoSetupCommand` 还提供了一个 `ensure()` 方法，用于在其他命令中自动检查并提示用户配置仓库设置。

**使用场景**：
- 在 `workflow branch create`、`workflow commit`、`workflow pr create` 等命令开始时调用
- 如果配置不存在，自动提示用户运行 `workflow repo setup`

**特点**：
- 仅在交互式环境（TTY）中提示
- 如果用户取消，不影响命令继续执行
- 如果配置已存在，立即返回

### 2. Repo Show 命令 (`show.rs`)

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

### 3. Repo Clean 命令 (`clean.rs`)

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

---

## 📝 配置文件格式

### 配置文件位置

```
项目根目录/.workflow/config.toml
```

### 配置格式示例

```toml
[branch]
prefix = "feature"
ignore = [
    "main",
    "master",
    "develop",
    "zw/important-feature",
]

[template.commit]
use_scope = false
default = "{{#if jira_key}}{{jira_key}}: {{subject}}{{else}}# {{subject}}{{/if}}"

[template.branch]
default = "{{jira_key}}-{{summary_slug}}"
feature = "feature/{{jira_key}}-{{summary_slug}}"

[template.pull_requests]
default = "## Description\n\n{{jira_summary}}\n\n..."

[pr]
auto_accept_change_type = true
```

### 配置节说明

#### `[branch]` 节

- `prefix` (可选) - 分支前缀，用于生成分支名时自动添加前缀
- `ignore` (可选) - 忽略分支列表，分支清理时会自动排除这些分支

#### `[template.commit]` 节

- `use_scope` (可选) - 是否使用 scope（Conventional Commits 格式）
- `default` (可选) - 默认提交模板（Handlebars 格式）

#### `[template.branch]` 节

- `default` (可选) - 默认分支模板
- `feature` (可选) - Feature 分支模板
- `bugfix` (可选) - Bugfix 分支模板
- `hotfix` (可选) - Hotfix 分支模板
- `refactoring` (可选) - Refactoring 分支模板
- `chore` (可选) - Chore 分支模板

#### `[template.pull_requests]` 节

- `default` (可选) - 默认 PR 模板（Handlebars 格式）

#### `[pr]` 节

- `auto_accept_change_type` (可选) - 是否自动接受变更类型选择

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数解析错误、命令不存在等
2. **命令层**：用户取消操作、输入验证失败等
3. **库层**：配置读写错误、文件系统错误等

### 错误类型

1. **文件系统错误**：
   - 目录创建失败
   - 文件读取失败
   - 文件写入失败

2. **配置解析错误**：
   - TOML 格式错误
   - 配置结构不匹配

3. **Git 仓库错误**：
   - 不在 Git 仓库中
   - 无法提取仓库名

### 容错机制

- **配置文件不存在**：返回默认配置（不报错）
- **配置解析失败**：返回错误，提示用户检查配置文件
- **Git 仓库检测失败**：返回错误，提示用户在 Git 仓库中运行

---

## 📋 使用示例

### Repo Setup 命令

```bash
# 交互式初始化仓库配置
workflow repo setup
```

### Repo Show 命令

```bash
# 显示当前仓库配置
workflow repo show
```

### Repo Clean 命令

```bash
# 清理本地分支和本地 tag
workflow repo clean

# 预览模式（不实际删除）
workflow repo clean --dry-run
```

### 基本使用（Lib 层）

```rust
use workflow::repo::RepoConfig;

// 检查配置是否存在
if !RepoConfig::exists()? {
    println!("Configuration not found");
}

// 加载配置
let config = RepoConfig::load()?;
println!("Branch prefix: {:?}", config.branch.prefix);

// 保存配置
let mut config = ProjectConfig::default();
config.branch.prefix = Some("feature".to_string());
RepoConfig::save(&config)?;

// 获取分支前缀
if let Some(prefix) = RepoConfig::get_branch_prefix() {
    println!("Branch prefix: {}", prefix);
}

// 获取忽略分支列表
let ignore_list = RepoConfig::get_ignore_branches();
println!("Ignored branches: {:?}", ignore_list);
```

---

## 🔄 与其他模块的集成

### 与 Template 模块的集成

- Repo 模块管理模板配置的存储和加载
- Template 模块使用 Repo 模块加载的配置进行模板渲染
- 配置存储在项目级配置文件中，支持团队共享

### 与 Branch 模块的集成

- Branch 模块使用 Repo 模块获取分支前缀和忽略列表
- 分支创建时自动应用配置的分支前缀
- 分支清理时自动排除配置的忽略列表

### 与命令层的集成

- `repo setup` 命令使用 Repo 模块进行配置保存
- `repo show` 命令使用 Repo 模块进行配置加载
- `branch` 命令使用 Repo 模块获取配置
- 其他命令使用 `RepoSetupCommand::ensure()` 确保配置存在

---

## 📝 扩展性

### 添加新配置项

1. 在 `lib/repo/config.rs` 中的 `ProjectConfig` 结构体中添加新字段
2. 在 `commands/repo/setup.rs` 的 `collect_config()` 方法中添加交互式收集逻辑
3. 在 `commands/repo/show.rs` 的 `show()` 方法中添加显示逻辑
4. 更新配置文件的保存和加载逻辑

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Template 模块架构文档](./template.md) - 模板系统
- [Branch 模块架构文档](./branch.md) - 分支管理
- [Git 模块架构文档](./git.md) - Git 操作相关

---

## ✅ 总结

Repo 模块采用清晰的设计原则：

1. **职责单一**：专注于仓库级配置管理
2. **无状态设计**：所有方法都是静态方法
3. **灵活配置**：支持多种配置类型和可选字段
4. **团队协作**：配置存储在项目根目录，可提交到 Git
5. **交互式配置**：提供完整的交互式配置流程
6. **自动检测**：在其他命令中自动检测并提示配置

**设计优势**：
- ✅ 职责清晰，易于维护
- ✅ 配置灵活，支持多种场景
- ✅ 团队协作友好，配置可版本控制
- ✅ 向后兼容，支持旧格式配置
- ✅ 用户体验友好，交互式配置
- ✅ 自动检测和提示，减少配置遗漏

**当前实现状态**：
- ✅ 配置检查功能完整实现
- ✅ 配置加载功能完整实现
- ✅ 配置保存功能完整实现
- ✅ 分支配置获取功能完整实现
- ✅ Repo Setup 功能完整实现
- ✅ Repo Show 功能完整实现
- ✅ Repo Clean 功能完整实现
- ✅ 交互式工作流完整
- ✅ 自动检测机制完整
- ✅ 分支和 tag 清理功能完整

通过分层架构和职责分离，实现了代码复用、易于维护和扩展的目标。命令层专注于用户交互和输出格式化，核心业务逻辑由 Lib 层提供，实现了清晰的职责分离。

---

**最后更新**: 2025-12-16

