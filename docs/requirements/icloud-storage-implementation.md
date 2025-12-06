# iCloud 存储实现文档

## 📋 概述

本文档详细说明如何在 Workflow 项目中实现 iCloud Drive 配置同步功能，包括设计决策、实现方案和测试策略。

---

## 🎯 设计目标

### 核心需求

1. **自动存储位置选择**：macOS 上优先使用 iCloud Drive，自动回退到本地
2. **分层存储策略**：配置文件同步，工作历史本地
3. **透明使用**：应用代码无需关心具体存储位置
4. **跨平台兼容**：非 macOS 系统使用本地存储
5. **用户友好**：清晰显示存储位置和同步状态

### 设计原则

- ✅ **配置应该同步**：GitHub token、Jira 配置等应在所有设备上一致
- ❌ **工作历史不同步**：每个设备的 PR 工作记录是独立的
- ✅ **优雅降级**：iCloud 不可用时自动使用本地，用户无感知
- ✅ **零配置**：无需用户手动选择，系统自动决策

---

## 📊 存储策略分析

### 目录分类

| 目录/文件 | 同步策略 | 存储位置 | 原因 |
|----------|---------|---------|------|
| `config/` | ✅ 同步 | iCloud（可用时） | 用户配置应多设备共享 |
| `config/workflow.toml` | ✅ 同步 | iCloud | 主配置文件 |
| `config/jira-status.toml` | ✅ 同步 | iCloud | Jira 状态缓存 |
| `config/jira-users.toml` | ✅ 同步 | iCloud | Jira 用户缓存 |
| `config/branch.toml` | ✅ 同步 | iCloud | 分支配置 |
| `work-history/` | ❌ 不同步 | 本地 | 设备特定的工作记录 |
| `completions/` | ❌ 不同步 | 本地 | Shell 补全脚本 |

### 存储路径对比

#### macOS + iCloud 可用

```
配置文件（同步）：
~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/
├── workflow.toml
├── jira-status.toml
├── jira-users.toml
└── branch.toml

工作历史（本地）：
~/.workflow/work-history/
├── github-com-owner-repo1.json
└── github-com-owner-repo2.json

补全脚本（本地）：
~/.workflow/completions/
├── workflow.bash
└── workflow.zsh
```

#### macOS + iCloud 不可用 / 其他系统

```
所有文件（本地）：
~/.workflow/
├── config/
│   ├── workflow.toml
│   ├── jira-status.toml
│   ├── jira-users.toml
│   └── branch.toml
├── work-history/
│   ├── github-com-owner-repo1.json
│   └── github-com-owner-repo2.json
└── completions/
    ├── workflow.bash
    └── workflow.zsh
```

---

## 🏗️ 架构设计

### 分层路径管理

```
┌─────────────────────────────────────────┐
│         应用层（业务代码）               │
│  config::Save(), jira::Cache, etc.      │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│         Paths API（公开接口）           │
│  config_dir(), work_history_dir()       │
└────────────────┬────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────┐
│       路径决策层（私有逻辑）            │
│  config_base_dir(), local_base_dir()    │
└────────────────┬────────────────────────┘
                 │
        ┌────────┴────────┐
        ▼                 ▼
┌──────────────┐  ┌──────────────┐
│  iCloud 路径  │  │  本地路径     │
│ (仅 macOS)   │  │ (所有平台)    │
└──────────────┘  └──────────────┘
```

### 决策流程

```
需要配置目录？
    ↓
调用 config_dir()
    ↓
内部调用 config_base_dir()
    ↓
是 macOS？
    ├─ 是 → try_icloud_base_dir()
    │           ↓
    │       iCloud 可用？
    │           ├─ 是 → 返回 iCloud 路径 ✅
    │           └─ 否 → local_base_dir() → 本地路径 ✅
    │
    └─ 否 → local_base_dir() → 本地路径 ✅

需要工作历史目录？
    ↓
调用 work_history_dir()
    ↓
直接调用 local_base_dir()
    ↓
返回本地路径 ✅（强制本地，永不同步）
```

