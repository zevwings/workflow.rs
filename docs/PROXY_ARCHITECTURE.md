# 代理管理模块架构文档

## 📋 概述

本文档描述 Workflow CLI 的代理管理模块架构，包括代理的开启、关闭和检查功能。该模块负责从 macOS 系统设置读取代理配置，并管理代理相关的环境变量。

**模块统计：**
- 总代码行数：约 600 行
- 文件数量：5 个核心文件
- 主要组件：4 个（SystemProxyReader, ProxyConfigGenerator, ProxyManager, ProxyInfo）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/proxy/
├── mod.rs                  # 模块声明和导出
├── proxy.rs                # 类型定义（ProxyType, ProxyInfo, ProxyConfig, 结果类型）
├── system_reader.rs        # 系统代理读取器（从 macOS 系统设置读取）
├── config_generator.rs     # 代理配置生成器（生成命令和环境变量）
└── manager.rs              # 代理管理器（协调其他组件，提供高级功能）
```

### CLI 入口层

```
src/main.rs                 # CLI 入口，参数解析
```

### 命令封装层

```
src/commands/proxy.rs       # 代理管理命令（CLI 接口）
```

### 依赖模块

- **`lib/base/shell/config.rs`**：Shell 配置文件管理（`ShellConfigManager`）
- **`lib/util/clipboard.rs`**：剪贴板操作

---

## 🏗️ 架构设计

### 设计原则

1. **单一职责原则**：每个组件只负责一个明确的功能
2. **迭代器模式**：使用 `ProxyType::all()` 迭代器统一处理所有代理类型
3. **策略模式**：通过 `temporary` 参数控制代理启用策略
4. **职责分离**：系统读取、配置生成、管理协调分离

### 核心组件

#### 1. ProxyType（枚举）

**职责**：定义代理类型（HTTP、HTTPS、SOCKS）

**位置**：`src/lib/proxy/proxy.rs`

**关键方法**：
- `all()` - 返回所有代理类型的迭代器
- `env_key()` - 返回对应的环境变量键名
- `url_scheme()` - 返回对应的 URL 协议方案

**设计优势**：
- 消除硬编码的环境变量名
- 统一处理所有代理类型
- 易于扩展新的代理类型

#### 2. ProxyInfo（结构体）

**职责**：存储代理配置信息

**位置**：`src/lib/proxy/proxy.rs`

**设计**：使用 `HashMap<ProxyType, ProxyConfig>` 存储，消除字段重复

**关键方法**：
- `new()` - 创建新实例
- `get_config(proxy_type)` - 获取指定代理类型的配置
- `get_config_mut(proxy_type)` - 获取可变引用
- `get_proxy_url(proxy_type)` - 获取代理 URL

**设计优势**：
- 消除字段重复（从 9 个字段减少到 1 个 HashMap）
- 易于扩展新的代理类型
- 使用 `ProxyType` 作为键，类型安全

#### 3. SystemProxyReader

**职责**：从 macOS 系统设置读取代理配置

**位置**：`src/lib/proxy/system_reader.rs`

**关键方法**：
- `read()` - 从系统设置读取代理配置

**关键特性**：
- 使用 `scutil --proxy` 命令读取系统代理设置
- 使用映射表简化解析逻辑
- 消除重复的 match 分支

#### 4. ProxyConfigGenerator

**职责**：生成代理命令和环境变量

**位置**：`src/lib/proxy/config_generator.rs`

**关键方法**：
- `generate_command(proxy_info)` - 生成 `export` 命令字符串
- `generate_env_vars(proxy_info)` - 生成环境变量 HashMap

**关键特性**：
- 提取公共逻辑（`generate_proxy_pairs()`）
- 减少代码重复
- 统一处理所有代理类型

#### 5. ProxyManager

**职责**：协调其他组件，提供高级代理管理功能

**位置**：`src/lib/proxy/manager.rs`

**关键方法**：
- `check_env_proxy()` - 检查环境变量中的代理设置
- `is_proxy_configured(proxy_info)` - 检查代理设置是否匹配
- `enable(temporary)` - 开启代理（支持临时模式和持久化模式）
- `disable()` - 关闭代理（同时从配置文件和当前 shell 移除）

**关键特性**：
- 协调 `SystemProxyReader` 和 `ProxyConfigGenerator`
- 支持临时模式和持久化模式
- 使用 `ProxyType` 迭代器统一处理

#### 6. ShellConfigManager

**职责**：通用的 Shell 配置文件管理

**位置**：`src/lib/base/shell/config.rs`

**关键方法**：
- `load_env_vars()` - 从配置块加载环境变量
- `save_env_vars(env_vars)` - 保存环境变量到配置块
- `set_env_vars(env_vars)` - 批量设置环境变量
- `remove_env_vars(keys)` - 从文件中移除指定的 export 语句
- `add_source(source_path, comment)` - 添加 source 语句
- `remove_source(source_path)` - 移除 source 语句

**关键特性**：
- 通用的 Shell 配置文件管理工具
- 供 Proxy 和 Completion 模块共用
- 支持环境变量和 source 语句管理

---

## 🔄 调用流程

### 整体架构流程

```
用户输入
  ↓
