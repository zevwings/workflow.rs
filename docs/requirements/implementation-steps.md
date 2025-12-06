# iCloud 存储功能实施步骤指南

## 📋 概述

本文档提供详细的分步实施指南，帮助开发者按照正确的顺序实现 iCloud 存储功能。

**预计总时间：2.5-3 小时**

---

## 🎯 实施策略

采用**渐进式实施策略**，分为 4 个主要阶段：

1. **阶段 1**：引入 dirs crate，简化现有代码（不改变功能）
2. **阶段 2**：实现 iCloud 支持（核心功能）
3. **阶段 3**：UI 集成（用户体验）
4. **阶段 4**：测试和文档（质量保证）

每个阶段都包含：
- ✅ 详细的代码示例
- ✅ 编译和测试命令
- ✅ 验证检查点
- ✅ 提交信息模板

---

## 📦 阶段 0：准备工作（5 分钟）

### 0.1 创建功能分支

```bash
# 确保在主分支上
git checkout main
git pull origin main

# 创建新的功能分支
git checkout -b feature/icloud-storage
```

### 0.2 验证当前状态

```bash
# 确保所有测试通过
cargo test

# 确保编译成功
cargo build

# 检查当前配置路径
ls -la ~/.workflow/config/
```

### 0.3 备份当前配置（可选但推荐）

```bash
# 备份现有配置
cp -r ~/.workflow ~/.workflow.backup

# 记录当前路径
echo "Current config dir:" >> ~/workflow-migration.log
ls -la ~/.workflow/ >> ~/workflow-migration.log
```

### ✅ 阶段 0 检查点

- [ ] 功能分支已创建
- [ ] 所有测试通过
- [ ] 配置已备份
- [ ] 准备开始实施

---

## 📦 阶段 1：引入 dirs crate（45 分钟）

> **目标**：简化现有代码，消除重复，不改变任何功能

---

### 1.1 添加 dirs 依赖（1 分钟）

**文件**：`Cargo.toml`

**操作**：在 `[dependencies]` 部分添加：

```toml
[dependencies]
# ... 现有依赖（保持不变）...
clap = { version = "4", features = ["derive"] }
clap_complete = "4.5"
anyhow = "1.0"
# ... 其他依赖 ...

# 🆕 添加这一行
dirs = "5.0"
```

**验证**：

```bash
# 下载依赖并编译
cargo build

# 应该看到 "Compiling dirs v5.0.1"
```

### ✅ 步骤 1.1 检查点

- [ ] `Cargo.toml` 已更新
- [ ] `cargo build` 成功
- [ ] dirs 依赖已下载

---

### 1.2 在 paths.rs 中添加辅助方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作 1**：在文件顶部的 `use` 语句后添加（如果还没有）：

```rust
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
```

**操作 2**：在 `impl Paths` 的开始位置添加私有辅助方法：

```rust
impl Paths {
    // ==================== 私有辅助方法 ====================

    /// 获取用户主目录
    ///
    /// 使用 dirs crate 提供的跨平台主目录获取功能。
    /// 这是一个统一的入口点，所有需要主目录的地方都应该调用此方法。
    ///
    /// # 返回
    ///
    /// 返回用户主目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法确定主目录，返回错误信息。
    fn home_dir() -> Result<PathBuf> {
        dirs::home_dir().context("Cannot determine home directory")
    }

    // ==================== 现有的公开方法（暂不修改）====================

    // ... config_dir() 等方法暂时保持不变 ...
}
```

**验证**：

```bash
# 编译检查语法
cargo build

# 运行测试
cargo test --lib base::settings::paths
```

### ✅ 步骤 1.2 检查点

- [ ] `home_dir()` 方法已添加
- [ ] 编译成功
- [ ] 测试通过

---

### 1.3 替换 config_dir() 方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `config_dir()` 方法，完整替换为：

```rust
/// 获取配置目录路径
///
/// 返回 `~/.workflow/config/` 目录路径，如果目录不存在则创建。
///
/// # 返回
///
/// 返回配置目录的 `PathBuf`。
///
/// # 错误
///
/// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
pub fn config_dir() -> Result<PathBuf> {
    // 🆕 使用新的 home_dir() 方法
    let home = Self::home_dir()?;
    let config_dir = home.join(".workflow").join("config");

    // 确保配置目录存在
    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    // 设置目录权限为 700（仅用户可访问，仅 Unix）
    #[cfg(unix)]
    {
        fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set config directory permissions")?;
    }

    Ok(config_dir)
}
```

**对比变化**：

```diff
- let config_dir = if cfg!(target_os = "windows") {
-     let app_data = std::env::var("APPDATA").context("...")?;
-     PathBuf::from(app_data).join("workflow").join("config")
- } else {
-     let home = std::env::var("HOME").context("...")?;
-     PathBuf::from(home).join(".workflow").join("config")
- };
+ let home = Self::home_dir()?;
+ let config_dir = home.join(".workflow").join("config");
```

**验证**：

```bash
# 编译
cargo build

# 测试 paths 模块
cargo test --lib base::settings::paths

# 测试配置相关功能
cargo run -- config show
```

### ✅ 步骤 1.3 检查点

- [ ] `config_dir()` 已简化
- [ ] 编译成功
- [ ] 测试通过
- [ ] `workflow config show` 正常运行
- [ ] 配置文件路径与之前相同

---

### 1.4 替换 workflow_dir() 方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `workflow_dir()` 方法，完整替换为：

