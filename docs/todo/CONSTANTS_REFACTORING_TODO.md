# 常量化重构待办事项

**创建日期**: 2025-12-18
**优先级**: P2-P3 (中低优先级)
**状态**: 待实施
**预估工作量**: 2-4小时

---

## 📋 概述

基于代码分析，识别出项目中可以常量化的字符串、错误消息和配置值。通过常量化这些内容，可以提升代码一致性、维护性和可读性。

## 🎯 待办事项

### P1 优先级 - 立即实施

#### 1. 分支验证错误消息常量化
- **状态**: 待实施
- **位置**: `src/commands/branch/rename.rs:332-371`
- **任务**: 创建分支验证错误消息常量
- **当前问题**: 7种不同的分支验证错误消息硬编码
- **解决方案**:
  ```rust
  pub mod branch_validation {
      pub const EMPTY_NAME: &str = "Branch name cannot be empty";
      pub const INVALID_DOT_POSITION: &str = "Branch name cannot start or end with '.'";
      pub const DOUBLE_DOT: &str = "Branch name cannot contain '..'";
      pub const CONTAINS_SPACES: &str = "Branch name cannot contain spaces";
      pub const INVALID_SPECIAL_CHAR: &str = "Branch name cannot contain special character";
      pub const TRAILING_SLASH: &str = "Branch name cannot end with '/'";
      pub const DOUBLE_SLASH: &str = "Branch name cannot contain consecutive slashes '//'";
  }
  ```
- **工作量**: 30分钟
- **收益**: 高 (错误消息一致性)

#### 2. API URL 常量化
- **状态**: 待实施
- **位置**:
  - `src/lib/pr/github/platform.rs:545`
  - `src/commands/check/check.rs:43`
- **任务**: 统一管理 API URL 常量
- **当前问题**: GitHub API URL 在多处硬编码
- **解决方案**:
  ```rust
  pub mod api_urls {
      pub const GITHUB_API_BASE: &str = "https://api.github.com";
      pub const GITHUB_BASE: &str = "https://github.com";
      pub const GITHUB_DOMAIN: &str = "github.com";
  }
  ```
- **工作量**: 20分钟
- **收益**: 中 (便于维护和配置)

### P2 优先级 - 下个版本

#### 3. 网络连接错误消息常量化
- **状态**: 待实施
- **位置**: `src/lib/base/http/retry.rs:362-365`
- **任务**: 统一网络错误消息
- **当前问题**: 网络相关错误消息分散
- **解决方案**:
  ```rust
  pub mod network_errors {
      pub const TIMEOUT: &str = "Network timeout";
      pub const CONNECTION_FAILED: &str = "Connection failed";
      pub const RATE_LIMIT_EXCEEDED: &str = "Rate limit exceeded";
  }
  ```
- **工作量**: 20分钟
- **收益**: 中 (用户体验一致性)

#### 4. Git 检查错误消息常量化
- **状态**: 待实施
- **位置**: `src/commands/check/check.rs:27,52`
- **任务**: 统一 Git 检查相关错误消息
- **当前问题**: Git 检查错误消息硬编码
- **解决方案**:
  ```rust
  pub mod git_check_errors {
      pub const NOT_GIT_REPO: &str = "Git check failed: Not in a Git repository";
      pub const NETWORK_CHECK_FAILED: &str = "Network check failed";
      pub const TESTS_FAILED: &str = "Tests failed";
      pub const LINT_CHECK_FAILED: &str = "Lint check failed";
  }
  ```
- **工作量**: 30分钟
- **收益**: 中 (检查流程一致性)

### P3 优先级 - 可选实施

#### 5. 文件操作错误消息常量化
- **状态**: 可选
- **位置**: 主要在测试文件中
- **任务**: 统一文件操作错误消息
- **当前问题**: 测试中重复的文件操作错误消息
- **解决方案**:
  ```rust
  pub mod file_operation_errors {
      pub const CREATE_DIR_FAILED: &str = "Failed to create directory";
      pub const READ_FILE_FAILED: &str = "Failed to read file";
      pub const WRITE_FILE_FAILED: &str = "Failed to write file";
      pub const LOAD_FIXTURE_FAILED: &str = "Failed to load fixture";
  }
  ```
- **工作量**: 45分钟
- **收益**: 低 (主要用于完整性)

#### 6. 配置相关消息常量化
- **状态**: 可选
- **位置**: 配置验证相关文件
- **任务**: 统一配置相关消息
- **解决方案**:
  ```rust
  pub mod config_messages {
      pub const VALIDATION_FAILED: &str = "Configuration validation failed";
      pub const CONFIG_HEADER: &str = "Configuration";
      pub const UNSUPPORTED_SHELL: &str = "Unsupported shell type";
  }
  ```
- **工作量**: 20分钟
- **收益**: 低 (使用频率较低)

---

## 🏗️ 实施计划

### 第一阶段：创建常量模块结构
- [ ] 创建 `src/lib/base/constants/` 目录
- [ ] 创建以下模块文件：
  - [ ] `mod.rs` - 模块导出
  - [ ] `errors.rs` - 错误消息常量
  - [ ] `api.rs` - API URL 常量
  - [ ] `network.rs` - 网络相关常量
  - [ ] `git.rs` - Git 相关常量
