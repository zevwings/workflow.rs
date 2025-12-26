# GitHub Workflows 脚本 Rust 迁移 TODO

## 📋 概述

本文档列出了将 `.github/workflows` 目录中的脚本逻辑迁移到 Rust 实现的待办事项。

**相关文档**: [`analysis/github-workflows-rust-migration.md`](../../analysis/github-workflows-rust-migration.md)

**状态**: 📋 待实施

**实现度**: 0%

---

## ✅ 已迁移到 Rust 的功能

以下功能已经通过 `cargo run --bin dev` 命令实现：

1. ✅ **测试报告生成** (`dev tests report generate`)
2. ✅ **测试指标收集** (`dev tests metrics collect`)
3. ✅ **性能回归分析** (`dev performance analyze`)
4. ✅ **测试趋势分析** (`dev tests trends analyze`)
5. ✅ **文档完整性检查** (`dev docs check integrity`)
6. ✅ **文档链接检查** (`dev docs check links`)

---

## 📋 待迁移功能 TODO

### 🔴 高优先级（复杂逻辑，适合 Rust）

#### 1. 版本号生成逻辑

- [ ] **命令**: `dev version generate`
- [ ] **位置**: `release.yml` lines 333-489
- [ ] **功能**:
  - [ ] 解析 git tags 获取最新版本
  - [ ] 分析 commit messages（Conventional Commits）
  - [ ] 根据提交类型确定版本递增策略（major/minor/patch）
  - [ ] 生成标准版本号或预发布版本号
  - [ ] 更新 Cargo.toml 和 Cargo.lock
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/version/mod.rs`
  - [ ] 实现版本号解析和生成
  - [ ] 集成 Conventional Commits 解析
- [ ] **依赖**: `git2` crate（已使用）

---

#### 2. Git Tag 操作 ⏸️ 延后实现

- [ ] **命令**: `dev tag create`
- [ ] **位置**: `release.yml` lines 1335-1459
- [ ] **功能**:
  - [ ] 创建 git tag
  - [ ] 推送 tag 到远程
  - [ ] 验证 tag 状态
  - [ ] 处理冲突情况
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/tag/mod.rs`
  - [ ] 实现 tag 创建、推送、验证
  - [ ] 处理冲突情况
- [ ] **依赖**: `git2` crate（已使用）
- [ ] **⚠️ 状态**: ⏸️ **延后实现** - git2 迁移存在一些问题，待解决后再实现

---

#### 3. Alpha Tag 清理 ⏸️ 延后实现

- [ ] **命令**: `dev tag cleanup`
- [ ] **位置**: `release.yml` lines 1760-1895
- [ ] **功能**:
  - [ ] 查找所有 alpha tags
  - [ ] 分析 tag 与合并提交的关系
  - [ ] 删除已合并的 alpha tags
- [ ] **实现**:
  - [ ] 扩展 `src/commands/dev/tag/mod.rs`
  - [ ] 实现清理逻辑
  - [ ] 分析 Git 历史
- [ ] **依赖**: `git2` crate（已使用）
- [ ] **⚠️ 状态**: ⏸️ **延后实现** - git2 迁移存在一些问题，待解决后再实现

---

#### 4. PR 创建和合并 ⏸️ 延后实现

- [ ] **命令**: `dev pr create` / `dev pr merge`
- [ ] **位置**: `release.yml` lines 959-1268
- [ ] **功能**:
  - [ ] 创建版本更新 PR
  - [ ] 检查 PR 状态
  - [ ] 等待 CI 完成
  - [ ] 合并 PR
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/pr/mod.rs`
  - [ ] 实现 GitHub API 客户端
  - [ ] 处理 PR 生命周期
- [ ] **依赖**: `octocrab` 或 `reqwest` + GitHub API
- [ ] **⚠️ 状态**: ⏸️ **延后实现** - git2 迁移存在一些问题，待解决后再实现

---

### 🟡 中优先级（中等复杂度）

#### 5. CI 跳过检查

- [ ] **命令**: `dev ci check-skip`
- [ ] **位置**: `ci.yml` lines 36-71
- [ ] **功能**:
  - [ ] 检查分支名称（是否为 `bump-version-*`）
  - [ ] 验证 PR 创建者
  - [ ] 输出 should_skip 标志
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/ci/mod.rs`
  - [ ] 实现分支检查逻辑
