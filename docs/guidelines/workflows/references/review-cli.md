# CLI 审查指南

> 专为 AI 助手设计的 CLI 命令检查指南，提供系统化的命令结构和参数复用检查方法，确保 CLI 命令完整性和一致性。

## 🎯 核心原则

**检查重点**：确保 CLI 命令的完整性、参数复用和补全脚本同步。

**关键目标**：
- ✅ 命令完整性：所有命令和子命令正确注册
- ✅ 参数复用：识别并使用已封装的共用参数
- ✅ 补全同步：补全脚本与实际命令结构一致
- ✅ 命名规范：参数命名遵循统一标准

---

## 📋 目录

- [检查目标](#-检查目标)
- [检查流程](#-检查流程)
- [完整性检查](#-完整性检查)
- [参数化检查](#-参数化检查)
- [参数抽取指南](#-参数抽取指南)
- [其他检查项](#-其他检查项)
- [检查清单](#-检查清单)
- [工具和测试](#-工具和测试)

---

## 🎯 检查目标

CLI 检查的主要目标：

1. **确保命令完整性**：所有命令和子命令都已正确注册和定义
2. **确保补全同步**：补全脚本与实际命令结构保持同步
3. **参数复用优化**：识别并复用已封装的共用参数
4. **参数提取优化**：识别可以提取为共用参数的重复定义
5. **命名一致性**：确保参数命名遵循统一规范

---

## 🔄 检查流程

### 步骤 1：完整性检查

1. **命令注册检查**：验证所有命令是否已在 `Commands` 枚举中注册
2. **子命令检查**：验证所有子命令是否已在对应枚举中定义
3. **文档注释检查**：验证所有命令和参数是否有文档注释
4. **补全脚本检查**：运行补全完整性测试

### 步骤 2：参数化检查

1. **参数复用检查**：检查是否应该使用已封装的参数（`OutputFormatArgs`、`DryRunArgs`、`JiraIdArg`）
2. **参数提取机会检查**：识别可以提取为共用参数的重复定义
3. **命名一致性检查**：检查参数命名是否遵循规范

### 步骤 3：其他检查

1. **参数验证检查**：检查参数验证逻辑是否统一
2. **命令结构检查**：检查命令层级和分组是否合理
3. **向后兼容性检查**：检查参数变更是否影响现有脚本

### 步骤 4：生成报告

1. **问题汇总**：汇总所有发现的问题
2. **优先级排序**：按影响范围和收益排序
3. **改进建议**：提供具体的改进方案

---

## ✅ 完整性检查

### 1.1 命令注册检查

**检查目标**：确保所有命令都已正确注册

**检查方法**：

1. **检查 `Commands` 枚举**：验证所有顶级命令是否已在 `src/lib/cli/commands.rs` 中注册

```bash
# 查看 Commands 枚举定义
grep -A 50 "pub enum Commands" src/lib/cli/commands.rs
```

2. **检查子命令枚举**：验证所有子命令是否已在对应枚举中定义

```bash
# 查看所有子命令枚举
grep -r "pub enum.*Subcommand\|pub enum.*Commands" src/lib/cli/
```

**检查清单**：
- [ ] 新增命令是否已在 `Commands` 枚举中注册
- [ ] 子命令是否已在对应枚举中定义
- [ ] 命令是否在 `src/lib/cli/mod.rs` 中正确导出

**位置**：`src/lib/cli/commands.rs`、`src/lib/cli/*.rs`

### 1.2 文档注释检查

**检查目标**：确保所有命令和参数都有文档注释

**检查方法**：

```bash
# 查找没有文档注释的命令
grep -r "pub enum.*Commands\|pub enum.*Subcommand" src/lib/cli/ -A 5 | grep -v "///"

# 查找没有文档注释的参数
grep -r "#\[arg" src/lib/cli/ -B 1 | grep -v "///"
```

**检查清单**：
- [ ] 所有命令枚举是否有 `///` 文档注释
- [ ] 所有命令变体是否有 `///` 文档注释
- [ ] 所有参数是否有 `///` 文档注释
- [ ] 文档注释是否清晰描述了命令和参数的用途

**示例**：

```rust
// ✅ 好的做法
/// Create a new Pull Request
///
/// Supports auto-detection of repository type (GitHub), and optionally uses AI to generate PR title.
Create {
    /// Jira ticket ID (optional, e.g., PROJ-123)
    #[arg(value-_name = "JIRA_ID")]
    jira-_id: Option<String>,
}

// ❌ 不好的做法
Create {
    #[arg(value-_name = "JIRA_ID")]
    jira-_id: Option<String>,  // 缺少文档注释
}
```

### 1.3 命名一致性检查

**检查目标**：确保参数命名遵循统一规范

**检查方法**：

```bash
# 查找 JIRA 相关参数（检查命名一致性）
grep -r "jira" src/lib/cli/ --include="*.rs" -i | grep "#\[arg\|:"

# 查找输出格式相关参数
grep -r "json\|yaml\|table\|markdown" src/lib/cli/ --include="*.rs" | grep "#\[arg"
```

**检查清单**：
- [ ] 相同语义的参数是否使用相同的命名（如 `jira-_id` vs `jira-_ticket`）
- [ ] `value-_name` 是否使用 `SCREAMING_SNAKE_CASE`（如 `JIRA_ID`）
- [ ] 参数字段名是否使用 `snake-_case`（如 `jira-_id`）
- [ ] 参数长名是否使用 `kebab-case`（clap 自动转换）

**参考**：见 [参数命名规范](#参数命名规范)

### 1.4 补全脚本完整性检查

**检查目标**：确保补全脚本包含所有命令和子命令

**检查方法**：

```bash
# 运行补全完整性测试
cargo test --test completeness

# 手动生成补全脚本验证
cargo run -- completion generate
```

**检查清单**：
- [ ] 运行补全完整性测试：`cargo test --test completeness`
- [ ] 新增命令是否包含在补全脚本中
- [ ] 所有 shell 类型（zsh, bash, fish, powershell, elvish）是否正常生成
- [ ] 补全脚本文件命名是否正确

**位置**：`tests/completion/completeness.rs`

---

## 🔧 参数化检查

### 2.1 参数复用检查

**检查目标**：检查是否应该使用已封装的参数但没有使用

**已封装的参数**（`src/lib/cli/args.rs`）：

1. **`OutputFormatArgs`**：输出格式选项（table、json、yaml、markdown）
2. **`DryRunArgs`**：Dry run 模式选项
3. **`JiraIdArg`**：可选 JIRA ID 参数

**检查方法**：

#### 检查 OutputFormatArgs 复用

```bash
# 查找定义了输出格式参数但没有使用 OutputFormatArgs 的命令
grep -r "json\|yaml\|table\|markdown" src/lib/cli/ --include="*.rs" | grep "#\[arg" | grep -v "OutputFormatArgs"
```

**检查清单**：
- [ ] 需要输出格式的命令是否使用了 `OutputFormatArgs`？
- [ ] 是否使用 `#[command(flatten)]` 复用参数组？

**当前使用情况**：
- ✅ `jira.rs`：Info、Related、Changelog、Comments 使用 `OutputFormatArgs`

#### 检查 DryRunArgs 复用

```bash
# 查找定义了 dry-run 参数但没有使用 DryRunArgs 的命令
grep -r "dry.*run\|dry-_run" src/lib/cli/ --include="*.rs" -i | grep "#\[arg" | grep -v "DryRunArgs"
```

**检查清单**：
- [ ] 需要 dry-run 模式的命令是否使用了 `DryRunArgs`？
- [ ] 是否使用 `#[command(flatten)]` 复用参数组？

**当前使用情况**：
- ✅ `pr.rs`：Create、Rebase、Pick 使用 `DryRunArgs`
- ✅ `branch.rs`：Create、Delete 使用 `DryRunArgs`
- ✅ `jira.rs`：Clean 使用 `DryRunArgs`
- ✅ `config.rs`：Import 使用 `DryRunArgs`
- ✅ `tag.rs`：使用 `DryRunArgs`

#### 检查 JiraIdArg 复用

```bash
# 查找定义了 jira 相关参数但没有使用 JiraIdArg 的命令
grep -r "jira" src/lib/cli/ --include="*.rs" -i | grep -E "#\[arg|:" | grep -v "JiraIdArg\|use.*JiraIdArg"
```

**检查清单**：
- [ ] 需要 JIRA ID 的命令是否使用了 `JiraIdArg`？
- [ ] 是否使用 `#[command(flatten)]` 复用参数组？

**当前使用情况**：
- ✅ `jira.rs`：所有子命令使用 `JiraIdArg`
- ✅ `log.rs`：所有子命令使用 `JiraIdArg`
- ✅ `branch.rs`：Create 使用 `JiraIdArg`
- ❌ `pr.rs`：Create 使用 `jira-_ticket: Option<String>`（应该使用 `JiraIdArg`）

**问题示例**：

```rust
// ❌ 不好的做法（pr.rs）
Create {
    /// Jira ticket ID (optional, e.g., PROJ-123)
    #[arg(value-_name = "JIRA_TICKET")]
    jira-_ticket: Option<String>,  // 应该使用 JiraIdArg
}

// ✅ 好的做法
Create {
    #[command(flatten)]
    jira-_id: JiraIdArg,  // 使用已封装的参数
}
```

### 2.2 参数提取机会检查

**检查目标**：识别可以提取为共用参数的重复定义

**提取标准**：

1. **硬性标准**：同一参数在 **2+ 个命令**中出现 → **必须提取**
2. **软性标准**：同一参数在 **3+ 个命令**中出现 → **强烈建议提取**
3. **语义一致性**：参数含义和用法必须一致
4. **命名一致性**：参数名和类型必须一致

**提取决策流程**：

```
1. 识别重复参数
   ├─ 出现次数 ≥ 2 → 进入评估
   └─ 出现次数 < 2 → 暂不提取

2. 评估语义一致性
   ├─ 含义相同 → 继续
   └─ 含义不同 → 不提取（即使命名相同）

3. 评估命名一致性
   ├─ 命名相同 → 直接提取
   └─ 命名不同 → 统一命名后提取

4. 评估验证逻辑
   ├─ 验证逻辑相同 → 提取到 args.rs
   └─ 验证逻辑不同 → 提取基础结构，验证逻辑保留在命令中
```

**检查方法**：

```bash
# 查找重复的参数模式
# 例如：查找所有 force 参数
grep -r "force" src/lib/cli/ --include="*.rs" | grep "#\[arg"

# 查找所有 limit 参数
grep -r "limit" src/lib/cli/ --include="*.rs" | grep "#\[arg"

# 查找所有 offset 参数
grep -r "offset" src/lib/cli/ --include="*.rs" | grep "#\[arg"
```

**检查清单**：
- [ ] 是否有参数在 2+ 个命令中重复出现？
- [ ] 重复参数的含义是否一致？
- [ ] 重复参数的命名是否一致？
- [ ] 是否可以提取为共用参数组？

**提取示例**：

```rust
// 如果多个命令都有 limit 和 offset 参数
// 可以提取为 PaginationArgs

/// 分页参数
#[derive(Args, Debug, Clone)]
pub struct PaginationArgs {
    /// Limit number of results to display
    #[arg(long, value-_name = "LIMIT")]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[arg(long, value-_name = "OFFSET")]
    pub offset: Option<usize>,
}
```

---

## 📝 参数抽取指南

### 3.1 何时提取参数

**必须提取的情况**（出现 2+ 次）：

- 参数在 2+ 个命令中出现
- 参数含义和用法完全一致
- 参数类型和验证逻辑相同

**建议提取的情况**（出现 3+ 次）：

- 参数在 3+ 个命令中出现
- 参数含义相似，可以统一
- 参数类型相同，验证逻辑可以统一

**不应提取的情况**：

- 参数只在一个命令中使用
- 参数含义不同（即使命名相同）
- 参数验证逻辑差异很大

### 3.2 如何提取参数

**步骤 1：创建参数结构体**

在 `src/lib/cli/args.rs` 中定义新的参数结构体：

```rust
use clap::Args;

/// 分页参数
#[derive(Args, Debug, Clone)]
pub struct PaginationArgs {
    /// Limit number of results to display
    #[arg(long, value-_name = "LIMIT")]
    pub limit: Option<usize>,

    /// Offset for pagination
    #[arg(long, value-_name = "OFFSET")]
    pub offset: Option<usize>,
}
```

**步骤 2：在 mod.rs 中导出**

在 `src/lib/cli/mod.rs` 中导出新参数：

```rust
pub use args::{DryRunArgs, JiraIdArg, OutputFormatArgs, PaginationArgs};
```

**步骤 3：在命令中使用**

使用 `#[command(flatten)]` 复用参数组：

```rust
use super::args::PaginationArgs;

#[derive(Subcommand)]
pub enum MySubcommand {
    List {
        #[command(flatten)]
        pagination: PaginationArgs,
    },
}
```

**步骤 4：更新所有使用该参数的命令**

替换所有重复的参数定义为使用新的参数组。

### 3.3 参数命名规范

**结构体字段名**：`snake-_case`（如 `jira-_id`、`dry-_run`）

**`value-_name`**：`SCREAMING_SNAKE_CASE`（如 `JIRA_ID`、`DRY_RUN`）

**参数长名**：`kebab-case`（clap 自动转换，如 `--jira-id`、`--dry-run`）

**参数短名**：单个字符（如 `-n`、`-f`）

**参考**：见 [开发规范 - 参数命名规范](../development.md#参数命名规范)

---

## 🔍 其他检查项

### 4.1 参数验证检查

**检查目标**：确保参数验证逻辑统一

**检查方法**：

```bash
# 查找参数验证逻辑
grep -r "parse\|validate\|check" src/lib/cli/ --include="*.rs" -A 5
```

**检查清单**：
- [ ] 相同类型的参数是否使用相同的验证逻辑？
- [ ] 参数验证错误消息是否统一？
- [ ] 是否使用了 clap 的内置验证功能？

### 4.2 命令结构检查

**检查目标**：确保命令层级和分组合理

**检查清单**：
- [ ] 子命令层级是否合理（不超过 2 层）？
- [ ] 命令分组是否合理（相关命令是否在同一模块）？
- [ ] 命令别名是否一致（如 `-n` vs `--dry-run`）？

### 4.3 向后兼容性检查

**检查目标**：确保参数变更不影响现有脚本

**检查清单**：
- [ ] 参数重命名是否影响现有脚本？
- [ ] 参数移除是否已标记为废弃？
- [ ] 新参数是否与现有参数冲突？

---

## 📋 检查清单

### 完整性检查清单

- [ ] 新增命令是否已在 `Commands` 枚举中注册
- [ ] 子命令是否已在对应枚举中定义
- [ ] 命令文档注释（`///`）是否完整
- [ ] 参数文档注释（`///`）是否完整
- [ ] 参数命名是否一致（如 `jira-_id` vs `jira-_ticket`）
- [ ] 运行补全完整性测试：`cargo test --test completeness`
- [ ] 新增命令是否包含在补全脚本中
- [ ] 所有 shell 类型是否正常生成

### 参数化检查清单

- [ ] 需要输出格式的命令是否使用了 `OutputFormatArgs`？
- [ ] 需要 dry-run 模式的命令是否使用了 `DryRunArgs`？
- [ ] 需要 JIRA ID 的命令是否使用了 `JiraIdArg`？
- [ ] 是否有参数在 2+ 个命令中重复出现？
- [ ] 重复参数是否已提取为共用参数组？
- [ ] 是否使用 `#[command(flatten)]` 复用参数组？

### 其他检查清单

- [ ] 参数验证逻辑是否统一
- [ ] 命令层级是否合理
- [ ] 命令分组是否合理
- [ ] 参数变更是否影响向后兼容性

---

## 🛠️ 工具和测试

### 自动化检查工具

#### 补全完整性测试

**位置**：`tests/completion/completeness.rs`

**运行命令**：
```bash
cargo test --test completeness
```

**检查内容**：
- CLI 结构包含所有顶级命令
- 所有子命令完整性
- 补全脚本生成功能
- 补全脚本文件命名

#### 参数检查测试

**位置**：`tests/args-_check.rs`

**运行命令**：
```bash
cargo test --test args-_check
```

**检查内容**：
- 是否应该使用 `JiraIdArg` 但使用了自定义参数
- 是否应该使用 `OutputFormatArgs` 但使用了自定义参数
- 是否应该使用 `DryRunArgs` 但使用了自定义参数
- 参数命名是否一致

**参考**：见 `tests/args-_check.rs`

### 手动检查工具

#### 查找重复参数

```bash
# 查找所有 jira 相关参数
grep -r "jira" src/lib/cli/ --include="*.rs" -i | grep "#\[arg"

# 查找所有 dry-run 相关参数
grep -r "dry.*run\|dry-_run" src/lib/cli/ --include="*.rs" -i

# 查找所有输出格式相关参数
grep -r "json\|yaml\|table\|markdown" src/lib/cli/ --include="*.rs" | grep "#\[arg"
```

#### 生成补全脚本

```bash
# 生成所有 shell 类型的补全脚本
cargo run -- completion generate

# 生成特定 shell 类型的补全脚本
cargo run -- completion generate --shell zsh
```

---

## 📚 参考文档

- [开发规范](../development.md) - 包含参数命名规范
- [提交前检查指南](../pre-commit.md) - 快速检查清单
- [代码检查指南](./review-code.md) - 代码优化检查
- [CLI 架构文档](../../architecture/cli.md) - CLI 架构设计

---

## 🔄 更新历史

- 2025-01-XX：初始版本，包含完整性检查、参数化检查和参数抽取指南

---

**最后更新**: 2025-12-16
