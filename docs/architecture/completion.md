# Shell Completion 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Shell Completion 模块架构，包括 completion 脚本的生成、安装、配置和卸载功能。该模块为 `workflow` 命令及其所有子命令提供 shell 自动补全支持。

**模块统计：**
- 总代码行数：约 500 行
- 文件数量：4 个核心文件
- 主要组件：3 个（Completion, Generate, Files）
- 支持的命令：workflow（包含所有子命令：pr, log, jira, branch, github, llm 等）
- 支持的 Shell：zsh, bash, fish, powershell, elvish

---

## 📁 模块结构

### 核心模块文件

```
src/lib/completion/
├── mod.rs                  # 模块声明和导出
├── completion.rs           # Completion 管理工具（配置、安装、卸载）
├── generate.rs             # Completion 脚本生成器
└── files.rs                # Completion 文件工具函数（文件名、列表）
```

### 依赖模块

- **`lib/base/settings/paths.rs`**：路径管理（`Paths::completion_dir()`）
- **`lib/base/shell/config.rs`**：Shell 配置文件管理（`ShellConfigManager`）
- **`lib/base/shell/detect.rs`**：Shell 检测（`Detect::shell()`）
- **`clap_complete`**：Completion 脚本生成库

### 模块集成

#### Shell 配置管理

- **`lib/base/shell/config.rs`**：`ShellConfigManager`
  - 添加 source 语句到 shell 配置文件
  - 从 shell 配置文件移除 source 语句

#### 路径管理

- **`lib/base/settings/paths.rs`**：`Paths`
  - `completion_dir()` - 获取 completion 目录路径

#### Shell 检测

- **`lib/base/shell/detect.rs`**：`Detect`
  - `shell()` - 检测当前 shell 类型

#### 回滚模块

- **`lib/rollback/rollback.rs`**：`RollbackManager`
  - 备份 completion 脚本文件
  - 恢复 completion 脚本文件

---

## 🏗️ 架构设计

### 设计原则

模块采用职责分离的设计模式，每个组件负责单一职责。

### 核心组件

#### 1. Completion（结构体）

- **职责**：Shell Completion 的配置和管理
- **功能**：
  - 配置 shell 配置文件以启用 completion
  - 创建 completion 配置文件（`.completions`）
  - 删除 completion 配置和文件
  - 获取 completion 文件列表
  - 生成所有 completion 脚本（委托给 `generate` 模块）

#### 2. Generate（函数模块）

- **职责**：生成各种 shell 的 completion 脚本文件
- **功能**：
  - 生成 `workflow` 命令的 completion（包含所有子命令）
  - 生成 `workflow` 命令及其所有子命令的 completion（包括 `pr`、`log`、`jira`、`llm` 等）
  - 支持多种 shell 类型（zsh, bash, fish, powershell, elvish）

#### 3. Files（函数模块）

- **职责**：Completion 文件命名和列表相关的工具函数
- **功能**：
  - 根据 shell 类型和命令名生成补全脚本文件名
  - 获取指定 shell 类型的所有补全脚本文件名
  - 获取所有 shell 类型的所有补全脚本文件名

### 设计模式

#### 1. 单一职责原则（SRP）

每个组件只负责一个明确的功能：
- `Completion`：只负责配置和管理
- `Generate`：只负责生成脚本
- `Files`：只负责文件命名和列表

#### 2. 委托模式

`Completion` 将具体的生成逻辑委托给 `Generate` 模块，保持接口简洁。

#### 3. 工具函数模式

`Files` 模块提供纯函数工具，无副作用，易于测试和复用。

### 错误处理

#### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **功能层**：文件操作错误、配置读写错误、shell 检测错误

#### 容错机制

- **Shell 检测失败**：提示用户手动指定 shell 类型
- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **配置写入失败**：保留原有配置，提示用户手动配置

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
Completion (管理层)
  ↓
Generate / Files / ShellConfigManager (功能层)
```

### 安装 Completion 流程

```
Completion::configure_shell_config(shell)
  ↓
  1. Detect::shell()                                    # 检测 shell 类型
  2. Paths::completion_dir()                           # 获取 completion 目录
  3. fs::create_dir_all()                              # 创建 completion 目录
  4. Completion::generate_all_completions()             # 生成 completion 脚本
     ├─ generate::generate_all_completions()
     │   └─ generate_workflow_completion()            # 生成 workflow completion（包含所有子命令：pr, log, jira, llm, github 等）
     └─ files::get_completion_filename()              # 获取文件名
  5. Completion::create_completion_config_file()       # 创建 .completions 配置文件
  6. ShellConfigManager::add_source()                  # 添加 source 语句到 shell 配置
