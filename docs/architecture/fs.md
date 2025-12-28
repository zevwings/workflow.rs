# FS 文件系统操作模块架构文档

## 📋 概述

FS 模块是 Workflow CLI 的基础设施模块之一，提供文件、路径、目录相关的工具函数。该模块采用模块化设计，分为三个子模块：文件操作（`file.rs`）、路径操作（`path.rs`）和目录操作（`directory.rs`），为整个项目提供统一的文件系统操作接口。

**模块统计：**
- 总代码行数：约 500+ 行
- 文件数量：4 个（`mod.rs`、`file.rs`、`path.rs`、`directory.rs`）
- 主要组件：3 个（`FileReader`、`FileWriter`、`PathAccess`、`DirectoryWalker`）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/fs/
├── mod.rs          # 模块导出和公共 API (12行)
├── file.rs         # 文件读写操作 (237行)
├── path.rs         # 路径处理工具 (63行)
└── directory.rs    # 目录操作工具 (136行)
```

### 依赖模块

- **`std::fs`**：标准库文件系统操作
- **`std::io`**：标准库 I/O 操作
- **`serde`**：序列化/反序列化（TOML、JSON）
- **`walkdir`**：目录遍历（`DirectoryWalker`）
- **`toml`**：TOML 文件解析和序列化
- **`serde_json`**：JSON 文件解析和序列化

### 模块集成

FS 模块被整个项目广泛使用：

- **配置管理**：
  - `Settings` 模块使用 `FileReader::toml()` 读取配置文件
  - `RepoConfig` 使用 `FileReader::toml()` 和 `FileWriter::write_toml_secure()` 管理配置
  - `MCPConfigManager` 使用 `FileReader::json()` 和 `FileWriter::write_json_secure()` 管理 MCP 配置

- **生命周期管理**：
  - `Lifecycle` 模块使用 `FileWriter` 写入补全脚本和配置文件
  - 使用 `PathAccess::ensure_dir_exists()` 创建目录结构

- **其他模块**：
  - `Checksum` 模块使用 `FileReader::open()` 读取文件
  - `Zip` 模块使用 `DirectoryWalker` 创建输出目录

---

## 🏗️ 架构设计

### 设计原则

1. **封装性**：封装常用文件操作，提供简洁的 API
2. **类型安全**：使用泛型支持类型安全的序列化/反序列化
3. **错误处理**：统一的错误处理，提供清晰的错误消息
4. **安全性**：提供安全写入方法（自动创建目录、设置权限）

### 核心组件

#### 1. FileReader 结构体（文件读取器）

**位置**：`file.rs`

**职责**：基于路径提供常用读取操作

**主要方法**：

##### `new(path: impl Into<PathBuf>) -> Self`

创建新的文件读取器。

##### `open() -> Result<BufReader<File>>`

打开文件并返回 `BufReader<File>`，用于流式读取。

##### `to_string() -> Result<String>`

读取文件内容为字符串。

##### `lines() -> Result<Vec<String>>`

读取文件的所有行，返回字符串向量。

##### `bytes() -> Result<Vec<u8>>`

读取文件内容为字节向量。

##### `toml<T>() -> Result<T>`

读取 TOML 文件并解析为类型 `T`（需要实现 `DeserializeOwned`）。

**示例**：
```rust
use workflow::base::fs::FileReader;

#[derive(serde::Deserialize)]
struct Config {
    name: String,
}

let config: Config = FileReader::new("config.toml").toml()?;
```

##### `json<T>() -> Result<T>`

读取 JSON 文件并解析为类型 `T`（需要实现 `DeserializeOwned`）。

**示例**：
```rust
use workflow::base::fs::FileReader;

#[derive(serde::Deserialize)]
struct Data {
    value: i32,
}

let data: Data = FileReader::new("data.json").json()?;
```

#### 2. FileWriter 结构体（文件写入器）

**位置**：`file.rs`

**职责**：基于路径提供常用写入操作

**主要方法**：

##### `new(path: impl Into<PathBuf>) -> Self`

创建新的文件写入器。

##### `ensure_parent_dir() -> Result<()>`

确保父目录存在，如果不存在则递归创建。

##### `set_permissions(mode: u32) -> Result<()>`（仅 Unix）

设置文件权限（八进制，如 `0o600`）。

##### `write_str(content: &str) -> Result<()>`

将字符串内容写入文件。

##### `write_str_with_dir(content: &str) -> Result<()>`

将字符串内容写入文件（自动创建父目录）。

##### `write_bytes(content: &[u8]) -> Result<()>`

将字节内容写入文件。

##### `write_bytes_with_dir(content: &[u8]) -> Result<()>`

将字节内容写入文件（自动创建父目录）。

##### `write_toml<T>(data: &T) -> Result<()>`

将类型 `T` 序列化为 TOML 并写入文件（需要实现 `Serialize`）。

##### `write_toml_secure<T>(data: &T) -> Result<()>`

将类型 `T` 序列化为 TOML 并写入文件（自动创建目录和设置权限 `0o600`）。

**示例**：
```rust
use workflow::base::fs::FileWriter;

