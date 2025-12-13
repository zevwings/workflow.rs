# 别名系统实现文档

## 📋 概述

本文档描述别名系统的完整实现方案，包括别名配置、别名展开和别名管理命令。

**状态**: 📋 待实现
**分类**: 用户体验优化
**优先级**: 中优先级
**依赖**: 配置文件管理系统（已实现）、CLI 命令解析系统（已实现）

---

## 🎯 需求目标

实现命令别名系统，以：
1. 简化常用命令输入，提高命令输入效率
2. 支持自定义别名，满足个人使用习惯
3. 支持别名嵌套和参数传递，提供灵活的扩展能力

---

## 📝 功能需求

### 1. 别名配置

#### 1.1 功能描述
支持在配置文件中定义别名，将简短别名映射到完整命令。

#### 1.2 配置格式
```toml
[aliases]
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"

# 支持嵌套别名
prc = "ci"  # prc -> ci -> pr create
```

#### 1.3 使用示例
```bash
workflow ci                                        # 等同于 workflow pr create
workflow cm                                        # 等同于 workflow pr merge
workflow js "project = PROJ"                       # 等同于 workflow jira search "project = PROJ"
workflow ji PROJ-123                               # 等同于 workflow jira info PROJ-123
```

### 2. 别名展开

#### 2.1 功能描述
在主入口处自动展开别名，将别名替换为完整命令。

#### 2.2 展开逻辑
1. **基本展开**：
   - 检查第一个参数是否是别名
   - 如果是，替换为别名值
   - 保留剩余参数

2. **嵌套别名处理**：
   - 使用 `HashSet` 跟踪已访问的别名（防止循环）
   - 递归展开嵌套别名
   - 最多展开深度限制（默认 10 层）

3. **参数传递**：
   - 别名展开后，将原始命令的剩余参数追加到展开后的命令
   - 例如：`workflow ci --title "test"` → `workflow pr create --title "test"`

### 3. 别名管理命令

#### 3.1 `alias list` - 列出所有别名
```bash
workflow alias list
```

**功能描述**：列出所有已定义的别名，使用表格格式显示。

**输出示例（表格格式）**：
```
┌─────────────────────────────────────────┐
│           Defined Aliases               │
├─────────────┬──────────────────────────┤
│ Alias Name  │ Command                  │
├─────────────┼──────────────────────────┤
│ ci          │ pr create                │
│ cm          │ pr merge                 │
│ js          │ jira search              │
│ ji          │ jira info                │
└─────────────┴──────────────────────────┘
```

**功能要求**：
- 使用表格格式显示别名列表（使用 `TableBuilder`）
- 表格包含两列：`Alias Name` 和 `Command`
- 如果没有别名，显示友好提示信息
- 使用 `TableStyle::Modern` 样式

#### 3.2 `alias add` - 添加别名
```bash
# 方式1：直接指定别名和命令
workflow alias add ci "pr create"
workflow alias add cm "pr merge"

# 方式2：交互式添加（不提供参数时）
workflow alias add
```

**交互式添加示例**：
```bash
$ workflow alias add

# 步骤1：输入别名名称
Enter alias name: ci

# 如果别名已存在，询问是否覆盖
Alias 'ci' already exists. Overwrite? (y/N)

# 步骤2：选择命令输入方式
How do you want to enter the command?
  > Select from common commands
    Enter manually

[↑↓: Move, Enter: Select, Esc: Cancel]

# 如果选择"从常用命令列表选择"
Select a command:
  > pr create
    pr merge
    pr status
    pr list
    jira info
    jira search
    branch create
    branch switch
    branch clean
```

**常用命令列表维护方式**：

推荐使用**方式4（混合方式）**：动态生成 + 配置文件优先级，既保证自动同步，又允许用户自定义。

#### 方式1：动态生成（推荐作为基础）

**实现方式**：
- 从 `Cli::command()` 获取命令结构
- 遍历所有顶级命令和子命令，生成完整命令列表
- 格式：`command subcommand`（如 `pr create`）

**优点**：
- ✅ 自动同步，无需手动维护
- ✅ 不会遗漏新命令
- ✅ 与 CLI 结构保持一致

**缺点**：
- ❌ 列表可能较长（50+ 个命令）
- ❌ 需要过滤或排序常用命令
- ❌ 用户体验可能不够友好（选项太多）

**适用场景**：作为基础数据源，配合其他方式使用

#### 方式2：硬编码常用命令（简单直接）

**实现方式**：
- 在代码中维护一个常用命令列表常量
- 只包含最常用的命令（10-15 个）

**优点**：
- ✅ 简单直接，列表精简
- ✅ 用户体验好（选项少，易于选择）
- ✅ 性能好（无需动态生成）

**缺点**：
- ❌ 需要手动更新
- ❌ 可能遗漏新命令
- ❌ 不同用户可能有不同的常用命令

**适用场景**：快速实现，或作为默认常用命令列表

**示例代码**：
```rust
const COMMON_COMMANDS: &[&str] = &[
    "pr create",
    "pr merge",
    "pr status",
    "pr list",
    "jira info",
    "jira search",
    "branch create",
    "branch switch",
    "branch clean",
];
```

#### 方式3：配置文件（灵活可配置）

**实现方式**：
- **选项A**：在 `workflow.toml` 中添加 `[aliases.common_commands]` 配置项
- **选项B（推荐）**：使用单独的 `commands.toml` 文件存储常用命令列表
- 用户可以自定义常用命令列表
- 默认值：使用硬编码的常用命令列表

**优点**：
- ✅ 灵活可配置，用户可自定义
- ✅ 不同用户可以有不同的常用命令
- ✅ 可以按使用频率排序
- ✅ **选项B**：职责分离，符合项目现有模式（类似 `llm.toml`、`jira-status.toml`）
- ✅ **选项B**：更易维护，常用命令列表可能经常更新

**缺点**：
- ❌ 需要额外的配置管理
- ❌ 新用户需要手动配置
- ❌ 配置可能过时
- ❌ **选项B**：增加文件数量（多一个配置文件）

**适用场景**：需要用户自定义的场景

