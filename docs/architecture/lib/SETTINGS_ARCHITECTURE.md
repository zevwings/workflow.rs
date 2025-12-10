# Settings 模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 Settings 模块架构，包括：
- TOML 配置文件的加载和管理
- 配置结构体定义（Jira、GitHub、日志、LLM、Codeup）
- 路径管理（配置文件路径、安装路径、Shell 路径）
- 默认值管理
- 配置验证功能

该模块为整个应用提供统一的配置管理基础设施，所有模块都通过 `Settings::get()` 获取配置。

**模块统计：**
- 总代码行数：约 950 行
- 文件数量：4 个核心文件
- 主要组件：3 个（Settings, Paths, defaults）
- 配置文件：workflow.toml, llm.toml
- 特殊功能：iCloud 存储支持（macOS）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/base/settings/
├── mod.rs          # 模块声明和导出 (11行)
├── settings.rs     # Settings 结构体和配置加载 (440行)
├── paths.rs        # 路径管理（包含 iCloud 支持，约 637行）
└── defaults.rs     # 默认值辅助函数 (52行)
```

### 依赖模块

- **`lib/base/http/`**：HTTP 客户端（用于配置验证）
- **`lib/jira/`**：Jira 类型定义（用于配置验证）
- **`lib/pr/`**：GitHub 客户端（用于配置验证）
- **`lib/base/util/`**：工具函数（日志宏、敏感值掩码）

### 模块集成

#### 使用场景

- **配置读取**：所有模块通过 `Settings::get()` 获取配置
- **配置验证**：使用 `Settings::verify()` 验证配置正确性
- **路径获取**：使用 `Paths::workflow_config()` 等获取配置文件路径

#### LLM 模块

- **`lib/base/llm/`**：LLM 客户端
  - `Settings::get().llm` - 获取 LLM 配置
  - `Paths::llm_config()` - 获取 LLM 配置文件路径

#### Jira 模块

- **`lib/jira/`**：Jira 客户端
  - `Settings::get().jira` - 获取 Jira 配置
  - `Paths::jira_status_config()` - 获取 Jira 状态配置文件路径
  - `Paths::jira_users_config()` - 获取 Jira 用户配置文件路径

#### PR 模块

- **`lib/pr/`**：PR 平台抽象
  - `Settings::get().github` - 获取 GitHub 配置
  - `Settings::get().codeup` - 获取 Codeup 配置

#### Shell 模块

- **`lib/base/shell/`**：Shell 配置管理
  - `Paths::config_file(shell)` - 获取 Shell 配置文件路径
  - `Paths::completion_dir()` - 获取 completion 目录路径

#### Completion 模块

- **`lib/completion/`**：Completion 管理
  - `Paths::completion_dir()` - 获取 completion 目录路径

#### Rollback 模块

- **`lib/rollback/`**：回滚管理
  - `Paths::completion_dir()` - 获取 completion 目录路径（用于备份）

---

## 🏗️ 架构设计

### 设计原则

1. **单例模式**：使用 `OnceLock` 实现线程安全的全局单例，配置只加载一次
2. **默认值优先**：配置文件不存在或字段缺失时使用默认值，不中断程序运行
3. **统一路径管理**：所有路径通过 `Paths` 统一管理，避免硬编码
4. **类型安全**：使用 Rust 类型系统保证配置结构体的类型安全
5. **配置验证**：提供配置验证功能，确保配置正确性

### 核心组件

#### 1. Settings（配置结构体）

**职责**：定义所有配置结构体和配置加载逻辑

**位置**：`src/lib/base/settings/settings.rs`

**主要结构体**：
- `Settings` - 主配置结构体（包含所有子配置）
- `JiraSettings` - Jira 配置
- `GitHubSettings` - GitHub 配置（支持多账号）
- `GitHubAccount` - GitHub 账号配置
- `LogSettings` - 日志配置
- `LLMSettings` - LLM 配置
- `CodeupSettings` - Codeup 配置

**关键方法**：
- `Settings::get()` - 获取缓存的全局单例（推荐使用）
- `Settings::load()` - 从 TOML 文件加载配置
- `Settings::verify()` - 验证并显示所有配置

**关键特性**：
- ✅ **单例模式**：使用 `OnceLock` 实现线程安全的全局单例
- ✅ **自动加载**：首次调用时自动从 TOML 文件加载
- ✅ **默认值支持**：使用 `#[serde(default)]` 和 `Default` trait
- ✅ **配置验证**：提供 Jira 和 GitHub 配置验证功能
- ✅ **权限检查**：Unix 系统下检查配置文件权限（建议 600）

