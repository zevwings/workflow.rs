# CI Workflow 指南

> 本文档描述了 CI Workflow 的配置、使用和故障排除指南。

---

## 📋 目录

- [概述](#-概述)
- [触发条件](#-触发条件)
- [Jobs 详细说明](#-jobs-详细说明)
- [与 Release Workflow 的关系](#-与-release-workflow-的关系)
- [版本更新分支的特殊处理](#-版本更新分支的特殊处理)
- [故障排除](#-故障排除)
- [最佳实践](#-最佳实践)

---

## 📋 概述

CI Workflow (`.github/workflows/ci.yml`) 用于在 Pull Request 时进行代码质量检查和测试，确保只有通过检查的代码才能合并到 master 分支。

### 主要功能

1. **代码质量检查**：格式化检查、Clippy 检查、编译检查
2. **测试验证**：运行完整的测试套件
3. **状态汇总**：汇总所有检查结果，满足分支保护规则要求
4. **版本更新分支处理**：自动跳过由 release workflow 创建的版本更新分支

---

## 🚀 触发条件

CI Workflow 在以下情况下触发：

### 1. Pull Request

**触发条件**：
- PR 目标分支为 `master` 或 `main`
- 任何源分支的 PR 都会触发

**行为**：
- 对所有 PR 运行 CI 检查
- 版本更新分支（`bump-version-*`）会跳过 lint 和测试，但会通过状态检查

### 2. 手动触发（workflow-_dispatch）

**触发条件**：
- 在 GitHub Actions 页面手动触发

**用途**：
- 测试 CI 配置
- 特殊情况下的验证

---

## 🔧 Jobs 详细说明

### 1. check-skip-ci

**目的**：检查是否应该跳过 CI 检查（仅针对版本更新分支）

**执行逻辑**：
1. 检查 PR 源分支是否为 `bump-version-*` 格式
2. 如果是版本更新分支：
   - 验证 PR 创建者是否与 `WORKFLOW_USER_NAME` 匹配
   - 如果匹配，设置 `should-_skip=true`
   - 如果不匹配，失败并报错
3. 如果不是版本更新分支，设置 `should-_skip=false`

**输出**：
- `should-_skip`: `true` 或 `false`

**安全机制**：
- 只有由 release workflow 创建的版本更新 PR 才能跳过 CI
- 防止恶意创建版本更新分支绕过 CI 检查

### 2. check-lint

**目的**：代码质量检查

**执行条件**：
- `check-skip-ci.outputs.should-_skip != 'true'`
- 即：非版本更新分支会执行此 job

**步骤**：
1. Checkout code
2. Setup Rust toolchain（包含 rustfmt 和 clippy）
3. Cache cargo registry
4. 安装系统依赖（Linux 需要 XCB 开发库）
5. `cargo fmt --check`：检查代码格式
6. `cargo clippy -- -D warnings`：运行 Clippy 检查
7. `cargo check`：编译检查

**失败条件**：
- 代码格式不符合要求
- Clippy 检查发现警告或错误
- 编译失败

### 3. tests

**目的**：运行测试套件

**执行条件**：
- `check-skip-ci.outputs.should-_skip != 'true'`
- 即：非版本更新分支会执行此 job

**步骤**：
1. Checkout code
2. Setup Rust toolchain
3. Cache cargo registry
4. 安装系统依赖（Linux 需要 XCB 开发库）
5. `cargo test --verbose`：运行所有测试

**失败条件**：
- 任何测试失败

### 4. check-status

**目的**：汇总所有检查的状态，满足分支保护规则要求

**执行条件**：
- `always()`：无论依赖 job 成功、失败或跳过，都会执行

**执行逻辑**：
1. **优先级 1**：如果 `check-lint` 和 `tests` 都被跳过，直接通过（版本更新分支的情况）
2. **优先级 2**：如果 `check-skip-ci` 成功且 `should-_skip=true`，直接通过
3. **优先级 3**：检查 `check-lint` 和 `tests` 的结果
   - 如果有失败的检查，此 job 失败
   - 如果所有检查都成功或跳过，此 job 成功

**重要性**：
- 此 job 的状态用于分支保护规则
- 必须通过此 job 才能合并 PR

---

## 🔗 与 Release Workflow 的关系

### 工作流配合

```
Release Workflow (master push)
  ↓
创建 bump-version-* PR
  ↓
触发 CI Workflow
  ↓
check-skip-ci 验证 PR 创建者
  ↓
跳过 lint 和 tests
  ↓
check-status 通过
  ↓
PR 可以合并
  ↓
合并后触发 Release Workflow 的 create-tag job
```

### 关键配合点

1. **PR 创建者验证**：
   - Release workflow 使用 `WORKFLOW_PAT` 创建 PR
   - CI workflow 验证 PR 创建者是否与 `WORKFLOW_USER_NAME` 匹配
   - 确保只有合法的版本更新 PR 才能跳过 CI

2. **状态检查**：
   - CI workflow 的 `check-status` job 必须通过
   - 分支保护规则要求此状态检查通过才能合并
   - 版本更新分支会跳过 lint/tests，但 `check-status` 仍然会通过

3. **分支命名约定**：
   - 版本更新分支必须遵循 `bump-version-*` 命名模式
   - CI workflow 通过分支名识别版本更新分支

---

## 🔄 版本更新分支的特殊处理

### 为什么需要特殊处理？

版本更新分支（`bump-version-*`）只包含版本号的更改（`Cargo.toml` 和 `Cargo.lock`），这些更改：
- 不涉及代码逻辑
- 不需要重新运行 lint 和测试
- 应该快速合并，以便继续发布流程

### 处理流程

1. **创建 PR**：
   - Release workflow 创建 `bump-version-{version}` 分支
   - 更新 `Cargo.toml` 和 `Cargo.lock` 中的版本号
   - 创建 PR 到 master 分支

2. **CI 验证**：
   - `check-skip-ci` job 验证 PR 创建者
   - 如果创建者匹配 `WORKFLOW_USER_NAME`，设置 `should-_skip=true`
   - `check-lint` 和 `tests` job 被跳过
   - `check-status` job 通过（因为应该跳过）

3. **合并 PR**：
   - 分支保护规则检查 `check-status` 状态
   - 状态为成功，允许合并
   - Release workflow 继续执行（创建 tag、构建、发布）

### 安全机制

- **PR 创建者验证**：只有由 release workflow 创建的 PR 才能跳过 CI
- **分支命名验证**：只有 `bump-version-*` 格式的分支才能跳过
- **双重验证**：既检查分支名，也检查 PR 创建者

---

## 🔧 故障排除

### 问题 1：CI 没有运行

**症状**：创建 PR 后，CI workflow 没有触发

**可能原因**：
- PR 目标分支不是 `master` 或 `main`
- Workflow 文件语法错误
- GitHub Actions 被禁用

**解决方案**：
1. 检查 PR 目标分支
2. 检查 `.github/workflows/ci.yml` 语法
3. 检查仓库设置中的 Actions 权限

### 问题 2：版本更新分支的 CI 验证失败

**症状**：`check-skip-ci` job 失败，提示 PR 创建者不匹配

**可能原因**：
- `WORKFLOW_USER_NAME` 未配置或配置错误
- PR 创建者与 `WORKFLOW_USER_NAME` 不匹配
- `WORKFLOW_PAT` 的所有者与 `WORKFLOW_USER_NAME` 不一致

**解决方案**：
1. 检查 `WORKFLOW_USER_NAME` Repository Variable 是否配置
2. 确认 `WORKFLOW_USER_NAME` 与 `WORKFLOW_PAT` 的所有者匹配
3. 检查 PR 创建者是否是预期的用户
4. 参考 [github-setup.md](./github-setup.md) 配置 `WORKFLOW_USER_NAME`

### 问题 3：check-status 失败

**症状**：`check-status` job 失败，导致 PR 无法合并

**可能原因**：
- `check-lint` 或 `tests` job 失败
- `check-skip-ci` 逻辑错误
- 版本更新分支的验证失败

**解决方案**：
1. 检查 `check-lint` 和 `tests` job 的状态
2. 如果是版本更新分支，检查 `check-skip-ci` 是否成功
3. 查看 `check-status` job 的日志，了解失败原因
4. 修复代码问题或配置问题

### 问题 4：版本更新分支无法跳过 CI

**症状**：版本更新分支仍然运行 lint 和 tests

**可能原因**：
- 分支名不符合 `bump-version-*` 格式
- `check-skip-ci` job 失败
- `should-_skip` output 未正确设置

**解决方案**：
1. 检查分支名是否符合 `bump-version-*` 格式
2. 检查 `check-skip-ci` job 是否成功
3. 查看 `check-skip-ci` job 的输出，确认 `should-_skip` 的值
4. 检查 `check-lint` 和 `tests` job 的 `if` 条件

---

## 💡 最佳实践

### 1. 保持 CI 快速

- 使用缓存加速构建
- 并行运行 lint 和 tests
- 只运行必要的检查

### 2. 清晰的错误信息

- CI 失败时提供清晰的错误信息
- 指出具体的失败原因和修复建议
- 链接到相关文档

### 3. 安全验证

- 严格验证版本更新分支的创建者
- 防止绕过 CI 检查
- 确保只有合法的自动化流程可以跳过 CI

### 4. 监控和日志

- 定期检查 CI 运行情况
- 关注失败的工作流
- 及时处理配置和代码问题

### 5. 与 Release Workflow 配合

- 确保 `WORKFLOW_USER_NAME` 配置正确
- 理解版本更新分支的处理逻辑
- 不要手动创建 `bump-version-*` 分支

---

## 📚 相关文档

- [GitHub Setup Guidelines](./github-setup.md)：GitHub 配置指南
- [Release Workflow Analysis](../requirements/RELEASE_WORKFLOW_ANALYSIS.md)：Release Workflow 详细分析
- [Development Guidelines](./development.md)：开发规范
- [Testing Guidelines](./testing.md)：测试指南

---

## 🔄 配置检查清单

在配置 CI Workflow 后，使用以下清单验证：

### 基本功能
- [ ] PR 可以触发 CI
- [ ] `check-lint` job 正常运行
- [ ] `tests` job 正常运行
- [ ] `check-status` job 正确汇总状态

### 版本更新分支处理
- [ ] `check-skip-ci` job 正确识别版本更新分支
- [ ] 版本更新分支跳过 lint 和 tests
- [ ] 版本更新分支的 `check-status` 通过
- [ ] PR 创建者验证正常工作

### 分支保护规则
- [ ] `check-status` job 在分支保护规则中配置为必需
- [ ] PR 必须通过 `check-status` 才能合并
- [ ] 版本更新分支可以正常合并

---

**最后更新**: 2025-12-23