**配置示例（选项A - workflow.toml）**：
```toml
[aliases]
common_commands = [
    "pr create",
    "pr merge",
    "jira info",
    "branch create",
]
```

**配置示例（选项B - commands.toml，推荐）**：
```toml
# ~/.workflow/config/commands.toml
common_commands = [
    "pr create",
    "pr merge",
    "pr status",
    "pr list",
    "jira info",
    "jira search",
    "branch create",
    "branch switch",
    "branch clean",
]
```

**推荐使用选项B（单独的 `commands.toml` 文件）**，原因：
1. **符合项目现有模式**：项目已经有分离配置文件的先例（`llm.toml`、`jira-status.toml`、`jira-users.toml`）
2. **职责分离**：别名定义（`workflow.toml`）和常用命令列表（`commands.toml`）分开管理
3. **更灵活**：用户可以单独管理常用命令列表，不影响主配置文件
4. **更易维护**：常用命令列表可能经常更新，单独文件更容易管理

#### 方式4：混合方式（推荐）

**实现方式**：
- **默认**：使用硬编码的常用命令列表（方式2）
- **可选**：如果配置文件中定义了常用命令，使用配置文件中的列表
- **备选**：如果用户选择"显示所有命令"，使用动态生成（方式1）

**优点**：
- ✅ 兼顾自动同步和用户体验
- ✅ 默认列表精简，用户体验好
- ✅ 支持用户自定义
- ✅ 支持查看所有命令

**缺点**：
- ❌ 实现稍复杂
- ❌ 需要处理多种数据源

**适用场景**：生产环境推荐方案

**实现示例**：
```rust
fn get_common_commands() -> Result<Vec<String>> {
    use crate::base::settings::paths::Paths;

    // 1. 优先从 commands.toml 配置文件读取
    let commands_config_path = Paths::commands_config()?;
    if commands_config_path.exists() {
        if let Ok(content) = fs::read_to_string(&commands_config_path) {
            if let Ok(config) = toml::from_str::<CommandsConfig>(&content) {
                if !config.common_commands.is_empty() {
                    return Ok(config.common_commands);
                }
            }
        }
    }

    // 2. 使用硬编码的默认常用命令列表
    Ok(COMMON_COMMANDS.iter().map(|s| s.to_string()).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandsConfig {
    #[serde(default)]
    pub common_commands: Vec<String>,
}

fn get_all_commands() -> Result<Vec<String>> {
    // 动态生成所有命令
    Self::get_available_commands()
}
```

**推荐方案**：
- **第一阶段**：使用方式2（硬编码常用命令），快速实现
- **第二阶段**：升级为方式4（混合方式），支持用户自定义和查看所有命令

[↑↓: Move, Enter: Select, Esc: Cancel]

# 如果选择"手动输入"
Enter command: pr create --title "test"

# 保存成功
Alias 'ci' = 'pr create' added successfully

# 询问是否更新补全脚本
Update completion scripts? (Y/n)
```

功能要求：
- **交互式添加**：
  - 如果没有提供参数，进入交互式模式
  - 步骤1：输入别名名称（使用 `InputDialog`）
    - 验证别名名称格式（不能为空，不能包含空格）
    - 检查别名是否已存在，如果存在提示用户并询问是否覆盖
  - 步骤2：输入或选择命令（两种方式）
    - 方式A：从常用命令列表中选择（使用 `SelectDialog`）
      - 显示常用命令列表（如：`pr create`, `pr merge`, `jira info`, `jira search` 等）
      - 用户可以选择一个常用命令
    - 方式B：手动输入命令（使用 `InputDialog`）
      - 提供选项让用户选择"手动输入"或"从列表选择"
      - 如果选择手动输入，允许用户输入任意命令
- **直接添加**：
  - 如果提供了别名和命令参数，直接添加
  - 检查别名是否已存在
  - 如果已存在，提示用户并询问是否覆盖
- **保存配置**：
  - 保存别名到配置文件
  - 显示成功消息
- **补全脚本更新确认**：添加别名后，询问用户是否更新补全脚本
  - 如果用户选择 `y`，自动调用 `workflow completion generate` 更新补全脚本
  - 如果用户选择 `n`，跳过补全脚本更新
  - 默认值：`y`（推荐更新）

#### 3.3 `alias remove` - 删除别名
```bash
# 方式1：直接指定别名名称
workflow alias remove ci

# 方式2：交互式选择（不提供参数时）
workflow alias remove
```

**交互式删除示例**：
```bash
$ workflow alias remove

# 显示别名列表（多选对话框）
Select aliases to remove:
  > [✓] ci = pr create
    [ ] cm = pr merge
    [✓] js = jira search
    [ ] ji = jira info

[↑↓: Move, Space: Toggle, Enter: Confirm, Esc: Cancel]

# 用户选择后，显示确认信息
Aliases to be removed:
  ci = pr create
  js = jira search

Are you sure you want to remove 2 alias/aliases? (y/N)
# 用户确认后删除，然后询问是否更新补全脚本
Update completion scripts? (Y/n)
```

功能要求：
- **交互式选择**：
  - 如果没有提供别名参数，显示所有已定义的别名列表
  - 使用 `MultiSelectDialog` 支持多选删除
  - 显示格式：`alias_name = command`（例如：`ci = pr create`）
  - 用户可以选择一个或多个别名进行删除
- **直接删除**：
  - 如果提供了别名参数，直接删除指定的别名
  - 检查别名是否存在
  - 如果不存在，提示用户
- **确认删除**：
  - 删除前显示将要删除的别名列表
  - 使用 `ConfirmDialog` 确认删除操作
  - 默认值：`false`（需要用户明确确认）
- **批量删除**：
  - 支持一次删除多个别名
  - 显示删除进度和结果
- **补全脚本更新确认**：删除别名后，询问用户是否更新补全脚本
  - 如果用户选择 `y`，自动调用 `workflow completion generate` 更新补全脚本
  - 如果用户选择 `n`，跳过补全脚本更新
  - 默认值：`y`（推荐更新）

---

## 🔧 技术设计

### 架构设计

别名系统采用三层架构：

```
CLI 入口层 (bin/workflow.rs)
  ↓ 别名展开
