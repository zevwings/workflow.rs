# Tag 模块架构文档

## 📋 概述

Tag 模块是 Workflow CLI 的 Git 操作模块的一部分，提供完整的 Git tag 操作功能，包括列出 tag、删除 tag、检查 tag 存在性和获取 tag 信息。该模块采用模块化设计，使用零大小结构体组织相关函数，通过统一的辅助函数减少代码重复。

**模块统计：**
- 总代码行数：约 286 行
- 文件数量：1 个（`src/lib/git/tag.rs`）
- 主要结构体：1 个（`GitTag`）
- 类型定义：1 个（`TagInfo`）

---

## 📁 模块结构

### 核心模块文件

```
src/lib/git/
├── mod.rs          # Git 模块声明和导出
├── tag.rs          # Tag 管理操作 (286行)
└── helpers.rs      # Git 操作辅助函数（共享）
```

### 依赖模块

- **`duct`**：命令执行库（执行 Git 命令）
- **`lib/git/helpers.rs`**：Git 操作辅助函数（`cmd_read`、`cmd_run`、`check_ref_exists`、`check_success`）

### 模块集成

- **Tag 命令集成** (`commands/tag/`)：
  - `GitTag::list_all_tags()` - 获取所有 tag
  - `GitTag::get_tag_info()` - 获取 tag 信息
  - `GitTag::delete_local()` - 删除本地 tag
  - `GitTag::delete_remote()` - 删除远程 tag

- **仓库清理集成** (`commands/repo/`)：
  - `GitTag::list_local_tags()` - 获取本地 tag 列表
  - `GitTag::is_tag_exists()` - 检查 tag 是否存在

---

## 🏗️ 架构设计

### 设计原则

1. **模块化设计**：Tag 操作集中在独立的结构体中，职责清晰
2. **零大小结构体**：使用 unit struct 组织相关函数，符合 Rust 最佳实践
3. **统一辅助函数**：通过 `helpers.rs` 提供统一的 Git 命令执行接口
4. **错误处理统一**：使用 `anyhow::Result` 和 `context` 提供清晰的错误信息
5. **类型安全**：使用 `TagInfo` 结构体封装 tag 信息

### 核心组件

#### 1. Tag 管理 (`tag.rs`)

**职责**：提供 tag 相关的所有操作

- **`GitTag`**：Tag 管理结构体（零大小结构体）

**主要方法**：
- `list_local_tags()` - 列出所有本地 tag
- `list_remote_tags()` - 列出所有远程 tag
- `list_all_tags()` - 列出所有 tag（本地和远程，合并去重）
- `is_tag_exists()` - 检查 tag 是否存在（本地或远程）
- `get_tag_info()` - 获取 tag 信息（名称、commit hash、存在位置）
- `delete_local()` - 删除本地 tag
- `delete_remote()` - 删除远程 tag
- `delete_both()` - 删除本地和远程 tag

**关键特性**：
- 支持列出本地和远程 tag
- 自动合并本地和远程 tag 列表（去重）
- 支持删除本地和远程 tag
- 远程 tag 删除支持两种方式（`--delete` 和 `:refs/tags/` 回退）

**使用场景**：
- Tag 删除命令：列出和删除 tag
- 仓库清理命令：列出本地 tag 用于清理

#### 2. 类型定义

**`TagInfo`**：Tag 信息结构体

```rust
pub struct TagInfo {
    pub name: String,              // Tag 名称
    pub commit_hash: String,        // Tag 指向的 commit hash
    pub exists_local: bool,        // Tag 是否在本地存在
    pub exists_remote: bool,       // Tag 是否在远程存在
}
```

**设计优势**：
- 封装 tag 的完整信息
- 便于传递和显示 tag 信息
- 类型安全，避免字符串拼接错误

### 设计模式

#### 1. 模块化设计模式

使用零大小结构体（unit struct）组织相关函数：

```rust
pub struct GitTag;  // 零大小结构体
impl GitTag {
    pub fn list_local_tags() -> Result<Vec<String>> { ... }
    // ...
}
```

**优势**：
- 职责清晰，符合单一职责原则
- 命名空间明确（`GitTag::list_local_tags()`）
- 易于维护和扩展