#[derive(serde::Serialize)]
struct Config {
    name: String,
}

let config = Config { name: "test".to_string() };
FileWriter::new("config.toml").write_toml_secure(&config)?;
```

##### `write_json<T>(data: &T) -> Result<()>`

将类型 `T` 序列化为 JSON 并写入文件（需要实现 `Serialize`）。

##### `write_json_secure<T>(data: &T) -> Result<()>`

将类型 `T` 序列化为 JSON 并写入文件（自动创建目录和设置权限 `0o600`）。

#### 3. PathAccess 结构体（路径助手）

**位置**：`path.rs`

**职责**：封装常用的目录/文件检查与创建操作

**主要方法**：

##### `new(path: impl Into<PathBuf>) -> Self`

创建新的路径助手。

##### `ensure_dir_exists() -> Result<()>`

确保目录存在（若不存在则递归创建）。

##### `ensure_parent_exists() -> Result<()>`

确保父目录存在（若父目录缺失则递归创建）。

##### `read_dir_safe() -> Result<Vec<PathBuf>>`

安全读取目录条目，忽略读取失败的条目。

##### `exists() -> bool`

检查路径是否存在。

##### `is_file() -> bool`

检查是否为文件。

##### `is_dir() -> bool`

检查是否为目录。

**示例**：
```rust
use workflow::base::fs::PathAccess;

let path = PathAccess::new("/path/to/dir");
path.ensure_dir_exists()?;

if path.exists() {
    if path.is_file() {
        println!("It's a file");
    } else if path.is_dir() {
        println!("It's a directory");
    }
}
```

#### 4. DirectoryWalker 结构体（目录遍历助手）

**位置**：`directory.rs`

**职责**：基于固定根路径提供目录遍历和创建操作

**主要方法**：

##### `new(path: impl Into<PathBuf>) -> Self`

创建新的目录遍历助手。

##### `list_dirs() -> Result<Vec<PathBuf>>`

递归列出所有子目录。

##### `list_files() -> Result<Vec<PathBuf>>`

递归列出所有文件。

##### `find_files(pattern: &str) -> Result<Vec<PathBuf>>`

递归查找匹配模式的文件（文件名包含给定模式）。

##### `list_direct_dirs() -> Result<Vec<PathBuf>>`

非递归列出直接子目录。

##### `list_direct_files() -> Result<Vec<PathBuf>>`

非递归列出直接文件。

##### `ensure_exists() -> Result<()>`

确保根目录存在，如果不存在则创建。

**示例**：
```rust
use workflow::base::fs::DirectoryWalker;

let walker = DirectoryWalker::new("/path/to/dir");
walker.ensure_exists()?;

// 递归列出所有文件
let files = walker.list_files()?;
for file in files {
    println!("File: {:?}", file);
}

// 查找匹配模式的文件
let config_files = walker.find_files("config")?;
```

---

## 🔄 调用流程与数据流

### 典型调用流程（配置读取）

```
配置文件路径
  ↓
FileReader::new(path)
  ↓
FileReader::toml<T>()
  ├─ 读取文件内容（fs::read_to_string）
  ├─ 解析 TOML（toml::from_str）
  └─ 返回类型 T
```

### 典型调用流程（配置写入）

```
配置数据 + 文件路径
  ↓
FileWriter::new(path)
  ↓
FileWriter::write_toml_secure<T>()
  ├─ 确保父目录存在（ensure_parent_dir）
  ├─ 序列化为 TOML（toml::to_string_pretty）
  ├─ 写入文件（fs::write）
  └─ 设置文件权限（set_permissions，仅 Unix）
```

### 典型调用流程（目录操作）

```
目录路径
  ↓
DirectoryWalker::new(path)
  ↓
DirectoryWalker::ensure_exists()
  ├─ 检查目录是否存在
  └─ 不存在则创建（fs::create_dir_all）
  ↓
DirectoryWalker::list_files()
  ├─ 遍历目录（WalkDir）
  ├─ 过滤文件
  └─ 返回文件列表
```

---

## 📋 使用示例

### 文件读取

```rust
use workflow::base::fs::{FileReader, FileWriter};