命令封装层 (commands/alias/)
  ↓ 调用
核心业务逻辑层 (lib/base/alias/)
  ↓ 使用
配置管理层 (lib/base/settings/)
```

### 核心模块结构

```
src/lib/base/alias/
├── mod.rs              # 模块声明和导出
├── manager.rs         # AliasManager 实现（别名加载、展开、管理）
└── commands_config.rs # CommandsConfig 实现（常用命令列表配置）

src/commands/alias/
├── mod.rs          # 命令模块声明
├── list.rs         # alias list 命令实现（表格显示）
├── add.rs          # alias add 命令实现
└── remove.rs       # alias remove 命令实现

src/lib/cli/
└── alias.rs        # AliasSubcommand 枚举定义
```

### 表格显示结构

```
src/commands/alias/list.rs
├── AliasRow        # 别名表格行结构体（实现 Tabled trait）
│   ├── alias_name  # 别名名称列
│   └── command     # 命令列
└── list()          # 使用 TableBuilder 显示表格
```

### 核心数据结构

#### 1. AliasConfig（配置结构体）

```rust
// src/lib/base/settings/settings.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // ... 其他配置
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}
```

#### 2. AliasManager（别名管理器）

```rust
// src/lib/base/alias/manager.rs
pub struct AliasManager;

impl AliasManager {
    /// 加载别名配置
    pub fn load() -> Result<HashMap<String, String>>;

    /// 展开别名（支持嵌套）
    pub fn expand(alias: &str, visited: &mut HashSet<String>) -> Result<String>;

    /// 展开命令行参数（处理第一个参数是否为别名）
    pub fn expand_args(args: Vec<String>) -> Result<Vec<String>>;

    /// 添加别名
    pub fn add(name: &str, command: &str) -> Result<()>;

    /// 删除别名
    pub fn remove(name: &str) -> Result<bool>;

    /// 列出所有别名
    pub fn list() -> Result<HashMap<String, String>>;

    /// 检查别名是否存在
    pub fn exists(name: &str) -> Result<bool>;

    /// 检查循环别名
    pub fn check_circular(name: &str, target: &str) -> Result<bool>;
}
```

### 别名展开算法

```rust
pub fn expand(alias: &str, visited: &mut HashSet<String>, depth: usize) -> Result<String> {
    const MAX_DEPTH: usize = 10;

    // 检查深度限制
    if depth > MAX_DEPTH {
        return Err(anyhow::anyhow!("Alias expansion depth exceeded maximum: {}", MAX_DEPTH));
    }

    // 检查循环引用
    if visited.contains(alias) {
        return Err(anyhow::anyhow!("Circular alias detected: {}", alias));
    }

    // 加载别名配置
    let aliases = Self::load()?;

    // 检查别名是否存在
    let command = aliases.get(alias)
        .ok_or_else(|| anyhow::anyhow!("Alias not found: {}", alias))?;

    // 标记为已访问
    visited.insert(alias.to_string());

    // 检查命令是否包含其他别名（递归展开）
    let parts: Vec<&str> = command.split_whitespace().collect();
    if let Some(first_part) = parts.first() {
        if aliases.contains_key(*first_part) {
            // 递归展开嵌套别名
            let expanded = Self::expand(first_part, visited, depth + 1)?;
            // 将展开后的命令与剩余部分组合
            let mut result = expanded.split_whitespace().collect::<Vec<_>>();
            result.extend_from_slice(&parts[1..]);
            return Ok(result.join(" "));
        }
    }

    Ok(command.clone())
}
```

### 主入口集成

在 `src/bin/workflow.rs` 中，在 `Cli::parse()` 之前进行别名展开：

```rust
fn main() -> Result<()> {
    // 安装 color-eyre（最早调用）
    color_eyre::install()?;

    // 初始化日志级别
    {
        let config_level = Settings::get()
            .log
            .level
            .as_ref()
            .and_then(|s| s.parse::<workflow::LogLevel>().ok());
        workflow::LogLevel::init(config_level);
    }

    // 初始化 tracing
    workflow::Tracer::init();

    // 别名展开：在解析前展开别名
    let args: Vec<String> = std::env::args().collect();
    let expanded_args = alias::AliasManager::expand_args(args)?;

    // 使用展开后的参数重新解析
    let cli = Cli::parse_from(expanded_args);

    // ... 后续命令分发逻辑
}
```

### 配置文件集成

#### 1. 别名配置（workflow.toml）

在 `src/lib/base/settings/settings.rs` 中添加别名配置：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfig {
    /// 别名映射表
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // ... 其他配置字段

    /// 别名配置
    #[serde(default)]
    pub aliases: AliasConfig,
}
```

#### 2. 常用命令配置（commands.toml，推荐）

**实现方式**：使用单独的 `commands.toml` 文件存储常用命令列表，类似于 `jira-status.toml` 和 `jira-users.toml` 的处理方式。

**文件位置**：`~/.workflow/config/commands.toml`

**配置结构**：
```rust
// src/lib/base/alias/commands_config.rs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandsConfig {
    /// 常用命令列表（用于交互式添加别名时的命令选择）
    #[serde(default)]
    pub common_commands: Vec<String>,
}
```

**路径管理**：在 `src/lib/base/settings/paths.rs` 中添加：
```rust
/// 获取常用命令配置文件路径
///
/// 返回 `~/.workflow/config/commands.toml` 的路径。
pub fn commands_config() -> Result<PathBuf> {
    Ok(Self::config_dir()?.join("commands.toml"))
}
```

**配置加载**：使用 `ConfigManager` 模式（参考 `jira/config.rs`）：
```rust
// src/lib/base/alias/commands_config.rs
use crate::base::settings::paths::Paths;
use crate::jira::config::ConfigManager;

pub type CommandsConfigManager = ConfigManager<CommandsConfig>;

impl CommandsConfig {
    /// 加载常用命令配置
    pub fn load() -> Result<Self> {
        let config_path = Paths::commands_config()?;
        let manager = CommandsConfigManager::new(config_path);
        Ok(manager.read())
    }
}
```

