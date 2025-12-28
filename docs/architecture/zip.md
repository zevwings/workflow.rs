# Zip 解压工具模块架构文档

## 📋 概述

Zip 模块是 Workflow CLI 的基础设施模块之一，提供 tar.gz 和 zip 文件解压功能。该模块分为两个子模块：tar.gz 解压（`tar.rs`）和 zip 文件解压（`zip_impl.rs`），通过统一的 `Unzip` 结构体提供向后兼容的接口。

**模块统计：**
- 总代码行数：约 160+ 行
- 文件数量：3 个（`mod.rs`、`tar.rs`、`zip_impl.rs`）
- 主要组件：1 个（`Unzip` 结构体）

---

## 📁 Lib 层架构（核心业务逻辑）

### 核心模块文件

```
src/lib/base/zip/
├── mod.rs          # 模块导出和统一接口 (82行)
├── tar.rs          # tar.gz 解压功能 (56行)
└── zip_impl.rs     # zip 文件解压功能 (77行)
```

### 依赖模块

- **`flate2`**：Gzip 解压（tar.gz 文件）
- **`tar`**：Tar 归档解压（tar.gz 文件）
- **`zip`**：ZIP 文件解压
- **`lib/base/fs/directory`**：目录操作（`DirectoryWalker`）
- **`lib/base/fs/file`**：文件读取（`FileReader`）

### 模块集成

#### Lifecycle 模块集成

- **更新功能**：
  - `LifecycleUpdateCommand` 使用 `Unzip::extract_tar_gz()` 解压下载的更新包
  - 使用 `Unzip::extract_zip()` 解压补全脚本包

**关键方法**：
- `LifecycleUpdateCommand::extract_archive()` - 使用 `Unzip` 解压文件

---

## 🏗️ 架构设计

### 设计原则

1. **统一接口**：通过 `Unzip` 结构体提供统一的解压接口
2. **向后兼容**：保持向后兼容的 API
3. **自动创建目录**：解压前自动创建输出目录
4. **错误处理**：提供清晰的错误消息和上下文信息

### 核心组件

#### Unzip 结构体（统一解压接口）

**位置**：`mod.rs`

**职责**：提供统一的文件解压接口，内部调用各个解压模块的函数

**主要方法**：

##### `extract_tar_gz(tar_gz_path: &Path, output_dir: &Path) -> Result<()>`

**功能**：解压 tar.gz 文件到指定目录

**流程**：
1. 创建输出目录（如果不存在）
2. 打开 tar.gz 文件
3. 创建 Gzip 解码器
4. 创建 Tar 归档读取器
5. 解压到目标目录

**实现**：
- 使用 `flate2::read::GzDecoder` 进行 Gzip 解压
- 使用 `tar::Archive` 进行 tar 归档解压

**示例**：
```rust
use workflow::base::zip::Unzip;
use std::path::Path;

Unzip::extract_tar_gz(
    Path::new("archive.tar.gz"),
    Path::new("./output")
)?;
```

##### `extract_zip(zip_path: &Path, output_dir: &Path) -> Result<()>`

**功能**：解压 zip 文件到指定目录

**流程**：
1. 创建输出目录（如果不存在）
2. 打开 zip 文件
3. 创建 ZipArchive 读取器
4. 遍历所有文件条目
5. 对于目录：创建目录
6. 对于文件：创建文件并写入内容

**实现**：
- 使用 `zip::ZipArchive` 读取 zip 文件
- 使用 `std::io::copy` 复制文件内容

**示例**：
```rust
use workflow::base::zip::Unzip;
use std::path::Path;

Unzip::extract_zip(
    Path::new("archive.zip"),
    Path::new("./output")
)?;
```

---

## 🔄 调用流程与数据流

### 典型调用流程（tar.gz 解压）

```
tar.gz 文件路径 + 输出目录
  ↓
Unzip::extract_tar_gz()
  ├─ 创建输出目录（DirectoryWalker::ensure_exists）
  ├─ 打开文件（FileReader::open）
  ├─ 创建 Gzip 解码器（GzDecoder）
  ├─ 创建 Tar 归档读取器（Archive）
  └─ 解压到目标目录（archive.unpack）
```

### 典型调用流程（zip 解压）

```
zip 文件路径 + 输出目录
  ↓
Unzip::extract_zip()
  ├─ 创建输出目录（DirectoryWalker::ensure_exists）
  ├─ 打开 zip 文件（File::open）
  ├─ 创建 ZipArchive 读取器
  ├─ 遍历所有文件条目
  │  ├─ 目录：创建目录（DirectoryWalker::ensure_exists）
  │  └─ 文件：创建文件并写入内容（File::create + io::copy）
  └─ 返回结果
```

---

## 📋 使用示例

### 基本使用

```rust
use workflow::base::zip::Unzip;
use std::path::Path;

// 解压 tar.gz 文件
Unzip::extract_tar_gz(
    Path::new("archive.tar.gz"),
    Path::new("./output")
)?;

// 解压 zip 文件
Unzip::extract_zip(
    Path::new("archive.zip"),
    Path::new("./output")
)?;
```

### 在更新流程中使用

```rust
use workflow::base::zip::Unzip;
use std::path::Path;

// 下载更新包后解压
let archive_path = Path::new("workflow-v1.0.0.tar.gz");
let output_dir = Path::new("./extracted");

Unzip::extract_tar_gz(&archive_path, &output_dir)?;
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

3. **解压错误**：
   - Gzip 解压失败（tar.gz）
   - Tar 归档解压失败（tar.gz）
   - Zip 文件读取失败
   - Zip 文件解压失败

### 容错机制

- **文件不存在**：返回文件操作错误
- **目录不存在**：自动创建目录（`DirectoryWalker::ensure_exists`）
- **解压失败**：返回解压错误，提示用户检查文件完整性

---

## 📝 扩展性

### 添加新的压缩格式支持

1. 创建新的解压模块文件（如 `7z.rs`）
2. 在 `Unzip` 实现中添加新方法（如 `extract_7z()`）
3. 使用相应的解压库实现功能

---

## 📚 相关文档

- [主架构文档](./architecture.md)
- [Lifecycle 模块架构文档](./lifecycle.md) - 更新功能使用解压工具
- [FS 模块架构文档](./fs.md) - 目录操作依赖
- [Checksum 模块架构文档](./checksum.md) - 文件完整性验证

---

## ✅ 总结

Zip 模块采用清晰的统一接口设计：

1. **统一接口**：通过 `Unzip` 结构体提供统一的解压接口
2. **向后兼容**：保持向后兼容的 API
3. **自动创建目录**：解压前自动创建输出目录
4. **错误处理**：提供清晰的错误消息和上下文信息

**设计优势**：
- ✅ 统一接口，易于使用
- ✅ 向后兼容，保持 API 稳定性
- ✅ 自动创建目录，提升用户体验
- ✅ 错误处理完善，提供清晰的错误信息

**当前实现状态**：
- ✅ tar.gz 解压功能完整实现
- ✅ zip 文件解压功能完整实现
- ✅ 已在更新流程中使用

---

**最后更新**: 2025-12-27

