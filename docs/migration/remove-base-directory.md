# 移除 base 目录迁移文档

## 文档信息

- **创建日期**: 2025-01-27
- **状态**: 迁移计划（待执行）
- **目标**: 将 `src/lib/base/` 下的所有子模块提升到 `src/lib/` 下，移除 `base` 目录

## 目录

1. [迁移目标](#迁移目标)
2. [影响分析](#影响分析)
3. [迁移步骤](#迁移步骤)
4. [检查清单](#检查清单)
5. [测试计划](#测试计划)
6. [回滚方案](#回滚方案)
7. [风险评估](#风险评估)

---

## 迁移目标

### 当前结构

```
src/lib/
├── base/              # 基础设施模块（待移除）
│   ├── alias/
│   ├── concurrent/
│   ├── constants/
│   ├── format/
│   ├── http/
│   ├── interactive/
│   ├── llm/
│   ├── logger/
│   ├── mcp/
│   ├── prompt/
│   ├── settings/
│   ├── shell/
│   └── util/
├── branch/            # 业务模块
├── jira/
├── pr/
└── ...
```

### 目标结构

```
src/lib/
├── alias/             # 从 base/ 提升
├── concurrent/        # 从 base/ 提升
├── constants/         # 从 base/ 提升
├── format/            # 从 base/ 提升
├── http/              # 从 base/ 提升
├── interactive/       # 从 base/ 提升
├── llm/               # 从 base/ 提升
├── logger/            # 从 base/ 提升
├── mcp/               # 从 base/ 提升
├── prompt/            # 从 base/ 提升
├── settings/          # 从 base/ 提升
├── shell/             # 从 base/ 提升
├── util/              # 从 base/ 提升
├── branch/            # 业务模块（保持不变）
├── jira/              # 业务模块（保持不变）
├── pr/                # 业务模块（保持不变）
└── infra/             # 基础设施服务层（已存在）
```

### 迁移目标

1. **扁平化结构**: 移除 `base` 中间层，所有模块直接在 `lib/` 下
2. **统一命名空间**: 基础设施模块与业务模块处于同一层级
3. **简化引用**: `crate::base::xxx` → `crate::xxx`
4. **保持功能**: 所有功能保持不变，仅重构目录结构

---

## 影响分析

### 统计数据

- **受影响文件**: 126 个文件
- **引用总数**: 319 处 `crate::base::` 引用
- **需要移动的模块**: 13 个子模块

### 需要移动的模块

| 模块 | 当前路径 | 目标路径 | 文件数 |
|------|---------|---------|--------|
| alias | `lib/base/alias/` | `lib/alias/` | 3 |
| concurrent | `lib/base/concurrent/` | `lib/concurrent/` | 2 |
| constants | `lib/base/constants/` | `lib/constants/` | 6 |
| format | `lib/base/format/` | `lib/format/` | 3 |
| http | `lib/base/http/` | `lib/http/` | 8 |
| interactive | `lib/base/interactive/` | `lib/interactive/` | 20+ |
| llm | `lib/base/llm/` | `lib/llm/` | 4 |
| logger | `lib/base/logger/` | `lib/logger/` | 4 |
| mcp | `lib/base/mcp/` | `lib/mcp/` | 2 |
| prompt | `lib/base/prompt/` | `lib/prompt/` | 6 |
| settings | `lib/base/settings/` | `lib/settings/` | 7 |
| shell | `lib/base/shell/` | `lib/shell/` | 4 |
| util | `lib/base/util/` | `lib/util/` | 10 |

### 引用模式分析

#### 1. 模块内部引用

```rust
// 当前
use crate::base::util::directory::DirectoryWalker;

// 迁移后
use crate::util::directory::DirectoryWalker;
```

#### 2. 跨模块引用

```rust
// 当前
use crate::base::http::HttpClient;
use crate::base::settings::Settings;

// 迁移后
use crate::http::HttpClient;
use crate::settings::Settings;
```

#### 3. 外部 API 导出

```rust
// 当前 (src/lib.rs)
pub use base::format::DisplayFormatter;
pub use base::settings::{LLMSettings, Paths, Settings};

// 迁移后
pub use format::DisplayFormatter;
pub use settings::{LLMSettings, Paths, Settings};
```

#### 4. 文档注释中的引用

```rust
// 当前
/// use workflow::base::http::HttpClient;

// 迁移后
/// use workflow::http::HttpClient;
```

---

## 迁移步骤

### 阶段 1: 准备工作

#### 1.1 创建备份分支

```bash
git checkout -b migration/remove-base-directory
git push -u origin migration/remove-base-directory
```

#### 1.2 验证当前状态

```bash
# 确保所有测试通过
cargo test

# 确保代码可以编译
cargo check

# 记录当前状态
git commit -m "chore: 记录迁移前的状态"
```

### 阶段 2: 目录移动

#### 2.1 移动所有子模块

```bash
# 在 src/lib/ 目录下执行
cd src/lib

# 移动所有 base 子目录到 lib 根目录
mv base/alias .
mv base/concurrent .
mv base/constants .
mv base/format .
mv base/http .
mv base/interactive .
mv base/llm .
mv base/logger .
mv base/mcp .
mv base/prompt .
mv base/settings .
mv base/shell .
mv base/util .

# 删除空的 base 目录
rmdir base
```

#### 2.2 验证目录结构

```bash
# 检查目录是否移动成功
ls -la src/lib/ | grep -E "(alias|concurrent|constants|format|http|interactive|llm|logger|mcp|prompt|settings|shell|util)"

# 确认 base 目录已删除
test ! -d src/lib/base && echo "base 目录已成功删除"
```

### 阶段 3: 更新模块声明

#### 3.1 更新 `src/lib.rs`

**当前代码**:
```rust
// 核心库模块声明
#[path = "lib/base/mod.rs"]
pub mod base;
#[path = "lib/branch/mod.rs"]
pub mod branch;
// ...
```

**更新后**:
```rust
// 核心库模块声明
#[path = "lib/alias/mod.rs"]
pub mod alias;
#[path = "lib/concurrent/mod.rs"]
pub mod concurrent;
#[path = "lib/constants/mod.rs"]
pub mod constants;
#[path = "lib/format/mod.rs"]
pub mod format;
#[path = "lib/http/mod.rs"]
pub mod http;
#[path = "lib/interactive/mod.rs"]
pub mod interactive;
#[path = "lib/llm/mod.rs"]
pub mod llm;
#[path = "lib/logger/mod.rs"]
pub mod logger;
#[path = "lib/mcp/mod.rs"]
pub mod mcp;
#[path = "lib/prompt/mod.rs"]
pub mod prompt;
#[path = "lib/settings/mod.rs"]
pub mod settings;
#[path = "lib/shell/mod.rs"]
pub mod shell;
#[path = "lib/util/mod.rs"]
pub mod util;
#[path = "lib/branch/mod.rs"]
pub mod branch;
// ... 其他业务模块保持不变
```

#### 3.2 更新 `pub use` 语句

**当前代码**:
```rust
// 从 base 模块重新导出基础设施类型，保持向后兼容
pub use base::format::DisplayFormatter;
pub use base::settings::{LLMSettings, Paths, Settings};
pub use base::util::{mask_sensitive_value, Browser, Checksum, Clipboard, Unzip};
pub use base::{
    Authorization, Detect, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig, LogLevel, Reload,
    ShellConfigManager, Logger,
};
pub use base::prompt::{
    find_language, generate_summarize_pr_system_prompt, get_language_instruction,
    get_supported_language_codes, get_supported_language_display_names, SupportedLanguage,
    GENERATE_BRANCH_SYSTEM_PROMPT, SUPPORTED_LANGUAGES,
};
pub use base::llm::get_language_requirement;
```

**更新后**:
```rust
// 重新导出基础设施类型
pub use format::DisplayFormatter;
pub use settings::{LLMSettings, Paths, Settings};
pub use util::{mask_sensitive_value, Browser, Checksum, Clipboard, Unzip};
pub use http::{Authorization, HttpClient, HttpResponse, HttpRetry, HttpRetryConfig};
pub use logger::{LogLevel, Logger};
pub use shell::{Detect, Reload, ShellConfigManager};
pub use prompt::{
    find_language, generate_summarize_pr_system_prompt, get_language_instruction,
    get_supported_language_codes, get_supported_language_display_names, SupportedLanguage,
    GENERATE_BRANCH_SYSTEM_PROMPT, SUPPORTED_LANGUAGES,
};
pub use llm::get_language_requirement;
```

### 阶段 4: 批量替换引用

#### 4.1 替换 `crate::base::` 为 `crate::`

```bash
# 使用 sed 或 find + sed 批量替换
find src -type f -name "*.rs" -exec sed -i '' 's/crate::base::/crate::/g' {} +
```

#### 4.2 替换 `workflow::base::` 为 `workflow::`（文档中）

```bash
# 替换文档注释中的引用
find src -type f -name "*.rs" -exec sed -i '' 's/workflow::base::/workflow::/g' {} +
```

#### 4.3 更新模块内部引用

某些模块内部可能使用了 `crate::base::` 引用其他 base 模块，需要手动检查：

```bash
# 查找所有模块内部的 base 引用
grep -r "crate::base::" src/lib/alias/
grep -r "crate::base::" src/lib/settings/
grep -r "crate::base::" src/lib/util/
# ... 其他模块
```

### 阶段 5: 更新模块内部引用

#### 5.1 检查模块间的相互引用

某些模块可能引用了其他 base 模块，需要更新：

**示例：`src/lib/settings/paths.rs`**
```rust
// 当前
use crate::base::util::directory::DirectoryWalker;

// 更新后
use crate::util::directory::DirectoryWalker;
```

#### 5.2 更新 infra 模块中的引用

**`src/lib/infra/adapters/config/settings.rs`**
```rust
// 当前
use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;
use crate::base::LogLevel;

// 更新后
use crate::settings::paths::Paths;
use crate::settings::Settings;
use crate::LogLevel;
```

**`src/lib/infra/adapters/config/env.rs`**
```rust
// 当前
use crate::base::LogLevel;

// 更新后
use crate::LogLevel;
```

**`src/lib/infra/traits/config_provider.rs`**
```rust
// 当前
use crate::base::LogLevel;

// 更新后
use crate::LogLevel;
```

### 阶段 6: 更新测试文件

#### 6.1 更新测试中的引用

```bash
# 查找测试文件中的 base 引用
find tests -type f -name "*.rs" -exec sed -i '' 's/crate::base::/crate::/g' {} +
```

#### 6.2 更新测试模块声明

如果测试中有模块声明，也需要更新：

```rust
// 当前
#[path = "../../../src/lib/base/settings/mod.rs"]
mod settings;

// 更新后
#[path = "../../../src/lib/settings/mod.rs"]
mod settings;
```

### 阶段 7: 更新文档

#### 7.1 更新代码文档注释

文档注释中的 `workflow::base::` 引用需要更新为 `workflow::`

#### 7.2 更新架构文档

更新 `docs/architecture.md` 中的模块结构说明

---

## 检查清单

### 目录结构检查

- [ ] 所有 13 个子模块已从 `lib/base/` 移动到 `lib/`
- [ ] `lib/base/` 目录已删除
- [ ] 目录结构符合目标结构

### 模块声明检查

- [ ] `src/lib.rs` 中已添加所有新模块的声明
- [ ] `src/lib.rs` 中已移除 `pub mod base;`
- [ ] `src/lib.rs` 中的 `pub use` 语句已更新

### 引用更新检查

- [ ] 所有 `crate::base::` 已替换为 `crate::`
- [ ] 所有 `workflow::base::` 已替换为 `workflow::`
- [ ] 模块内部引用已更新
- [ ] infra 模块中的引用已更新

### 编译检查

- [ ] `cargo check` 通过
- [ ] `cargo build` 通过
- [ ] 无编译错误
- [ ] 无编译警告（或只有预期的警告）

### 测试检查

- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 测试覆盖率不降低

### 文档检查

- [ ] 代码文档注释已更新
- [ ] 架构文档已更新
- [ ] README 中的引用已更新（如有）

---

## 测试计划

### 单元测试

```bash
# 运行所有单元测试
cargo test --lib

# 运行特定模块的测试
cargo test --lib alias
cargo test --lib settings
cargo test --lib http
# ...
```

### 集成测试

```bash
# 运行所有集成测试
cargo test --test '*'

# 运行特定功能的集成测试
cargo test --test settings
```

### 功能测试

```bash
# 测试 CLI 命令
cargo run -- pr create --help
cargo run -- jira info --help
cargo run -- config show
# ...
```

### 编译测试

```bash
# 检查编译
cargo check

# 检查所有目标
cargo check --all-targets

# 检查 release 构建
cargo check --release
```

---

## 回滚方案

### 如果迁移失败

#### 方案 1: Git 回滚

```bash
# 回滚到迁移前的提交
git reset --hard HEAD~1

# 或回滚到特定提交
git reset --hard <commit-hash>
```

#### 方案 2: 手动恢复

如果已经部分迁移，可以手动恢复：

```bash
# 恢复目录结构
cd src/lib
mkdir base
mv alias concurrent constants format http interactive llm logger mcp prompt settings shell util base/

# 恢复 src/lib.rs
git checkout HEAD -- src/lib.rs

# 恢复所有文件中的引用
git checkout HEAD -- src/
```

### 回滚检查清单

- [ ] 目录结构已恢复
- [ ] 所有引用已恢复
- [ ] 代码可以编译
- [ ] 测试通过

---

## 风险评估

### 高风险项 ⚠️

1. **大量引用更新**: 319 处引用需要更新，容易遗漏
   - **缓解措施**: 使用自动化脚本批量替换，然后手动检查

2. **模块间依赖**: 某些模块可能相互依赖，需要仔细检查
   - **缓解措施**: 先编译检查，逐个修复错误

3. **测试覆盖**: 可能遗漏某些边界情况
   - **缓解措施**: 运行完整的测试套件

### 中风险项 ⚠️

1. **文档更新**: 文档中的引用可能遗漏
   - **缓解措施**: 使用 grep 查找所有 `base::` 引用

2. **外部依赖**: 如果有外部代码依赖 `workflow::base::`
   - **缓解措施**: 检查是否有外部使用者，提前通知

### 低风险项 ✅

1. **功能变更**: 仅重构目录结构，不改变功能
2. **API 兼容**: 通过 `pub use` 重新导出，保持 API 兼容
3. **编译错误**: Rust 编译器会捕获所有引用错误

---

## 自动化脚本

### 批量替换脚本

```bash
#!/bin/bash
# migrate-base-directory.sh

set -e

echo "开始迁移 base 目录..."

# 1. 移动目录
echo "步骤 1: 移动目录..."
cd src/lib
for dir in alias concurrent constants format http interactive llm logger mcp prompt settings shell util; do
    if [ -d "base/$dir" ]; then
        mv "base/$dir" .
        echo "  ✓ 移动 $dir"
    fi
done
rmdir base
echo "  ✓ 删除 base 目录"

# 2. 替换引用
echo "步骤 2: 替换引用..."
cd ../..
find src -type f -name "*.rs" -exec sed -i '' 's/crate::base::/crate::/g' {} +
find src -type f -name "*.rs" -exec sed -i '' 's/workflow::base::/workflow::/g' {} +
find tests -type f -name "*.rs" -exec sed -i '' 's/crate::base::/crate::/g' {} +
echo "  ✓ 替换完成"

# 3. 检查编译
echo "步骤 3: 检查编译..."
cargo check 2>&1 | head -50

echo "迁移完成！请手动更新 src/lib.rs 和检查编译错误。"
```

### 验证脚本

```bash
#!/bin/bash
# verify-migration.sh

set -e

echo "验证迁移结果..."

# 检查 base 目录是否已删除
if [ -d "src/lib/base" ]; then
    echo "❌ base 目录仍然存在"
    exit 1
fi
echo "✓ base 目录已删除"

# 检查所有模块是否已移动
modules=("alias" "concurrent" "constants" "format" "http" "interactive" "llm" "logger" "mcp" "prompt" "settings" "shell" "util")
for module in "${modules[@]}"; do
    if [ ! -d "src/lib/$module" ]; then
        echo "❌ $module 模块未找到"
        exit 1
    fi
done
echo "✓ 所有模块已移动"

# 检查是否还有 base 引用
if grep -r "crate::base::" src/ --include="*.rs" | grep -v "migration" > /dev/null; then
    echo "⚠️  仍有 crate::base:: 引用，请检查："
    grep -r "crate::base::" src/ --include="*.rs" | head -10
else
    echo "✓ 无残留的 crate::base:: 引用"
fi

# 检查编译
echo "检查编译..."
if cargo check 2>&1 | grep -q "error"; then
    echo "❌ 编译错误，请检查"
    cargo check 2>&1 | grep "error" | head -10
    exit 1
else
    echo "✓ 编译通过"
fi

echo "验证完成！"
```

---

## 后续工作

### 迁移后的清理

1. **移除迁移脚本**: 迁移完成后删除临时脚本
2. **更新 CI/CD**: 确保 CI 配置不需要更新
3. **更新文档**: 更新所有相关文档
4. **代码审查**: 进行完整的代码审查

### 长期优化

1. **统一命名**: 考虑是否需要统一模块命名规范
2. **依赖分析**: 分析模块间的依赖关系，优化结构
3. **文档完善**: 更新架构文档，说明新的模块结构

---

## 参考资料

- [Rust 模块系统](https://doc.rust-lang.org/book/ch07-02-defining-modules-to-control-scope-and-privacy.html)
- [Cargo 项目布局](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [项目架构文档](../architecture.md)

---

**文档状态**: 迁移计划完成，待执行

**最后更新**: 2025-01-27