impl Default for AliasConfig {
    fn default() -> Self {
        Self {
            aliases: HashMap::new(),
            common_commands: None,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // ... 其他默认值
            aliases: AliasConfig::default(),
        }
    }
}
```

**配置文件示例**：

**workflow.toml**（别名定义）：
```toml
[aliases]
# 别名定义
ci = "pr create"
cm = "pr merge"
js = "jira search"
ji = "jira info"
```

**commands.toml**（常用命令列表，推荐）：
```toml
# ~/.workflow/config/commands.toml
# 常用命令列表（用于交互式添加别名时的命令选择）
common_commands = [
    "pr create",
    "pr merge",
    "pr status",
    "pr list",
    "jira info",
    "jira search",
    "branch create",
    "branch switch",
    "branch clean",
]
```

### CLI 命令结构

在 `src/lib/cli/alias.rs` 中定义别名子命令：

```rust
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum AliasSubcommand {
    /// 列出所有别名
    List,

    /// 添加别名（支持交互式添加）
    Add {
        /// 别名名称（可选，如果不提供则进入交互式模式）
        name: Option<String>,
        /// 别名对应的命令（可选，如果不提供则进入交互式模式）
        command: Option<String>,
    },

    /// 删除别名（支持交互式多选）
    Remove {
        /// 别名名称（可选，如果不提供则进入交互式选择模式）
        name: Option<String>,
    },
}
```

在 `src/lib/cli/commands.rs` 中添加别名命令：

```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... 其他命令

    /// 别名管理
    Alias {
        #[command(subcommand)]
        subcommand: AliasSubcommand,
    },
}
```

---

## 📋 实现步骤

### 第一阶段：核心功能实现

#### 1.1 创建别名管理模块

**文件**: `src/lib/base/alias/mod.rs`
```rust
mod manager;

pub use manager::AliasManager;
```

**文件**: `src/lib/base/alias/manager.rs`
- 实现 `AliasManager::load()` - 从配置加载别名
- 实现 `AliasManager::expand()` - 别名展开逻辑（支持嵌套）
- 实现 `AliasManager::expand_args()` - 命令行参数展开
- 实现循环检测和深度限制

#### 1.2 配置文件集成

**文件**: `src/lib/base/settings/settings.rs`
- 定义 `AliasConfig` 结构体，包含：
  - `aliases: HashMap<String, String>` - 别名映射表
- 在 `Settings` 结构体中添加 `aliases: AliasConfig` 字段
- 在 `Default` 实现中添加默认空别名配置
- 确保配置加载和保存时正确处理别名字段

**文件**: `src/lib/base/alias/commands_config.rs`（新建）
- 定义 `CommandsConfig` 结构体，包含：
  - `common_commands: Vec<String>` - 常用命令列表
- 实现 `CommandsConfig::load()` 方法，从 `commands.toml` 加载配置
- 使用 `ConfigManager` 模式（参考 `jira/config.rs`）

**文件**: `src/lib/base/settings/paths.rs`
- 添加 `commands_config()` 方法，返回 `~/.workflow/config/commands.toml` 路径

#### 1.3 主入口集成

**文件**: `src/bin/workflow.rs`
- 在 `main()` 函数中，在 `Cli::parse()` 之前添加别名展开逻辑
- 使用 `AliasManager::expand_args()` 展开命令行参数
- 使用展开后的参数调用 `Cli::parse_from()`

### 第二阶段：别名管理命令

#### 2.1 CLI 命令结构定义

**文件**: `src/lib/cli/alias.rs`
- 定义 `AliasSubcommand` 枚举（List、Add、Remove）
- 添加命令参数和帮助信息

**文件**: `src/lib/cli/commands.rs`
- 在 `Commands` 枚举中添加 `Alias` 变体
- 导入 `AliasSubcommand`

#### 2.2 命令实现

**文件**: `src/commands/alias/mod.rs`
```rust
mod list;
mod add;
mod remove;