---

## 💻 具体实现

### 步骤 1：在 `paths.rs` 中实现核心逻辑

```rust
use anyhow::{Context, Result};
use clap_complete::shells::Shell;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 路径管理器
pub struct Paths;

impl Paths {
    // ==================== 私有辅助方法 ====================

    /// 获取用户主目录
    fn home_dir() -> Result<PathBuf> {
        dirs::home_dir().context("Cannot determine home directory")
    }

    /// 尝试获取 iCloud 基础目录（仅 macOS）
    ///
    /// 检查 iCloud Drive 是否可用，如果可用则返回 .workflow 目录路径。
    ///
    /// # 返回
    ///
    /// - `Some(PathBuf)` - iCloud Drive 可用且成功创建目录
    /// - `None` - iCloud Drive 不可用或创建目录失败
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

    /// 获取本地基础目录（总是可用）
    ///
    /// 返回 `~/.workflow/` 目录（Unix）。
    /// 此方法作为回退方案，确保在任何情况下都能获取到有效路径。
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

    /// 获取配置基础目录（支持 iCloud）
    ///
    /// 决策逻辑：
    /// 1. 在 macOS 上，优先尝试使用 iCloud Drive
    /// 2. 如果 iCloud 不可用，回退到本地目录
    /// 3. 在其他平台上，直接使用本地目录
    fn config_base_dir() -> Result<PathBuf> {
        // macOS 上尝试 iCloud
        if let Some(icloud_dir) = Self::try_icloud_base_dir() {
            return Ok(icloud_dir);
        }

        // 回退到本地
        Self::local_base_dir()
    }

    // ==================== 公开 API ====================

    /// 获取配置目录路径（支持 iCloud 同步）
    ///
    /// 返回配置文件存储目录。在 macOS 上，如果 iCloud Drive 可用，
    /// 配置将保存到 iCloud 并自动同步到其他设备。
    ///
    /// # 路径示例
    ///
    /// - macOS + iCloud：`~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/`
    /// - macOS 无 iCloud / 其他系统：`~/.workflow/config/`
    ///
    /// # 返回
    ///
    /// 返回配置目录的 `PathBuf`。
    ///
    /// # 错误
    ///
    /// 如果无法创建目录或设置权限，返回相应的错误信息。
    pub fn config_dir() -> Result<PathBuf> {
        let config_dir = Self::config_base_dir()?.join("config");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        // 设置目录权限为 700（仅用户可访问）
        #[cfg(unix)]
        {
            fs::set_permissions(&config_dir, fs::Permissions::from_mode(0o700))
                .context("Failed to set config directory permissions")?;
        }

        Ok(config_dir)
    }

    /// 获取工作流目录路径（支持 iCloud）
    ///
    /// 返回工作流基础目录。如果配置在 iCloud，此方法返回 iCloud 路径。
    pub fn workflow_dir() -> Result<PathBuf> {
        Self::config_base_dir()
    }

    /// 获取工作历史目录路径（强制本地，不同步）
    ///
    /// 返回 `~/.workflow/work-history/`（总是本地路径）。
    ///
    /// **重要**：工作历史是设备本地的，不应该跨设备同步，因为：
    /// - 每个设备的工作历史是独立的
    /// - 避免多设备冲突（不同设备可能处理不同的 PR）
    /// - 防止历史记录混乱
    /// - 性能考虑（本地读写更快）
    ///
    /// # 路径示例
    ///
    /// - 所有平台：`~/.workflow/work-history/`
    ///
    /// # 返回
    ///
    /// 返回工作历史目录的 `PathBuf`。
    pub fn work_history_dir() -> Result<PathBuf> {
        // 强制使用本地路径，不使用 iCloud
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

    /// 获取补全脚本目录路径（强制本地）
    ///
    /// 返回 `~/.workflow/completions/`（总是本地路径）。
    /// Shell 补全脚本是本地安装的，不需要同步。
    pub fn completion_dir() -> Result<PathBuf> {
        Ok(Self::local_base_dir()?.join("completions"))
    }

    // ==================== 配置文件路径 ====================

    /// 获取主配置文件路径
    pub fn workflow_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("workflow.toml"))
    }

    /// 获取 LLM 配置文件路径
    pub fn llm_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("llm.toml"))
    }

    /// 获取 Jira 状态配置文件路径
    pub fn jira_status_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-status.toml"))
    }

    /// 获取 Jira 用户配置文件路径
    pub fn jira_users_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("jira-users.toml"))
    }

    /// 获取分支配置文件路径
    pub fn branch_config() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("branch.toml"))
    }

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

    // ==================== Shell 路径和安装路径（保持不变）====================

    pub fn config_file(shell: &Shell) -> Result<PathBuf> {
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

    pub fn command_names() -> &'static [&'static str] {
        &["workflow"]
    }

    pub fn binary_install_dir() -> String {
        if cfg!(target_os = "windows") {
            let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| {
                std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\User".to_string())
            });
            format!("{}\\Programs\\workflow\\bin", local_app_data)
        } else {
            "/usr/local/bin".to_string()
        }
    }

    pub fn binary_paths() -> Vec<String> {
        let install_dir = Self::binary_install_dir();
        let install_path = PathBuf::from(&install_dir);
        Self::command_names()
            .iter()
            .map(|name| {
                let binary_name = Self::binary_name(name);
                install_path
                    .join(&binary_name)
                    .to_string_lossy()
                    .to_string()
            })
            .collect()
    }

    pub fn binary_name(name: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
    }
}
```

