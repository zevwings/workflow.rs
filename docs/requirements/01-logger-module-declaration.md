# 需求：修复 logger 模块声明问题

**优先级**: 🔴 高
**类型**: Bug Fix
**影响范围**: `crates/app/src/`

## 问题描述

当前 `crates/app/src/` 目录下存在 `logger/` 目录及其实现文件，但在 `lib.rs` 中没有声明该模块。

### 当前状态

```
crates/app/src/
├── lib.rs              # ❌ 没有声明 logger 模块
├── logger/
│   ├── mod.rs         # ✓ 模块存在
│   └── setup.rs       # ✓ 实现存在
```

**lib.rs 当前内容**:
```rust
pub mod cli;
pub mod commands;
pub mod registry;
pub(crate) mod utils;
pub(crate) mod workflows;
// ❌ 缺少：pub(crate) mod logger;
```

## 为什么需要修改

### 1. **编译一致性问题**
- 未声明的模块不会被 Rust 编译器处理
- 可能导致代码实际未被编译，形成"僵尸代码"
- IDE 工具可能无法正确识别和提供代码补全

### 2. **可维护性风险**
- 代码存在但未使用，增加维护负担
- 开发者可能误以为功能已实现
- 未来重构时容易遗漏

### 3. **违反 Rust 模块系统规范**
- Rust 要求所有模块必须在父模块中显式声明
- 当前状态违反了这一基本原则

## 解决方案

### 方案 A：添加模块声明（推荐）

如果 logger 功能需要使用：

```rust
// crates/app/src/lib.rs
pub mod cli;
pub mod commands;
pub mod registry;
pub(crate) mod logger;  // ✅ 添加声明
pub(crate) mod utils;
pub(crate) mod workflows;
```

**优点**:
- 简单直接，一行代码修复
- 保留现有功能
- 符合 Rust 规范

### 方案 B：删除未使用的模块

如果 logger 功能已废弃或未使用：

```bash
# 删除整个 logger 目录
rm -rf crates/app/src/logger/
```

**优点**:
- 清理僵尸代码
- 减少维护负担

## 验证方法

修复后进行验证：

```bash
# 1. 检查编译是否通过
cargo build

# 2. 检查 logger 模块是否可以导入
# 在代码中测试
use crate::logger::setup_logger;

# 3. 搜索 logger 的使用位置
rg "logger::" crates/app/
rg "setup_logger" crates/app/
```

## 影响评估

- **影响文件**: `crates/app/src/lib.rs`
- **破坏性**: 无（只是修复声明）
- **测试需求**: 验证编译通过
- **迁移成本**: 极低（一行代码）

## 行动建议

1. 首先检查 `logger` 模块是否在项目中被使用
2. 如果被使用，添加模块声明（方案 A）
3. 如果未被使用，删除整个目录（方案 B）
4. 提交代码并验证 CI/CD 通过

## 相关文件

- [crates/app/src/lib.rs](../../crates/app/src/lib.rs)
- [crates/app/src/logger/mod.rs](../../crates/app/src/logger/mod.rs)
- [crates/app/src/logger/setup.rs](../../crates/app/src/logger/setup.rs)