pub use list::AliasListCommand;
pub use add::AliasAddCommand;
pub use remove::AliasRemoveCommand;
```

**文件**: `src/commands/alias/list.rs`
- 实现 `AliasListCommand::list()` - 列出所有别名
- **表格显示**：
  - 定义 `AliasRow` 结构体，实现 `Tabled` trait
  - 包含两列：`Alias Name` 和 `Command`
  - 使用 `TableBuilder` 构建表格
  - 使用 `TableStyle::Modern` 样式
  - 如果没有别名，显示友好提示信息

**文件**: `src/commands/alias/add.rs`
- 实现 `AliasAddCommand::add()` - 添加别名
  - 参数：`name: Option<String>`, `command: Option<String>`
- **常用命令列表维护**：
  - 实现 `get_available_commands()` 函数，从 `Cli::command()` 动态提取所有命令
  - 或者：维护常用命令列表常量（需要手动更新）
  - 或者：从配置文件读取用户自定义的常用命令列表
- **交互式添加模式**（当 `name` 或 `command` 为 `None` 时）：
  - 步骤1：输入别名名称
    - 使用 `InputDialog` 获取别名名称
    - 验证器：检查别名名称格式（不能为空，不能包含空格）
    - 检查别名是否已存在，如果存在使用 `ConfirmDialog` 询问是否覆盖
  - 步骤2：选择命令输入方式
    - 使用 `SelectDialog` 让用户选择：
      - "从常用命令列表选择"
      - "手动输入命令"
  - 步骤3A：从常用命令列表选择（如果用户选择列表）
    - 使用 `SelectDialog` 显示常用命令列表
    - **常用命令列表维护**（推荐使用动态生成方式）：
      - 方式1（推荐）：从 `Cli::command()` 动态提取所有命令和子命令
      - 方式2：硬编码常用命令列表常量（需要手动维护）
      - 方式3：从配置文件读取用户自定义的常用命令列表
      - 方式4（最佳）：动态生成 + 配置文件优先级（默认动态生成，用户可自定义）
  - 步骤3B：手动输入命令（如果用户选择手动输入）
    - 使用 `InputDialog` 获取命令
    - 验证器：检查命令不能为空
- **直接添加模式**（当 `name` 和 `command` 都提供时）：
  - 检查别名是否已存在
  - 如果已存在，使用 `ConfirmDialog` 询问是否覆盖
  - 检查循环别名
- **保存配置**：
  - 保存别名到配置文件
  - 显示成功消息
- **补全脚本更新确认**：
  - 使用 `ConfirmDialog` 询问用户是否更新补全脚本
  - 如果确认，调用 `Completion::generate_all_completions()` 更新补全脚本
  - 显示更新结果（成功或失败）

**文件**: `src/commands/alias/remove.rs`
- 实现 `AliasRemoveCommand::remove()` - 删除别名
  - 参数：`name: Option<String>` - 可选别名名称
- **交互式选择模式**（当 `name` 为 `None` 时）：
  - 加载所有别名列表
  - 使用 `MultiSelectDialog` 显示别名列表（格式：`alias_name = command`）
  - 支持多选删除
  - 如果列表为空，提示用户没有可删除的别名
- **直接删除模式**（当 `name` 为 `Some(name)` 时）：
  - 检查别名是否存在
  - 如果不存在，提示用户并返回错误
- **确认删除**：
  - 显示将要删除的别名列表
  - 使用 `ConfirmDialog` 确认删除（默认值：`false`）
  - 如果用户取消，不执行删除操作
- **批量删除**：
  - 遍历选中的别名，逐个删除
  - 显示删除进度和结果
  - 统计成功和失败的删除数量
- **补全脚本更新确认**：
  - 使用 `ConfirmDialog` 询问用户是否更新补全脚本
  - 如果确认，调用 `Completion::generate_all_completions()` 更新补全脚本
  - 显示更新结果（成功或失败）

#### 2.3 主入口命令分发

**文件**: `src/bin/workflow.rs`
- 在 `match cli.command` 中添加 `Commands::Alias` 分支
- 分发到对应的子命令处理函数

### 第三阶段：测试和优化

#### 3.1 单元测试

**文件**: `src/lib/base/alias/manager.rs`（在 `#[cfg(test)]` 模块中）
- 测试别名加载
- 测试别名展开（基本、嵌套、循环检测）
- 测试参数传递
- 测试边界情况（空配置、不存在的别名等）

#### 3.2 集成测试

**文件**: `tests/alias/`
- `expand.rs` - 测试别名展开功能
- `commands.rs` - 测试别名管理命令
- `circular.rs` - 测试循环别名检测

#### 3.3 错误处理优化

- 添加清晰的错误信息
- 处理配置文件不存在的情况
- 处理配置文件格式错误的情况

### 第四阶段：文档和补全（可选）

#### 4.1 补全脚本集成（可选）

**文件**: `src/lib/completion/generate.rs`
- 在 `generate_workflow()` 中读取别名配置
- 将别名作为命令选项添加到补全脚本中
- 注意：需要重新生成补全脚本才能看到别名

**补全脚本自动更新功能**：
- 在 `alias add` 和 `alias remove` 命令中，添加确认对话框
- 使用 `ConfirmDialog::new("Update completion scripts? (Y/n)")`
  - 默认值：`true`（推荐更新）
  - 如果用户确认，调用 `Completion::generate_all_completions()` 自动更新补全脚本
  - 显示更新结果，如果失败则显示警告信息（但不影响别名操作本身）

**实现示例 - 别名列表（表格显示）**：
```rust
// 在 AliasListCommand::list() 中
use crate::base::util::{TableBuilder, TableStyle};
use crate::alias::AliasManager;
use tabled::Tabled;

/// 别名表格行
#[derive(Tabled, Clone)]
struct AliasRow {
    #[tabled(rename = "Alias Name")]
    alias_name: String,
    #[tabled(rename = "Command")]
    command: String,
}

pub fn list() -> Result<()> {
    log_break!();
    log_message!("Alias List");

    let aliases = AliasManager::list()?;

    if aliases.is_empty() {
        log_info!("No aliases defined");
        log_message!("Run 'workflow alias add' to add an alias.");
        return Ok(());
    }

    // 构建表格数据
    let rows: Vec<AliasRow> = aliases
        .iter()
        .map(|(alias_name, command)| AliasRow {
            alias_name: alias_name.clone(),
            command: command.clone(),
        })
        .collect();

    // 显示表格
    let table = TableBuilder::new(rows)
        .with_title("Defined Aliases")
        .with_style(TableStyle::Modern)
        .render();

    log_message!("{}", table);
    log_success!("Found {} alias/aliases", aliases.len());

    Ok(())
}
```