### 步骤 2：在 `setup.rs` 中显示存储信息

在 `src/commands/config/setup.rs` 中添加：

```rust
impl SetupCommand {
    /// 运行初始化设置流程
    pub fn run() -> Result<()> {
        log_success!("Starting Workflow CLI initialization...\n");

        // 加载现有配置
        let existing_config = Self::load_existing_config()?;

        // 收集配置信息
        let config = Self::collect_config(&existing_config)?;

        // 保存配置
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

        // 验证配置
        let settings = Settings::load();
        settings.verify()?;

        log_break!();
        log_success!("Initialization completed successfully!");
        log_message!("You can now use the Workflow CLI commands.");

        Ok(())
    }

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

### 步骤 3：在 `config show` 中显示存储信息

在 `src/commands/config/show.rs` 中添加：

```rust
use crate::base::settings::paths::Paths;

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

    // ... 现有的配置显示逻辑 ...

    Ok(())
}
```

---

## 🧪 测试

### 单元测试

在 `src/lib/base/settings/paths.rs` 末尾添加：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_exists() {
        let config_dir = Paths::config_dir().unwrap();
        assert!(config_dir.exists());
        assert!(config_dir.is_dir());
    }

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
                assert_ne!(
                    config.parent().unwrap().parent().unwrap(),
                    work_history.parent().unwrap().parent().unwrap()
                );
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
    fn test_all_config_files_in_same_dir() {
        let workflow_config = Paths::workflow_config().unwrap();
        let jira_status = Paths::jira_status_config().unwrap();
        let jira_users = Paths::jira_users_config().unwrap();
        let branch_config = Paths::branch_config().unwrap();

        // 所有配置文件应该在同一个目录下
        assert_eq!(
            workflow_config.parent().unwrap(),
            jira_status.parent().unwrap()
        );
        assert_eq!(
            workflow_config.parent().unwrap(),
            jira_users.parent().unwrap()
        );
        assert_eq!(
            workflow_config.parent().unwrap(),
            branch_config.parent().unwrap()
        );
    }
}
```

### 集成测试

创建 `tests/icloud_storage.rs`：

