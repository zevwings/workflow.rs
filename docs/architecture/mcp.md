# MCP 配置管理模块架构文档

## 📋 概述

MCP (Model Context Protocol) 配置管理模块是 Workflow CLI 的基础设施模块之一，提供 MCP 配置文件的读写和管理功能。该模块支持读取和写入 `.cursor/mcp.json` 配置文件，检测已配置的 MCP 服务器，合并配置（不覆盖已有配置），以及验证配置格式。

**模块统计：**
- 总代码行数：约 126 行
- 文件数量：2 个（`mod.rs`、`config.rs`）
- 主要组件：3 个（`MCPConfigManager`、`MCPConfig`、`MCPServerConfig`）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/mcp/
├── mod.rs          # 模块导出和公共 API (9行)
└── config.rs       # MCP 配置管理实现 (126行)
```

### 依赖模块

- **`serde`**：序列化/反序列化（JSON）
- **`lib/base/fs/file`**：文件读写（`FileReader`、`FileWriter`）
- **`std::collections::HashMap`**：MCP 服务器配置存储

### 模块集成

MCP 模块主要用于 Cursor IDE 的 MCP 服务器配置管理：

- **配置管理**：
  - 读取和写入 `.cursor/mcp.json` 配置文件
  - 检测已配置的 MCP 服务器
  - 合并配置（不覆盖已有配置）

---

## 🏗️ 架构设计

### 设计原则

1. **项目级配置**：配置文件存储在项目根目录的 `.cursor/mcp.json`
2. **配置合并**：合并配置时不覆盖已有配置，只添加新配置
3. **自动创建**：如果配置文件不存在，自动创建
4. **类型安全**：使用结构体定义配置格式，保证类型安全

### 核心组件

#### 1. MCPServerConfig 结构体（MCP 服务器配置）

**位置**：`config.rs`

**职责**：表示单个 MCP 服务器的配置

**字段**：
- `command: String` - 命令（如 "npx"）
- `args: Vec<String>` - 命令参数
- `env: HashMap<String, String>` - 环境变量（可选）

**示例**：
```json
{
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-github"],
  "env": {
    "GITHUB_TOKEN": "your-token"
  }
}
```

#### 2. MCPConfig 结构体（MCP 配置文件结构）

**位置**：`config.rs`

**职责**：表示完整的 MCP 配置文件结构

**字段**：
- `mcp_servers: HashMap<String, MCPServerConfig>` - MCP 服务器配置（键为服务器名称）

**示例**：
```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "your-token"
      }
    }
  }
}
```

#### 3. MCPConfigManager 结构体（MCP 配置管理器）

**位置**：`config.rs`

**职责**：管理 MCP 配置文件的读写和操作

**字段**：
- `config_path: PathBuf` - 配置文件路径

**主要方法**：

##### `new() -> Result<Self>`

创建新的配置管理器。

**配置文件位置**：统一使用项目目录下的 `.cursor/mcp.json`。

##### `config_path() -> &PathBuf`

获取配置文件路径。

##### `read() -> Result<MCPConfig>`

读取配置文件。

**行为**：
- 如果文件不存在，返回默认配置（空的 `mcp_servers`）
- 如果文件存在，读取并解析 JSON

##### `write(config: &MCPConfig) -> Result<()>`

写入配置文件。

**行为**：
- 自动创建目录和文件
- 设置适当的权限（Unix 系统：`0o600`）

##### `update<F>(f: F) -> Result<()>`

更新配置文件。

**流程**：
1. 读取现有配置
2. 应用更新函数
3. 写回文件

**示例**：
```rust
manager.update(|config| {
    config.mcp_servers.insert("new-server".to_string(), server_config);
})?;
```

##### `merge(new_config: &MCPConfig) -> Result<()>`

合并配置。

**行为**：
- 将新配置合并到现有配置中
- 不覆盖已有的 MCP 服务器配置
- 如果服务器已存在，合并环境变量（不覆盖已有环境变量）

**示例**：
```rust
let new_config = MCPConfig {
    mcp_servers: {
        let mut map = HashMap::new();
        map.insert("github".to_string(), github_config);
        map
    },
};
manager.merge(&new_config)?;
```

##### `detect_configured_servers() -> Result<HashSet<String>>`

检测已配置的 MCP 服务器。

**返回**：已配置的 MCP 服务器名称集合。

##### `is_configured(server_name: &str) -> Result<bool>`

检查特定 MCP 服务器是否已配置。

**返回**：如果已配置返回 `true`，否则返回 `false`。

---

## 🔄 调用流程与数据流

### 典型调用流程（读取配置）

```
项目根目录
  ↓
MCPConfigManager::new()
  ├─ 检测配置文件路径（.cursor/mcp.json）
  └─ 返回配置管理器
  ↓
