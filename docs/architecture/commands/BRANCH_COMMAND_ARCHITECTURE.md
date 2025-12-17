# 分支管理命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的分支管理命令模块架构，包括：
- 分支创建功能（支持从 JIRA ticket 创建，使用 LLM 生成分支名）
- 分支切换功能（支持直接切换和交互式选择，自动处理未提交更改）
- 分支重命名功能（支持本地和远程分支重命名，提供完整的交互式流程）
- 本地分支清理功能
- 分支忽略列表管理功能

**注意**：分支前缀配置已迁移到 `workflow repo setup` 命令，通过项目级配置文件（`.workflow/config.toml`）管理。

分支管理命令提供智能的分支清理功能，可以安全地删除已合并的分支，同时保留重要的分支（如 main/master、develop、当前分支和用户配置的忽略分支）。同时支持为不同仓库配置不同的分支前缀，用于生成分支名时自动添加前缀。新增的分支创建功能支持从 JIRA ticket 创建分支，使用 LLM 自动生成分支名，并支持从默认分支创建。分支切换功能支持快速切换分支，当分支不存在时自动询问是否创建，并自动处理未提交的更改。分支重命名功能提供完整的交互式流程，支持重命名本地和远程分支，包含多重验证和确认机制。

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
├── mod.rs          # 分支命令模块声明
├── create.rs       # 分支创建命令（~347 行）
├── switch.rs       # 分支切换命令（~104 行）
├── rename.rs       # 分支重命名命令（~357 行）
├── clean.rs        # 分支清理命令（~195 行）
├── ignore.rs       # 分支忽略列表管理命令（~199 行）
└── helpers.rs      # 辅助函数（分支选择等，~260 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（确认、预览等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/git/`) 的功能
- 管理项目级分支配置文件（`.workflow/config.toml`）

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
.workflow/config.toml (项目级配置文件)
```

### 命令分发流程

```
src/main.rs::main()
  ↓
Cli::parse() (解析命令行参数)
  ↓
match cli.subcommand {
  BranchSubcommand::Create { jira_id, from_default, dry_run } => CreateCommand::execute()
  BranchSubcommand::Switch { branch_name } => SwitchCommand::execute()
  BranchSubcommand::Rename => BranchRenameCommand::execute()
  BranchSubcommand::Clean { dry_run } => BranchCleanCommand::clean()
  BranchSubcommand::Ignore { subcommand } => match subcommand {
    IgnoreSubcommand::Add { branch_name } => BranchIgnoreCommand::add()
    IgnoreSubcommand::Remove { branch_name } => BranchIgnoreCommand::remove()
    IgnoreSubcommand::List => BranchIgnoreCommand::list()
  }
    // Note: Branch prefix command has been removed.
    // Use 'workflow repo setup' to configure branch prefix.
  }
}
```

---

## 1. 分支创建命令 (`create.rs`)

### 相关文件

```
src/commands/branch/create.rs (~347 行)
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::BranchSubcommand::Create { jira_id, from_default, dry_run }
  ↓
commands/branch/create.rs::CreateCommand::execute(jira_id, from_default, dry_run)
  ↓
  1. 解析 JIRA ticket ID（可选，如果未提供则交互式输入）
  2. 确定分支类型（如果仓库前缀存在则使用，否则交互式选择）
  3. 确定分支名：
     - 如果有 JIRA ticket：使用 LLM 从 ticket 信息生成分支名
     - 否则：交互式输入分支名
  4. 格式化分支名（使用模板：{type}/{jira-ticket}-{branch-name}）
  5. 确定基础分支（--from-default 则从默认分支创建，否则从当前分支创建）
  6. Dry-run 模式（如果启用，只预览不执行）
  7. 切换到基础分支（如果需要）
  8. 创建新分支
