# Git 模块 gix 迁移计划 TODO

## 概述

本文档记录了将项目中的 Git 模块从命令行调用方式迁移到 `gix` (gitoxide) 纯 Rust 实现的详细计划。

## 迁移目标

- **性能提升**：消除 95% 的进程启动开销，提升 10-100 倍性能
- **部署简化**：减少对系统 Git 安装的依赖
- **类型安全**：使用强类型 API 减少运行时错误
- **跨平台一致性**：纯 Rust 实现，避免平台差异

## 迁移范围分析

### ✅ 第一阶段：核心模块迁移（立即开始）

#### 1. 仓库操作模块 (`src/lib/git/repo.rs`)
- [ ] **优先级：高** - 基础仓库检测和操作
- [ ] 迁移 `GitRepo::is_git_repo()` - 使用 `gix::Repository::discover()`
- [ ] 迁移 `GitRepo::get_remote_url()` - 使用 `repo.find_remote("origin")?.url()`
- [ ] 迁移 `GitRepo::fetch()` - 使用 `remote.fetch()`
- [ ] 迁移 `GitRepo::prune_remote()` - 使用 gix 远程操作
- [ ] 迁移 `GitRepo::extract_repo_name()` - 保持逻辑，改用 gix 获取 URL
- [ ] 更新相关测试用例

#### 2. 提交管理模块 (`src/lib/git/commit.rs`)
- [ ] **优先级：高** - 核心提交功能
- [ ] 迁移 `GitCommit::status()` - 使用 `repo.statuses()`
- [ ] 迁移 `GitCommit::has_changes()` - 使用 gix 状态检查
- [ ] 迁移 `GitCommit::has_staged()` - 使用 gix index 状态
- [ ] 迁移 `GitCommit::add_all()` - 使用 `index.add_all()`
- [ ] 迁移 `GitCommit::commit()` - 使用 `repo.commit()`
- [ ] 迁移 `GitCommit::get_last_commit_info()` - 使用 `repo.head_commit()`
- [ ] 迁移 `GitCommit::amend()` - 使用 gix commit 修改
- [ ] 保留 `GitCommit::reset_hard()` 的复杂 rebase 部分
- [ ] 更新相关测试用例

#### 3. 分支管理模块 (`src/lib/git/branch.rs`)
- [ ] **优先级：高** - 分支操作核心功能
- [ ] 迁移 `GitBranch::get_current_branch()` - 使用 `repo.head()?.shorthand()`
- [ ] 迁移 `GitBranch::branch_exists()` - 使用 gix 分支检查
- [ ] 迁移 `GitBranch::create_and_switch()` - 使用 `repo.branch()` + checkout
- [ ] 迁移 `GitBranch::switch_to_branch()` - 使用 gix checkout
- [ ] 迁移 `GitBranch::get_default_branch()` - 使用 gix 远程信息
- [ ] 迁移 `GitBranch::pull_latest()` - 使用 `remote.fetch()` + merge
- [ ] 迁移 `GitBranch::force_push()` - 使用 gix push 操作
- [ ] 迁移 `GitBranch::delete_remote_branch()` - 使用 gix 远程删除
- [ ] 迁移 `GitBranch::merge()` - 使用 gix merge 操作
- [ ] 更新相关测试用例

#### 4. 暂存管理模块 (`src/lib/git/stash.rs`)
- [ ] **优先级：中** - Stash 操作
- [ ] 迁移 `GitStash::push()` - 使用 gix stash 功能
- [ ] 迁移 `GitStash::pop()` - 使用 gix stash apply + drop
- [ ] 迁移 `GitStash::has_conflicts()` - 使用 gix 冲突检测
- [ ] 迁移 `GitStash::list()` - 使用 gix stash 列表
- [ ] 更新相关测试用例

#### 5. Tag 管理模块 (`src/lib/git/tag.rs`)
- [ ] **优先级：中** - Tag 操作
- [ ] 迁移 `GitTag::list_tags()` - 使用 `repo.tag_names()`
- [ ] 迁移 `GitTag::delete_local_tag()` - 使用 gix tag 删除
- [ ] 迁移 `GitTag::delete_remote_tag()` - 使用 gix 远程 tag 删除
- [ ] 迁移 `GitTag::create_tag()` - 使用 gix tag 创建
- [ ] 更新相关测试用例

#### 6. 配置管理模块 (`src/lib/git/config.rs`)
- [ ] **优先级：低** - 配置操作
- [ ] 迁移 `GitConfig::set_global_email()` - 使用 gix config
- [ ] 迁移 `GitConfig::set_global_name()` - 使用 gix config
- [ ] 迁移 `GitConfig::get_global_config()` - 使用 gix config 读取
- [ ] 更新相关测试用例

### ⚠️ 第二阶段：高级功能迁移（需要额外开发）

