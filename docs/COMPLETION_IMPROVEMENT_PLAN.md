# Completion 模块改进方案

## 📋 概述

本文档描述 Completion 模块的改进方案，主要目标是：
1. 安装时只生成当前 shell 类型的 completion 脚本
2. 支持所有 shell 类型（zsh, bash, fish, powershell, elvish）
3. 新增 `workflow completion generate` 子命令，自动检测当前 shell 并生成对应的 completion（行为与安装流程一致）

---

## 🎯 改进目标

### 1. 简化安装流程
- **当前问题**：安装时会生成 zsh 和 bash 两种类型的 completion 脚本，即使当前只使用一种 shell
- **改进方案**：安装时只生成当前检测到的 shell 类型的 completion 脚本
- **好处**：减少不必要的文件生成，简化安装流程

### 2. 支持所有 Shell 类型
- **当前问题**：配置文件只支持 zsh 和 bash
- **改进方案**：支持所有 shell 类型（zsh, bash, fish, powershell, elvish）
- **好处**：用户可以在任何 shell 环境下使用 completion

### 3. 灵活的 Completion 管理
- **当前问题**：用户切换 shell 时需要重新安装
- **改进方案**：新增 `workflow completion generate` 子命令，自动检测当前 shell 并生成对应的 completion
- **实现方式**：使用 `Detect::shell()` 检测当前 shell，生成 completion 脚本并保存到对应的 shell 配置文件（通过 `ShellConfigManager`）
- **好处**：用户可以随时为当前使用的 shell 生成 completion，无需重新安装，行为与安装流程完全一致

---

## 📁 需要改动的文件

### 1. `src/commands/install.rs`
**改动内容：**
- 移除生成 zsh 和 bash 的逻辑（第 52-68 行）
- 只生成当前检测到的 shell 类型
- 简化安装流程

**改动前：**
```rust
// 生成当前检测到的 shell 类型的补全脚本
Completion::generate_all_completions(...)?;

// 生成 zsh 补全脚本（如果当前不是 zsh）
if shell != Shell::Zsh {
    Completion::generate_all_completions(Some("zsh".to_string()), ...)?;
}

// 生成 bash 补全脚本（如果当前不是 bash）
if shell != Shell::Bash {
    Completion::generate_all_completions(Some("bash".to_string()), ...)?;
}
```

**改动后：**
```rust
// 只生成当前检测到的 shell 类型的补全脚本
let shell_type_str = shell.to_string();
Completion::generate_all_completions(
    Some(shell_type_str),
    Some(completion_dir.to_string_lossy().to_string()),
)?;
```

### 2. `src/lib/completion/completion.rs`
**改动内容：**
- `create_completion_config_file()` 方法需要根据 shell 类型生成不同的配置
- 支持所有 shell 类型（zsh, bash, fish, powershell, elvish）

**不同 Shell 的配置方式：**

#### Zsh
```bash
# ~/.workflow/.completions
fpath=($HOME/.workflow/completions $fpath)
if [[ -f $HOME/.workflow/completions/_workflow ]]; then
    source $HOME/.workflow/completions/_workflow
    source $HOME/.workflow/completions/_pr
    source $HOME/.workflow/completions/_qk
fi
```

#### Bash
```bash
# ~/.workflow/.completions
for f in $HOME/.workflow/completions/*.bash; do
    [[ -f "$f" ]] && source "$f"
done
```

#### Fish
```fish
# ~/.config/fish/config.fish (直接写入)
source $HOME/.workflow/completions/workflow.fish
source $HOME/.workflow/completions/pr.fish
source $HOME/.workflow/completions/qk.fish
```

#### PowerShell
```powershell
# ~/.config/powershell/Microsoft.PowerShell_profile.ps1 (直接写入)
. $HOME/.workflow/completions/_workflow.ps1
. $HOME/.workflow/completions/_pr.ps1
. $HOME/.workflow/completions/_qk.ps1
```