#### 2. 辅助函数模式

通过 `helpers.rs` 提供统一的 Git 命令执行接口：

```rust
// 统一接口
cmd_read(&["tag", "-l"])
cmd_run(&["tag", "-d", tag_name])
check_ref_exists(&format!("refs/tags/{}", tag_name))
```

**优势**：
- 减少代码重复
- 统一错误处理格式
- 提高代码可维护性

#### 3. 回退模式

`delete_remote()` 方法实现自动回退：

```rust
// 优先使用 --delete 方式
let result = cmd_run(&["push", "origin", "--delete", tag_name]);

if result.is_err() {
    // 回退到使用 :refs/tags/ 方式
    cmd_run(&["push", "origin", &format!(":refs/tags/{}", tag_name)])?;
}
```

**优势**：
- 支持不同 Git 版本和远程仓库配置
- 自动适配不同环境
- 提高兼容性

### 错误处理

#### 分层错误处理

1. **辅助函数层**：统一错误上下文
   ```rust
   cmd_read(&["tag", "-l"])
       .wrap_err("Failed to list local tags")
   ```

2. **业务逻辑层**：添加业务上下文
   ```rust
   GitTag::delete_local(tag_name)
       .wrap_err_with(|| format!("Failed to delete local tag: {}", tag_name))
   ```

3. **命令层**：用户友好的错误提示

#### 容错机制

- **Tag 不存在**：返回明确的错误信息
- **删除失败**：提供清晰的错误信息和解决建议
- **远程 tag 删除失败**：自动回退到备用方法

---

## 🔄 调用流程与数据流

### 整体架构流程

```
调用者（命令层或其他模块）
  ↓
lib/git/tag.rs (核心业务逻辑层)
  ├── GitTag::list_local_tags()      # 列出本地 tag
  ├── GitTag::list_remote_tags()     # 列出远程 tag
  ├── GitTag::list_all_tags()        # 列出所有 tag
  ├── GitTag::is_tag_exists()        # 检查 tag 是否存在
  ├── GitTag::get_tag_info()         # 获取 tag 信息
  ├── GitTag::delete_local()         # 删除本地 tag
  └── GitTag::delete_remote()         # 删除远程 tag
  ↓
helpers.rs (辅助函数层)
  ├── cmd_read()
  ├── cmd_run()
  ├── check_ref_exists()
  └── check_success()
  ↓
duct::cmd (命令执行层)
  └── git 命令
```

### 典型调用示例

#### 1. 列出所有 Tag

```
GitTag::list_all_tags()
  ↓
GitTag::list_local_tags()  # 获取本地 tag
GitTag::list_remote_tags() # 获取远程 tag
  ↓
helpers::cmd_read()  # 执行 git tag -l
helpers::cmd_read()  # 执行 git ls-remote --tags
  ↓
合并去重，构建 TagInfo 列表
```

#### 2. 删除 Tag

```
GitTag::delete_local(tag_name)
  ↓
helpers::cmd_run()  # 执行 git tag -d <tag_name>
```

```
GitTag::delete_remote(tag_name)
  ↓
helpers::cmd_run()  # 执行 git push origin --delete <tag_name>
  ↓
如果失败，回退到 git push origin :refs/tags/<tag_name>
```

#### 3. 获取 Tag 信息

```
GitTag::get_tag_info(tag_name)
  ↓
GitTag::is_tag_exists(tag_name)  # 检查存在性
  ↓
helpers::cmd_read()  # 获取 commit hash（git rev-parse 或 git ls-remote）
  ↓
构建 TagInfo 结构体
```

### 数据流

#### 列出 Tag 数据流

```
用户请求（列出所有 tag）
  ↓
GitTag::list_all_tags()
  ↓
获取本地 tag（GitTag::list_local_tags()）
  ↓
获取远程 tag（GitTag::list_remote_tags()）
  ↓
合并去重
  ↓
获取每个 tag 的 commit hash
  ↓
构建 TagInfo 列表
  ↓
返回结果
```

#### 删除 Tag 数据流

