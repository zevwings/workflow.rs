# 模块组织规范

> 本文档定义了 Workflow CLI 项目的模块组织规范和最佳实践，所有贡献者都应遵循这些规范。

---

## 📋 目录

- [概述](#-概述)
- [目录结构](#-目录结构)
- [模块职责](#-模块职责)
- [模块依赖规则](#-模块依赖规则)
- [平台特定代码组织](#-平台特定代码组织)
- [相关文档](#-相关文档)

---

## 📋 概述

本文档定义了模块组织规范，包括目录结构、模块职责、模块依赖规则和平台特定代码组织。

### 核心原则

- **三层架构**：遵循项目的三层架构设计
- **职责分离**：各层职责清晰，避免混淆
- **依赖管理**：避免循环依赖，保持依赖方向清晰

### 使用场景

- 创建新模块时参考
- 重构模块时使用
- 代码审查时检查

---

## 目录结构

遵循项目的三层架构：

```
src/
├── main.rs              # CLI 入口
├── lib.rs               # 库入口
├── bin/                 # 独立可执行文件
│   └── install.rs
├── commands/            # 命令封装层
│   ├── pr/
│   ├── log/
│   └── ...
└── lib/                 # 核心业务逻辑层
    ├── base/           # 基础模块
    ├── pr/             # PR 模块
    ├── jira/           # Jira 模块
    └── ...
```

---

## 模块职责

- **`commands/`**：CLI 命令封装，处理用户交互、参数解析
- **`lib/`**：核心业务逻辑，可复用的功能模块
- **`bin/`**：独立的可执行文件入口

---

## 模块依赖规则

- **命令层** → **库层**：命令层可以依赖库层，但不能反向依赖
- **库层内部**：可以相互依赖，但避免循环依赖
- **基础模块**：`lib/base/` 不依赖其他业务模块

---

## 平台特定代码组织

项目支持跨平台开发（macOS、Linux、Windows），需要正确处理平台特定代码。

### 使用条件编译组织平台特定代码

使用 Rust 的条件编译属性 `#[cfg(...)]` 来组织平台特定代码：

```rust
// 平台特定的函数实现
#[cfg(target_os = "macos")]
fn get_system_path() -> PathBuf {
    PathBuf::from("/usr/local/bin")
}

#[cfg(target_os = "linux")]
fn get_system_path() -> PathBuf {
    PathBuf::from("/usr/bin")
}

#[cfg(target_os = "windows")]
fn get_system_path() -> PathBuf {
    PathBuf::from("C:\\Program Files\\Workflow")
}

// 平台特定的模块
#[cfg(target_os = "macos")]
mod macos_specific;

#[cfg(target_os = "linux")]
mod linux_specific;

#[cfg(target_os = "windows")]
mod windows_specific;
```

### 平台特定代码的模块化组织

对于复杂的平台特定功能，建议使用独立的模块文件：

```
src/lib/base/util/
├── mod.rs
├── platform.rs          # 平台检测（跨平台）
├── clipboard.rs         # 剪贴板功能（平台特定）
├── clipboard_macos.rs   # macOS 实现
├── clipboard_linux.rs   # Linux 实现
└── clipboard_windows.rs # Windows 实现
```

在 `mod.rs` 中组织：

```rust
// mod.rs
pub mod platform;

#[cfg(target_os = "macos")]
mod clipboard_macos;
#[cfg(target_os = "linux")]
mod clipboard_linux;
#[cfg(target_os = "windows")]
mod clipboard_windows;

#[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
pub mod clipboard;
```

### 平台特定依赖管理

在 `Cargo.toml` 中使用条件依赖：

```toml
# 仅在特定平台启用依赖
[target.'cfg(all(not(target_env = "musl"), not(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))))'.dependencies]
clipboard = "0.5"
```

**规则**：
- 使用 `target.'cfg(...)'.dependencies` 来指定平台特定依赖
- 明确说明为什么某些平台禁用某些依赖（如 Linux ARM64 和 musl 静态链接版本不支持剪贴板功能）
- 在文档中说明平台限制

### 平台检测工具

使用 `Platform` 结构体进行运行时平台检测：

```rust
use crate::base::util::Platform;

let platform = Platform::detect();
if platform.is_macos() {
    // macOS 特定逻辑
} else if platform.is_linux() {
    // Linux 特定逻辑
} else if platform.is_windows() {
    // Windows 特定逻辑
}
```

**使用场景**：
- **编译时检测**：使用 `#[cfg(...)]` 条件编译，适用于编译期已知的平台差异
- **运行时检测**：使用 `Platform::detect()`，适用于需要在运行时判断平台的场景

### 平台特定功能的测试要求

**测试覆盖要求**：
- 平台特定功能必须添加单元测试
- 使用条件编译确保测试只在对应平台运行：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn test_macos_specific_feature() {
        // macOS 特定测试
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_specific_feature() {
        // Linux 特定测试
    }
}
```

**跨平台测试策略**：
- 在 CI/CD 中为所有支持的平台运行测试
- 使用 GitHub Actions 的矩阵构建策略测试多个平台
- 确保所有平台的测试都能通过

**测试注意事项**：
- 某些功能可能在某些平台上不可用（如 Linux ARM64 和 musl 静态链接版本不支持剪贴板功能）
- 在测试中正确处理平台限制，避免在不支持的平台上运行相关测试

### 平台特定代码的最佳实践

1. **优先使用条件编译**：对于编译期已知的平台差异，使用 `#[cfg(...)]` 条件编译
2. **运行时检测作为补充**：对于需要在运行时判断的场景，使用 `Platform::detect()`
3. **模块化组织**：将平台特定代码组织到独立的模块文件中，保持代码清晰
4. **文档说明**：在代码和文档中明确说明平台限制和平台特定行为
5. **测试覆盖**：确保所有平台的代码都有相应的测试覆盖

### 相关文档

- `src/lib/base/util/platform.rs` - 平台检测工具实现
- `.github/workflows/release.yml` - 跨平台构建和测试配置
- `Cargo.toml` - 平台特定依赖配置示例

---

## 🔍 故障排除

### 问题 1：模块依赖混乱

**症状**：模块依赖方向不清晰，存在循环依赖

**解决方案**：

1. 检查模块依赖是否符合三层架构规则
2. 避免命令层和库层之间的反向依赖
3. 使用依赖图工具分析依赖关系

### 问题 2：平台特定代码组织混乱

**症状**：平台特定代码分散在多个文件中，难以维护

**解决方案**：

1. 使用独立的模块文件组织平台特定代码
2. 在 `mod.rs` 中统一管理平台特定模块
3. 使用条件编译属性明确平台限制

---

## 📚 相关文档

### 开发规范

- [代码风格规范](./code-style.md) - 代码风格规范
- [命名规范](./naming.md) - 命名规范

### 架构文档

- [架构文档](../../architecture/architecture.md) - 项目架构总览

---

## ✅ 检查清单

使用本规范时，请确保：

- [ ] 遵循三层架构设计
- [ ] 模块职责清晰
- [ ] 模块依赖方向正确
- [ ] 平台特定代码组织清晰
- [ ] 平台特定功能有测试覆盖

---

**最后更新**: 2025-12-23