#### Elvish
```elvish
# ~/.elvish/rc.elv (直接写入)
source $HOME/.workflow/completions/workflow.elv
source $HOME/.workflow/completions/pr.elv
source $HOME/.workflow/completions/qk.elv
```

**改动要点：**
- zsh 和 bash：继续使用 `~/.workflow/.completions` 统一配置文件
- fish, powershell, elvish：直接写入各自的配置文件（不使用统一配置文件）

### 3. `src/main.rs`
**改动内容：**
- 添加 `Completion` 子命令到 `Commands` 枚举
- 添加 `CompletionSubcommand` 枚举

**新增代码：**
```rust
#[derive(Subcommand)]
enum Commands {
    // ... 现有命令 ...

    /// 管理 Shell Completion
    ///
    /// 生成和管理 shell completion 脚本。
    Completion {
        #[command(subcommand)]
        subcommand: CompletionSubcommand,
    },
}

/// Completion 管理子命令
#[derive(Subcommand)]
enum CompletionSubcommand {
    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到对应的配置文件。
    /// 行为与安装流程完全一致：使用 `Detect::shell()` 检测当前 shell，
    /// 生成 completion 脚本，并通过 `ShellConfigManager` 保存到对应的 shell 配置文件。
    Generate,
}
```

### 4. `src/commands/completion.rs`（新建）
**功能：**
- 实现 `generate` 子命令
- 自动检测当前 shell 类型（使用 `Detect::shell()`）
- 生成 completion 脚本并应用到对应 shell 配置文件（使用 `ShellConfigManager`）
- 行为与安装流程完全一致

**实现要点：**
```rust
pub struct CompletionCommand;

impl CompletionCommand {
    /// 生成 completion 脚本
    ///
    /// 自动检测当前 shell 类型，生成对应的 completion 脚本并应用到配置文件。
    /// 行为与安装流程完全一致。
    pub fn generate() -> Result<()> {
        // 1. 自动检测当前 shell 类型（使用 Detect::shell()）
        let shell = Detect::shell()
            .context("Failed to detect current shell type")?;

        // 2. 生成 completion 脚本（与安装流程一致）
        let completion_dir = Paths::completion_dir()?;
        Completion::generate_all_completions(
            Some(shell.to_string()),
            Some(completion_dir.to_string_lossy().to_string()),
        )?;

        // 3. 应用到对应的 shell 配置文件（使用 ShellConfigManager）
        Completion::configure_shell_config(&shell)?;

        Ok(())
    }
}
```

### 5. `src/commands/mod.rs`
**改动内容：**
- 添加 `completion` 模块声明
- 导出 `CompletionCommand`

---

## 🔄 改进后的流程

### 安装流程（简化）

```
workflow install 或 ./install
  ↓
  1. Detect::shell()                    # 检测当前 shell 类型（例如：zsh）
  2. Completion::generate_all_completions("zsh", ...)  # 只生成 zsh 的脚本
  3. Completion::configure_shell_config(&Shell::Zsh)    # 只配置 zsh 的配置文件
  ↓
  完成（只生成和配置当前 shell）
```

### 新命令流程（与安装流程一致）

```
workflow completion generate
  ↓
  1. Detect::shell()                    # 自动检测当前 shell 类型（例如：bash）
  2. Completion::generate_all_completions("bash", ...)  # 生成当前 shell 的脚本
  3. Completion::configure_shell_config(&Shell::Bash)  # 通过 ShellConfigManager 保存到配置文件
  ↓
  完成（为当前 shell 生成和配置，行为与安装流程完全一致）
```

### 多 Shell 支持场景

```
用户场景：用户在 zsh 环境下安装了 workflow
  ↓
  安装时：只生成 zsh 的 completion 脚本，只配置 ~/.zshrc
  ↓
  用户切换到 bash：
  ↓
  运行：workflow completion generate
  ↓
  自动检测：Detect::shell() 检测到当前是 bash
  ↓
  生成：Completion::generate_all_completions("bash", ...) 生成 bash 脚本
  ↓
  配置：Completion::configure_shell_config(&Shell::Bash) 通过 ShellConfigManager 保存到 ~/.bash_profile
  ↓
  结果：生成 bash 的 completion 脚本，配置 ~/.bash_profile
  ↓
  现在两个 shell 都支持 completion
```

