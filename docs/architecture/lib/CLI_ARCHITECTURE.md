# CLI 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 CLI 模块架构，包括：
- CLI 命令结构的定义和管理
- 命令行参数解析（使用 clap）
- 与补全脚本生成器的集成
- 命令结构的层次设计

CLI 模块是 Workflow CLI 的基础设施模块，定义了整个 CLI 的命令结构，为 `bin/workflow.rs` 和补全脚本生成器提供统一的命令结构定义。

**模块统计：**
- 总代码行数：约 526 行
- 文件数量：1 个核心文件
- 主要组件：1 个主结构体（`Cli`）+ 多个命令枚举（`Commands`、各种 `Subcommand`）
- 支持命令：15+ 个顶级命令，50+ 个子命令

---

## 📁 模块结构

### 核心模块文件

```
src/lib/cli/
└── mod.rs        # CLI 命令结构定义（~526 行）
```

### 依赖模块

- **`clap`**：命令行参数解析库
  - `Parser` - 主命令结构体派生宏
  - `Subcommand` - 子命令枚举派生宏
  - `CommandFactory` - 命令工厂 trait（用于生成补全脚本）

### 模块集成

#### 主入口集成

**位置**：`src/bin/workflow.rs`

**使用方式**：
```rust
use workflow::cli::Cli;

let cli = Cli::parse();
match cli.command {
    Some(Commands::Llm { subcommand }) => {
        // 处理 LLM 命令
    }
    // ...
}
```

#### 补全脚本生成器集成

**位置**：`src/lib/completion/generate.rs`

**使用方式**：
```rust
use workflow::cli::get_cli_command;

let command = get_cli_command();
clap_complete::generate(shell, &mut command, "workflow", &mut output);
```

---

## 🏗️ 架构设计

### 设计原则

1. **单一来源**：命令结构定义在一个地方，确保补全脚本与实际命令结构同步
2. **层次化设计**：使用主命令 → 子命令的层次结构
3. **类型安全**：使用 Rust 枚举和结构体确保类型安全
4. **易于扩展**：添加新命令只需在枚举中添加新变体

### 核心组件

#### 1. Cli 结构体（主命令结构）

**职责**：定义 CLI 主入口，包含所有顶级命令

**定义**：
```rust
#[derive(Parser)]
#[command(name = "workflow")]
#[command(about = "Workflow CLI tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
```

**关键特性**：
- 使用 `clap::Parser` 派生宏自动生成参数解析
- 支持子命令模式（`subcommand`）
- 命令为可选（`Option<Commands>`），允许不带参数运行

**使用场景**：
- `bin/workflow.rs` 中使用 `Cli::parse()` 解析命令行参数
- 补全脚本生成器使用 `Cli::command()` 生成补全脚本

#### 2. Commands 枚举（顶级命令）

**职责**：定义所有顶级命令

**定义**：
```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Manage proxy settings (on/off/check)
    Proxy {
        #[command(subcommand)]
        subcommand: ProxySubcommand,
        /// Temporary mode
        #[arg(short, long)]
        temporary: bool,
    },
    /// Run environment checks
    Check,
    /// Initialize or update configuration
    Setup,
    // ... 其他命令
}
```

**关键特性**：
- 使用 `clap::Subcommand` 派生宏自动生成子命令解析
- 每个命令变体可以包含参数和子命令
- 文档注释（`///`）自动成为命令帮助信息

**命令列表**：
- `Proxy` - 代理管理（包含子命令）
- `Check` - 环境检查
- `Setup` - 配置初始化
- `Config` - 配置查看
- `Uninstall` - 卸载
- `Version` - 版本信息
- `Update` - 更新
- `LogLevel` - 日志级别管理（包含子命令）
- `GitHub` - GitHub 账号管理（包含子命令）
- `Llm` - LLM 配置管理（包含子命令）
- `Completion` - 补全脚本管理（包含子命令）
- `Branch` - 分支管理（包含子命令）
- `Pr` - PR 操作（包含子命令）
- `Log` - 日志操作（包含子命令）
- `Jira` - Jira 操作（包含子命令）

#### 3. 子命令枚举（Subcommands）

**职责**：定义各个顶级命令的子命令

**示例**：
```rust
/// Proxy management subcommands
#[derive(Subcommand)]
pub enum ProxySubcommand {
    /// Enable proxy (set environment variables)
    On,
    /// Disable proxy (clear environment variables)
    Off,
    /// Check proxy status and configuration
    Check,
}

/// LLM configuration management subcommands
#[derive(Subcommand)]
pub enum LLMSubcommand {
    /// Show current LLM configuration
    Show,
    /// Setup LLM configuration
    Setup,
}
```

**关键特性**：
- 每个顶级命令可以有独立的子命令枚举
- 子命令枚举也使用 `clap::Subcommand` 派生宏
- 文档注释自动成为子命令帮助信息