// 读取文本文件
let content = FileReader::new("file.txt").to_string()?;

// 读取所有行
let lines = FileReader::new("file.txt").lines()?;

// 读取 TOML 配置
#[derive(serde::Deserialize)]
struct Config {
    name: String,
}
let config: Config = FileReader::new("config.toml").toml()?;

// 读取 JSON 数据
#[derive(serde::Deserialize)]
struct Data {
    value: i32,
}
let data: Data = FileReader::new("data.json").json()?;
```

### 文件写入

```rust
use workflow::base::fs::FileWriter;

// 写入文本文件
FileWriter::new("file.txt").write_str("Hello, World!")?;

// 写入文本文件（自动创建目录）
FileWriter::new("path/to/file.txt").write_str_with_dir("Content")?;

// 写入 TOML 配置（安全模式）
#[derive(serde::Serialize)]
struct Config {
    name: String,
}
let config = Config { name: "test".to_string() };
FileWriter::new("config.toml").write_toml_secure(&config)?;

// 写入 JSON 数据（安全模式）
#[derive(serde::Serialize)]
struct Data {
    value: i32,
}
let data = Data { value: 42 };
FileWriter::new("data.json").write_json_secure(&data)?;
```

### 路径操作

```rust
use workflow::base::fs::PathAccess;

let path = PathAccess::new("/path/to/dir");

// 确保目录存在
path.ensure_dir_exists()?;

// 确保父目录存在
let file_path = PathAccess::new("/path/to/file.txt");
file_path.ensure_parent_exists()?;

// 检查路径
if path.exists() {
    if path.is_file() {
        println!("It's a file");
    } else if path.is_dir() {
        println!("It's a directory");
    }
}

// 读取目录条目
let entries = path.read_dir_safe()?;
```

### 目录遍历

```rust
use workflow::base::fs::DirectoryWalker;

let walker = DirectoryWalker::new("/path/to/dir");

// 确保目录存在
walker.ensure_exists()?;

// 递归列出所有文件
let files = walker.list_files()?;
for file in files {
    println!("File: {:?}", file);
}

// 递归列出所有目录
let dirs = walker.list_dirs()?;
for dir in dirs {
    println!("Directory: {:?}", dir);
}

// 查找匹配模式的文件
let config_files = walker.find_files("config")?;

// 非递归列出直接文件
let direct_files = walker.list_direct_files()?;
```

---

## 🔍 错误处理

### 错误类型

1. **文件操作错误**：
   - 文件打开失败
   - 文件读取失败
   - 文件写入失败

2. **目录操作错误**：
   - 目录创建失败
   - 目录读取失败

3. **序列化/反序列化错误**：
   - TOML 解析失败
   - JSON 解析失败
   - 序列化失败

### 容错机制

- **文件不存在**：返回文件操作错误
- **目录不存在**：自动创建目录（`ensure_*` 方法）
- **解析失败**：返回解析错误，提示用户检查文件格式
- **权限设置失败**：在非 Unix 系统上静默忽略

---

## 📝 扩展性

### 添加新的文件格式支持

1. 在 `FileReader` 中添加新方法（如 `yaml<T>()`）
2. 使用相应的解析库（如 `serde_yaml`）
3. 在 `FileWriter` 中添加对应的写入方法

### 添加新的目录操作

1. 在 `DirectoryWalker` 中添加新方法
2. 使用 `walkdir` 或标准库实现功能

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Settings 模块架构文档](./settings.md) - 使用文件操作读取配置
- [Repo 模块架构文档](./repo.md) - 使用文件操作管理仓库配置
- [Checksum 模块架构文档](./checksum.md) - 使用文件操作读取文件
- [Zip 模块架构文档](./zip.md) - 使用目录操作创建输出目录

---

## ✅ 总结

FS 模块采用清晰的模块化设计：

1. **封装性**：封装常用文件操作，提供简洁的 API
2. **类型安全**：使用泛型支持类型安全的序列化/反序列化
3. **安全性**：提供安全写入方法（自动创建目录、设置权限）
4. **易用性**：统一的错误处理，提供清晰的错误消息

**设计优势**：
- ✅ 封装性好，简化文件操作
- ✅ 类型安全，使用泛型保证类型正确
- ✅ 安全性高，自动创建目录和设置权限
- ✅ 易于使用，统一的 API 和错误处理

**当前实现状态**：
- ✅ 文件读取功能完整实现
- ✅ 文件写入功能完整实现
- ✅ 路径操作功能完整实现
- ✅ 目录遍历功能完整实现
- ✅ 已在整个项目中广泛使用

---

**最后更新**: 2025-12-27