```rust
use workflow::base::settings::paths::Paths;
use std::fs;

#[test]
fn test_config_persistence() {
    // 获取配置目录
    let config_dir = Paths::config_dir().unwrap();

    // 创建测试文件
    let test_file = config_dir.join("test.txt");
    fs::write(&test_file, "test content").unwrap();

    // 验证文件存在
    assert!(test_file.exists());

    // 读取文件
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "test content");

    // 清理
    fs::remove_file(&test_file).ok();
}

#[test]
fn test_work_history_independence() {
    let config_dir = Paths::config_dir().unwrap();
    let work_history_dir = Paths::work_history_dir().unwrap();

    // 在两个目录下创建同名文件
    let config_test = config_dir.join("test.json");
    let history_test = work_history_dir.join("test.json");

    fs::write(&config_test, r#"{"type": "config"}"#).unwrap();
    fs::write(&history_test, r#"{"type": "history"}"#).unwrap();

    // 验证两个文件独立存在
    assert!(config_test.exists());
    assert!(history_test.exists());

    // 验证内容不同
    let config_content = fs::read_to_string(&config_test).unwrap();
    let history_content = fs::read_to_string(&history_test).unwrap();
    assert_ne!(config_content, history_content);

    // 清理
    fs::remove_file(&config_test).ok();
    fs::remove_file(&history_test).ok();
}

#[test]
#[cfg(target_os = "macos")]
fn test_icloud_detection() {
    // 测试 iCloud 检测逻辑
    let is_icloud = Paths::is_config_in_icloud();
    let location = Paths::storage_location();

    if is_icloud {
        assert_eq!(location, "iCloud Drive (synced across devices)");

        // 验证配置目录在 iCloud 路径下
        let config_dir = Paths::config_dir().unwrap();
        let path_str = config_dir.to_string_lossy();
        assert!(path_str.contains("com~apple~CloudDocs"));
    } else {
        assert_eq!(location, "Local storage");
    }
}
```

### 手动测试清单

#### macOS 测试

```bash
# 1. 测试 iCloud 可用情况
# 确保 iCloud Drive 已启用
workflow setup

# 检查配置位置
ls -la ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/

# 验证工作历史在本地
ls -la ~/.workflow/work-history/

# 2. 测试 iCloud 不可用情况
# 系统设置 → iCloud → 关闭 iCloud Drive
workflow setup

# 检查配置位置（应该在本地）
ls -la ~/.workflow/config/

# 3. 测试配置持久化
workflow setup  # 初始化配置
cat ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/workflow.toml

# 4. 测试多设备同步（需要多台 Mac）
# 设备 A: workflow setup
# 等待几秒
# 设备 B: ls ~/Library/Mobile\ Documents/com~apple~CloudDocs/.workflow/config/
# 应该看到同步的配置文件
```

#### Linux/其他系统测试

```bash
# 验证总是使用本地存储
workflow setup

# 检查配置位置
ls -la ~/.workflow/config/

# 验证存储信息显示正确
workflow config show
```

---

## 📊 路径对比表

### 配置文件路径

| 文件 | macOS (iCloud) | macOS (本地) / 其他系统 |
|------|---------------|----------------------|
| workflow.toml | `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/workflow.toml` | `~/.workflow/config/workflow.toml` |
| jira-status.toml | `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/jira-status.toml` | `~/.workflow/config/jira-status.toml` |
| jira-users.toml | `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/jira-users.toml` | `~/.workflow/config/jira-users.toml` |
| branch.toml | `~/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config/branch.toml` | `~/.workflow/config/branch.toml` |

### 工作历史路径（总是本地）

| 文件 | 所有平台 |
|------|---------|
| work-history/ | `~/.workflow/work-history/` |
| github-com-owner-repo.json | `~/.workflow/work-history/github-com-owner-repo.json` |

---

## ⚠️ 注意事项

### 1. iCloud 同步延迟

```rust
// iCloud 同步可能需要几秒到几分钟
// 在同一台设备上，文件操作是立即的
// 跨设备同步有延迟，这是正常现象

// 应用不需要处理同步逻辑，macOS 会自动处理
```

### 2. 冲突处理

```
如果两台设备同时修改同一配置文件：
1. macOS 会自动处理冲突
2. 通常会保留两个版本
3. 用户可以在 Finder 中查看和解决冲突

应用层面不需要特殊处理。
```