#### 2. Paths（路径管理器）

**职责**：统一管理所有路径信息

**位置**：`src/lib/base/settings/paths.rs`

**路径分类**：
- **配置路径**：配置文件目录和文件路径
- **安装路径**：二进制文件和补全脚本路径
- **Shell 路径**：Shell 配置文件和 completion 目录

**关键方法**：
- **配置路径**：
  - `config_dir()` - 获取配置目录（`~/.workflow/config/`）
  - `workflow_config()` - 获取主配置文件路径
  - `llm_config()` - 获取 LLM 配置文件路径
  - `jira_status_config()` - 获取 Jira 状态配置文件路径
  - `jira_users_config()` - 获取 Jira 用户配置文件路径
  - `workflow_dir()` - 获取工作流目录（`~/.workflow/`）
  - `work_history_dir()` - 获取工作历史记录目录
- **安装路径**：
  - `command_names()` - 获取所有命令名称
  - `binary_install_dir()` - 获取二进制文件安装目录
  - `binary_paths()` - 获取所有二进制文件完整路径
  - `completion_dir()` - 获取 completion 目录路径
- **Shell 路径**：
  - `config_file(shell)` - 获取 Shell 配置文件路径

**关键特性**：
- ✅ **自动创建目录**：路径不存在时自动创建
- ✅ **权限管理**：Unix 系统下自动设置目录权限（700）
- ✅ **多 Shell 支持**：支持 zsh、bash、fish、powershell、elvish
- ✅ **路径统一管理**：所有路径集中管理，避免硬编码
- ✅ **iCloud 存储支持**：macOS 上自动使用 iCloud Drive 存储配置（可选）

#### iCloud 存储支持（macOS）

**功能概述**：
在 macOS 上，`Paths` 模块支持自动将配置文件存储到 iCloud Drive，实现跨设备同步。该功能通过 `config_base_dir()` 方法实现，自动在 iCloud 和本地存储之间选择最佳位置。

**设计原则**：
1. **自动选择**：系统自动选择最佳存储位置，用户无需手动配置
2. **透明回退**：iCloud 不可用时自动回退到本地存储，确保功能可用
3. **选择性同步**：配置同步到 iCloud，工作历史等保持本地（避免冲突）
4. **平台兼容**：非 macOS 平台自动使用本地存储，无需特殊处理
5. **用户控制**：支持通过环境变量 `WORKFLOW_DISABLE_ICLOUD` 强制使用本地存储

**核心方法**：

- **`try_icloud_base_dir()`** (私有方法，仅 macOS)：
  - 检查 iCloud Drive 是否可用（`~/Library/Mobile Documents/com~apple~CloudDocs`）
  - 如果可用，创建并返回 `~/.workflow/` 目录在 iCloud 中的路径
  - 设置目录权限为 700（仅用户可访问）
  - 如果不可用或创建失败，返回 `None`

- **`config_base_dir()`** (私有方法)：
  - 决策逻辑：
    1. 检查环境变量 `WORKFLOW_DISABLE_ICLOUD`，如果已设置则使用本地存储
    2. 在 macOS 上尝试 iCloud 存储
    3. 如果 iCloud 不可用，回退到本地存储
  - 返回配置基础目录路径

**存储位置决策流程**：

```
config_base_dir()
  ↓
检查环境变量 WORKFLOW_DISABLE_ICLOUD
  ├─ 已设置 → 使用本地存储 (~/.workflow/)
  └─ 未设置 → 继续
  ↓
[仅 macOS] 尝试 iCloud 存储
  ├─ iCloud 可用 → 使用 iCloud (~/Library/Mobile Documents/.../.workflow/)
  └─ iCloud 不可用 → 回退到本地存储
  ↓
返回路径
```

**使用场景**：
- **配置同步**：配置文件存储在 iCloud，自动同步到其他 macOS 设备
- **工作历史保持本地**：`work_history_dir()` 和 `completion_dir()` 强制使用本地存储，避免冲突

**环境变量控制**：
- `WORKFLOW_DISABLE_ICLOUD=1`：强制使用本地存储，即使 iCloud 可用

**查询 API**：
- `Paths::is_config_in_icloud()` - 检查是否使用 iCloud 存储
- `Paths::storage_location()` - 获取存储位置描述
- `Paths::storage_info()` - 获取详细存储信息