MCPConfigManager::read()
  ├─ 检查文件是否存在
  ├─ 不存在：返回默认配置
  ├─ 存在：读取文件（FileReader::json）
  └─ 返回 MCPConfig
```

### 典型调用流程（写入配置）

```
MCPConfig + 文件路径
  ↓
MCPConfigManager::write()
  ├─ 确保父目录存在
  ├─ 序列化为 JSON（serde_json::to_string_pretty）
  ├─ 写入文件（FileWriter::write_json_secure）
  └─ 设置文件权限（Unix：0o600）
```

### 典型调用流程（合并配置）

```
新配置 + 现有配置
  ↓
MCPConfigManager::merge()
  ├─ 读取现有配置
  ├─ 遍历新配置的服务器
  │  ├─ 服务器已存在：合并环境变量（不覆盖）
  │  └─ 服务器不存在：直接添加
  ├─ 更新配置
  └─ 写回文件
```

---

## 📋 使用示例

### 基本使用

```rust
use workflow::base::mcp::config::MCPConfigManager;

// 创建配置管理器
let manager = MCPConfigManager::new()?;

// 读取配置
let config = manager.read()?;

// 检查服务器是否已配置
if manager.is_configured("github")? {
    println!("GitHub MCP server is configured");
}

// 检测所有已配置的服务器
let servers = manager.detect_configured_servers()?;
for server in servers {
    println!("Configured server: {}", server);
}
```

### 写入配置

```rust
use workflow::base::mcp::config::{MCPConfigManager, MCPConfig, MCPServerConfig};
use std::collections::HashMap;

let manager = MCPConfigManager::new()?;

// 创建服务器配置
let server_config = MCPServerConfig {
    command: "npx".to_string(),
    args: vec!["-y", "@modelcontextprotocol/server-github".to_string()],
    env: {
        let mut env = HashMap::new();
        env.insert("GITHUB_TOKEN".to_string(), "your-token".to_string());
        env
    },
};

// 创建配置
let mut config = MCPConfig::default();
config.mcp_servers.insert("github".to_string(), server_config);

// 写入配置
manager.write(&config)?;
```

### 更新配置

```rust
let manager = MCPConfigManager::new()?;

// 更新配置
manager.update(|config| {
    // 添加新服务器
    let new_server = MCPServerConfig {
        command: "npx".to_string(),
        args: vec!["-y", "@modelcontextprotocol/server-jira".to_string()],
        env: HashMap::new(),
    };
    config.mcp_servers.insert("jira".to_string(), new_server);
})?;
```

### 合并配置

```rust
let manager = MCPConfigManager::new()?;

// 创建新配置
let mut new_config = MCPConfig::default();
let github_server = MCPServerConfig {
    command: "npx".to_string(),
    args: vec!["-y", "@modelcontextprotocol/server-github".to_string()],
    env: HashMap::new(),
};
new_config.mcp_servers.insert("github".to_string(), github_server);

// 合并配置（不覆盖已有配置）
manager.merge(&new_config)?;
```

---

## 🔍 错误处理

### 错误类型

1. **文件操作错误**：
   - 文件读取失败
   - 文件写入失败
   - 目录创建失败

2. **配置解析错误**：
   - JSON 格式错误
   - 配置结构不匹配

3. **路径错误**：
   - 无法获取当前工作目录

### 容错机制

- **文件不存在**：返回默认配置（不报错）
- **配置解析失败**：返回解析错误，提示用户检查配置文件格式
- **目录不存在**：自动创建目录（`FileWriter::write_json_secure`）

---

## 📝 扩展性

### 添加新的配置字段

1. 在 `MCPServerConfig` 或 `MCPConfig` 结构体中添加新字段
2. 使用 `serde` 属性控制序列化/反序列化
3. 更新相关方法以支持新字段

### 添加新的配置操作

1. 在 `MCPConfigManager` 实现中添加新方法
2. 遵循现有的设计模式（读取、更新、写入）

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [FS 模块架构文档](./fs.md) - 文件操作依赖
- [Settings 模块架构文档](./settings.md) - 配置管理相关

---

## ✅ 总结

MCP 模块采用清晰的配置管理设计：

1. **项目级配置**：配置文件存储在项目根目录，便于版本控制
2. **配置合并**：合并配置时不覆盖已有配置，保护用户设置
3. **自动创建**：如果配置文件不存在，自动创建
4. **类型安全**：使用结构体定义配置格式，保证类型安全

**设计优势**：
- ✅ 项目级配置，便于版本控制
- ✅ 配置合并，保护用户设置
- ✅ 自动创建，提升用户体验
- ✅ 类型安全，使用 Rust 类型系统保证安全性

**当前实现状态**：
- ✅ 配置读取功能完整实现
- ✅ 配置写入功能完整实现
- ✅ 配置更新功能完整实现
- ✅ 配置合并功能完整实现
- ✅ 服务器检测功能完整实现

---

**最后更新**: 2025-12-27

