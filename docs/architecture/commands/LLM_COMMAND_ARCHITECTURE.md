# LLM 命令模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的 LLM 配置管理命令模块架构，包括：
- LLM 配置的交互式设置功能
- LLM 配置的查看和展示功能
- 与 Settings 模块的集成关系

这些命令负责管理 Workflow CLI 的 LLM 配置（provider, url, key, model, language），使用统一的 `ConfigManager` 进行配置更新。

**模块统计：**
- 命令数量：2 个（`setup`, `show`）
- 总代码行数：约 406 行
- 文件数量：3 个
- 主要依赖：`lib/base/settings/`、`lib/jira/config.rs`、`lib/base/llm/`

---

## 📁 相关文件

### CLI 入口层

```
src/bin/workflow.rs
```

### 命令封装层

```
src/commands/llm/
├── mod.rs        # 模块声明和导出（~11 行）
├── setup.rs      # LLM 配置设置命令（~210 行）
└── show.rs       # LLM 配置查看命令（~80 行）
```

**职责**：
- 解析命令参数
- 处理用户交互（输入、选择等）
- 格式化输出
- 调用核心业务逻辑层 (`lib/base/settings/`) 的功能

### 依赖模块

命令层通过调用 `lib/` 模块提供的 API 实现功能，具体实现细节请参考相关模块文档：
- **`lib/base/settings/`**：配置管理（`Settings`、`Paths`、`defaults`）
  - `Settings::load()` - 加载配置
  - `Paths::workflow_config()` - 获取配置文件路径
  - `default_llm_model()` - 获取默认模型
- **`lib/jira/config.rs`**：配置管理器（`ConfigManager`）
  - `ConfigManager::update()` - 更新配置
- **`lib/base/llm/`**：LLM 客户端（配置读取，不直接调用）
- **`lib/commands/config/helpers.rs`**：配置辅助函数
  - `select_language()` - 选择语言

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
bin/workflow.rs (CLI 入口，参数解析)
  ↓