**子命令枚举列表**：
- `ProxySubcommand` - 代理管理子命令（On, Off, Check）
- `LogLevelSubcommand` - 日志级别管理子命令（Set, Check）
- `GitHubSubcommand` - GitHub 账号管理子命令（List, Current, Add, Remove, Switch, Update）
- `LLMSubcommand` - LLM 配置管理子命令（Show, Setup）
- `CompletionSubcommand` - 补全脚本管理子命令（Generate, Check, Remove）
- `BranchSubcommand` - 分支管理子命令（Clean, Ignore）
- `IgnoreSubcommand` - 分支忽略列表管理子命令（Add, Remove, List）
- `PRCommands` - PR 操作子命令（Create, Merge, Close, Status, List, Update, Sync, Rebase, Pick, Summarize, Approve, Comment）
- `LogSubcommand` - 日志操作子命令（Download, Find, Search）
- `JiraSubcommand` - Jira 操作子命令（Info, Attachments, Clean）

#### 4. get_cli_command 函数

**职责**：获取 CLI 命令结构，用于生成补全脚本

**定义**：
```rust
/// 获取 CLI 命令结构（用于生成补全脚本）
///
/// 返回 `Cli` 结构体的 `Command` 实例，用于自动生成 shell completion 脚本。
/// 这样可以确保补全脚本与实际命令结构保持同步。
pub fn get_cli_command() -> clap::Command {
    Cli::command()
}
```

**关键特性**：
- 使用 `CommandFactory` trait 的 `command()` 方法生成命令结构
- 返回 `clap::Command` 实例，可直接用于补全脚本生成
- 确保补全脚本与实际命令结构同步

**使用场景**：
- 补全脚本生成器使用此函数获取命令结构
- 自动生成各种 shell 的补全脚本（zsh, bash, fish, powershell, elvish）

### 设计模式

#### 1. 命令模式（Command Pattern）

**模式说明**：
使用枚举定义命令结构，每个命令变体代表一个具体的命令。

**优势**：
- ✅ 类型安全：编译时检查命令结构
- ✅ 易于扩展：添加新命令只需在枚举中添加新变体
- ✅ 自动生成帮助信息：文档注释自动成为帮助信息

**实现**：
```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Command description
    CommandName {
        /// Argument description
        #[arg(short, long)]
        arg: Option<String>,
    },
}
```

#### 2. 层次化命令结构

**模式说明**：
使用主命令 → 子命令的层次结构，支持多级命令嵌套。

**优势**：
- ✅ 清晰的命令组织：相关命令组织在一起
- ✅ 易于理解：命令结构直观
- ✅ 支持复杂命令：支持参数、选项、子命令

**实现**：
```rust
Commands::Pr {
    subcommand: PRCommands::Create {
        jira_ticket: Option<String>,
        title: Option<String>,
        // ...
    }
}
```

#### 3. 单一来源原则（Single Source of Truth）

**模式说明**：
命令结构定义在一个地方，确保补全脚本与实际命令结构同步。

**优势**：
- ✅ 避免不一致：补全脚本自动与实际命令结构同步
- ✅ 易于维护：只需在一个地方修改命令结构
- ✅ 类型安全：编译时检查命令结构

**实现**：
```rust
// 定义命令结构
pub struct Cli { ... }

// 主入口使用
let cli = Cli::parse();

// 补全脚本生成器使用
let command = Cli::command();
```

---

## 🔄 调用流程与数据流

### 整体架构流程

```
用户输入命令
  ↓
bin/workflow.rs::main()
  ↓
Cli::parse() (clap 解析)
  ↓
match cli.command
  ↓
  ├─ Commands::Proxy { subcommand } → match subcommand
  ├─ Commands::Llm { subcommand } → match subcommand
  └─ ... 其他命令
```

### 命令解析流程

```
命令行输入: "workflow llm setup"
  ↓
Cli::parse()
  ↓
clap 解析
  ↓
Commands::Llm {
    subcommand: LLMSubcommand::Setup
}
  ↓
bin/workflow.rs::match Commands::Llm
  ↓
LLMSetupCommand::setup()
```

### 补全脚本生成流程

```
补全脚本生成器调用
  ↓
get_cli_command()
  ↓
Cli::command() (CommandFactory trait)
  ↓
clap 生成命令结构
  ↓
clap_complete::generate()
  ↓
生成补全脚本文件
```

### 数据流

#### 命令执行数据流

```
用户输入
  ↓
CLI 模块（命令结构定义）
  ↓
bin/workflow.rs（命令分发）
  ↓
commands/*（命令实现）
  ↓
lib/*（业务逻辑）
```

#### 补全脚本生成数据流

```
补全脚本生成器
  ↓
CLI 模块（get_cli_command）
  ↓
clap_complete::generate()
  ↓
补全脚本文件
```

---

## 📋 使用示例

### 基本使用（在 bin/workflow.rs 中）

```rust
use workflow::cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Llm { subcommand }) => match subcommand {
            LLMSubcommand::Show => LLMShowCommand::show()?,
            LLMSubcommand::Setup => LLMSetupCommand::setup()?,
        },
        Some(Commands::Pr { subcommand }) => match subcommand {
            PRCommands::Create { jira_ticket, title, .. } => {
                // 处理 PR 创建命令
            },
            // ... 其他 PR 子命令
        },
        None => {
            // 显示帮助信息
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
```