### 3. 工作历史独立性

```rust
// 工作历史必须保持设备独立
// 原因：
// 1. 不同设备可能处理不同的 PR
// 2. PR ID 可能在不同仓库中重复
// 3. 避免历史记录混乱

// 实现：work_history_dir() 总是调用 local_base_dir()
```

### 4. 向后兼容

```
现有用户升级后：
1. 配置文件仍在 ~/.workflow/config/
2. 首次运行时会检测 iCloud
3. 如果 iCloud 可用，会将配置复制到 iCloud
4. 工作历史保持在原位置
```

### 5. 错误处理

```rust
// iCloud 不可用时自动回退
if let Some(icloud) = try_icloud_base_dir() {
    Ok(icloud)  // 使用 iCloud
} else {
    local_base_dir()  // 回退到本地，用户无感知
}
```

---

## 📝 实施清单

- [ ] 步骤 1：在 `Cargo.toml` 添加 `dirs = "5.0"`
- [ ] 步骤 2：在 `paths.rs` 实现 iCloud 支持
  - [ ] 添加 `try_icloud_base_dir()` 方法
  - [ ] 添加 `config_base_dir()` 方法
  - [ ] 更新 `config_dir()` 使用新逻辑
  - [ ] 确保 `work_history_dir()` 总是本地
  - [ ] 添加 `is_config_in_icloud()` 方法
  - [ ] 添加 `storage_location()` 方法
  - [ ] 添加 `storage_info()` 方法
- [ ] 步骤 3：更新 `setup.rs` 显示存储信息
- [ ] 步骤 4：更新 `config/show.rs` 显示存储位置
- [ ] 步骤 5：添加单元测试
- [ ] 步骤 6：添加集成测试
- [ ] 步骤 7：在 macOS 上测试 iCloud 场景
- [ ] 步骤 8：测试 iCloud 不可用场景
- [ ] 步骤 9：在 Linux 上测试（确保使用本地）
- [ ] 步骤 10：测试配置迁移（从本地到 iCloud）
- [ ] 步骤 11：更新用户文档
- [ ] 步骤 12：提交代码

---

## 🎯 用户体验示例

### 初始化时的输出（macOS + iCloud）

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

### 初始化时的输出（本地存储）

```
✅ Configuration saved successfully!

📦 Storage Information
─────────────────────────────────────────────────────────────────
  Type: Local storage

  📁 Configuration:
     /Users/username/.workflow/config

  📁 Work History:
     /Users/username/.workflow/work-history
─────────────────────────────────────────────────────────────────
```

### `workflow config show` 输出

```
============== Configuration Overview ==============

Storage: iCloud Drive (synced across devices)
Path: /Users/username/Library/Mobile Documents/com~apple~CloudDocs/.workflow/config
  ✅ Synced across your Apple devices

GitHub Configuration
────────────────────────────────────────────────────
...
```

---

## 🎨 总结

### 核心特性

1. ✅ **自动 iCloud 检测**：无需用户配置
2. ✅ **优雅降级**：iCloud 不可用时自动本地
3. ✅ **分层存储**：配置同步，工作历史本地
4. ✅ **跨平台兼容**：非 macOS 自动使用本地
5. ✅ **用户友好**：清晰显示存储位置和同步状态

### 技术亮点

- 🎯 **编译期优化**：`#[cfg(target_os = "macos")]` 零运行时开销
- 🔒 **类型安全**：`PathBuf` 保证路径正确性
- 🚀 **零配置**：系统自动决策存储位置
- 📦 **最小依赖**：只需 `dirs` crate
- 🧪 **完善测试**：单元测试 + 集成测试全覆盖

### 设计优势

相比 Go 实现：
- ✅ 更好的类型安全（`PathBuf` vs `string`）
- ✅ 更清晰的代码结构（分层路径管理）
- ✅ 更好的性能（编译期平台检测）
- ✅ 更强的测试覆盖（Rust 测试生态）

---

**最后更新**：2024-12-06