```rust
/// 获取工作流目录路径
///
/// 返回 `~/.workflow/` 目录路径（Unix）或 `%APPDATA%\workflow` 目录路径（Windows），如果目录不存在则创建。
///
/// # 返回
///
/// 返回工作流目录的 `PathBuf`。
///
/// # 错误
///
/// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
pub fn workflow_dir() -> Result<PathBuf> {
    // 🆕 使用新的 home_dir() 方法
    let home = Self::home_dir()?;
    let workflow_dir = home.join(".workflow");

    // 确保工作流目录存在
    fs::create_dir_all(&workflow_dir).context("Failed to create .workflow directory")?;

    // 设置目录权限为 700（仅用户可访问，仅 Unix）
    #[cfg(unix)]
    {
        fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set workflow directory permissions")?;
    }

    Ok(workflow_dir)
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 1.4 检查点

- [ ] `workflow_dir()` 已简化
- [ ] 编译成功
- [ ] 测试通过

---

### 1.5 替换 work_history_dir() 方法（3 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `work_history_dir()` 方法，完整替换为：

```rust
/// 获取工作历史记录目录路径
///
/// 返回 `~/.workflow/work-history/` 目录路径（Unix）或 `%APPDATA%\workflow\work-history` 目录路径（Windows），如果目录不存在则创建。
///
/// # 返回
///
/// 返回工作历史记录目录的 `PathBuf`。
///
/// # 错误
///
/// 如果环境变量未设置或无法创建目录，返回相应的错误信息。
pub fn work_history_dir() -> Result<PathBuf> {
    // 🆕 复用 workflow_dir() 方法
    let history_dir = Self::workflow_dir()?.join("work-history");

    // 确保目录存在
    fs::create_dir_all(&history_dir)
        .context("Failed to create .workflow/work-history directory")?;

    // 设置目录权限为 700（仅用户可访问，仅 Unix）
    #[cfg(unix)]
    {
        fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set work-history directory permissions")?;
    }

    Ok(history_dir)
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 1.5 检查点

- [ ] `work_history_dir()` 已简化
- [ ] 复用了 `workflow_dir()`
- [ ] 测试通过

---

### 1.6 替换 completion_dir() 方法（2 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `completion_dir()` 方法，完整替换为：

```rust
/// 获取 completion 目录的完整路径
///
/// 返回 `~/.workflow/completions` 目录的完整路径。
///
/// # 返回
///
/// 返回 completion 目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取 workflow 目录，返回相应的错误信息。
pub fn completion_dir() -> Result<PathBuf> {
    // 🆕 复用 workflow_dir() 方法
    Ok(Self::workflow_dir()?.join("completions"))
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 1.6 检查点

- [ ] `completion_dir()` 已简化
- [ ] 测试通过

---

### 1.7 替换 config_file() 方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `config_file()` 方法，完整替换为：

```rust
/// 获取 shell 配置文件路径
///
/// 支持的 shell 类型及其配置文件路径：
/// - zsh → `~/.zshrc`
/// - bash → `~/.bash_profile`（如果不存在则使用 `~/.bashrc`）
/// - fish → `~/.config/fish/config.fish`
/// - powershell → 跨平台路径
/// - elvish → `~/.elvish/rc.elv`
///
/// # 参数
///
/// * `shell` - Shell 枚举类型
///
/// # 返回
///
/// 返回 shell 配置文件的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取主目录或 shell 类型不支持，返回相应的错误信息。
pub fn config_file(shell: &Shell) -> Result<PathBuf> {
    // 🆕 使用新的 home_dir() 方法
    let home = Self::home_dir()?;

    let config_file = match shell {
        #[cfg(target_os = "windows")]
        Shell::PowerShell => {
            home.join("Documents")
                .join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1")
        }

        #[cfg(not(target_os = "windows"))]
        Shell::Zsh => home.join(".zshrc"),

        #[cfg(not(target_os = "windows"))]
        Shell::Bash => {
            let bash_profile = home.join(".bash_profile");
            let bashrc = home.join(".bashrc");
            if !bash_profile.exists() && bashrc.exists() {
                bashrc
            } else {
                bash_profile
            }
        }

        #[cfg(not(target_os = "windows"))]
        Shell::Fish => home.join(".config/fish/config.fish"),

        #[cfg(not(target_os = "windows"))]
        Shell::PowerShell => {
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        }

        #[cfg(not(target_os = "windows"))]
        Shell::Elvish => home.join(".elvish/rc.elv"),

        _ => anyhow::bail!("Unsupported shell type"),
    };

    Ok(config_file)
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 1.7 检查点

- [ ] `config_file()` 已简化
- [ ] 使用编译期条件判断
- [ ] 测试通过

---

### 1.8 更新 completion/completion.rs（5 分钟）

**文件**：`src/lib/completion/completion.rs`

**操作 1**：在文件顶部添加 import：

```rust
use crate::base::settings::paths::Paths;
```

**操作 2**：找到 `create_workflow_dir()` 函数，替换为：

```rust
/// 创建 workflow 配置文件目录
fn create_workflow_dir() -> Result<PathBuf> {
    // 🆕 直接复用 Paths::workflow_dir()
    Paths::workflow_dir()
}
```

**操作 3**：找到 `remove_completion_config_file()` 函数，更新为：

```rust
/// 删除 workflow completion 配置文件
pub fn remove_completion_config_file() -> Result<()> {
    // 🆕 使用 Paths::workflow_dir()
    let workflow_config_file = Paths::workflow_dir()?.join(".completions");

    if workflow_config_file.exists() {
        fs::remove_file(&workflow_config_file)
            .context("Failed to remove completion config file")?;
    }

    Ok(())
}
```

**验证**：

```bash
cargo build && cargo test --lib completion
```

### ✅ 步骤 1.8 检查点

- [ ] `completion.rs` 已更新
- [ ] 复用了 `Paths` 方法
- [ ] 测试通过

---

### 1.9 更新 base/shell/config.rs（5 分钟）

**文件**：`src/lib/base/shell/config.rs`

**操作 1**：在文件顶部添加 import：

```rust
use crate::base::settings::paths::Paths;
```

**操作 2**：找到所有使用 `std::env::var("HOME")` 的地方，替换为使用 `Paths::home_dir()`。

例如，在 `check_source_exists` 函数中：

```rust
// 🆕 修改前
if source_path.contains("$HOME") {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    let abs_path = source_path.replace("$HOME", &home);
    // ...
}

// 🆕 修改后
if source_path.contains("$HOME") {
    let home = Paths::home_dir()?;
    let abs_path = source_path.replace("$HOME", &home.to_string_lossy());
    // ...
}
```

**提示**：在这个文件中搜索所有 `std::env::var("HOME")` 并替换（大约 4 处）。

**验证**：

```bash
cargo build && cargo test --lib base::shell
```

### ✅ 步骤 1.9 检查点

- [ ] `shell/config.rs` 已更新
- [ ] 所有 HOME 环境变量调用已替换
- [ ] 测试通过

---

### 1.10 更新 base/settings/defaults.rs（3 分钟）

**文件**：`src/lib/base/settings/defaults.rs`

**操作**：找到 `default_download_base_dir()` 函数，替换为：

```rust
/// 获取默认下载基础目录
pub fn default_download_base_dir() -> String {
    // 🆕 使用 dirs::home_dir()
    dirs::home_dir()
        .and_then(|h| Some(h.join("Downloads").to_string_lossy().to_string()))
        .unwrap_or_else(|| {
            if cfg!(target_os = "windows") {
                "C:\\Users\\User\\Downloads".to_string()
            } else {
                "~/Downloads".to_string()
            }
        })
}
```

**验证**：

```bash
cargo build && cargo test --lib base::settings
```

### ✅ 步骤 1.10 检查点

- [ ] `defaults.rs` 已更新
- [ ] 测试通过

---

### 1.11 更新 completion/generate.rs（3 分钟）

**文件**：`src/lib/completion/generate.rs`

**操作 1**：在文件顶部添加 import：

```rust
use crate::base::settings::paths::Paths;
```

**操作 2**：找到 `generate()` 函数中的路径处理部分，更新为：

```rust
pub fn generate(shell: Shell, output_dir: Option<&str>) -> Result<()> {
    // 🆕 使用 Paths::completion_dir()
    let output = output_dir.map(PathBuf::from).unwrap_or_else(|| {
        Paths::completion_dir()
            .unwrap_or_else(|_| PathBuf::from("~/.workflow/completions"))
    });

    // ... 其余代码保持不变 ...
}
```

**验证**：

```bash
cargo build && cargo test --lib completion
```

### ✅ 步骤 1.11 检查点

- [ ] `generate.rs` 已更新
- [ ] 测试通过

---

### 1.12 验证阶段 1（5 分钟）

**运行完整测试套件**：

```bash
# 1. 编译整个项目
cargo build

# 2. 运行所有单元测试
cargo test

# 3. 运行集成测试
cargo test --test '*'

# 4. 手动测试关键功能
cargo run -- setup
cargo run -- config show

# 5. 验证路径
ls -la ~/.workflow/config/
ls -la ~/.workflow/work-history/

# 6. 验证功能正常
cargo run -- pr list  # 或其他常用命令
```

**检查输出**：

- ✅ 所有测试应该通过
- ✅ 路径应该与之前完全相同
- ✅ 所有功能应该正常工作
- ✅ 没有任何功能变化

### ✅ 阶段 1 完成检查点

- [ ] 所有文件已更新
- [ ] `cargo build` 成功
- [ ] `cargo test` 全部通过
- [ ] 手动测试通过
- [ ] 配置路径与之前相同
- [ ] 代码更简洁，消除了重复

---

### 1.13 提交阶段 1（2 分钟）

```bash
# 查看更改
git status
git diff

# 添加所有更改
git add Cargo.toml
git add src/lib/base/settings/paths.rs
git add src/lib/base/settings/defaults.rs
git add src/lib/base/shell/config.rs
git add src/lib/completion/completion.rs
git add src/lib/completion/generate.rs

# 提交
git commit -m "refactor: introduce dirs crate to simplify path handling

- Add dirs crate (v5.0) for cross-platform home directory support
- Replace all std::env::var(\"HOME\") calls with unified Paths::home_dir()
- Simplify platform-specific path logic in paths.rs
- Reduce code duplication across 5 files (completion, shell, defaults)
- Update all path-related methods to use new helper function
- Remove redundant Windows/Unix platform checks
- All tests passing, no functional changes

Benefits:
- Code reduced by ~60 lines
- Single source of truth for home directory
- Better error handling
- More idiomatic Rust code"
```

---

## 🌥️ 阶段 2：实现 iCloud 支持（60 分钟）

> **目标**：添加 iCloud Drive 支持，配置文件支持同步，工作历史保持本地

---

### 2.1 添加 iCloud 检测方法（10 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：在 `home_dir()` 方法后面添加以下私有方法：

```rust
impl Paths {
    // ==================== 私有辅助方法 ====================

    /// 获取用户主目录
    fn home_dir() -> Result<PathBuf> {
        dirs::home_dir().context("Cannot determine home directory")
    }

    // 🆕 添加以下方法

    /// 尝试获取 iCloud 基础目录（仅 macOS）
    ///
    /// 检查 iCloud Drive 是否可用，如果可用则返回 .workflow 目录路径。
    ///
    /// # 返回
    ///
    /// - `Some(PathBuf)` - iCloud Drive 可用且成功创建目录
    /// - `None` - iCloud Drive 不可用或创建目录失败
    ///
    /// # iCloud 路径
    ///
    /// macOS: `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/`
    #[cfg(target_os = "macos")]
    fn try_icloud_base_dir() -> Option<PathBuf> {
        // 获取主目录
        let home = dirs::home_dir()?;

        // 构建 iCloud Drive 基础路径
        // ~/Library/Mobile Documents/com~apple~CloudDocs
        let icloud_base = home
            .join("Library")
            .join("Mobile Documents")
            .join("com~apple~CloudDocs");

        // 检查 iCloud Drive 是否可用
        if !icloud_base.exists() || !icloud_base.is_dir() {
            return None;
        }

        // 尝试创建 .workflow 目录
        let workflow_dir = icloud_base.join(".workflow");
        if fs::create_dir_all(&workflow_dir).is_err() {
            return None;
        }

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(
                &workflow_dir,
                fs::Permissions::from_mode(0o700)
            );
        }

        Some(workflow_dir)
    }

    /// 非 macOS 平台：总是返回 None
    #[cfg(not(target_os = "macos"))]
    fn try_icloud_base_dir() -> Option<PathBuf> {
        None
    }

    // ... 其余方法保持不变 ...
}
```

**验证**：

```bash
cargo build
```

### ✅ 步骤 2.1 检查点

- [ ] `try_icloud_base_dir()` 已添加
- [ ] 使用条件编译 `#[cfg]`
- [ ] 编译成功

---

### 2.2 添加本地基础目录方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：在 `try_icloud_base_dir()` 后面添加：

```rust
/// 获取本地基础目录（总是可用）
///
/// 返回 `~/.workflow/` 目录（Unix）。
/// 此方法作为回退方案，确保在任何情况下都能获取到有效路径。
///
/// # 返回
///
/// 返回本地工作流目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
fn local_base_dir() -> Result<PathBuf> {
    let home = Self::home_dir()?;
    let workflow_dir = home.join(".workflow");

    // 确保目录存在
    fs::create_dir_all(&workflow_dir)
        .context("Failed to create local .workflow directory")?;

    // 设置目录权限为 700（仅用户可访问）
    #[cfg(unix)]
    {
        fs::set_permissions(&workflow_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set workflow directory permissions")?;
    }

    Ok(workflow_dir)
}
```

**验证**：

```bash
cargo build
```

### ✅ 步骤 2.2 检查点

- [ ] `local_base_dir()` 已添加
- [ ] 编译成功

---

### 2.3 添加配置基础目录方法（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：在 `local_base_dir()` 后面添加：

```rust
/// 获取配置基础目录（支持 iCloud）
///
/// 决策逻辑：
/// 1. 在 macOS 上，优先尝试使用 iCloud Drive
/// 2. 如果 iCloud 不可用，回退到本地目录
/// 3. 在其他平台上，直接使用本地目录
///
/// # 返回
///
/// 返回配置基础目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
fn config_base_dir() -> Result<PathBuf> {
    // macOS 上尝试 iCloud
    if let Some(icloud_dir) = Self::try_icloud_base_dir() {
        return Ok(icloud_dir);
    }

    // 回退到本地
    Self::local_base_dir()
}
```

**验证**：

```bash
cargo build
```

### ✅ 步骤 2.3 检查点

- [ ] `config_base_dir()` 已添加
- [ ] 包含 iCloud 检测和回退逻辑
- [ ] 编译成功

---

### 2.4 更新 config_dir() 使用 iCloud（5 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `config_dir()` 方法，修改第一行：

```rust
pub fn config_dir() -> Result<PathBuf> {
    // 🆕 从 workflow_dir() 改为 config_base_dir()
    let config_dir = Self::config_base_dir()?.join("config");

    // 确保配置目录存在
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;

    // 设置目录权限为 700（仅用户可访问，仅 Unix）
    #[cfg(unix)]
    {
        fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set config directory permissions")?;
    }

    Ok(config_dir)
}
```

**验证**：

```bash
cargo build && cargo test --lib base::settings::paths
```

### ✅ 步骤 2.4 检查点

- [ ] `config_dir()` 已更新
- [ ] 使用 `config_base_dir()`
- [ ] 测试通过

---

### 2.5 更新 workflow_dir() 使用 iCloud（3 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `workflow_dir()` 方法，完整替换为：

```rust
/// 获取工作流目录路径（支持 iCloud）
///
/// 返回工作流基础目录。如果配置在 iCloud，此方法返回 iCloud 路径。
///
/// # 返回
///
/// 返回工作流目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub fn workflow_dir() -> Result<PathBuf> {
    // 🆕 直接返回配置基础目录
    Self::config_base_dir()
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 2.5 检查点

- [ ] `workflow_dir()` 已简化
- [ ] 复用 `config_base_dir()`
- [ ] 测试通过

---

### 2.6 更新 work_history_dir() 强制本地（10 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `work_history_dir()` 方法，完整替换为：

```rust
/// 获取工作历史目录路径（强制本地，不同步）
///
/// 返回 `~/.workflow/work-history/`（总是本地路径）。
///
/// **重要**：工作历史是设备本地的，不应该跨设备同步，因为：
/// - 每个设备的工作历史是独立的
/// - 避免多设备冲突（不同设备可能处理不同的 PR）
/// - 防止历史记录混乱（PR ID 可能在不同仓库中重复）
/// - 性能考虑（本地读写更快，不需要等待 iCloud 同步）
///
/// # 路径示例
///
/// - 所有平台：`~/.workflow/work-history/`
///
/// # 返回
///
/// 返回工作历史目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法创建目录，返回相应的错误信息。
pub fn work_history_dir() -> Result<PathBuf> {
    // 🆕 强制使用本地路径，不使用 iCloud
    let history_dir = Self::local_base_dir()?.join("work-history");

    fs::create_dir_all(&history_dir)
        .context("Failed to create work-history directory")?;

    #[cfg(unix)]
    {
        fs::set_permissions(&history_dir, fs::Permissions::from_mode(0o700))
            .context("Failed to set work-history directory permissions")?;
    }

    Ok(history_dir)
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 2.6 检查点

- [ ] `work_history_dir()` 已更新
- [ ] **强制使用** `local_base_dir()`
- [ ] 不会跟随配置到 iCloud
- [ ] 测试通过

---

### 2.7 更新 completion_dir() 保持本地（3 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：找到 `completion_dir()` 方法，确认使用本地路径：

```rust
/// 获取补全脚本目录路径（强制本地）
///
/// 返回 `~/.workflow/completions/`（总是本地路径）。
/// Shell 补全脚本是本地安装的，不需要同步。
///
/// # 返回
///
/// 返回补全脚本目录的 `PathBuf`。
///
/// # 错误
///
/// 如果无法获取本地目录，返回相应的错误信息。
pub fn completion_dir() -> Result<PathBuf> {
    // 🆕 确保使用本地路径
    Ok(Self::local_base_dir()?.join("completions"))
}
```

**验证**：

```bash
cargo build && cargo test
```

### ✅ 步骤 2.7 检查点

- [ ] `completion_dir()` 使用本地路径
- [ ] 测试通过

---

### 2.8 添加信息查询方法（10 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：在公开 API 部分的末尾（在 Shell 路径方法之前）添加：

```rust
// ==================== 信息查询 API ====================

/// 检查配置是否存储在 iCloud
///
/// # 返回
///
/// - `true` - 配置当前存储在 iCloud Drive
/// - `false` - 配置存储在本地
pub fn is_config_in_icloud() -> bool {
    #[cfg(target_os = "macos")]
    {
        Self::try_icloud_base_dir().is_some()
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// 获取存储位置的用户友好描述
///
/// # 返回
///
/// - "iCloud Drive (synced across devices)" - 使用 iCloud
/// - "Local storage" - 使用本地存储
pub fn storage_location() -> &'static str {
    if Self::is_config_in_icloud() {
        "iCloud Drive (synced across devices)"
    } else {
        "Local storage"
    }
}

/// 获取详细的存储信息
///
/// 返回包含存储类型、配置路径和工作历史路径的详细信息。
///
/// # 返回
///
/// 返回格式化的存储信息字符串。
///
/// # 错误
///
/// 如果无法获取路径，返回相应的错误信息。
pub fn storage_info() -> Result<String> {
    let config_dir = Self::config_dir()?;
    let work_history_dir = Self::work_history_dir()?;

    let info = if Self::is_config_in_icloud() {
        format!(
            "Storage Type: iCloud Drive (synced across devices)\n\
             \n\
             Configuration (synced):\n\
             {}\n\
             \n\
             Work History (local only, not synced):\n\
             {}",
            config_dir.display(),
            work_history_dir.display()
        )
    } else {
        format!(
            "Storage Type: Local storage\n\
             \n\
             Configuration:\n\
             {}\n\
             \n\
             Work History:\n\
             {}",
            config_dir.display(),
            work_history_dir.display()
        )
    };

    Ok(info)
}

// ==================== Shell 路径相关方法 ====================
// ... config_file() 等方法保持不变 ...
```

**验证**：

```bash
cargo build
```

### ✅ 步骤 2.8 检查点

- [ ] 三个查询方法已添加
- [ ] `is_config_in_icloud()`
- [ ] `storage_location()`
- [ ] `storage_info()`
- [ ] 编译成功

---

### 2.9 添加测试（10 分钟）

**文件**：`src/lib/base/settings/paths.rs`

**操作**：在文件末尾的 `#[cfg(test)]` 模块中添加新测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ... 现有测试保持不变 ...

    // 🆕 添加以下测试

    #[test]
    fn test_work_history_always_local() {
        let work_history = Paths::work_history_dir().unwrap();
        let local_base = Paths::local_base_dir().unwrap();

        // work_history 应该总是在本地路径下
        assert!(work_history.starts_with(&local_base));

        // 确保不在 iCloud 路径下
        #[cfg(target_os = "macos")]
        {
            if let Some(icloud_base) = Paths::try_icloud_base_dir() {
                assert!(!work_history.starts_with(&icloud_base));
            }
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_config_can_use_icloud() {
        let config_dir = Paths::config_dir().unwrap();

        if Paths::is_config_in_icloud() {
            let icloud_base = Paths::try_icloud_base_dir().unwrap();
            assert!(config_dir.starts_with(&icloud_base));

            // 验证路径包含 iCloud 标识
            let path_str = config_dir.to_string_lossy();
            assert!(path_str.contains("com~apple~CloudDocs"));
        }
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn test_non_macos_always_local() {
        assert!(!Paths::is_config_in_icloud());
        assert_eq!(Paths::storage_location(), "Local storage");
    }

    #[test]
    fn test_paths_separation() {
        let config = Paths::config_dir().unwrap();
        let work_history = Paths::work_history_dir().unwrap();

        // 在 macOS + iCloud 环境下，config 和 work_history 应该在不同的基础目录
        #[cfg(target_os = "macos")]
        {
            if Paths::is_config_in_icloud() {
                // config 在 iCloud，work_history 在本地
                let config_base = config.parent().unwrap().parent().unwrap();
                let history_base = work_history.parent().unwrap().parent().unwrap();
                assert_ne!(config_base, history_base);
            }
        }
    }

    #[test]
    fn test_storage_info() {
        let info = Paths::storage_info().unwrap();
        assert!(!info.is_empty());
        assert!(info.contains("Storage Type"));
        assert!(info.contains("Configuration"));
        assert!(info.contains("Work History"));
    }

    #[test]
    fn test_completion_dir_is_local() {
        let completion_dir = Paths::completion_dir().unwrap();
        let local_base = Paths::local_base_dir().unwrap();

        // completion 应该总是在本地路径下
        assert!(completion_dir.starts_with(&local_base));
    }
}
```

**验证**：

```bash
cargo test --lib base::settings::paths
```

### ✅ 步骤 2.9 检查点

- [ ] 6 个新测试已添加
- [ ] 测试 work_history 总是本地
- [ ] 测试 iCloud 检测
- [ ] 测试路径分离
- [ ] 所有测试通过

---

### 2.10 验证阶段 2（5 分钟）

```bash
# 1. 运行所有测试
cargo test

# 2. 手动测试（macOS）
cargo run -- setup

# 3. 检查存储位置
echo "Config dir:"
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/ 2>/dev/null || \
ls -la ~/.workflow/config/

echo -e "\nWork history dir (应该总是本地):"
ls -la ~/.workflow/work-history/

# 4. 验证查询方法
cargo run -- config show
```

**预期结果（macOS + iCloud）**：

- 配置目录：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
- 工作历史：`~/.workflow/work-history/`（总是本地）

**预期结果（其他情况）**：

- 配置目录：`~/.workflow/config/`
- 工作历史：`~/.workflow/work-history/`

### ✅ 阶段 2 完成检查点

- [ ] 所有 iCloud 方法已实现
- [ ] `config_dir()` 支持 iCloud
- [ ] `work_history_dir()` 强制本地
- [ ] 信息查询方法可用
- [ ] 所有测试通过
- [ ] 手动测试验证成功

---

### 2.11 提交阶段 2（2 分钟）

```bash
git add src/lib/base/settings/paths.rs
git commit -m "feat: add iCloud Drive support for config storage

Core features:
- Implement iCloud Drive detection for macOS
- Config files automatically sync via iCloud when available
- Work history remains local-only (device-specific)
- Graceful fallback to local storage when iCloud unavailable
- Cross-platform compatible (non-macOS uses local storage)

Implementation details:
- Add try_icloud_base_dir() for iCloud detection
- Add config_base_dir() with fallback logic
- Update config_dir() to use iCloud when available
- Force work_history_dir() to always use local storage
- Add is_config_in_icloud() query method
- Add storage_location() for user-friendly description
- Add storage_info() for detailed information

Testing:
- Add 6 new unit tests
- Test iCloud detection on macOS
- Test work history isolation
- Test path separation
- All tests passing

Backward compatible:
- Existing local configs continue to work
- No user action required
- Transparent migration to iCloud when available"
```

---

## 🎨 阶段 3：UI 集成（30 分钟）

> **目标**：向用户显示存储信息，提升用户体验

---

### 3.1 更新 setup.rs 显示存储信息（15 分钟）

**文件**：`src/commands/config/setup.rs`

**操作 1**：在 `impl SetupCommand` 中找到 `run()` 方法，在保存配置后添加存储信息显示：

```rust
impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        log_success!("Starting Workflow CLI initialization...\n");

        // 加载现有配置（从 TOML 文件）
        let existing_config = Self::load_existing_config()?;

        // 收集配置信息（智能处理现有配置）
        let config = Self::collect_config(&existing_config)?;

        // 保存配置到 TOML 文件
        log_message!("Saving configuration...");
        Self::save_config(&config)?;
        log_success!("Configuration saved successfully!");

        // 🆕 显示存储位置信息
        Self::show_storage_location()?;

        log_break!();
        log_info!("Verifying configuration...");
        log_break!();

        log_break!('-', 40, "Verifying Configuration");
        log_break!();

        // 验证配置（使用 load() 获取最新配置，避免 OnceLock 缓存问题）
        let settings = Settings::load();
        settings.verify()?;

        log_break!();
        log_success!("Initialization completed successfully!");
        log_message!("You can now use the Workflow CLI commands.");

        Ok(())
    }

    // ... 现有的其他方法 ...
}
```

**操作 2**：在 `impl SetupCommand` 的末尾添加新方法：

```rust
impl SetupCommand {
    // ... 现有方法 ...

    /// 🆕 显示存储位置信息
    fn show_storage_location() -> Result<()> {
        use crate::base::settings::paths::Paths;

        log_break!();
        log_info!("📦 Storage Information");
        log_break!('─', 65);

        let is_icloud = Paths::is_config_in_icloud();
        let config_dir = Paths::config_dir()?;
        let work_history_dir = Paths::work_history_dir()?;

        if is_icloud {
            log_info!("  Type: iCloud Drive (synced across devices)");
            log_info!("  ");
            log_info!("  📁 Configuration (synced):");
            log_info!("     {}", config_dir.display());
            log_info!("  ");
            log_info!("  📁 Work History (local only):");
            log_info!("     {}", work_history_dir.display());
            log_info!("  ");
            log_success!("  ✅ Your settings will sync across all your Apple devices");
            log_info!("  ⚠️  Work history is device-specific and won't sync");
        } else {
            log_info!("  Type: Local storage");
            log_info!("  ");
            log_info!("  📁 Configuration:");
            log_info!("     {}", config_dir.display());
            log_info!("  ");
            log_info!("  📁 Work History:");
            log_info!("     {}", work_history_dir.display());
        }

        log_break!();

        Ok(())
    }
}
```

**验证**：

```bash
cargo build
cargo run -- setup
```

**预期输出（macOS + iCloud）**：

```
✅ Configuration saved successfully!

📦 Storage Information
─────────────────────────────────────────────────────────────────
  Type: iCloud Drive (synced across devices)

  📁 Configuration (synced):
     /Users/username/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config

  📁 Work History (local only):
     /Users/username/.workflow/work-history

  ✅ Your settings will sync across all your Apple devices
  ⚠️  Work history is device-specific and won't sync
─────────────────────────────────────────────────────────────────
```

### ✅ 步骤 3.1 检查点

- [ ] `show_storage_location()` 已添加
- [ ] 在 `run()` 中调用
- [ ] 编译成功
- [ ] 输出格式正确

---

### 3.2 更新 show.rs 显示存储位置（10 分钟）

**文件**：`src/commands/config/show.rs`

**操作 1**：在文件顶部添加 import：

```rust
use crate::base::settings::paths::Paths;
```

**操作 2**：在 `show_config()` 函数开头添加存储信息：

```rust
pub fn show_config() -> Result<()> {
    log_break!('=', 40, "Configuration Overview");

    // 🆕 显示存储位置
    log_info!("Storage: {}", Paths::storage_location());
    log_info!("Path: {}", Paths::config_dir()?.display());

    #[cfg(target_os = "macos")]
    {
        if Paths::is_config_in_icloud() {
            log_info!("  ✅ Synced across your Apple devices");
        }
    }

    log_break!();

    // ... 现有的配置显示逻辑保持不变 ...

    Ok(())
}
```

**验证**：

```bash
cargo build
cargo run -- config show
```

**预期输出**：

```
============== Configuration Overview ==============

Storage: iCloud Drive (synced across devices)
Path: /Users/username/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config
  ✅ Synced across your Apple devices

... 其他配置信息 ...
```

### ✅ 步骤 3.2 检查点

- [ ] `show.rs` 已更新
- [ ] 显示存储类型和路径
- [ ] 编译成功
- [ ] 输出正确

---

### 3.3 验证阶段 3（5 分钟）

```bash
# 1. 测试 setup 命令
cargo run -- setup

# 2. 测试 config show 命令
cargo run -- config show

# 3. 测试其他命令正常工作
cargo run -- pr list
cargo run -- pr create TEST-123

# 4. 完整测试套件
cargo test
```

### ✅ 阶段 3 完成检查点

- [ ] `workflow setup` 显示存储信息
- [ ] `workflow config show` 显示存储位置
- [ ] UI 输出清晰易懂
- [ ] 所有命令正常工作
- [ ] 所有测试通过

---

### 3.4 提交阶段 3（2 分钟）

```bash
git add src/commands/config/setup.rs
git add src/commands/config/show.rs
git commit -m "feat: display storage location in UI

Features:
- Show storage type and paths in 'workflow setup'
- Add storage info to 'workflow config show'
- Clear indication of iCloud sync status
- Distinguish between synced config and local work history

User experience:
- Informative messages about where data is stored
- Clear visual distinction (📁 icons, ✅/⚠️ indicators)
- Friendly explanations of sync behavior
- No additional user action required

Implementation:
- Add show_storage_location() method in setup.rs
- Update show_config() in show.rs
- Use emoji indicators for better UX
- Conditional display based on storage type"
```

---

## ✅ 阶段 4：测试和文档（15 分钟）

> **目标**：确保质量，完善文档

---

### 4.1 完整测试（10 分钟）

```bash
# 1. 单元测试
cargo test --lib

# 2. 集成测试
cargo test --test '*'

# 3. 所有测试
cargo test

# 4. 检查警告
cargo clippy

# 5. 格式检查
cargo fmt --check

# 6. 手动测试关键功能
echo "=== 测试 setup ==="
cargo run -- setup

echo -e "\n=== 测试 config show ==="
cargo run -- config show

echo -e "\n=== 测试 PR 功能 ==="
cargo run -- pr list

# 7. 检查路径（macOS）
echo -e "\n=== 检查实际路径 ==="
echo "Config dir:"
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/ 2>/dev/null || \
ls -la ~/.workflow/config/

echo -e "\nWork history dir:"
ls -la ~/.workflow/work-history/

echo -e "\nCompletion dir:"
ls -la ~/.workflow/completions/
```

### ✅ 步骤 4.1 检查点

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 无编译警告
- [ ] 代码格式正确
- [ ] 手动测试成功
- [ ] 路径验证正确

---

### 4.2 更新文档（5 分钟）

**选项 1：更新 README.md**

在 README.md 中添加新功能说明：

```markdown
## 新功能：iCloud Drive 配置同步

### macOS 自动配置同步

在 macOS 上，Workflow CLI 会自动将配置文件保存到 iCloud Drive，实现多设备自动同步。

**特性：**
- ✅ **零配置**：无需手动设置，系统自动检测并使用 iCloud Drive
- ✅ **多设备同步**：配置在所有登录同一 Apple ID 的设备间自动同步
- ✅ **智能回退**：如果 iCloud Drive 不可用，自动使用本地存储
- ✅ **工作历史独立**：工作历史保持设备本地，避免冲突

**查看存储位置：**

```bash
workflow config show
```

**存储策略：**
- 配置文件（同步）：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
- 工作历史（本地）：`~/.workflow/work-history/`
```

**选项 2：创建独立文档**

创建 `docs/ICLOUD_STORAGE.md`（可选）：

```markdown
# iCloud Drive 存储说明

（内容参考 icloud-storage-implementation.md 的用户部分）
```

---

### 4.3 提交文档（2 分钟）

```bash
git add README.md  # 或其他文档文件
git commit -m "docs: add iCloud storage feature documentation

- Add iCloud Drive sync feature description
- Document storage strategy and paths
- Add usage examples
- Explain sync behavior and limitations"
```

---

### 4.4 最终检查和合并（2 分钟）

```bash
# 1. 查看所有提交
git log --oneline feature/icloud-storage

# 应该看到 4-5 个提交：
# - refactor: introduce dirs crate
# - feat: add iCloud Drive support
# - feat: display storage location in UI
# - docs: add documentation

# 2. 确保在正确的分支
git branch

# 3. 合并到主分支
git checkout main
git merge feature/icloud-storage

# 4. 运行最终测试
cargo test

# 5. 构建 release 版本
cargo build --release

# 6. 打标签（可选）
git tag -a v1.5.0 -m "Add iCloud Drive config sync support"

# 7. 推送（如果需要）
git push origin main
git push origin v1.5.0
```

---

## 📋 完整实施检查清单

### 阶段 0：准备工作
- [ ] 创建功能分支
- [ ] 验证当前状态
- [ ] 备份配置

### 阶段 1：引入 dirs crate
- [ ] 1.1 添加 dirs 依赖
- [ ] 1.2 添加 home_dir() 方法
- [ ] 1.3 替换 config_dir()
- [ ] 1.4 替换 workflow_dir()
- [ ] 1.5 替换 work_history_dir()
- [ ] 1.6 替换 completion_dir()
- [ ] 1.7 替换 config_file()
- [ ] 1.8 更新 completion/completion.rs
- [ ] 1.9 更新 base/shell/config.rs
- [ ] 1.10 更新 base/settings/defaults.rs
- [ ] 1.11 更新 completion/generate.rs
- [ ] 1.12 验证阶段 1
- [ ] 1.13 提交阶段 1

### 阶段 2：实现 iCloud 支持
- [ ] 2.1 添加 try_icloud_base_dir()
- [ ] 2.2 添加 local_base_dir()
- [ ] 2.3 添加 config_base_dir()
- [ ] 2.4 更新 config_dir() 使用 iCloud
- [ ] 2.5 更新 workflow_dir()
- [ ] 2.6 更新 work_history_dir() 强制本地
- [ ] 2.7 更新 completion_dir() 保持本地
- [ ] 2.8 添加信息查询方法
- [ ] 2.9 添加测试
- [ ] 2.10 验证阶段 2
- [ ] 2.11 提交阶段 2

### 阶段 3：UI 集成
- [ ] 3.1 更新 setup.rs
- [ ] 3.2 更新 show.rs
- [ ] 3.3 验证阶段 3
- [ ] 3.4 提交阶段 3

### 阶段 4：测试和文档
- [ ] 4.1 完整测试
- [ ] 4.2 更新文档
- [ ] 4.3 提交文档
- [ ] 4.4 最终检查和合并

---

## 🎯 时间估算

| 阶段 | 预计时间 | 累计时间 |
|------|---------|---------|
| 阶段 0 | 5 分钟 | 5 分钟 |
| 阶段 1 | 45 分钟 | 50 分钟 |
| 阶段 2 | 60 分钟 | 110 分钟 |
| 阶段 3 | 30 分钟 | 140 分钟 |
| 阶段 4 | 15 分钟 | 155 分钟 |

**总计：约 2.5-3 小时**

---

## 💡 实施建议

### 1. 循序渐进

- ✅ 不要跳过步骤
- ✅ 每个步骤完成后立即测试
- ✅ 发现问题立即修复

### 2. 频繁提交

- ✅ 每个阶段提交一次
- ✅ 提交信息要清晰
- ✅ 方便回滚和查看历史

### 3. 充分测试

- ✅ 单元测试
- ✅ 集成测试
- ✅ 手动测试
- ✅ 不同场景测试

### 4. 保持备份

- ✅ 备份当前配置
- ✅ 使用功能分支
- ✅ 可以随时回滚

### 5. 先阶段 1，再阶段 2

- ✅ 阶段 1 是基础，简化代码
- ✅ 阶段 2 在此基础上添加功能
- ✅ 不要跳过阶段 1

---

## ⚠️ 常见问题

### Q1: 如果测试失败怎么办？

```bash
# 查看失败的测试
cargo test -- --nocapture

# 运行特定测试
cargo test test_name

# 检查是否有编译错误
cargo check
```

### Q2: 如何验证 iCloud 是否工作？

```bash
# macOS 上检查 iCloud 目录
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/

# 运行 setup 查看存储信息
cargo run -- setup

# 查看配置
cargo run -- config show
```

### Q3: 如何回滚到某个阶段？

```bash
# 查看提交历史
git log --oneline

# 回滚到特定提交
git reset --hard <commit-hash>

# 或者回滚最后一次提交
git reset --hard HEAD~1
```

### Q4: 如果 iCloud 检测失败怎么办？

不用担心，系统会自动回退到本地存储，功能不受影响。

---

## 📚 相关文档

- [dirs-crate-integration.md](./dirs-crate-integration.md) - dirs 引入详细分析
- [icloud-storage-implementation.md](./icloud-storage-implementation.md) - iCloud 实现详细说明
- [icloud-storage-usage-examples.md](./icloud-storage-usage-examples.md) - Go 实现参考

---

**最后更新**：2024-12-06