#### 7. Cherry-pick 操作模块 (`src/lib/git/cherry_pick.rs`)
- [ ] **优先级：中** - 需要手动实现 cherry-pick 逻辑
- [ ] 研究 gix 的三向合并 API
- [ ] 实现 `GitCherryPick::apply()` - 手动实现 cherry-pick 逻辑
- [ ] 实现 `GitCherryPick::apply_no_commit()` - 应用但不提交
- [ ] 实现 `GitCherryPick::continue_cherry_pick()` - 继续操作
- [ ] 实现 `GitCherryPick::abort()` - 中止操作
- [ ] 实现 `GitCherryPick::is_in_progress()` - 状态检查
- [ ] 更新相关测试用例

### ❌ 第三阶段：保留混合模式

#### 8. 复杂 Rebase 操作
- [ ] **决策：保留命令行** - 交互式 rebase 复杂度高
- [ ] 保留 `src/lib/commit/squash.rs` 中的交互式 rebase
- [ ] 保留 `src/lib/commit/reword.rs` 中的交互式 rebase
- [ ] 保留 `src/commands/branch/rename.rs` 中的复杂 rebase

#### 9. Pre-commit 工具集成
- [ ] **决策：保留外部调用** - pre-commit 是独立 Python 工具
- [ ] 保留 `src/lib/git/pre_commit.rs` 中的 `pre-commit` 工具调用
- [ ] 可选：迁移 Git 原生 hooks 执行到 gix

## 技术实施计划

### 阶段 1：环境准备
- [ ] 更新 `Cargo.toml` 依赖
  ```toml
  [dependencies]
  gix = { version = "0.66", features = ["max-performance"] }

  [features]
  default = ["gix-backend"]
  gix-backend = ["gix"]
  git2-backend = ["git2"]  # 保留作为后备
  ```

### 阶段 2：创建 gix 实现
- [ ] 创建 `src/lib/git/gix/` 目录结构
- [ ] 实现 gix 版本的各个模块
- [ ] 保持与现有 API 的兼容性

### 阶段 3：渐进式迁移
- [ ] 使用 feature flag 控制迁移进度
- [ ] 确保所有现有测试通过
- [ ] 添加 gix 特定的性能测试

### 阶段 4：性能基准测试
- [ ] 创建性能基准测试套件
- [ ] 对比迁移前后的性能数据
- [ ] 优化性能瓶颈

## 风险评估与缓解

### 高风险项目
1. **Cherry-pick 实现复杂度**
   - 风险：gix 没有直接 API，需要手动实现
   - 缓解：先实现基础功能，复杂场景保留命令行

2. **测试用例兼容性**
   - 风险：gix 行为可能与 Git CLI 有细微差异
   - 缓解：详细的集成测试，逐步验证

3. **网络认证兼容性**
   - 风险：企业环境的特殊认证需求
   - 缓解：保留 git2 作为后备选项

### 中风险项目
1. **API 行为差异**
   - 风险：gix 与 Git CLI 的输出格式差异
   - 缓解：适配层统一输出格式

2. **性能回归**
   - 风险：某些操作可能比预期慢
   - 缓解：性能基准测试和优化

## 测试策略

### 单元测试
- [ ] 为每个迁移的模块创建 gix 特定测试
- [ ] 确保 API 兼容性测试通过
- [ ] 添加性能回归测试

### 集成测试
- [ ] 更新现有集成测试以支持 gix
- [ ] 创建端到端工作流测试
- [ ] 验证与外部工具的集成

### 性能测试
- [ ] 创建操作性能基准
- [ ] 对比命令行 vs gix 性能
- [ ] 内存使用情况测试

## 文档更新

### 开发文档
- [ ] 更新 `docs/guidelines/DEVELOPMENT_GUIDELINES.md`
- [ ] 创建 gix 使用指南
- [ ] 更新 API 文档

### 用户文档
- [ ] 更新安装说明（减少 Git 依赖）
- [ ] 更新故障排除指南
- [ ] 性能改进说明

## 发布计划

### Beta 版本 (v1.7.0-beta)
- [ ] 完成第一阶段核心模块迁移
- [ ] 通过所有现有测试
- [ ] 性能基准验证

### 正式版本 (v1.7.0)
- [ ] 完成第二阶段高级功能
- [ ] 完整的测试覆盖
- [ ] 文档更新完成

### 后续版本 (v1.8.0+)
- [ ] 优化性能
- [ ] 完善错误处理
- [ ] 社区反馈集成

## 成功指标

### 性能指标
- [ ] Git 操作性能提升 10-100 倍
- [ ] 内存使用减少 50%+
- [ ] 启动时间减少 80%+

### 质量指标
- [ ] 所有现有测试通过率 100%
- [ ] 新增测试覆盖率 > 90%
- [ ] 零关键 bug

### 用户体验指标
- [ ] 安装步骤减少（无需系统 Git）
- [ ] 跨平台一致性提升
- [ ] 错误信息更清晰

## 备注

- 本迁移计划采用渐进式策略，确保项目稳定性
- 保留 git2 作为后备选项，以应对特殊场景
- 重点关注性能提升和用户体验改善
- 定期评估进度并调整计划

---

**创建日期：** 2024-12-17
**最后更新：** 2024-12-17
**负责人：** 开发团队
**预计完成时间：** 2-3 个开发周期