main.rs (CLI 入口，参数解析)
  ↓
commands/proxy.rs (命令封装层)
  ↓
ProxyManager (协调层)
  ↓
SystemProxyReader / ProxyConfigGenerator / ShellConfigManager (功能层)
```

### 开启代理 (`on`)

```
main.rs::Commands::Proxy::On { temporary }
  ↓
commands/proxy.rs::ProxyCommand::on(temporary)
  ↓
  1. ProxyManager::enable(temporary)
     ├─ SystemProxyReader::read()                    # 获取系统代理设置
     ├─ ProxyManager::is_proxy_configured()          # 检查代理是否已配置
     ├─ ProxyConfigGenerator::generate_command()     # 生成代理命令
     ├─ ProxyConfigGenerator::generate_env_vars()    # 生成环境变量
     └─ ShellConfigManager::set_env_vars()          # 保存到配置文件（如果非临时模式）
  ↓
  2. Clipboard::copy()                                # 复制代理命令到剪贴板
```

**模式说明**：
- **默认模式**（`temporary = false`）：写入 shell 配置文件，新开 shell 自动启用
- **临时模式**（`temporary = true`）：不写入配置文件，只在当前 shell 生效

### 关闭代理 (`off`)

```
main.rs::Commands::Proxy::Off
  ↓
commands/proxy.rs::ProxyCommand::off()
  ↓
  1. ProxyManager::disable()
     ├─ ProxyManager::collect_current_proxy()        # 收集当前代理设置（环境变量和配置文件）
     ├─ ProxyManager::remove_from_config_file()      # 从配置文件移除
     └─ ProxyManager::generate_unset_command()       # 生成 unset 命令
  ↓
  2. Clipboard::copy()                                 # 复制 unset 命令到剪贴板
```

**行为说明**：
- 同时从 shell 配置文件和当前 shell 环境变量中移除代理设置
- 生成 `unset` 命令用于当前 shell 会话

### 检查代理 (`check`)

```
main.rs::Commands::Proxy::Check
  ↓
commands/proxy.rs::ProxyCommand::check()
  ↓
  1. SystemProxyReader::read()                       # 获取系统代理设置
  2. ProxyManager::check_env_proxy()                 # 检查环境变量
  3. ShellConfigManager::load_env_vars()              # 加载配置文件
  4. 显示系统代理设置（使用 ProxyType::all() 迭代器）
  5. 显示环境变量设置（区分当前会话和配置文件）
  6. ProxyManager::is_proxy_configured()             # 检查代理是否已正确配置