**实现示例 - 别名添加（交互式）**：
```rust
// 在 AliasAddCommand::add() 中
use crate::base::dialog::{ConfirmDialog, InputDialog, SelectDialog};
use crate::completion::Completion;
use crate::alias::AliasManager;

pub fn add(name: Option<String>, command: Option<String>) -> Result<()> {
    let (alias_name, alias_command) = if let (Some(name), Some(cmd)) = (name, command) {
        // 直接添加模式
        (name, cmd)
    } else {
        // 交互式添加模式
        let aliases = AliasManager::list()?;

        // 步骤1：输入别名名称
        let name = InputDialog::new("Enter alias name")
            .with_validator(|input: &str| {
                if input.trim().is_empty() {
                    Err("Alias name cannot be empty".to_string())
                } else if input.contains(' ') {
                    Err("Alias name cannot contain spaces".to_string())
                } else if aliases.contains_key(input.trim()) {
                    Err(format!("Alias '{}' already exists", input.trim()))
                } else {
                    Ok(())
                }
            })
            .prompt()
            .wrap_err("Failed to get alias name")?;

        // 检查别名是否已存在（如果验证器没捕获到）
        if aliases.contains_key(&name) {
            let should_overwrite = ConfirmDialog::new(format!(
                "Alias '{}' already exists. Overwrite? (y/N)",
                name
            ))
            .with_default(false)
            .prompt()
            .unwrap_or(false);

            if !should_overwrite {
                log_info!("Operation cancelled");
                return Ok(());
            }
        }

        // 步骤2：选择命令输入方式
        let input_method = SelectDialog::new(
            "How do you want to enter the command?",
            vec!["Select from common commands", "Enter manually"]
        )
        .prompt()
        .wrap_err("Failed to select input method")?;

        // 步骤3：获取命令
        let cmd = if input_method == "Select from common commands" {
            // 从常用命令列表选择（使用混合方式）
            let commands = Self::get_common_commands()?;

            // 可选：如果用户想查看所有命令，可以提供选项
            // let all_commands = Self::get_available_commands()?;
            // 在对话框中添加"显示所有命令"选项，然后切换到 all_commands

            SelectDialog::new("Select a command", commands)
                .prompt()
                .wrap_err("Failed to select command")?
        } else {
            // 手动输入
            InputDialog::new("Enter command")
                .with_validator(|input: &str| {
                    if input.trim().is_empty() {
                        Err("Command cannot be empty".to_string())
                    } else {
                        Ok(())
                    }
                })
                .prompt()
                .wrap_err("Failed to get command")?
        };

        (name, cmd)
    };

    // 检查循环别名
    if let Err(e) = AliasManager::check_circular(&alias_name, &alias_command) {
        color_eyre::eyre::bail!("Circular alias detected: {}", e);
    }

    // 保存别名
    AliasManager::add(&alias_name, &alias_command)?;
    log_success!("Alias '{}' = '{}' added successfully", alias_name, alias_command);

    // 询问是否更新补全脚本
    let should_update = ConfirmDialog::new("Update completion scripts? (Y/n)")
        .with_default(true)
        .prompt()
        .unwrap_or(false);

    if should_update {
        match Completion::generate_all_completions(None, None) {
            Ok(_) => {
                log_success!("Completion scripts updated successfully");
            }
            Err(e) => {
                log_warning!("Failed to update completion scripts: {}", e);
                log_info!("You can manually update them later with: workflow completion generate");
            }
        }
    }

    Ok(())
}

/// 获取常用命令列表（混合方式）
///
/// 优先级：commands.toml 配置文件 > 硬编码默认列表
fn get_common_commands() -> Result<Vec<String>> {
    use crate::base::alias::commands_config::CommandsConfig;

    // 1. 优先从 commands.toml 配置文件读取
    if let Ok(config) = CommandsConfig::load() {
        if !config.common_commands.is_empty() {
            return Ok(config.common_commands);
        }
    }

    // 2. 使用硬编码的默认常用命令列表
    const DEFAULT_COMMON_COMMANDS: &[&str] = &[
        "pr create",
        "pr merge",
        "pr status",
        "pr list",
        "jira info",
        "jira search",
        "branch create",
        "branch switch",
        "branch clean",
    ];

    Ok(DEFAULT_COMMON_COMMANDS.iter().map(|s| s.to_string()).collect())
}

/// 从 CLI 结构动态获取所有可用命令
///
/// 遍历所有顶级命令和子命令，生成完整命令列表（格式：`command subcommand`）
fn get_available_commands() -> Result<Vec<String>> {
    use clap::CommandFactory;
    use crate::cli::Cli;

    let cmd = Cli::command();
    let mut commands = Vec::new();

    // 遍历所有顶级命令
    for top_level_cmd in cmd.get_subcommands() {
        let top_level_name = top_level_cmd.get_name();

        // 跳过 alias 命令本身（避免循环）
        if top_level_name == "alias" {
            continue;
        }

        // 如果顶级命令有子命令，遍历子命令
        let subcommands: Vec<_> = top_level_cmd.get_subcommands().collect();
        if !subcommands.is_empty() {
            for subcmd in subcommands {
                let subcmd_name = subcmd.get_name();
                // 跳过内部命令（如 help, version 等）
                if subcmd_name != "help" && subcmd_name != "version" {
                    commands.push(format!("{} {}", top_level_name, subcmd_name));
                }
            }
        } else {
            // 如果顶级命令没有子命令，直接添加
            commands.push(top_level_name.to_string());
        }
    }

    // 排序并去重
    commands.sort();
    commands.dedup();

    Ok(commands)
}
```

**实现示例 - 别名删除（交互式多选）**：
```rust
// 在 AliasRemoveCommand::remove() 中
use crate::base::dialog::{ConfirmDialog, MultiSelectDialog};
use crate::completion::Completion;
use crate::alias::AliasManager;

pub fn remove(name: Option<String>) -> Result<()> {
    let aliases = AliasManager::list()?;

    if aliases.is_empty() {
        log_info!("No aliases defined");
        return Ok(());
    }

    let names_to_remove = if let Some(name) = name {
        // 直接删除模式
        if !aliases.contains_key(&name) {
            color_eyre::eyre::bail!("Alias '{}' not found", name);
        }
        vec![name]
    } else {
        // 交互式选择模式
        let options: Vec<String> = aliases
            .iter()
            .map(|(alias_name, command)| format!("{} = {}", alias_name, command))
            .collect();

        let selected = MultiSelectDialog::new("Select aliases to remove", options)
            .prompt()
            .wrap_err("Failed to select aliases")?;

        if selected.is_empty() {
            log_info!("No aliases selected");
            return Ok(());
        }

        // 从选中的字符串中提取别名名称（格式：alias_name = command）
        selected
            .iter()
            .filter_map(|s| s.split('=').next().map(|n| n.trim().to_string()))
            .collect()
    };

    // 显示将要删除的别名
    log_break!();
    log_message!("Aliases to be removed:");
    for name in &names_to_remove {
        if let Some(command) = aliases.get(name) {
            log_info!("  {} = {}", name, command);
        }
    }

    // 确认删除
    let confirmed = ConfirmDialog::new(format!(
        "Are you sure you want to remove {} alias/aliases?",
        names_to_remove.len()
    ))
    .with_default(false)
    .prompt()
    .wrap_err("Failed to get user confirmation")?;

    if !confirmed {
        log_info!("Operation cancelled");
        return Ok(());
    }

    // 批量删除
    let mut removed_count = 0;
    for name in &names_to_remove {
        match AliasManager::remove(name) {
            Ok(true) => {
                log_success!("Alias '{}' removed successfully", name);
                removed_count += 1;
            }
            Ok(false) => {
                log_warning!("Alias '{}' not found (may have been removed already)", name);
            }
            Err(e) => {
                log_warning!("Failed to remove alias '{}': {}", name, e);
            }
        }
    }

    if removed_count > 0 {
        log_success!("Successfully removed {} alias/aliases", removed_count);

        // 询问是否更新补全脚本
        let should_update = ConfirmDialog::new("Update completion scripts? (Y/n)")
            .with_default(true)
            .prompt()
            .unwrap_or(false);

        if should_update {
            match Completion::generate_all_completions(None, None) {
                Ok(_) => {
                    log_success!("Completion scripts updated successfully");
                }
                Err(e) => {
                    log_warning!("Failed to update completion scripts: {}", e);
                    log_info!("You can manually update them later with: workflow completion generate");
                }
            }
        }
    }

    Ok(())
}
```