- [ ] 在 `src/lib/base/mod.rs` 中导出常量模块

### 第二阶段：实施 P1 优先级常量化
- [ ] 实施分支验证错误消息常量化
  - [ ] 定义常量
  - [ ] 替换 `src/commands/branch/rename.rs` 中的硬编码字符串
  - [ ] 运行测试验证
- [ ] 实施 API URL 常量化
  - [ ] 定义 API URL 常量
  - [ ] 替换相关文件中的硬编码 URL
  - [ ] 运行测试验证

### 第三阶段：实施 P2 优先级常量化
- [ ] 实施网络连接错误消息常量化
- [ ] 实施 Git 检查错误消息常量化
- [ ] 全面测试和验证

### 第四阶段：可选实施 P3 优先级
- [ ] 文件操作错误消息常量化（按需）
- [ ] 配置相关消息常量化（按需）

---

## 📂 推荐的模块结构

```
src/lib/base/constants/
├── mod.rs              // 模块导出和重新导出
├── errors.rs           // 通用错误消息常量
├── api.rs              // API URL 和端点常量
├── network.rs          // 网络相关常量
├── git.rs              // Git 操作相关常量
└── validation.rs       // 验证相关常量
```

### 模块导出示例

**constants/mod.rs**:
```rust
//! 项目常量模块
//!
//! 统一管理项目中使用的字符串常量、错误消息、API URL 等

pub mod errors;
pub mod api;
pub mod network;
pub mod git;
pub mod validation;

// 重新导出常用常量
pub use errors::*;
pub use api::*;
```

---

## 📊 收益评估

### 预期收益
- ✅ **一致性提升**: 错误消息格式统一，用户体验更好
- ✅ **维护性改善**: 集中管理常量，修改更容易
- ✅ **国际化准备**: 为将来的多语言支持奠定基础
- ✅ **代码可读性**: 常量名称比硬编码字符串更有语义
- ✅ **重构安全**: IDE 可以更好地支持重构和查找引用

### 实施成本
- ⏱️ **开发时间**: 2-4 小时（根据实施范围）
- 🧪 **测试工作**: 需要验证所有引用位置正确
- 👀 **代码审查**: 需要仔细检查常量使用的正确性

### 风险评估
- 🟢 **低风险**: 主要是字符串替换，不影响核心业务逻辑
- 🟡 **注意事项**: 确保格式化字符串的参数正确传递
- 🟡 **测试覆盖**: 需要确保所有错误路径都有测试覆盖

---

## 🔧 实施指南

### 代码示例

**使用前**:
```rust
color_eyre::eyre::bail!("Branch name cannot be empty");
```

**使用后**:
```rust
use workflow::base::constants::validation::branch;
color_eyre::eyre::bail!("{}", branch::EMPTY_NAME);
```

### 格式化字符串处理

**使用前**:
```rust
color_eyre::eyre::bail!("Branch name cannot contain special character: '{}'", ch);
```

**使用后**:
```rust
use workflow::base::constants::validation::branch;
color_eyre::eyre::bail!("{}: '{}'", branch::INVALID_SPECIAL_CHAR, ch);
```

---

## ✅ 验收标准

### 功能验收
- [ ] 所有硬编码字符串已替换为常量引用
- [ ] 格式化字符串的参数传递正确
- [ ] 错误消息的语义和格式保持一致

### 质量验收
- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] `cargo fmt` 格式化通过
- [ ] `cargo clippy` 检查无警告
- [ ] 常量命名符合项目规范

### 文档验收
- [ ] 常量模块有完整的文档注释
- [ ] 使用示例清晰明确
- [ ] 相关开发文档已更新

---

## 🔄 检查清单

### 实施前检查
- [ ] 确认所有需要常量化的字符串位置
- [ ] 设计合理的常量模块结构和命名
- [ ] 确保不会影响现有功能

### 实施中检查
- [ ] 逐步替换，避免大批量修改
- [ ] 每个模块完成后运行相关测试
- [ ] 保持 Git 提交的原子性

### 实施后检查
- [ ] 全量测试通过
- [ ] 代码审查完成
- [ ] 文档更新完成
- [ ] 性能无回归

---

## 📚 相关文档

- [开发规范](../guidelines/DEVELOPMENT_GUIDELINES.md) - 常量命名和模块组织规范
- [代码质量改进TODO](./CODE_QUALITY_IMPROVEMENTS_TODO.md) - 相关的代码质量改进项目

---

## 📝 实施记录

### 完成状态
- [ ] 创建常量模块结构
- [ ] 分支验证错误消息常量化 (P1)
- [ ] API URL 常量化 (P1)
- [ ] 网络连接错误消息常量化 (P2)
- [ ] Git 检查错误消息常量化 (P2)
- [ ] 文件操作错误消息常量化 (P3, 可选)
- [ ] 配置相关消息常量化 (P3, 可选)

### 实施日志
- **2025-12-18**: 创建 TODO 文档，识别常量化需求

---

**最后更新**: 2025-12-18