```

---

## 📦 模块职责

### SystemProxyReader

**职责**：从 macOS 系统设置读取代理配置

**核心功能**：
- 使用 `scutil --proxy` 命令读取系统代理设置
- 解析输出并构建 `ProxyInfo` 结构体
- 使用映射表简化解析逻辑

**使用场景**：
- 开启代理时读取系统代理设置
- 检查代理时显示系统代理配置

### ProxyConfigGenerator

**职责**：生成代理命令和环境变量

**核心功能**：
- 生成 `export` 命令字符串（用于当前 shell 会话）
- 生成环境变量 HashMap（用于保存到配置文件）
- 提取公共逻辑，减少代码重复

**使用场景**：
- 开启代理时生成代理命令和环境变量
- 所有需要生成代理配置的场景

### ProxyManager

**职责**：协调其他组件，提供高级代理管理功能

**核心功能**：
- 检查环境变量中的代理设置
- 检查代理设置是否匹配系统配置
- 开启代理（支持临时模式和持久化模式）
- 关闭代理（同时从配置文件和当前 shell 移除）

**使用场景**：
- 所有代理管理操作（开启、关闭、检查）
- 通过 `ProxyManager` 提供统一的接口

### ProxyInfo

**职责**：存储代理配置信息

**核心功能**：
- 使用 `HashMap<ProxyType, ProxyConfig>` 存储代理配置
- 提供类型安全的配置访问接口
- 根据代理类型获取代理 URL

**使用场景**：
- 存储从系统读取的代理配置
- 所有需要访问代理配置的场景

---

## 🔗 与其他模块的集成

### Shell 配置管理

- **`lib/base/shell/config.rs`**：`ShellConfigManager`
  - 环境变量管理（load, save, remove）
  - Source 语句管理（add, remove, has）
  - 配置块管理

**集成方式**：
- Proxy 模块使用 `ShellConfigManager` 管理 shell 配置文件中的环境变量
- Completion 模块也使用 `ShellConfigManager` 管理 source 语句
- 两个模块共享同一套工具，减少代码重复

### 工具模块

- **`lib/util/clipboard.rs`**：剪贴板操作
  - 复制代理命令到剪贴板
  - 复制 unset 命令到剪贴板

**集成方式**：
- 命令层使用 `Clipboard` 复制命令到剪贴板
- 用户可以直接粘贴命令到 shell 执行

---

## 🎯 设计模式

### 1. 单一职责原则（SRP）

每个组件只负责单一功能：
- `SystemProxyReader`：只负责读取系统代理设置
- `ProxyConfigGenerator`：只负责生成命令和环境变量
- `ProxyManager`：只负责协调和提供高级功能
- `ShellConfigManager`：只负责 Shell 配置文件管理

### 2. 迭代器模式

使用 `ProxyType::all()` 迭代器统一处理所有代理类型，消除硬编码：

```rust
ProxyType::all()
    .filter_map(|pt| {
        // 统一处理逻辑
    })
    .collect()
```

**优势**：
- 消除硬编码的环境变量名
- 添加新代理类型时自动支持
- 代码更简洁、可维护

### 3. 策略模式

通过 `temporary` 参数控制代理启用策略：
- **持久化模式**：写入配置文件
- **临时模式**：不写入配置文件

**优势**：
- 保持向后兼容（默认持久化模式）
- 增加灵活性，满足不同使用场景
- 用户可以根据需要选择模式

---

## 🔍 错误处理

### 分层错误处理

1. **CLI 层**：参数验证错误
2. **命令层**：用户交互错误、业务逻辑错误
3. **功能层**：系统调用错误、文件操作错误、配置读写错误

### 容错机制

- **系统代理读取失败**：提示用户检查系统设置
- **文件操作失败**：提供清晰的错误提示和手动操作建议
- **配置解析失败**：返回默认值或空配置

### 错误处理示例

```rust
// SystemProxyReader::read() 错误处理
let output = cmd("scutil", &["--proxy"])
    .read()
    .context("Failed to get system proxy settings")?;

// ProxyManager::enable() 错误处理
ShellConfigManager::set_env_vars(&env_vars)
    .context("Failed to save proxy settings to shell config")?;
```

---

## 📊 数据流

### 代理管理数据流

```
macOS 系统代理设置
  ↓
SystemProxyReader::read()
  ↓
ProxyInfo (HashMap<ProxyType, ProxyConfig>)
  ↓
ProxyConfigGenerator::generate_env_vars()
  ↓
ShellConfigManager::set_env_vars() (持久化模式)
  ↓
Shell 配置文件 (~/.zshrc, ~/.bash_profile)
```

### 当前 Shell 会话数据流

```
ProxyInfo
  ↓
ProxyConfigGenerator::generate_command()
  ↓
export http_proxy=... https_proxy=... all_proxy=...
  ↓
用户执行 eval $(workflow proxy on)
  ↓
当前 Shell 环境变量
```

### 代理检查数据流

```
系统代理设置 (SystemProxyReader)
  ↓
环境变量 (ProxyManager::check_env_proxy)
  ↓
配置文件 (ShellConfigManager::load_env_vars)
  ↓
比较和显示 (ProxyManager::is_proxy_configured)
```

---

## 📝 扩展性

### 添加新代理类型

1. 在 `ProxyType` 枚举中添加新类型
2. 实现 `env_key()` 和 `url_scheme()` 方法
3. 更新 `SystemProxyReader` 的映射表（如果需要）
4. 所有使用 `ProxyType::all()` 迭代器的代码会自动支持新类型

**示例**：
```rust
pub enum ProxyType {
    Http,
    Https,
    Socks,
    Ftp,  // 新增 FTP 代理
}