**实现示例 - 别名添加（补全脚本更新）**：
```rust
// 在 AliasAddCommand::add() 中
use crate::base::dialog::ConfirmDialog;
use crate::completion::Completion;

// 保存别名后
log_success!("Alias '{}' added successfully", name);

// 询问是否更新补全脚本
let should_update = ConfirmDialog::new("Update completion scripts? (Y/n)")
    .with_default(true)
    .prompt()
    .unwrap_or(false);

if should_update {
    match Completion::generate_all_completions(None, None) {
        Ok(_) => {
            log_success!("Completion scripts updated successfully");
        }
        Err(e) => {
            log_warning!("Failed to update completion scripts: {}", e);
            log_info!("You can manually update them later with: workflow completion generate");
        }
    }
}
```

**注意事项**：
- 补全脚本更新是可选功能，即使更新失败也不应该影响别名操作本身
- 使用 `unwrap_or(false)` 处理用户取消的情况，确保程序继续执行
- 更新失败时只显示警告，不抛出错误，让用户知道可以稍后手动更新
- 默认值设置为 `true`，因为大多数用户希望补全脚本与别名保持同步

#### 4.2 文档更新

- 更新 `README.md`，添加别名系统说明
- 更新帮助信息
- 创建别名系统架构文档（如需要）

---

## 🧪 测试计划

### 单元测试

#### 1. 别名加载测试
```rust
#[test]
fn test_load_aliases() {
    // 测试从配置文件加载别名
    // 测试空配置
    // 测试配置格式错误
}
```

#### 2. 别名展开测试
```rust
#[test]
fn test_expand_basic_alias() {
    // 测试基本别名展开
    // workflow ci -> workflow pr create
}

#[test]
fn test_expand_nested_alias() {
    // 测试嵌套别名展开
    // workflow prc -> workflow ci -> workflow pr create
}

#[test]
fn test_expand_with_args() {
    // 测试参数传递
    // workflow ci --title "test" -> workflow pr create --title "test"
}

#[test]
fn test_circular_alias() {
    // 测试循环别名检测
    // a -> b, b -> a 应该报错
}

#[test]
fn test_max_depth() {
    // 测试最大深度限制
}
```

#### 3. 别名管理测试
```rust
#[test]
fn test_add_alias() {
    // 测试直接添加别名（提供参数）
    // 测试别名已存在的情况
    // 测试循环别名检测
    // 测试交互式添加（不提供参数）
    // 测试从常用命令列表选择
    // 测试手动输入命令
    // 测试别名名称验证（空值、空格等）
}

#[test]
fn test_remove_alias() {
    // 测试直接删除别名（提供参数）
    // 测试别名不存在的情况
    // 测试交互式选择删除（不提供参数）
    // 测试多选删除
    // 测试删除确认对话框
    // 测试批量删除
}

#[test]
fn test_list_aliases() {
    // 测试列出所有别名
}

#[test]
fn test_completion_update_prompt() {
    // 测试补全脚本更新确认对话框
    // 测试用户选择 y 时自动更新补全脚本
    // 测试用户选择 n 时跳过更新
    // 测试补全脚本更新失败时的错误处理
}
```

### 集成测试

#### 1. 命令行展开测试
```rust
#[test]
fn test_command_line_expansion() {
    // 测试完整的命令行别名展开流程
    // 模拟 workflow ci --title "test"
}
```

#### 2. 命令执行测试
```rust
#[test]
fn test_alias_commands() {
    // 测试 workflow alias list
    // 测试 workflow alias add
    // 测试 workflow alias remove
}
```

### 边界情况测试

1. **空配置**：配置文件不存在或别名配置为空
2. **格式错误**：配置文件格式错误
3. **循环别名**：检测并防止循环别名
4. **深度限制**：超过最大展开深度
5. **不存在的别名**：使用不存在的别名时应该返回原命令
6. **别名冲突**：别名与现有命令名称冲突时的处理

---

## ✅ 验收标准

### 功能验收

- [ ] 能够在配置文件中定义别名
- [ ] 别名能够正确展开为完整命令
- [ ] 支持命令参数传递（`workflow ci --title "test"`）
- [ ] 支持别名嵌套（别名引用别名）
- [ ] 能够添加新别名（`workflow alias add <name> <command>`）
- [ ] 能够交互式添加别名（`workflow alias add`，不提供参数）
- [ ] 支持从常用命令列表选择
- [ ] 支持手动输入命令
- [ ] 能够删除别名（`workflow alias remove <name>`）
- [ ] 能够交互式选择删除别名（`workflow alias remove`，不提供参数）
- [ ] 支持多选删除别名
- [ ] 能够列出所有别名（`workflow alias list`）
- [ ] 使用表格格式显示别名列表（两列：Alias Name 和 Command）
- [ ] 表格样式美观（使用 TableStyle::Modern）
- [ ] 没有别名时显示友好提示信息
- [ ] 添加/删除别名后询问是否更新补全脚本
- [ ] 用户确认后自动更新补全脚本
- [ ] 补全脚本更新失败时显示警告但不影响别名操作

### 边界情况