**示例**：
```rust
use workflow::base::settings::Paths;

// 自动选择存储位置（iCloud 或本地）
let config_dir = Paths::config_dir()?;

// 检查是否使用 iCloud
if Paths::is_config_in_icloud() {
    log_message!("配置存储在 iCloud，会自动同步");
}

// 强制使用本地存储
std::env::set_var("WORKFLOW_DISABLE_ICLOUD", "1");
let local_config_dir = Paths::config_dir()?;  // 总是返回本地路径
```

#### 3. defaults（默认值模块）

**职责**：提供配置结构体的默认值实现和辅助函数

**位置**：`src/lib/base/settings/defaults.rs`

**主要函数**：
- `default_log_folder()` - 默认日志文件夹名称（"logs"）
- `default_download_base_dir()` - 默认下载基础目录
- `default_download_base_dir_option()` - 默认下载基础目录（Option 类型）
- `default_log_settings()` - 默认 LogSettings 实例
- `default_response_format()` - 默认 LLM 响应格式路径
- `default_llm_provider()` - 默认 LLM Provider（"openai"）
- `default_llm_model(provider)` - 根据 Provider 获取默认模型

**关键特性**：
- ✅ **集中管理**：所有默认值集中在一个模块
- ✅ **Provider 特定默认值**：LLM 模型根据 Provider 提供不同默认值

### 设计模式

#### 1. 单例模式

通过 `OnceLock` 实现线程安全的全局单例：

```rust
pub fn get() -> &'static Settings {
    static SETTINGS: OnceLock<Settings> = OnceLock::new();
    SETTINGS.get_or_init(Self::load)
}
```

**优势**：
- 配置只加载一次，提高性能
- 线程安全，可以在多线程环境中安全使用
- 统一管理，所有模块使用同一个配置实例

#### 2. 默认值模式

使用 `#[serde(default)]` 和 `Default` trait 提供默认值：

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub jira: JiraSettings,
    // ...
}
```

**优势**：
- 配置文件不存在或字段缺失时使用默认值
- 不中断程序运行
- 提供合理的默认值，提升用户体验

#### 3. 路径统一管理模式

通过 `Paths` 结构体统一管理所有路径：

```rust
pub struct Paths;

impl Paths {
    pub fn workflow_config() -> Result<PathBuf> { ... }
    pub fn completion_dir() -> Result<PathBuf> { ... }
    // ...
}
```

**优势**：
- 避免硬编码路径
- 集中管理，易于维护
- 自动创建目录和设置权限

### 错误处理

#### 分层错误处理

1. **路径获取失败**：
   - `HOME` 环境变量未设置：返回错误
   - 目录创建失败：返回错误

2. **配置文件读取失败**：
   - 文件不存在：使用默认值，不报错
   - 读取失败：使用默认值，不报错
   - 解析失败：使用默认值，不报错

3. **配置验证失败**：
   - Jira 验证失败：显示警告，不中断流程
   - GitHub 验证失败：显示警告，不中断流程

#### 容错机制

- **配置文件不存在**：使用默认值，不报错
- **字段缺失**：使用默认值，不报错
- **TOML 解析失败**：使用默认值，不报错
- **配置验证失败**：显示警告，不中断流程

---

## 🔄 调用流程与数据流

### 整体架构流程

```
应用启动或首次使用
  ↓
Settings::get() (获取全局单例)
  ↓
Settings::load() (从 TOML 文件加载)
  ├─ Paths::workflow_config() (获取配置文件路径)
  ├─ 检查文件是否存在
  ├─ 读取 TOML 文件内容
  ├─ 解析为 Settings 结构体
  └─ 使用默认值填充缺失字段
  ↓
缓存到 OnceLock（单例）
  ↓
返回 Settings 引用
```

### 配置加载流程

```
Settings::load()
  ↓
1. Paths::workflow_config()              # 获取配置文件路径
  ↓
2. 检查文件是否存在
  ├─ 不存在 → 返回默认值
  └─ 存在 → 继续
  ↓
3. 检查文件权限（Unix 系统）
  ├─ 权限过宽 → 警告
  └─ 继续
  ↓
4. 读取文件内容
  ├─ 读取失败 → 返回默认值
  └─ 成功 → 继续
  ↓
5. 解析 TOML
  ├─ 解析失败 → 返回默认值
  └─ 成功 → 使用 #[serde(default)] 填充缺失字段
  ↓
6. 返回 Settings 实例
```

### 配置验证流程

```
Settings::verify()
  ↓
1. print_log()                            # 显示日志配置
2. print_llm()                            # 显示 LLM 配置
3. verify_codeup()                        # 显示 Codeup 配置
4. verify_jira()                          # 验证 Jira 配置
  ├─ 检查配置是否完整
  ├─ 发送 HTTP 请求验证
  └─ 显示验证结果