```

### 功能说明

分支创建命令提供智能的分支创建功能：

1. **JIRA 集成**：
   - 支持从 JIRA ticket ID 创建分支
   - 使用 LLM 从 ticket 信息自动生成分支名
   - 自动验证 JIRA ticket 格式

2. **分支类型确定**：
   - 优先使用仓库配置的分支前缀作为分支类型
   - 如果未配置，则交互式选择（feature/bugfix/refactoring/hotfix/chore）

3. **分支名生成**：
   - 从 JIRA ticket：使用 LLM 生成分支名 slug
   - 手动输入：支持非英文输入，自动转换为 slug
   - 格式化：`{type}/{jira-ticket}-{branch-name}`

4. **基础分支选择**：
   - `--from-default`：从默认分支（main/master）创建
   - 默认：从当前分支创建（可选拉取最新更改）

5. **安全机制**：
   - Dry-run 模式：预览将要创建的分支名
   - 自动处理未提交的更改（stash）
   - 自动拉取最新更改（可选）

### 关键步骤说明

1. **JIRA ticket 解析**：
   - 如果提供了 ticket ID，验证格式
   - 如果未提供，交互式输入（可选）
   - 获取 ticket 信息用于生成分支名

2. **分支类型确定**：
   - 检查仓库配置的分支前缀（`get_branch_prefix()`）
   - 如果前缀匹配已知分支类型，直接使用
   - 否则，交互式选择分支类型

3. **分支名生成**：
   - 从 JIRA：调用 `PullRequestLLM::generate()` 生成分支名
   - 手动输入：使用 `BranchNaming::sanitize_and_translate_branch_name()` 转换为 slug
   - 格式化：使用 `BranchNaming::from_type_and_slug()` 生成最终分支名

4. **分支创建流程**：
   - 如果 `--from-default`：切换到默认分支并拉取最新更改
   - 否则：询问是否拉取当前分支最新更改
   - 自动处理未提交更改（stash push/pop）
   - 创建并切换到新分支

### 数据流

```
用户输入 (workflow branch create [JIRA_ID] [--from-default] [--dry-run])
  ↓
解析 JIRA ticket ID（可选）
  ↓
确定分支类型（仓库前缀或交互式选择）
  ↓
确定分支名（LLM 生成或交互式输入）
  ↓
格式化分支名
  ↓
确定基础分支
  ↓
Dry-run 预览（如果启用）
  ↓
切换到基础分支（如果需要）
  ↓
创建新分支
```

### 依赖模块

- **`lib/branch/`**：分支命名和类型管理（`BranchNaming`、`BranchType`）
- **`lib/jira/`**：JIRA ticket 信息获取（`Jira::get_ticket_info()`）
- **`lib/pr/llm.rs`**：LLM 分支名生成（`PullRequestLLM::generate()`）
- **`lib/git/`**：Git 操作（`GitBranch`、`GitCommit`、`GitStash`）
- **`commands/branch/helpers.rs`**：分支前缀获取（`get_branch_prefix()`）
- **`commands/pr/helpers.rs`**：Stash 处理辅助函数（`handle_stash_pop_result()`）

---

## 2. 分支切换命令 (`switch.rs`)

### 相关文件

```
src/commands/branch/switch.rs (~104 行)
src/commands/branch/helpers.rs (分支选择辅助函数)
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::BranchSubcommand::Switch { branch_name }
  ↓
commands/branch/switch.rs::SwitchCommand::execute(branch_name)
  ↓
  1. 如果提供了分支名：直接使用
  2. 如果未提供分支名：交互式选择分支（使用 helpers::select_branch()）
     - 分支数量 > 25：自动启用 fuzzy filter（支持搜索）
     - 分支数量 <= 25：使用普通 selector
  3. 检查是否已在目标分支（如果是则退出）
  4. 检查分支是否存在（本地或远程）
  5. 如果分支不存在：使用 ConfirmDialog 询问是否创建
  6. 检查未提交更改：如果有则自动 stash
  7. 切换或创建分支（GitBranch::checkout_branch()）
  8. 如果切换失败且之前有 stash：恢复 stash
  9. 如果之前有 stash：恢复 stash（使用 handle_stash_pop_result()）
```

### 功能说明

分支切换命令提供快速、智能的分支切换功能：

1. **直接切换**：
   - 支持直接指定分支名切换
   - 如果分支不存在，使用 `ConfirmDialog` 询问是否创建

2. **交互式选择**：
   - 不带参数时自动进入交互式选择
   - 显示所有可用分支（本地 + 远程，已去重）
   - 标记当前分支（显示 "[current]"）
   - 排除当前分支或标记为当前（默认选择）

3. **智能搜索**：
   - 分支数量 > 25：自动启用 fuzzy filter，支持输入关键词实时过滤
   - 分支数量 <= 25：使用普通 selector，通过方向键浏览
   - 使用 `fuzzy-matcher` crate 进行模糊匹配

4. **自动处理未提交更改**：
   - 自动检测未提交的更改
   - 切换前自动 stash
   - 切换后自动恢复 stash
   - 如果切换失败，自动恢复之前的 stash

### 关键步骤说明

1. **分支选择**：
   - 使用 `helpers::select_branch()` 统一的分支选择辅助函数
   - 支持标记当前分支、排除当前分支、设置默认索引等选项
   - 根据分支数量自动决定是否启用 fuzzy filter

2. **分支存在性检查**：
   - 使用 `GitBranch::is_branch_exists()` 检查本地和远程
   - 如果都不存在，使用 `ConfirmDialog` 询问是否创建

3. **Stash 处理**：
   - 使用 `GitCommit::has_commit()` 检查未提交更改
   - 使用 `GitStash::stash_push()` 保存更改
   - 使用 `GitStash::stash_pop()` 恢复更改
   - 使用 `handle_stash_pop_result()` 处理 stash pop 结果（处理冲突等）

### 数据流

```
用户输入 (workflow branch switch [BRANCH_NAME])
  ↓
