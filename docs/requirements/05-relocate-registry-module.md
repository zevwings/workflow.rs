# 需求：重新考虑 registry 模块位置

**优先级**: 🟢 低
**类型**: Architecture Improvement
**影响范围**: `crates/app/src/registry/`, 项目整体架构

## 问题描述

当前 `registry` 模块位于 `crates/app/src/` 内部，但其职责是组合所有底层 crates（storage, services, llm），在架构层次上应该更独立。

### 当前状态

```
workflow-rs/
├── crates/
│   ├── app/
│   │   └── src/
│   │       ├── registry/      # ❌ 依赖注入容器在这里
│   │       ├── commands/
│   │       └── ...
│   ├── storage/
│   ├── services/
│   ├── llm/
│   └── domain/
```

### registry 实际职责

```rust
// crates/app/src/registry/mod.rs
static APP_INITIALIZED: LazyLock<()> = LazyLock::new(|| {
    // 0. 注册配置上下文
    context::register_context()?;

    // 1. 注册 LLM 层服务
    register_llm()?;

    // 2. 注册 storage 层服务
    register_storage()?;

    // 3. 注册 services 层服务
    register_services()?;

    // 4. 注册 app 层服务
    app::register_app()?;
});
```

**职责**：
- ✓ 全局依赖注入容器
- ✓ 组合所有 crate 的注册逻辑
- ✓ 管理应用启动初始化

## 为什么需要修改

### 1. **架构层次错位**

**依赖关系**：
```
registry 依赖 → storage, services, llm, domain
app 依赖 → registry
```

**问题**：
- `registry` 是**底层基础设施**，但位于 `app` 内部
- `app` 不应该包含底层基础设施

**理想架构**：
```
app → registry → (storage, services, llm, domain)
```

### 2. **违反单一职责原则**

`crates/app` 的职责应该是：
- ✓ CLI 命令实现
- ✓ 用户交互逻辑
- ✓ 二进制入口

`crates/app` 不应该包含：
- ✗ 依赖注入容器
- ✗ 全局服务注册
- ✗ 模块初始化逻辑

### 3. **可重用性受限**

如果未来需要开发其他应用（如 Web API、TUI）：

**当前**：
```rust
// 新的 web-api crate
use app::registry::get_service;  // ❌ 依赖整个 app crate
```

**问题**：
- Web API 不需要 CLI 命令实现
- 但必须依赖整个 `app` crate 才能使用 `registry`

**理想**：
```rust
// 新的 web-api crate
use registry::get_service;  // ✅ 仅依赖 registry crate
```

### 4. **测试复杂度增加**

测试底层 crate 时，需要：

```rust
// storage/tests/integration.rs
use app::registry::get_service;  // ❌ 测试 storage 却要依赖 app
```

**问题**：
- 循环依赖风险
- 测试边界不清晰

## 解决方案

### 方案 A：提升为独立 crate（推荐）

```
workflow-rs/
├── crates/
│   ├── registry/           # ✅ 独立 crate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── app.rs
│   │   │   └── context/
│   │   └── Cargo.toml
│   ├── app/
│   ├── storage/
│   ├── services/
│   └── ...
```

**Cargo.toml**：
```toml
# crates/registry/Cargo.toml
[package]
name = "registry"

[dependencies]
domain = { path = "../domain" }
storage = { path = "../storage" }
services = { path = "../services" }
llm = { path = "../llm" }
shaku = "0.8"

# crates/app/Cargo.toml
[dependencies]
registry = { path = "../registry" }  # ✅ app 依赖 registry
```

**优点**：
- ✅ 架构层次清晰
- ✅ 可重用性高
- ✅ 测试边界清晰
- ✅ 符合依赖倒置原则

**缺点**：
- ⚠️ 增加一个 crate（项目复杂度略增）

### 方案 B：保持现状但重命名

如果不想增加 crate，至少应该明确命名：

```
crates/app/src/
├── di/             # Dependency Injection（而非 registry）
│   ├── container.rs
│   ├── app.rs
│   └── context/
```

**优点**：
- ✅ 改动最小
- ✅ 命名更准确

**缺点**：
- ❌ 架构问题未解决
- ❌ 可重用性仍受限

### 推荐：方案 A

理由：
1. 未来可能需要开发 Web API、TUI 等其他应用
2. `registry` 是基础设施，应该独立
3. 有利于测试和模块化