---

## 🏗️ 架构设计

### 配置文件策略

采用**混合方案**：

1. **Zsh 和 Bash**：
   - 使用统一的 `~/.workflow/.completions` 配置文件
   - 在各自的 shell 配置文件中添加 `source ~/.workflow/.completions`
   - 配置文件内部检测 shell 类型并加载相应的脚本

2. **Fish, PowerShell, Elvish**：
   - 直接写入各自的配置文件
   - 不使用统一配置文件
   - 因为它们的配置文件格式和加载方式不同

### 方法签名改动

#### `create_completion_config_file()`
```rust
// 改动前
fn create_completion_config_file(_shell: &Shell) -> Result<PathBuf>

// 改动后
fn create_completion_config_file(shell: &Shell) -> Result<Option<PathBuf>>
// 返回 Option，因为 fish/powershell/elvish 不使用统一配置文件
```

#### `configure_shell_config()`
```rust
// 改动前
pub fn configure_shell_config(shell: &Shell) -> Result<()>

// 改动后（签名不变，但内部逻辑改变）
pub fn configure_shell_config(shell: &Shell) -> Result<()> {
    match shell {
        Shell::Zsh | Shell::Bash => {
            // 创建统一配置文件并添加到 shell 配置文件
        }
        Shell::Fish => {
            // 直接写入 ~/.config/fish/config.fish
        }
        Shell::PowerShell => {
            // 直接写入 PowerShell profile
        }
        Shell::Elvish => {
            // 直接写入 ~/.elvish/rc.elv
        }
    }
}
```

---

## 📊 数据流对比

### 改进前（安装时）

```
安装命令
  ↓
  生成 zsh completion 脚本
  生成 bash completion 脚本（即使当前不是 bash）
  创建统一配置文件（支持 zsh 和 bash）
  添加到 ~/.zshrc 或 ~/.bash_profile
  ↓
  结果：生成了两种 shell 的脚本，但可能只需要一种
```

### 改进后（安装时）

```
安装命令
  ↓
  检测当前 shell（例如：zsh）
  只生成 zsh completion 脚本
  创建 zsh 配置文件
  添加到 ~/.zshrc
  ↓
  结果：只生成需要的脚本，更简洁
```

### 改进后（新命令）

```
workflow completion generate
  ↓
  自动检测当前 shell（例如：bash）
  生成 bash completion 脚本
  通过 ShellConfigManager 保存到 ~/.bash_profile
  ↓
  结果：为当前 shell 生成和配置，行为与安装流程完全一致
```

---

## 🔍 实现细节

### Shell 类型检测

`workflow completion generate` 命令使用 `Detect::shell()` 自动检测当前 shell 类型，与安装流程完全一致：

```rust
// 在 commands/completion.rs 中
let shell = Detect::shell()
    .context("Failed to detect current shell type")?;
```

这样确保了：
- 行为与安装流程一致
- 不需要用户手动指定 shell 类型
- 自动适配当前使用的 shell

### 配置文件生成逻辑

```rust
fn create_completion_config_file(shell: &Shell) -> Result<Option<PathBuf>> {
    match shell {
        Shell::Zsh => {
            // 生成 zsh 配置
            let config_content = format!(
                "fpath=($HOME/.workflow/completions $fpath)\n\
                 if [[ -f $HOME/.workflow/completions/_workflow ]]; then\n\
                     source $HOME/.workflow/completions/_workflow\n\
                     source $HOME/.workflow/completions/_pr\n\
                     source $HOME/.workflow/completions/_qk\n\
                 fi\n"
            );
            // 写入 ~/.workflow/.completions
            Ok(Some(config_file))
        }
        Shell::Bash => {
            // 生成 bash 配置
            let config_content = format!(
                "for f in $HOME/.workflow/completions/*.bash; do\n\
                     [[ -f \"$f\" ]] && source \"$f\"\n\
                 done\n"
            );
            // 写入 ~/.workflow/.completions
            Ok(Some(config_file))
        }
        Shell::Fish | Shell::PowerShell | Shell::Elvish => {
            // 这些 shell 不使用统一配置文件
            // 配置会直接写入各自的配置文件
            Ok(None)
        }
    }
}
```