- [ ] 处理循环别名（防止无限递归）
- [ ] 处理不存在的别名（返回原命令，不报错）
- [ ] 处理空别名配置
- [ ] 处理别名名称冲突（与现有命令冲突时给出提示）
- [ ] 处理配置文件不存在的情况
- [ ] 处理配置文件格式错误的情况

### 用户体验

- [ ] 别名展开对用户透明
- [ ] 错误信息清晰友好
- [ ] 命令帮助信息完整
- [ ] 配置文件不存在时使用默认空配置
- [ ] 补全脚本更新确认对话框清晰易懂
- [ ] 补全脚本更新过程有进度提示
- [ ] 补全脚本更新失败时有友好的错误提示
- [ ] 交互式选择界面清晰友好（显示 `alias_name = command` 格式）
- [ ] 多选删除支持空格键切换选择状态
- [ ] 删除前显示确认信息，包含将要删除的别名列表
- [ ] 批量删除时显示删除进度和结果统计
- [ ] 交互式添加流程清晰，步骤明确
- [ ] 别名名称验证友好（提示不能为空、不能包含空格等）
- [ ] 常用命令列表包含常用操作
- [ ] 常用命令列表维护方式合理（推荐动态生成）
- [ ] 动态生成的命令列表与实际命令结构同步
- [ ] 命令列表排序合理，易于查找
- [ ] 支持覆盖已存在的别名（有确认提示）
- [ ] 表格显示清晰易读，列对齐正确
- [ ] 表格标题居中显示

### 代码质量

- [ ] 代码通过 `cargo fmt` 格式化
- [ ] 代码通过 `cargo clippy` 检查（无警告）
- [ ] 所有测试通过
- [ ] 代码覆盖率 > 80%
- [ ] 遵循项目开发规范

---

## 🔍 是否可以通过经典代码补全实现？

**结论：部分功能可以通过补全实现，但核心功能（别名展开）必须通过运行时实现。**

### 1. 别名展开（核心功能）❌ 不能通过补全实现

**原因**：
- Shell 补全（completion）只是在用户输入时提供建议，不会实际修改命令
- 别名展开需要在命令执行前实际替换参数（`workflow ci` → `workflow pr create`）
- 这是一个运行时功能，必须在主入口处实现

**实现方式**：
- 在 `src/bin/workflow.rs` 中，在 `Cli::parse()` 之前进行别名展开
- 读取配置文件中的别名，检查第一个参数是否是别名
- 如果是别名，展开为完整命令，重新构建命令行参数
- 然后使用展开后的参数调用 `Cli::parse()`

### 2. 别名在补全中显示 ⚠️ 部分可以，但有局限性

**可行性**：
- 可以在补全脚本生成时读取配置文件，将别名作为命令选项包含在补全脚本中
- 当用户输入 `workflow ` 后按 Tab，可以显示所有可用的命令，包括别名

**局限性**：
- 补全脚本是静态生成的（在安装时生成一次）
- 每次添加/删除别名后，需要重新生成补全脚本才能看到变化
- 需要修改 `src/lib/completion/generate.rs`，在生成补全脚本时读取配置文件

**实现方式**：
```rust
// 在 generate_workflow() 中
let mut cmd = crate::cli::Cli::command();

// 读取别名配置
let aliases = AliasManager::load()?;
for (alias_name, _) in aliases.iter() {
    // 将别名作为命令选项添加到补全脚本中
    cmd = cmd.subcommand(Command::new(alias_name));
}
```

### 3. 别名管理命令的补全 ⚠️ 部分可以，需要自定义补全函数

**可行性**：
- `workflow alias` 的子命令（`list`、`add`、`remove`）可以通过 `clap_complete` 自动补全
- `workflow alias remove <alias>` 时补全别名列表需要动态读取配置文件

**局限性**：
- `clap_complete` 生成的补全脚本是静态的，无法动态读取配置文件
- 某些 shell（如 zsh）支持自定义补全函数，可以动态读取配置文件
- 这需要编写 shell 特定的补全函数，超出了 `clap_complete` 的能力范围

**实现方式**：
- 对于 zsh：可以编写自定义补全函数，在补全时动态读取配置文件
- 对于 bash：可以使用 `compgen` 和动态补全函数
- 对于其他 shell：可能需要手动维护补全脚本

### 总结

| 功能 | 是否可通过补全实现 | 实现方式 |
|------|-------------------|---------|
| 别名展开 | ❌ 否 | 必须在运行时实现（主入口处） |
| 别名在补全中显示 | ⚠️ 部分可以 | 修改补全脚本生成逻辑，静态包含别名 |
| 别名管理命令补全 | ⚠️ 部分可以 | 子命令自动补全，别名列表需要自定义补全函数 |

**推荐实现策略**：
1. **核心功能**：在运行时实现别名展开（必须）
2. **增强体验**：在补全脚本生成时包含别名（可选，但需要重新生成）
3. **高级功能**：为别名管理命令编写自定义补全函数（可选，复杂）

---

## 📚 相关文档

- [配置架构文档](../architecture/lib/SETTINGS_ARCHITECTURE.md) - 配置文件管理
- [CLI 架构文档](../architecture/lib/CLI_ARCHITECTURE.md) - 命令解析
- [开发规范文档](../guidelines/DEVELOPMENT_GUIDELINES.md) - 开发规范和最佳实践

---

## 📝 实现注意事项

### 1. 错误处理

- 使用 `anyhow::Result<T>` 作为返回类型
- 使用 `Context` 添加上下文信息
- 提供清晰的错误消息

### 2. 配置管理

- 使用现有的 `Settings` 系统
- 确保配置加载和保存的原子性
- 处理配置文件不存在的情况

### 3. 性能考虑

- 别名展开应该在命令解析前进行，避免重复解析
- 使用 `HashMap` 存储别名，提高查找效率
- 缓存别名配置（如果需要）

### 4. 安全性

- 防止命令注入（如果别名值包含用户输入）
- 验证别名名称的有效性
- 限制别名值的长度和格式

### 5. 向后兼容性

- 确保现有命令不受影响
- 别名不应该覆盖现有命令
- 配置文件格式变更时提供迁移路径

---

**创建日期**: 2025-01-27
**最后更新**: 2025-01-27
