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

### 依赖模块

- **`lib/base/shell/config.rs`**：Shell 配置文件管理（`ShellConfigManager`）
- **`lib/base/util/clipboard.rs`**：剪贴板操作（`Clipboard`）

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
- `ensure_proxy_enabled()` - 确保代理已启用（如果系统代理已启用，自动设置环境变量）

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
调用者（命令层或其他模块）
  ↓
ProxyManager (协调层)
  ↓
SystemProxyReader / ProxyConfigGenerator / ShellConfigManager (功能层)
```

### 开启代理流程

```
ProxyManager::enable(temporary)
  ↓
  1. SystemProxyReader::read()                    # 获取系统代理设置
  2. ProxyManager::is_proxy_configured()          # 检查代理是否已配置
  3. ProxyConfigGenerator::generate_command()     # 生成代理命令
  4. ProxyConfigGenerator::generate_env_vars()    # 生成环境变量
  5. ShellConfigManager::set_env_vars()          # 保存到配置文件（如果非临时模式）
```

**模式说明**：
- **默认模式**（`temporary = false`）：写入 shell 配置文件，新开 shell 自动启用
- **临时模式**（`temporary = true`）：不写入配置文件，只在当前 shell 生效

### 关闭代理流程

```
ProxyManager::disable()
  ↓
  1. ProxyManager::collect_current_proxy()        # 收集当前代理设置（环境变量和配置文件）
  2. ProxyManager::remove_from_config_file()      # 从配置文件移除
  3. ProxyManager::generate_unset_command()       # 生成 unset 命令
```

**行为说明**：
- 同时从 shell 配置文件和当前 shell 环境变量中移除代理设置
- 生成 `unset` 命令用于当前 shell 会话

### 检查代理流程

```
ProxyManager::check_env_proxy()
  ↓
  1. SystemProxyReader::read()                       # 获取系统代理设置
  2. ProxyManager::check_env_proxy()                 # 检查环境变量
  3. ShellConfigManager::load_env_vars()              # 加载配置文件
  4. ProxyManager::is_proxy_configured()             # 检查代理是否已正确配置
```

### 自动启用代理流程

```
ProxyManager::ensure_proxy_enabled()
  ↓
  1. SystemProxyReader::read()                       # 获取系统代理设置
  2. is_system_proxy_enabled()                       # 检查系统代理是否启用
  3. is_proxy_configured()                           # 检查环境变量是否已配置
  4. ProxyConfigGenerator::generate_env_vars()       # 生成环境变量
  5. std::env::set_var()                             # 在当前进程中设置环境变量
```

**行为说明**：
- 如果系统代理（VPN）未启用，静默跳过，不影响正常流程
- 如果系统代理已启用但环境变量未设置，自动在当前进程中设置环境变量
- 如果环境变量已配置，无需操作
- 主要用于在需要网络访问的命令执行前自动启用代理

### 数据流

#### 代理管理数据流

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

- [主架构文档](../ARCHITECTURE.md)
- [Settings 模块架构文档](./SETTINGS_ARCHITECTURE.md)

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

// 自动启用代理（如果系统代理已启用）
ProxyManager::ensure_proxy_enabled()?;
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