分支选择（直接指定或交互式选择）
  ↓
检查是否已在目标分支
  ↓
检查分支是否存在
  ↓
如果不存在：询问是否创建
  ↓
检查未提交更改并 stash（如果需要）
  ↓
切换或创建分支
  ↓
恢复 stash（如果之前有 stash）
```

### 依赖模块

- **`lib/git/`**：Git 操作（`GitBranch`、`GitCommit`、`GitStash`）
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::get_all_branches()` - 获取所有分支（本地+远程）
  - `GitBranch::is_branch_exists()` - 检查分支是否存在
  - `GitBranch::checkout_branch()` - 切换/创建分支
  - `GitCommit::has_commit()` - 检查是否有未提交的更改
  - `GitStash::stash_push()` - 保存未提交的更改
  - `GitStash::stash_pop()` - 恢复保存的更改
- **`commands/branch/helpers.rs`**：分支选择辅助函数（`select_branch()`）
- **`commands/pr/helpers.rs`**：Stash 处理辅助函数（`handle_stash_pop_result()`）
- **`lib/base/dialog/`**：对话框（`SelectDialog`、`ConfirmDialog`）

---

## 3. 分支重命名命令 (`rename.rs`)

### 相关文件

```
src/commands/branch/rename.rs (~357 行)
src/commands/branch/helpers.rs (分支选择辅助函数)
src/main.rs (命令入口)
```

### 调用流程

```
src/main.rs::BranchSubcommand::Rename
  ↓
commands/branch/rename.rs::BranchRenameCommand::execute()
  ↓
  1. 运行环境检查（CheckCommand::run_all()）
  2. 选择要重命名的分支（完全交互式）：
     - 询问是否重命名当前分支（ConfirmDialog）
     - 如果否：从分支列表选择（使用 helpers::select_branch()）
  3. 输入并验证新分支名（完全交互式）：
     - 输入新分支名（InputDialog）
     - 验证分支名格式（Git 规范）
     - 检查本地是否存在（如果存在则提示错误，重新输入）
  4. 检查新分支名是否与旧分支名相同（如果是则退出）
  5. 预览和确认（完全交互式）：
     - 检查是否是默认分支（需要额外警告）
     - 显示预览信息（旧分支名、新分支名、是否当前分支、远程分支状态）
     - 最终确认（ConfirmDialog）
  6. 执行重命名（完全交互式）：
     - 重命名本地分支（GitBranch::rename()）
     - 检查远程分支是否存在
     - 如果存在：显示警告，询问是否更新远程分支
     - 如果用户选择更新：二次确认后执行远程分支重命名（GitBranch::rename_remote()）
  7. 显示完成信息
```

### 功能说明

分支重命名命令提供完整的交互式分支重命名功能：

1. **完全交互式流程**：
   - 所有操作都通过交互式提示完成，无需记忆复杂参数
   - 清晰的步骤引导，每一步都有明确提示

2. **分支选择**：
   - 优先询问是否重命名当前分支（最常用场景）
   - 如果否，从分支列表选择（使用统一的分支选择辅助函数）

3. **新分支名验证**：
   - 验证分支名格式（Git 规范）：
     - 不能为空、不能以 `.` 开头或结尾
     - 不能包含 `..`、空格、特殊字符（`~ ^ : ? * [ \`）
     - 不能以 `/` 结尾、不能包含连续斜杠 `//`
     - 不能是保留名称（`HEAD`、`FETCH_HEAD` 等）
   - 检查本地是否存在（如果存在则提示错误，重新输入）

4. **预览和确认**：
   - 检查是否是默认分支（需要额外警告和确认）
   - 显示完整的预览信息：
     - 旧分支名、新分支名
     - 是否当前分支
     - 远程分支状态（本地分支、远程分支、远程跟踪）
   - 最终确认（ConfirmDialog）