- [ ] **依赖**: `git2` crate（已使用）

---

#### 6. 文档检查报告生成

- [ ] **命令**: `dev docs report generate`
- [ ] **位置**: `document-check.yml` lines 92-143
- [ ] **功能**:
  - [ ] 生成文档检查报告（基于检查结果或 GITHUB_OUTPUT）
  - [ ] 统一的报告模板系统
  - [ ] 支持多种报告格式（Markdown、HTML 等）
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/docs/report.rs`
  - [ ] 实现模板系统
  - [ ] 基于文档检查结果生成报告
- [ ] **依赖**: 模板引擎 crate（`handlebars` 或 `tera`）

---

#### 7. CI 检查验证（门控）

- [ ] **命令**: `dev ci verify`
- [ ] **位置**: `ci.yml` lines 898-1008
- [ ] **功能**:
  - [ ] 验证所有 CI job 的状态
  - [ ] 汇总检查结果
  - [ ] 输出最终验证状态（通过/失败）
  - [ ] 支持分支保护规则集成
- [ ] **实现**:
  - [ ] 扩展 `src/commands/dev/ci/mod.rs`
  - [ ] 实现状态检查逻辑
- [ ] **依赖**: GitHub Actions 上下文（`needs`）

---

### 🟢 低优先级（简单脚本，可保持现状）

#### 8. Homebrew Formula 更新

- [ ] **命令**: `dev homebrew update`
- [ ] **位置**: `release.yml` lines 1627-1736
- [ ] **功能**:
  - [ ] 从模板生成或更新 Formula 文件
  - [ ] 更新版本号和下载 URL
  - [ ] 验证 Formula 文件语法
  - [ ] Git 操作（配置、提交、推送）
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/homebrew/mod.rs`
  - [ ] 实现模板系统
  - [ ] 实现 Git 操作
- [ ] **依赖**:
  - [ ] 模板引擎 crate（`handlebars` 或 `tera`）
  - [ ] `git2` crate（已使用）
  - [ ] 可选：Ruby 语法验证（可通过外部命令调用）
- [ ] **注意**: 如果只是简单的文本替换，可以保持 Shell 脚本

---

#### 9. 文件哈希计算

- [ ] **命令**: `dev checksum calculate`
- [ ] **位置**: `release.yml` lines 614-626, 699-711, 851-866
- [ ] **功能**:
  - [ ] 计算文件 SHA256
  - [ ] 跨平台支持
- [ ] **实现**:
  - [ ] 创建 `src/commands/dev/checksum/mod.rs`
  - [ ] 实现哈希计算逻辑
- [ ] **依赖**: `sha2` crate
- [ ] **注意**: 可选迁移，Shell 脚本已足够简单

---

## 🚀 实施计划

### 阶段 1: 立即实现（不依赖 git2 迁移）

#### 1. 版本号生成逻辑
- [ ] 创建 `src/commands/dev/version/mod.rs`
- [ ] 实现版本号解析和生成
- [ ] 集成 Conventional Commits 解析
- [ ] **注意**: 需要 git2，但版本号生成逻辑相对独立，可以先实现

#### 5. CI 跳过检查
- [ ] 创建 `src/commands/dev/ci/mod.rs`
- [ ] 实现分支检查逻辑
- [ ] 验证 PR 创建者

#### 6. 文档检查报告生成
- [ ] 创建 `src/commands/dev/docs/report.rs`
- [ ] 实现模板系统
- [ ] 基于文档检查结果生成报告

#### 7. CI 检查验证（门控）
- [ ] 扩展 `src/commands/dev/ci/mod.rs`
- [ ] 实现状态检查逻辑
- [ ] 支持分支保护规则集成