impl ProxyType {
    fn env_key(&self) -> &'static str {
        match self {
            // ...
            Self::Ftp => "ftp_proxy",
        }
    }
}
```

### 添加新功能

- 所有功能都通过 `ProxyManager` 提供统一的接口
- 新功能可以添加到 `ProxyManager` 或创建新的组件
- 保持单一职责原则，避免组件职责过重

---

## 📚 相关文档

- [主架构文档](./ARCHITECTURE.md)
- [配置管理模块架构文档](./CONFIG_ARCHITECTURE.md)

---

## 📋 使用示例

### 基本使用

```rust
use workflow::ProxyManager;

// 开启代理（持久化模式）
let result = ProxyManager::enable(false)?;
if let Some(cmd) = result.proxy_command {
    println!("Run: eval $({})", cmd);
}

// 开启代理（临时模式）
let result = ProxyManager::enable(true)?;
if let Some(cmd) = result.proxy_command {
    println!("Run: {}", cmd);
}

// 关闭代理
let result = ProxyManager::disable()?;
if let Some(cmd) = result.unset_command {
    println!("Run: {}", cmd);
}

// 检查代理
let env_proxy = ProxyManager::check_env_proxy();
let is_configured = ProxyManager::is_proxy_configured(&proxy_info);
```

### 使用 SystemProxyReader

```rust
use workflow::SystemProxyReader;

// 读取系统代理设置
let proxy_info = SystemProxyReader::read()?;

// 检查代理配置
for proxy_type in workflow::ProxyType::all() {
    if let Some(config) = proxy_info.get_config(proxy_type) {
        if config.enable {
            println!("{}: {}:{}",
                proxy_type.env_key(),
                config.address.as_deref().unwrap_or("N/A"),
                config.port.map(|p| p.to_string()).unwrap_or_else(|| "N/A".to_string())
            );
        }
    }
}
```

### 使用 ProxyConfigGenerator

```rust
use workflow::{ProxyConfigGenerator, SystemProxyReader};

// 读取系统代理设置
let proxy_info = SystemProxyReader::read()?;

// 生成代理命令
if let Some(cmd) = ProxyConfigGenerator::generate_command(&proxy_info) {
    println!("Command: {}", cmd);
}

// 生成环境变量
let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
for (key, value) in &env_vars {
    println!("{}={}", key, value);
}
```

### 使用 ShellConfigManager

```rust
use workflow::base::shell::ShellConfigManager;

// 加载环境变量
let env_vars = ShellConfigManager::load_env_vars()?;

// 设置环境变量
let mut proxy_vars = std::collections::HashMap::new();
proxy_vars.insert("http_proxy".to_string(), "http://proxy:8080".to_string());
ShellConfigManager::set_env_vars(&proxy_vars)?;

// 移除环境变量
ShellConfigManager::remove_env_vars(&["http_proxy", "https_proxy"])?;
```

---

## ✅ 总结

Proxy 模块采用清晰的分层架构设计：

1. **类型定义层**：`ProxyType`、`ProxyInfo`、`ProxyConfig` 提供类型安全的数据结构
2. **功能层**：`SystemProxyReader`、`ProxyConfigGenerator` 提供单一职责的功能组件
3. **协调层**：`ProxyManager` 协调其他组件，提供高级代理管理功能
4. **工具层**：`ShellConfigManager` 提供通用的 Shell 配置文件管理

**设计优势**：
- ✅ **职责分离**：每个组件只负责单一功能，易于测试和维护
- ✅ **代码复用**：使用 `ProxyType` 迭代器统一处理，消除代码重复
- ✅ **易于扩展**：添加新代理类型只需扩展枚举
- ✅ **类型安全**：使用枚举和 HashMap 替代字符串硬编码
- ✅ **灵活配置**：支持临时模式和持久化模式

**重构成果**：
- ✅ 消除所有硬编码的环境变量名
- ✅ 消除字段重复（9 个字段 → 1 个 HashMap）
- ✅ 消除代码重复（提取公共逻辑）
- ✅ 提高可维护性和可扩展性

通过职责分离、迭代器模式和策略模式，实现了代码复用、易于维护和扩展的目标。