### 补全脚本生成（在 completion/generate.rs 中）

```rust
use workflow::cli::get_cli_command;
use clap_complete::{generate, shells::Shell};

fn generate_completion(shell: Shell) -> Result<()> {
    let mut command = get_cli_command();
    let mut output = Vec::new();

    generate(shell, &mut command, "workflow", &mut output);

    // 保存补全脚本到文件
    std::fs::write("completion_script", output)?;

    Ok(())
}
```

### 添加新命令

#### 步骤 1：在 Commands 枚举中添加新命令

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令

    /// New command description
    NewCommand {
        /// Argument description
        #[arg(short, long)]
        arg: Option<String>,

        #[command(subcommand)]
        subcommand: Option<NewCommandSubcommand>,
    },
}
```

#### 步骤 2：定义子命令枚举（如果需要）

```rust
#[derive(Subcommand)]
pub enum NewCommandSubcommand {
    /// Subcommand description
    SubCommand {
        /// Subcommand argument
        value: String,
    },
}
```

#### 步骤 3：在 bin/workflow.rs 中添加命令处理

```rust
match cli.command {
    // ... 现有命令

    Some(Commands::NewCommand { arg, subcommand }) => {
        match subcommand {
            Some(NewCommandSubcommand::SubCommand { value }) => {
                // 处理子命令
            },
            None => {
                // 处理主命令
            }
        }
    },
}
```

---

## 📝 扩展性

### 添加新命令

1. 在 `Commands` 枚举中添加新命令变体
2. 如果需要子命令，创建新的子命令枚举
3. 在 `bin/workflow.rs` 中添加命令处理逻辑
4. 补全脚本会自动更新（无需手动修改）

### 添加命令参数

1. 在命令变体中添加字段
2. 使用 `#[arg(...)]` 属性配置参数
3. 在命令处理逻辑中使用参数

**示例**：
```rust
Commands::NewCommand {
    /// Required argument
    value: String,

    /// Optional argument with short and long flags
    #[arg(short, long)]
    option: Option<String>,

    /// Flag (boolean)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    flag: bool,
}
```

### 修改命令帮助信息

1. 修改命令或参数的文档注释（`///`）
2. clap 会自动将文档注释转换为帮助信息

**示例**：
```rust
/// This is the command description
///
/// This will appear in the help text.
Commands::NewCommand {
    /// This is the argument description
    value: String,
}
```

---

## 🔍 命令结构维护

### 命令结构同步机制

**问题**：如何确保补全脚本与实际命令结构同步？

**解决方案**：
1. **单一来源**：命令结构定义在 `lib/cli/mod.rs` 中
2. **自动生成**：补全脚本通过 `get_cli_command()` 自动生成
3. **编译时检查**：Rust 类型系统确保命令结构正确

**优势**：
- ✅ 无需手动维护补全脚本
- ✅ 命令结构变更自动反映到补全脚本
- ✅ 编译时检查，避免运行时错误

### 命令结构验证

**验证方式**：
1. **编译时验证**：Rust 编译器检查类型和语法
2. **运行时验证**：clap 验证参数和选项
3. **测试验证**：单元测试验证命令结构

**示例测试**：
```rust
#[test]
fn test_cli_parsing() {
    let cli = Cli::try_parse_from(["workflow", "llm", "setup"]).unwrap();
    assert!(matches!(cli.command, Some(Commands::Llm { .. })));
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [补全脚本管理架构文档](./COMPLETION_ARCHITECTURE.md) - 补全脚本生成器
- [LLM 命令架构文档](../commands/LLM_COMMAND_ARCHITECTURE.md) - LLM 命令实现
- [PR 命令架构文档](../commands/PR_COMMAND_ARCHITECTURE.md) - PR 命令实现

---

## ✅ 总结

CLI 模块采用清晰的层次化命令结构设计：

1. **单一来源**：命令结构定义在一个地方，确保补全脚本与实际命令结构同步
2. **类型安全**：使用 Rust 枚举和结构体确保类型安全
3. **易于扩展**：添加新命令只需在枚举中添加新变体

**设计优势**：
- ✅ **类型安全**：编译时检查命令结构
- ✅ **自动同步**：补全脚本自动与实际命令结构同步
- ✅ **易于维护**：只需在一个地方修改命令结构
- ✅ **用户友好**：自动生成帮助信息和补全脚本

**当前实现状态**：
- ✅ 15+ 个顶级命令
- ✅ 50+ 个子命令
- ✅ 支持参数、选项、子命令
- ✅ 自动生成帮助信息
- ✅ 自动生成补全脚本（zsh, bash, fish, powershell, elvish）

**技术栈**：
- **clap**：命令行参数解析库
- **Rust 枚举**：命令结构定义
- **派生宏**：自动生成参数解析代码