### 直接写入配置文件的逻辑

```rust
fn configure_shell_config_direct(shell: &Shell) -> Result<()> {
    let completion_dir = Paths::completion_dir()?;
    let config_path = Paths::config_file(shell)?;

    match shell {
        Shell::Fish => {
            let content = format!(
                "\n# Workflow CLI completions\n\
                 source {}/workflow.fish\n\
                 source {}/pr.fish\n\
                 source {}/qk.fish\n",
                completion_dir.display(),
                completion_dir.display(),
                completion_dir.display(),
            );
            ShellConfigManager::append_to_file(&config_path, &content)?;
        }
        Shell::PowerShell => {
            // PowerShell 使用 . 而不是 source
            let content = format!(
                "\n# Workflow CLI completions\n\
                 . {}/_workflow.ps1\n\
                 . {}/_pr.ps1\n\
                 . {}/_qk.ps1\n",
                completion_dir.display(),
                completion_dir.display(),
                completion_dir.display(),
            );
            ShellConfigManager::append_to_file(&config_path, &content)?;
        }
        Shell::Elvish => {
            let content = format!(
                "\n# Workflow CLI completions\n\
                 source {}/workflow.elv\n\
                 source {}/pr.elv\n\
                 source {}/qk.elv\n",
                completion_dir.display(),
                completion_dir.display(),
                completion_dir.display(),
            );
            ShellConfigManager::append_to_file(&config_path, &content)?;
        }
        _ => {}
    }

    Ok(())
}
```

---

## ⚠️ 潜在问题和解决方案

### 1. 用户切换 Shell

**问题**：用户在 zsh 环境下安装，后来切换到 bash，bash 没有 completion

**解决方案**：
- 用户在 bash 环境下运行 `workflow completion generate`
- 命令会自动检测当前是 bash，生成 bash 的 completion 脚本并配置
- 行为与安装流程完全一致，用户无需关心 shell 类型

### 2. 卸载时的清理

**问题**：卸载时需要清理所有 shell 的配置

**解决方案**：
- `remove_completion_files()` 已经会删除所有 shell 类型的文件
- `remove_completion_config()` 需要支持清理所有 shell 类型的配置
- 可以遍历所有支持的 shell 类型进行清理

### 3. 配置文件冲突

**问题**：如果用户手动修改了配置文件，我们的修改可能会冲突

**解决方案**：
- 使用 `ShellConfigManager` 的现有机制（检查是否已存在）
- 对于直接写入的配置文件，检查是否已包含我们的配置块

### 4. 向后兼容性

**问题**：现有用户已经安装了 zsh 和 bash 的 completion

**解决方案**：
- 卸载时清理所有文件
- 重新安装时只生成当前 shell 的
- 不影响现有功能

---

## 📝 实现步骤

### 阶段 1：简化安装流程
1. 修改 `install.rs`，移除生成多种 shell 的逻辑
2. 测试安装流程

### 阶段 2：支持所有 Shell 类型
1. 修改 `create_completion_config_file()` 支持所有 shell
2. 实现直接写入配置文件的逻辑（fish, powershell, elvish）
3. 测试所有 shell 类型的配置

### 阶段 3：新增子命令
1. 创建 `commands/completion.rs`
2. 修改 `main.rs` 添加子命令
3. 实现 `generate` 子命令
4. 测试新命令