```

**设计说明**：
- 配置文件 `~/.workflow/.completions` 同时支持 zsh 和 bash
- 配置文件在运行时检测当前 shell 类型（通过 `$ZSH_VERSION` 和 `$BASH_VERSION`）
- 安装时会同时生成 zsh 和 bash 的补全脚本，确保用户切换 shell 时补全功能仍然可用

### 卸载 Completion 流程

```
Completion::remove_completion_config(shell)
  ↓
  1. Detect::shell()                                    # 检测 shell 类型
  2. Completion::remove_completion_files()             # 删除 completion 脚本文件
     └─ files::get_all_completion_files()              # 获取所有 shell 类型的文件列表
  3. Completion::remove_completion_config_file()       # 删除 .completions 配置文件
  4. ShellConfigManager::remove_source()               # 从 shell 配置文件移除 source 语句
```

### 数据流

#### Completion 安装数据流

```
clap::Command (命令定义)
  ↓
clap_complete::generate() (生成 completion 脚本)
  ↓
Completion 脚本文件（_workflow 或 workflow.bash）
  ↓
~/.workflow/completions/ 目录
  ↓
~/.workflow/.completions 配置文件（source 语句）
  ↓
Shell 配置文件 (~/.zshrc, ~/.bash_profile) (source ~/.workflow/.completions)
  ↓
Shell 环境（启用 completion）
```

#### Completion 文件命名规则

| Shell 类型 | 文件命名规则 | 示例 |
|-----------|------------|------|

| zsh | `_{command}` | `_workflow` |
| bash | `{command}.bash` | `workflow.bash` |
| fish | `{command}.fish` | `workflow.fish` |
| powershell | `_{command}.ps1` | `_workflow.ps1` |
| elvish | `{command}.elv` | `workflow.elv` |

---

## 📝 扩展性

### 添加新命令的 Completion

1. 在 `generate.rs` 中添加新的生成函数（如 `generate_new_command_completion()`）
2. 在 `generate_all_completions()` 中调用新函数
3. 在 `files.rs` 的 `get_all_completion_files()` 中添加新命令名
4. 更新 `completion.rs` 中的命令列表（如 `get_completion_files()`）

**示例**：
```rust
// generate.rs
pub fn generate_new_command_completion(shell: &ClapShell, output_dir: &Path) -> Result<()> {
    let mut cmd = Command::new("new_command")
        .about("New command description")
        .subcommand(/* ... */);

    let mut buffer = Vec::new();
    generate(*shell, &mut cmd, "new_command", &mut buffer);

    let filename = get_completion_filename(&shell.to_string(), "new_command")?;
    let output_file = output_dir.join(&filename);
    fs::write(&output_file, buffer)?;
    Ok(())
}
```

### 添加新 Shell 支持

1. 在 `files.rs` 的 `get_completion_filename()` 中添加新 shell 类型的命名规则
2. 在 `generate.rs` 的 `generate_all_completions()` 中添加 shell 类型解析
3. 在 `completion.rs` 的 `create_completion_config_file()` 中添加新 shell 的配置逻辑

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [生命周期管理命令模块架构文档](../commands/LIFECYCLE_COMMAND_ARCHITECTURE.md)
- [回滚模块架构文档](./ROLLBACK_ARCHITECTURE.md)

---

## 📋 使用示例

### 基本使用

```rust
use workflow::completion::Completion;

// 配置 shell completion
Completion::configure_shell_config(&shell)?;

// 移除 completion 配置
Completion::remove_completion_config(&shell)?;
```

---

## ✅ 总结

Shell Completion 模块采用清晰的职责分离设计：

1. **单一职责**：每个组件只负责单一功能
2. **委托模式**：Completion 委托给 Generate 模块生成脚本
3. **工具函数**：Files 模块提供纯函数工具

**设计优势**：
- ✅ **易于扩展**：添加新命令只需扩展生成函数
- ✅ **类型安全**：使用 clap_complete::Shell 枚举类型
- ✅ **代码复用**：Files 模块提供通用工具函数
- ✅ **多 Shell 支持**：支持 zsh、bash、fish、powershell、elvish

---

**最后更新**: 2025-12-16