5. **远程分支处理**：
   - 自动检测远程分支是否存在
   - 如果存在：显示警告信息（影响其他协作者、PR、CI/CD 等）
   - 询问是否更新远程分支
   - 如果用户选择更新：二次确认后执行远程分支重命名
   - 如果用户选择不更新：显示手动更新命令提示

6. **安全机制**：
   - 环境检查（运行所有检查）
   - 多重验证（分支名格式、存在性检查）
   - 多重确认（预览确认、远程分支更新确认）
   - 默认分支额外警告

### 关键步骤说明

1. **分支名格式验证**：
   - 实现完整的 Git 分支名规范验证
   - 包含所有 Git 不允许的字符和格式
   - 提供清晰的错误提示

2. **默认分支检测**：
   - 使用 `GitBranch::get_default_branch()` 获取默认分支
   - 如果是默认分支，显示额外警告和确认

3. **远程分支重命名**：
   - 使用 `GitBranch::has_remote_branch()` 检查远程分支
   - 使用 `GitBranch::rename_remote()` 重命名远程分支
   - 包含推送新分支、删除旧分支、更新远程跟踪设置

4. **远程跟踪检查**：
   - 使用 `git config --get branch.{name}.remote` 检查远程跟踪设置
   - 在预览信息中显示远程跟踪状态

### 数据流

```
用户输入 (workflow branch rename)
  ↓
环境检查（CheckCommand::run_all()）
  ↓
选择要重命名的分支（交互式）
  ↓
输入并验证新分支名（交互式，循环直到通过）
  ↓
检查新分支名是否与旧分支名相同
  ↓
预览和确认（交互式）
  ↓
重命名本地分支
  ↓
检查远程分支并处理（交互式）
  ↓
显示完成信息
```

### 依赖模块

- **`lib/git/`**：Git 操作（`GitBranch`）
  - `GitBranch::current_branch()` - 获取当前分支
  - `GitBranch::get_default_branch()` - 获取默认分支
  - `GitBranch::get_local_branches()` - 获取所有本地分支
  - `GitBranch::is_branch_exists()` - 检查分支是否存在
  - `GitBranch::has_remote_branch()` - 检查远程分支是否存在
  - `GitBranch::rename()` - 重命名本地分支
  - `GitBranch::rename_remote()` - 重命名远程分支
- **`commands/branch/helpers.rs`**：分支选择辅助函数（`select_branch()`）
- **`commands/check/`**：环境检查（`CheckCommand::run_all()`）
- **`lib/base/dialog/`**：对话框（`SelectDialog`、`ConfirmDialog`、`InputDialog`）

---

## 4. 分支清理命令 (`clean.rs`)

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
   - 配置文件排除：从项目级配置（`.workflow/config.toml`）读取忽略列表
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
   - 配置文件路径：`.workflow/config.toml`（项目级配置）
   - 存储忽略分支列表和分支前缀
   - 使用 `RepoConfig` 进行配置读写

### 数据流

```
用户输入 (workflow branch clean [--dry-run])
  ↓
环境检查 (CheckCommand::run_all())
  ↓
获取分支信息 (GitBranch, GitRepo)
  ↓
读取配置文件 (BranchConfig::get_ignore_branches_for_current_repo())
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

## 5. 分支忽略列表管理命令 (`ignore.rs`)

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
   [owner/repo]
   branch_prefix = "feature"
   branch_ignore = ["branch1", "branch2"]
   ```

2. **仓库名提取**：
   - 使用 `GitRepo::extract_repo_name()` 提取仓库名
   - 格式：`owner/repo`（如 `github.com/owner/repo` → `owner/repo`）

3. **配置管理**：
   - 使用 `ConfigManager<BranchConfig>` 进行配置读写
   - 自动创建配置文件（如果不存在）
   - 按仓库分组管理，支持多仓库配置

---

## 6. 辅助函数 (`helpers.rs`)

### 相关文件

```
src/commands/branch/helpers.rs (~330 行)
```

### 功能说明

辅助函数模块提供分支配置文件的完整管理功能：

1. **配置结构体**：
   - `ProjectBranchConfig` - 项目级分支配置（包含前缀和忽略列表）
   - `RepoConfig` - 项目级配置管理