```
用户请求（删除 tag）
  ↓
GitTag::delete_local(tag_name) / GitTag::delete_remote(tag_name)
  ↓
执行 Git 命令（git tag -d / git push origin --delete）
  ↓
返回结果
```

---

## 📋 使用示例

### 基本使用

```rust
use workflow::git::GitTag;

// 列出所有本地 tag
let local_tags = GitTag::list_local_tags()?;

// 列出所有远程 tag
let remote_tags = GitTag::list_remote_tags()?;

// 列出所有 tag（本地和远程，合并去重）
let all_tags = GitTag::list_all_tags()?;

// 检查 tag 是否存在
let (exists_local, exists_remote) = GitTag::is_tag_exists("v1.0.0")?;

// 获取 tag 信息
let tag_info = GitTag::get_tag_info("v1.0.0")?;
println!("Tag: {}, Commit: {}", tag_info.name, tag_info.commit_hash);

// 删除本地 tag
GitTag::delete_local("v1.0.0")?;

// 删除远程 tag
GitTag::delete_remote("v1.0.0")?;

// 删除本地和远程 tag
GitTag::delete_both("v1.0.0")?;
```

### 批量操作

```rust
use workflow::git::GitTag;

// 列出所有 tag
let all_tags = GitTag::list_all_tags()?;

// 过滤需要删除的 tag（例如：所有 v1.x 版本的 tag）
let tags_to_delete: Vec<String> = all_tags
    .iter()
    .filter(|tag| tag.name.starts_with("v1."))
    .map(|tag| tag.name.clone())
    .collect();

// 批量删除
for tag_name in tags_to_delete {
    if let Err(e) = GitTag::delete_both(&tag_name) {
        eprintln!("Failed to delete tag {}: {}", tag_name, e);
    }
}
```

---

## 📝 扩展性

### 添加新的 Tag 操作

1. 在 `tag.rs` 中添加方法
2. 使用 `helpers.rs` 中的辅助函数
3. 添加文档注释
4. 在 `mod.rs` 中导出（如需要）

**示例**：
```rust
// tag.rs
impl GitTag {
    pub fn create_tag(tag_name: &str, message: Option<&str>) -> Result<()> {
        let mut args = vec!["tag"];

        if let Some(msg) = message {
            args.push("-a");
            args.push(tag_name);
            args.push("-m");
            args.push(msg);
        } else {
            args.push(tag_name);
        }

        helpers::cmd_run(&args)
            .wrap_err_with(|| format!("Failed to create tag: {}", tag_name))
    }
}
```

### 添加新的 Tag 信息字段

1. 在 `TagInfo` 结构体中添加新字段
2. 更新相关方法以填充新字段

**示例**：
```rust
pub struct TagInfo {
    pub name: String,
    pub commit_hash: String,
    pub exists_local: bool,
    pub exists_remote: bool,
    pub created_date: Option<String>,  // 新增字段
}
```

---

## 📚 相关文档

- [主架构文档](../ARCHITECTURE.md)
- [Git 模块架构文档](./GIT_ARCHITECTURE.md) - Git 操作相关
- [Tag 命令架构文档](../commands/TAG_COMMAND_ARCHITECTURE.md) - Tag 命令层详细说明

---

## ✅ 总结

Tag 模块采用清晰的模块化设计：

1. **模块化结构**：Tag 操作集中在独立的结构体中，职责清晰
2. **统一辅助函数**：通过 `helpers.rs` 提供统一的命令执行接口
3. **类型安全**：使用 `TagInfo` 结构体封装 tag 信息
4. **错误处理统一**：使用 `anyhow::Result` 和 `context` 提供清晰的错误信息
5. **易于扩展**：模块化设计便于添加新功能
6. **完整功能**：支持列出、检查、删除本地和远程 tag

**设计优势**：
- ✅ **职责清晰**：每个方法负责单一功能领域
- ✅ **代码复用**：统一的辅助函数减少重复代码
- ✅ **易于维护**：模块化设计，低耦合
- ✅ **类型安全**：结构体保证类型安全
- ✅ **兼容性好**：自动回退机制支持不同 Git 版本和配置

通过模块化设计和统一辅助函数，实现了代码复用、易于维护和扩展的目标。

---

**最后更新**: 2025-12-16