### 阶段 4：完善和测试
1. 测试所有 shell 类型的安装和生成
2. 测试卸载流程
3. 更新文档

---

## 🎯 改进效果

### 1. 安装流程简化

**改进前：**
- 安装时会生成 zsh 和 bash 两种类型的 completion 脚本
- 即使当前只使用一种 shell，也会生成多种脚本
- 创建统一配置文件同时支持 zsh 和 bash

**改进后：**
- 安装时只生成当前检测到的 shell 类型的 completion 脚本
- 使用 `Detect::shell()` 自动检测当前 shell
- 只配置当前 shell 的配置文件
- 更简洁高效，减少不必要的文件生成

### 2. 多 Shell 支持

**改进前：**
- `ShellConfigManager` 只支持 zsh 和 bash
- 配置文件只支持 zsh 和 bash
- 其他 shell（fish, powershell, elvish）无法使用 completion

**改进后（已实现）：**
- `ShellConfigManager::get_config_path()` 已支持所有 shell 类型（通过 `Paths::config_file()`）
- `Paths::config_file()` 支持 zsh, bash, fish, powershell, elvish
- 配置文件策略：
  - zsh 和 bash：使用统一的 `~/.workflow/.completions` 配置文件
  - fish, powershell, elvish：直接写入各自的配置文件

### 3. 灵活的 Completion 管理

**改进前：**
- 用户切换 shell 时需要重新安装整个 workflow
- 无法为特定 shell 单独生成 completion

**改进后（设计完成）：**
- 新增 `workflow completion generate` 子命令
- 自动检测当前 shell 类型（使用 `Detect::shell()`）
- 生成 completion 脚本并保存到对应的 shell 配置文件（通过 `ShellConfigManager`）
- 行为与安装流程完全一致，用户可以在任意 shell 环境下运行
- 支持为不同 shell 分别生成和配置 completion

---

## 📚 相关文档

- [Completion 模块架构文档](./COMPLETION_ARCHITECTURE.md)
- [安装/卸载模块架构文档](./INSTALL_ARCHITECTURE.md)

---

## 🔄 后续优化（✅ 已完成）

### 1. 状态检查功能 ✅
- **功能**：支持 `workflow completion check` 检查当前已安装的 shell 类型
- **实现**：
  - ✅ 检测系统中已安装的 shell（通过检查 `/etc/shells` 或环境变量）
  - ✅ 检查哪些 shell 已配置 completion（检查对应的配置文件是否包含 completion 配置）
  - ✅ 显示已配置和未配置的 shell 列表
- **用途**：帮助用户了解哪些 shell 已配置 completion，哪些还需要配置
- **输出示例**：
  ```
  $ workflow completion check

  已安装的 shell：
  ✓ zsh  - 已配置 completion (~/.zshrc)
  ✓ bash - 已配置 completion (~/.bash_profile)
  ✗ fish - 未配置 completion

  提示：运行 `workflow completion generate` 为未配置的 shell 生成 completion
  ```

### 2. 移除功能 ✅
- **功能**：支持 `workflow completion remove` 移除已配置的 completion
- **实现方式**：
  - ✅ 检测当前已安装的 shell 类型
  - ✅ 检查哪些 shell 已配置 completion
  - ✅ 显示多选列表，列出所有已配置 completion 的 shell
  - ✅ 用户可以选择一个或多个 shell（使用空格选择，Enter 确认删除）
  - ✅ 确认后移除选中 shell 的 completion 配置和脚本文件
- **交互示例**：
  ```
  $ workflow completion remove

  检测到以下 shell 已配置 completion：
  [ ] zsh  (~/.zshrc)
  [x] bash (~/.bash_profile)
  [x] fish (~/.config/fish/config.fish)

  使用空格选择，Enter 确认删除，Esc 取消
  > 确认删除选中的 completion？(y/N)
  ```

**注意**：`workflow completion generate` 自动检测当前 shell，不需要任何参数。上述功能已全部实现并可用。