2. **配置管理函数**：
   - `RepoConfig::get_branch_prefix()` - 获取当前仓库的分支前缀（从项目级配置）
   - `RepoConfig::get_ignore_branches()` - 获取当前仓库的忽略分支列表（从项目级配置）
   - `RepoConfig::load()` - 读取项目级配置文件
   - `RepoConfig::save()` - 保存项目级配置文件
   - `RepoSetupCommand::run()` - 交互式设置项目级配置（包括分支前缀）
   - `BranchIgnoreCommand::add()` - 添加分支到忽略列表（保存到项目级配置）
   - `BranchIgnoreCommand::remove()` - 从忽略列表移除分支（从项目级配置）
   - `BranchIgnoreCommand::list()` - 列出忽略分支（从项目级配置）

### 配置文件结构

```toml
[branch]
prefix = "feature"
ignore = ["feature-branch", "hotfix-branch"]
```

**注意**：
- 配置文件路径：`.workflow/config.toml`（项目级配置）
- `prefix` 字段是可选的，如果未设置则不会序列化到配置文件
- 配置可以提交到 Git，团队成员共享
- 分支前缀通过 `workflow repo setup` 命令配置

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

每个命令都是一个独立的结构体，实现统一的方法接口：
- `CreateCommand::execute()` - 创建分支
- `BranchCleanCommand::clean()` - 清理分支
- `BranchIgnoreCommand::add()` - 添加忽略分支
- `BranchIgnoreCommand::remove()` - 移除忽略分支
- `BranchIgnoreCommand::list()` - 列出忽略分支
- `RepoSetupCommand::run()` - 设置项目级配置（包括分支前缀和忽略列表）
- `RepoShowCommand::show()` - 显示项目级配置

#### 2. 配置管理模式

使用 `RepoConfig` 管理项目级配置文件：
- 配置文件路径：`.workflow/config.toml`（项目级配置）
- 每个项目有独立的配置
- 自动创建配置文件（如果不存在）
- 同时管理分支前缀和忽略列表，统一配置结构
- 配置可以提交到 Git，团队成员共享

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

### Create 命令

```bash
# 从 JIRA ticket 创建分支（交互式选择分支类型）
workflow branch create PROJ-123

# 从 JIRA ticket 创建分支，从默认分支创建
workflow branch create PROJ-123 --from-default

# 手动输入创建分支（交互式输入分支名和类型）
workflow branch create

# 预览模式（不实际创建）
workflow branch create PROJ-123 --dry-run
```

### Switch 命令

```bash
# 直接指定分支名切换（不存在时询问是否创建）
workflow branch switch feature/new-feature

# 交互式选择分支（分支数量 > 25 时自动启用搜索）
workflow branch switch
```

### Rename 命令

```bash
# 交互式重命名分支（完全交互式，支持本地和远程分支）
workflow branch rename
```

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

### 分支前缀配置

**注意**：分支前缀配置已迁移到 `workflow repo setup` 命令。

```bash
# 配置项目级设置（包括分支前缀）
workflow repo setup

# 显示项目级配置（包括分支前缀）
workflow repo show
```

---

## ✅ 总结

分支管理命令层采用清晰的分层架构设计：

1. **智能创建**：支持从 JIRA ticket 创建分支，使用 LLM 自动生成分支名
2. **快速切换**：支持直接切换和交互式选择，自动处理未提交更改，分支不存在时自动询问是否创建
3. **安全重命名**：提供完整的交互式流程，支持本地和远程分支重命名，包含多重验证和确认机制
4. **智能清理**：自动识别已合并分支，安全删除
5. **灵活配置**：支持按仓库配置忽略列表和分支前缀
6. **安全机制**：预览、确认、分类处理，确保操作安全
7. **分支前缀管理**：支持为不同仓库配置不同的分支前缀，自动应用到分支名生成

**设计优势**：
- ✅ **智能创建**：LLM 自动生成分支名，支持 JIRA 集成
- ✅ **快速切换**：支持直接切换和交互式选择，智能搜索（自动启用 fuzzy filter），自动处理未提交更改
- ✅ **安全重命名**：完全交互式流程，多重验证和确认，支持本地和远程分支重命名
- ✅ **安全性**：多重确认机制，防止误删重要分支
- ✅ **智能性**：自动识别已合并分支，分类处理
- ✅ **灵活性**：支持按仓库配置忽略列表和分支前缀
- ✅ **用户友好**：清晰的预览和确认提示，首次使用自动提示配置
- ✅ **仓库级别配置**：不同仓库可以配置不同的分支前缀，满足不同项目的命名规范

---

**最后更新**: 2025-12-16