#### 8. Homebrew Formula 更新
- [ ] 创建 `src/commands/dev/homebrew/mod.rs`
- [ ] 实现模板系统
- [ ] 实现 Git 操作（可使用 git2，但主要是文件操作）

#### 9. 文件哈希计算
- [ ] 创建 `src/commands/dev/checksum/mod.rs`
- [ ] 实现哈希计算逻辑

---

### 阶段 2: 延后实现（等待 git2 迁移问题解决）⏸️

#### 2. Git Tag 操作
- [ ] 创建 `src/commands/dev/tag/mod.rs`
- [ ] 实现 tag 创建、推送、验证
- [ ] 处理冲突情况
- [ ] **⚠️ 状态**: 等待 git2 迁移问题解决

#### 3. Alpha Tag 清理
- [ ] 扩展 `src/commands/dev/tag/mod.rs`
- [ ] 实现清理逻辑
- [ ] 分析 Git 历史
- [ ] **⚠️ 状态**: 等待 git2 迁移问题解决

#### 4. PR 创建和合并
- [ ] 创建 `src/commands/dev/pr/mod.rs`
- [ ] 实现 GitHub API 客户端
- [ ] 处理 PR 生命周期
- [ ] **⚠️ 状态**: 等待 git2 迁移问题解决（PR 操作可能需要 Git 操作）

---

## 📊 优先级总结

| 优先级 | 功能 | 复杂度 | 收益 | 实现顺序 | 状态 |
|--------|------|--------|------|----------|------|
| 🔴 高 | 版本号生成 | 高 | 高 | 阶段 1 | ✅ 立即实现 |
| 🟡 中 | CI 跳过检查 | 低 | 低 | 阶段 1 | ✅ 立即实现 |
| 🟡 中 | 文档检查报告生成 | 中 | 中 | 阶段 1 | ✅ 立即实现 |
| 🟡 中 | CI 检查验证 | 中 | 低 | 阶段 1 | ✅ 立即实现 |
| 🟢 低 | Homebrew Formula | 低 | 低 | 阶段 1 | ✅ 立即实现 |
| 🟢 低 | 文件哈希计算 | 低 | 低 | 阶段 1 | ✅ 立即实现 |
| 🔴 高 | Git Tag 操作 | 中 | 高 | 阶段 2 | ⏸️ 延后（git2 迁移问题） |
| 🔴 高 | Alpha Tag 清理 | 高 | 中 | 阶段 2 | ⏸️ 延后（git2 迁移问题） |
| 🔴 高 | PR 创建和合并 | 中 | 高 | 阶段 2 | ⏸️ 延后（git2 迁移问题） |

---

## 📝 注意事项

1. **git2 迁移问题** ⚠️
   - Git Tag 操作、Alpha Tag 清理、PR 创建和合并功能依赖 git2
   - 当前 git2 迁移存在一些问题，这些功能延后实现
   - 待 git2 迁移问题解决后，再实现阶段 2 的功能

2. **GitHub API 认证**
   - 需要支持 `GITHUB_TOKEN` 和 `WORKFLOW_PAT`
   - 实现安全的 token 管理

3. **跨平台兼容性**
   - 确保所有 Git 操作在 Windows/macOS/Linux 上正常工作
   - 处理路径差异

4. **错误处理**
   - 提供清晰的错误信息
   - 支持 `continue-on-error` 场景

5. **向后兼容**
   - 保持现有工作流的接口
   - 逐步迁移，不破坏现有流程

6. **测试**
   - 为每个新功能添加单元测试
   - 集成测试验证工作流集成

---

## 🔗 相关文档

- [迁移分析报告](../../analysis/github-workflows-rust-migration.md)
- [开发工具脚本 README](../../../scripts/dev/README.md)
- [dev 命令实现状态](../../../analysis/dev-commands-implementation-status.md)

---

**最后更新**: 2025-12-25