5. verify_github()                        # 验证 GitHub 配置
  ├─ 遍历所有账号
  ├─ 使用 API Token 验证每个账号
  └─ 显示验证结果
```

---

```
workflow.toml (TOML 文件)
  ↓
fs::read_to_string() (读取文件)
  ↓
toml::from_str() (解析 TOML)
  ↓
Settings 结构体（使用 #[serde(default)] 填充缺失字段）
  ↓
OnceLock 缓存（单例）
  ↓
Settings::get() (返回引用)
```

#### 配置验证数据流

```
Settings::verify()
  ↓
显示配置（print_log, print_llm, verify_codeup）
  ↓
验证 Jira（verify_jira）
  ├─ HttpClient::global() (发送 HTTP 请求)
  └─ 显示验证结果
  ↓
验证 GitHub（verify_github）
  ├─ GitHub::get_user_info() (验证每个账号)
  └─ 显示验证结果
```

---

## 📝 扩展性

### 添加新配置项

1. 在 `settings.rs` 中添加新的配置结构体（如 `NewSettings`）
2. 在 `Settings` 结构体中添加新字段（使用 `#[serde(default)]`）
3. 在 `defaults.rs` 中添加默认值函数（如需要）
4. 在 `Settings::verify()` 中添加验证逻辑（如需要）

**示例**：
```rust
// settings.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewSettings {
    pub key: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {
    // ...
    #[serde(default)]
    pub new: NewSettings,
}
```

### 添加新路径

1. 在 `paths.rs` 的 `Paths` 实现中添加新方法
2. 使用 `config_dir()` 或 `workflow_dir()` 作为基础路径
3. 自动创建目录和设置权限（如需要）

**示例**：
```rust
impl Paths {
    pub fn new_config_file() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("new-config.toml"))
    }
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [配置管理命令模块架构文档](../commands/CONFIG_COMMAND_ARCHITECTURE.md) - 命令层如何使用 Settings 模块
- [LLM 模块架构文档](./LLM_ARCHITECTURE.md) - LLM 模块如何使用 Settings
- [Jira 模块架构文档](./JIRA_ARCHITECTURE.md) - Jira 模块如何使用 Settings
- [Shell 模块架构文档](./SHELL_ARCHITECTURE.md) - Shell 模块如何使用 Paths

**注意**：iCloud 存储功能是 Settings 模块的一部分，通过 `Paths` 结构体实现。详细说明见本文档的"Paths（路径管理器）"章节。

---

## 📋 使用示例

### 基本使用

```rust
use workflow::base::settings::Settings;

// 获取全局配置（推荐）
let settings = Settings::get();

// 获取 Jira 配置
if let Some(email) = &settings.jira.email {
    log_message!("Jira email: {}", email);
}

// 获取 GitHub 当前账号
if let Some(account) = settings.github.get_current_account() {
    log_message!("GitHub account: {}", account.name);
}

// 获取 LLM 配置
log_message!("LLM provider: {}", settings.llm.provider);
```

### 使用 Paths

```rust
use workflow::base::settings::Paths;

// 获取配置文件路径
let config_path = Paths::workflow_config()?;

// 获取 completion 目录
let completion_dir = Paths::completion_dir()?;

// 获取 Shell 配置文件路径
use clap_complete::shells::Shell;
let zshrc = Paths::config_file(&Shell::Zsh)?;
```

### 配置验证

```rust
use workflow::base::settings::Settings;

// 加载配置（不使用缓存）
let settings = Settings::load();

// 验证配置
settings.verify()?;
```

---

## ✅ 总结

Settings 模块采用清晰的模块化设计：

1. **单例模式**：`Settings::get()` 实现线程安全的全局单例
2. **默认值支持**：使用 `#[serde(default)]` 和 `Default` trait
3. **路径统一管理**：`Paths` 统一管理所有路径
4. **配置验证**：提供 Jira 和 GitHub 配置验证功能
5. **易于扩展**：添加新配置项只需扩展结构体

**设计优势**：
- ✅ **高性能**：单例模式，配置只加载一次
- ✅ **容错性**：配置文件不存在或字段缺失时使用默认值
- ✅ **类型安全**：使用 Rust 类型系统保证类型安全
- ✅ **统一管理**：所有路径集中管理，避免硬编码
- ✅ **易于扩展**：添加新配置项只需扩展结构体
- ✅ **跨设备同步**：macOS 上支持 iCloud 存储，自动同步配置

通过单例模式、默认值支持、路径统一管理和 iCloud 存储支持，实现了高性能、容错性、易于维护和跨设备同步的目标。