commands/llm/* (命令封装层)
  ↓
lib/base/settings/* (通过 API 调用，具体实现见相关模块文档)
  ↓
~/.workflow/config/workflow.toml (配置文件)
```

### 命令分发流程

```
bin/workflow.rs::main()
  ↓
Cli::parse() (clap 解析)
  ↓
match Commands::Llm { subcommand }
  ↓
  ├─ LLMSubcommand::Setup → LLMSetupCommand::setup()
  └─ LLMSubcommand::Show → LLMShowCommand::show()
```

### 数据流

#### Setup 命令数据流

```
用户输入（交互式）
  ↓
LLMSetupCommand::setup()
  ↓
  1. Settings::load()                    # 加载现有配置
  2. 交互式收集配置项
     ├─ Provider 选择（openai/deepseek/proxy）
     ├─ URL 输入（仅 proxy 需要）
     ├─ API Key 输入
     ├─ Model 输入
     └─ Language 选择
  3. ConfigManager::update()              # 更新配置到 TOML
  4. 输出成功信息
```

#### Show 命令数据流

```
用户输入
  ↓
LLMShowCommand::show()
  ↓
  1. Settings::load()                    # 加载配置
  2. 检查配置是否为空
  3. 格式化输出配置信息
     ├─ Provider
     ├─ URL（仅 proxy 模式）
     ├─ API Key（掩码显示）
     ├─ Model
     └─ Language
```

---

## 1. Setup 命令 (`setup.rs`)

### 相关文件

```
src/commands/llm/setup.rs
```

### 调用流程

```
bin/workflow.rs::Commands::Llm { subcommand: LLMSubcommand::Setup }
  ↓
LLMSetupCommand::setup()
  ↓
  1. Settings::load()                    # 加载现有配置
  2. 交互式收集配置项
     ├─ Select::new()                    # Provider 选择
     ├─ Input::new()                     # URL 输入（仅 proxy）
     ├─ Input::new()                     # API Key 输入
     ├─ Input::new()                     # Model 输入
     └─ select_language()                # Language 选择
  3. ConfigManager::new()                # 创建配置管理器
  4. ConfigManager::update()              # 更新配置
  5. 输出成功信息
```

### 功能说明

1. **交互式配置收集**：
   - 提供友好的交互式界面
   - 显示当前配置值（如果存在）
   - 支持保留现有配置（按 Enter 跳过）

2. **Provider 选择**：
   - 支持三种 Provider：`openai`、`deepseek`、`proxy`
   - 根据选择的 Provider 动态调整配置项

3. **配置项验证**：
   - Proxy Provider 要求 Model 必须配置
   - 其他 Provider 的 Model 为可选

4. **配置保存**：
   - 使用 `ConfigManager` 原子性更新配置
   - 保存到 `workflow.toml` 文件

### 关键步骤说明

#### 步骤 1：Provider 选择

```rust
let llm_providers = vec!["openai", "deepseek", "proxy"];
let llm_provider_idx = Select::new()
    .with_prompt(&llm_provider_prompt)
    .items(&llm_providers)
    .default(current_provider_idx)
    .interact()?;
```

**特性**：
- 显示当前配置的 Provider
- 支持键盘上下选择
- 默认选中当前配置

#### 步骤 2：URL 配置（仅 Proxy）

```rust
let llm_url = match llm_provider.as_str() {
    "openai" => None,      // openai 不使用 proxy URL
    "deepseek" => None,    // deepseek 不使用 proxy URL
    "proxy" => {
        // 交互式输入 URL
        let llm_url_input: String = Input::new()
            .with_prompt(&llm_url_prompt)
            .allow_empty(true)
            .interact_text()?;
        // 处理输入
    }
};
```

**特性**：
- 仅 Proxy Provider 需要配置 URL
- 支持保留现有配置（按 Enter）
- 其他 Provider 自动设置为 `None`

#### 步骤 3：API Key 配置

```rust
let key_prompt = match llm_provider.as_str() {
    "openai" => "OpenAI API key...",
    "deepseek" => "DeepSeek API key...",
    "proxy" => "LLM proxy key...",
    _ => "LLM API key...",
};
```

**特性**：
- 根据 Provider 显示不同的提示信息
- 支持掩码显示现有配置（`***`）
- 支持保留现有配置（按 Enter）

#### 步骤 4：Model 配置

```rust
let llm_model_input: String = Input::new()
    .with_prompt(&model_prompt)
    .allow_empty(!is_proxy)  // Proxy 必须配置 Model
    .validate_with(|input: &String| -> Result<(), &str> {
        if input.is_empty() && is_proxy {
            Err("Model is required for proxy provider")
        } else {
            Ok(())
        }
    })
    .interact_text()?;
```

**特性**：
- Proxy Provider 要求 Model 必须配置
- 其他 Provider 的 Model 为可选
- 使用默认模型（如果未配置）

#### 步骤 5：Language 配置

```rust
let llm_language = select_language(current_language)?;
```

**特性**：
- 使用统一的语言选择函数
- 支持多语言选择
- 默认语言为 `en`

#### 步骤 6：配置保存

```rust
let config_path = Paths::workflow_config()?;
let manager = ConfigManager::<Settings>::new(config_path);

manager.update(|settings| {
    settings.llm.provider = llm_provider.clone();
    settings.llm.url = llm_url.clone();
    settings.llm.key = llm_key.clone();
    settings.llm.model = llm_model.clone();
    settings.llm.language = llm_language.clone();
})?;
```

**特性**：
- 原子性更新配置
- 使用闭包更新配置项
- 自动保存到 TOML 文件

---

## 2. Show 命令 (`show.rs`)

### 相关文件

```
src/commands/llm/show.rs
```

### 调用流程

```
bin/workflow.rs::Commands::Llm { subcommand: LLMSubcommand::Show }
  ↓
LLMShowCommand::show()
  ↓
  1. Settings::load()                    # 加载配置
  2. is_empty_config()                    # 检查配置是否为空
  3. 格式化输出配置信息
     ├─ Provider
     ├─ URL（仅 proxy 模式）
     ├─ API Key（掩码显示）
     ├─ Model
     └─ Language
  4. 输出成功信息
```

### 功能说明

1. **配置展示**：
   - 显示所有 LLM 配置项
   - 根据 Provider 类型显示不同的配置项
   - 掩码显示敏感信息（API Key）

2. **空配置检测**：
   - 检测配置是否为空（使用默认值）
   - 如果为空，提示用户运行 `workflow llm setup`

3. **格式化输出**：
   - 使用日志宏格式化输出
   - 区分已配置和未配置的项
   - 显示默认值信息

### 关键步骤说明

#### 步骤 1：配置加载和检查

```rust
let settings = Settings::load();
let llm = &settings.llm;

if Self::is_empty_config(llm) {
    log_warning!("No LLM configuration found.");
    log_message!("Run 'workflow llm setup' to configure LLM settings.");
    return Ok(());
}
```

**特性**：
- 检查配置是否为空（使用默认值）
- 如果为空，提示用户配置

#### 步骤 2：配置展示

```rust
// Provider
log_info!("Provider: {}", llm.provider);

// URL（仅 proxy 模式）
if llm.provider == "proxy" {
    if let Some(ref url) = llm.url {
        log_info!("Proxy URL: {}", url);
    } else {
        log_warning!("Proxy URL: Not configured");
    }
}

// API Key（掩码显示）
if let Some(ref key) = llm.key {
    let masked = if key.len() > 8 {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    } else {
        "***".to_string()
    };
    log_info!("API Key: {}", masked);
} else {
    log_warning!("API Key: Not configured");
}
```

**特性**：
- 根据 Provider 类型显示不同的配置项
- 掩码显示 API Key（前 4 位 + `...` + 后 4 位）
- 区分已配置和未配置的项

#### 步骤 3：空配置检测

```rust
fn is_empty_config(llm: &LLMSettings) -> bool {
    llm.url.is_none()
        && llm.key.is_none()
        && llm.model.is_none()
        && llm.provider == default_llm_provider()
        && llm.language == default_language()
}
```

**特性**：
- 检查所有配置项是否使用默认值
- 如果所有项都是默认值，视为空配置

---

## 🏗️ 架构设计

### 设计模式

#### 1. 命令模式

**模式说明**：
每个命令封装为一个结构体，提供静态方法执行命令逻辑。

**优势**：
- ✅ 清晰的命令边界
- ✅ 易于测试和维护
- ✅ 统一的命令接口

**实现**：
```rust
pub struct LLMSetupCommand;

impl LLMSetupCommand {
    pub fn setup() -> Result<()> {
        // 命令逻辑
    }
}
```

#### 2. 交互式配置模式

**模式说明**：
使用 `dialoguer` crate 提供交互式配置界面，支持选择、输入等操作。

**优势**：
- ✅ 用户友好的交互体验
- ✅ 支持默认值和验证
- ✅ 统一的交互接口

**实现**：
```rust
use dialoguer::{Input, Select};

// 选择
let idx = Select::new()
    .with_prompt("Select provider")
    .items(&providers)
    .default(current_idx)
    .interact()?;

// 输入
let input: String = Input::new()
    .with_prompt("Enter API key")
    .allow_empty(true)
    .interact_text()?;
```

### 错误处理

#### 分层错误处理

1. **CLI 层**：参数解析错误（clap）
2. **命令层**：用户交互错误（dialoguer）、配置验证错误
3. **库层**：配置加载错误（Settings）、文件操作错误（ConfigManager）

#### 容错机制

- **配置不存在**：使用默认配置
- **用户取消**：返回错误，不保存配置
- **配置验证失败**：提示用户重新输入

---

## 📋 使用示例

### Setup 命令

```bash
# 交互式设置 LLM 配置
workflow llm setup

# 执行流程：
# 1. 选择 Provider（openai/deepseek/proxy）
# 2. 输入 URL（仅 proxy 需要）
# 3. 输入 API Key
# 4. 输入 Model（proxy 必须，其他可选）
# 5. 选择输出语言
# 6. 保存配置
```

**示例输出**：
```
========================================
LLM Configuration Setup
========================================

  LLM/AI Configuration
─────────────────────────────────────────────────
Select LLM provider [current: openai]
> openai
  deepseek
  proxy

OpenAI API key [current: ***] (press Enter to keep)
> sk-...

OpenAI model (optional, press Enter to skip)
> gpt-4

[语言选择界面...]

✓ LLM configuration saved successfully!
ℹ Provider: openai
ℹ Model: gpt-4
ℹ Output Language: en
```

### Show 命令

```bash
# 查看当前 LLM 配置
workflow llm show
```

**示例输出（有配置）**：
```
========================================
LLM Configuration
========================================

ℹ Provider: openai
ℹ API Key: sk-...abcd
ℹ Model: gpt-4
ℹ Output Language: en

✓ LLM configuration displayed.
```

**示例输出（无配置）**：
```
========================================
LLM Configuration
========================================

⚠ No LLM configuration found.
Run 'workflow llm setup' to configure LLM settings.
```

---

## 📝 扩展性

### 添加新的 Provider

1. 在 `setup.rs` 的 `llm_providers` 列表中添加新 Provider
2. 在 URL 配置逻辑中添加新 Provider 的处理（如果需要）
3. 在 API Key 提示中添加新 Provider 的提示信息
4. 在 Model 配置中添加新 Provider 的默认模型（如果需要）

**示例**：
```rust
// 1. 添加 Provider
let llm_providers = vec!["openai", "deepseek", "proxy", "new_provider"];

// 2. URL 配置（如果需要）
"new_provider" => {
    // 处理新 Provider 的 URL 配置
}

// 3. API Key 提示
"new_provider" => "New Provider API key...",
```

### 添加新的配置项

1. 在 `Settings` 结构体中添加新字段
2. 在 `setup.rs` 中添加交互式配置逻辑
3. 在 `show.rs` 中添加配置展示逻辑
4. 在 `ConfigManager::update()` 中添加配置更新逻辑

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [LLM 模块架构文档](../lib/LLM_ARCHITECTURE.md) - LLM 客户端实现
- [Settings 模块架构文档](../lib/SETTINGS_ARCHITECTURE.md) - 配置管理模块
- [配置管理命令架构文档](./CONFIG_COMMAND_ARCHITECTURE.md) - 配置管理命令（包含 setup 命令）

---

## ✅ 总结

LLM 命令模块采用清晰的命令模式设计：

1. **交互式配置**：提供友好的交互式配置界面
2. **配置验证**：根据 Provider 类型验证配置项
3. **统一管理**：使用 `ConfigManager` 统一管理配置更新

**设计优势**：
- ✅ **用户友好**：交互式配置，支持默认值和验证
- ✅ **类型安全**：根据 Provider 类型动态调整配置项
- ✅ **易于扩展**：添加新 Provider 只需配置，无需修改代码
- ✅ **统一接口**：使用统一的配置管理接口

**当前实现状态**：
- ✅ Setup 命令（交互式配置）
- ✅ Show 命令（配置展示）
- ✅ 支持三种 Provider（openai、deepseek、proxy）
- ✅ 配置验证和错误处理
- ✅ 掩码显示敏感信息

**配置说明**：
- 配置文件位置：`~/.workflow/config/workflow.toml`（或 iCloud 路径）
- 配置项：`provider`、`url`（仅 proxy）、`key`、`model`、`language`
- 默认值：Provider 为 `openai`，Language 为 `en`

---

**最后更新**: 2025-12-16