## 迁移步骤（方案 A）

### 步骤 1：创建新 crate

```bash
# 创建 registry crate
mkdir -p crates/registry/src
cd crates/registry

# 创建 Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "registry"
version = "0.1.0"
edition = "2021"

[dependencies]
domain = { path = "../domain" }
storage = { path = "../storage" }
services = { path = "../services" }
llm = { path = "../llm" }
shaku = "0.8"
EOF
```

### 步骤 2：移动代码

```bash
# 移动 app/src/registry → registry/src
mv crates/app/src/registry/* crates/registry/src/

# 创建 lib.rs
mv crates/registry/src/mod.rs crates/registry/src/lib.rs
```

### 步骤 3：更新 workspace

```toml
# Cargo.toml（根目录）
[workspace]
members = [
    "crates/app",
    "crates/registry",  # ✅ 添加
    "crates/storage",
    "crates/services",
    # ...
]
```

### 步骤 4：更新依赖

```toml
# crates/app/Cargo.toml
[dependencies]
registry = { path = "../registry" }  # ✅ 新增
```

### 步骤 5：更新导入

```rust
// crates/app/src/bin/workflow.rs
// 迁移前
use app::registry::{get_global_config_repository, get_path_service};

// 迁移后
use registry::{get_global_config_repository, get_path_service};
```

```bash
# 批量替换
find crates/app/src -type f -name "*.rs" -exec sed -i '' 's/app::registry/registry/g' {} +
find crates/app/src -type f -name "*.rs" -exec sed -i '' 's/crate::registry/registry/g' {} +
```

### 步骤 6：清理 app/src

```rust
// crates/app/src/lib.rs
// 移除
// pub mod registry;  // ❌ 删除

// 只保留
pub mod cli;
pub mod commands;
pub(crate) mod logger;
pub(crate) mod utils;
pub(crate) mod workflows;
```

## 影响评估

### 影响范围

| 类型 | 影响 |
|------|------|
| **新增 crate** | `crates/registry/` |
| **移动文件** | `app/src/registry/*` → `registry/src/` |
| **更新依赖** | `app/Cargo.toml` 添加 `registry` |
| **导入路径** | 所有 `app::registry` → `registry` |
| **破坏性** | 无（仅内部重构） |

### 优点

- ✅ **架构清晰**：依赖关系符合分层原则
- ✅ **高可重用性**：其他应用可直接使用 `registry`
- ✅ **测试隔离**：底层 crate 测试不依赖 `app`
- ✅ **模块独立**：每个 crate 职责单一

### 风险

- ⚠️ **迁移工作量**：需要创建新 crate 并更新导入
- ⚠️ **测试需求**：确保所有依赖注入正常工作
- ⚠️ **编译时间**：可能略微增加（新增 crate）

## 实施计划

### 阶段 1：评估（1 天）
- [ ] 确认是否有多应用需求（Web API, TUI）
- [ ] 评估当前测试对 `registry` 的依赖
- [ ] 决定是否执行（如无需求可暂缓）

### 阶段 2：迁移（2-3 天）
- [ ] 创建 `crates/registry/` 目录和 Cargo.toml
- [ ] 移动 `app/src/registry` 代码
- [ ] 更新 workspace 配置
- [ ] 更新所有导入路径

### 阶段 3：验证（1 天）
- [ ] 运行 `cargo build --workspace`
- [ ] 运行 `cargo test --workspace`
- [ ] 手动测试所有依赖注入
- [ ] 更新文档

## 何时执行

建议在以下情况下执行：
- ✅ 需要开发 Web API 或其他应用
- ✅ 测试时遇到循环依赖问题
- ✅ 进行大规模架构重构时

可以暂缓的情况：
- ⏸️ 仅有 CLI 应用，无扩展需求
- ⏸️ 当前架构运行良好，无明显问题

## 验证方法

```bash
# 1. 检查 workspace 编译
cargo build --workspace

# 2. 运行所有测试
cargo test --workspace

# 3. 确认 registry crate 独立性
cd crates/registry
cargo build  # 应该能独立编译

# 4. 确认导入路径更新完成
rg "app::registry" crates/  # 应该无结果
```

## 相关文档

- [crates/app/src/registry/](../../crates/app/src/registry/)
- [架构文档](../architecture.md)
- [依赖注入设计](../architecture.md#依赖注入)
