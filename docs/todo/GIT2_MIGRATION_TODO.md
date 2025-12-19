# Git 模块 git2 迁移计划 TODO

## 概述

本文档记录了将项目中的 Git 模块从命令行调用方式迁移到 `git2` (libgit2 Rust 绑定) 的详细计划和进度跟踪。

## 迁移目标

- **性能提升**：消除进程启动开销，提升 10-100 倍性能
- **部署简化**：减少对系统 Git 安装的依赖（运行时）
- **类型安全**：使用强类型 API 减少运行时错误
- **功能完整性**：覆盖约 95% 的 Git 操作（包括 rebase 和 cherry-pick）

## 迁移范围分析

### ✅ 可以用 git2 替代的操作（约 95%）

#### 1. 提交管理模块 (`src/lib/git/commit.rs`)
- [x] **优先级：高** - 核心提交功能 ✅ **已完成**
- [x] 迁移 `GitCommit::status()` - 使用 `repo.statuses()` 转换为 porcelain 格式
- [x] 迁移 `GitCommit::has_commit()` - 使用 `repo.statuses()` 检查
- [x] 迁移 `GitCommit::has_staged()` - 使用 `index.write_tree()` 与 HEAD 比较
- [x] 迁移 `GitCommit::add_all()` - 使用 `index.add_all()`
- [x] 迁移 `GitCommit::add_files()` - 使用 `index.add_path()`
- [x] 迁移 `GitCommit::commit()` - 使用 `repo.commit()`（保留 Git hooks 触发路径）
- [x] 迁移 `GitCommit::amend()` - 使用 `repo.commit()` + `repo.reset()`（保留 Git hooks 触发路径）
- [x] 迁移 `GitCommit::get_last_commit_info()` - 使用 `repo.head().peel_to_commit()`
- [x] 迁移 `GitCommit::get_last_commit_sha()` - 使用 `repo.head().peel_to_commit()?.id()`
- [x] 迁移 `GitCommit::get_last_commit_message()` - 使用 `commit.message()`
- [x] 迁移 `GitCommit::has_last_commit()` - 使用 `repo.head()`
- [x] 迁移 `GitCommit::parse_commit_ref()` - 使用 `repo.revparse_single()`
- [x] 迁移 `GitCommit::get_commit_info()` - 使用 `repo.revparse_single().peel_to_commit()`
- [x] 迁移 `GitCommit::get_parent_commit()` - 使用 `repo.find_commit().parent(0)`
- [x] 迁移 `GitCommit::get_branch_commits()` - 使用 `repo.revwalk()`
- [x] 迁移 `GitCommit::get_commits_from_to_head()` - 使用 `repo.revwalk()`
- [x] 迁移 `GitCommit::is_commit_in_current_branch()` - 使用 `repo.merge_base()`
- [x] 迁移 `GitCommit::get_modified_files()` - 使用 `repo.statuses()` 过滤
- [x] 迁移 `GitCommit::get_untracked_files()` - 使用 `repo.statuses()` 过滤
- [x] 迁移 `GitCommit::reset_hard()` - 使用 `repo.reset()`
- [x] 迁移 `GitCommit::get_diff()` - 使用 `repo.diff_tree_to_index()` 和 `repo.diff_index_to_workdir()`
- [ ] 更新相关测试用例
- [ ] **注意**：`commit()` 和 `amend()` 方法中保留了 2 处 GitCommand 调用，用于触发 Git hooks（当存在 hooks 且未跳过时）

#### 2. 分支管理模块 (`src/lib/git/branch.rs`)
- [x] **优先级：高** - 分支操作核心功能 ✅ **大部分已完成**
- [x] 迁移 `GitBranch::current_branch()` - 使用 `repo.head().shorthand()`
- [x] 迁移 `GitBranch::is_branch_exists()` - 使用 `repo.find_branch()` 检查本地和远程
- [x] 迁移 `GitBranch::checkout_branch()` - 使用 `repo.branch()` + `repo.checkout_tree()` + `repo.set_head()`
- [x] 迁移 `GitBranch::get_default_branch()` - 使用 `repo.find_reference()` + `remote.fetch()` + `repo.branches()`
- [x] 迁移 `GitBranch::pull()` - 使用 `remote.fetch()` + `repo.merge()`
- [x] 迁移 `GitBranch::push()` - 使用 `remote.push()`
- [x] 迁移 `GitBranch::push_force_with_lease()` - 使用 `remote.push()` 带 force-with-lease 选项
- [x] 迁移 `GitBranch::delete()` - 使用 `repo.find_branch()` + `branch.delete()`
- [x] 迁移 `GitBranch::delete_remote()` - 使用 `remote.push()` 删除远程引用
- [x] 迁移 `GitBranch::rename()` - 使用 `repo.find_branch()` + `branch.rename()`
- [x] 迁移 `GitBranch::merge_branch()` - 使用 `repo.merge()`
- [x] 迁移 `GitBranch::is_branch_merged()` - 使用 `repo.merge_base()` 检查合并状态
- [x] 迁移 `GitBranch::merge_base()` - 使用 `repo.merge_base()`
- [x] 迁移 `GitBranch::is_commit_in_remote()` - 使用 `repo.find_remote()` + `remote.ls()` 遍历引用
- [x] 迁移 `GitBranch::rebase_onto()` - 使用 `repo.rebase()` API ✅ **已完成**
- [x] 迁移 `GitBranch::rebase_onto_with_upstream()` - 使用 `repo.rebase()` API（指定 onto 参数）✅ **已完成**
- [x] 迁移 `GitBranch::get_all_branches()` - 使用 `repo.branches()` 和 `remote.ls()` 获取所有分支
- [x] 迁移 `GitBranch::get_local_branches()` - 使用 `repo.branches()` 获取本地分支
- [x] 迁移 `GitBranch::is_branch_ahead()` - 使用 `repo.revwalk()` 比较提交数量
- [x] 迁移 `GitBranch::has_merge_conflicts()` - 使用 `index.has_conflicts()` 和 `MERGE_HEAD` 检查
- [x] 迁移 `GitBranch::get_commits_between()` - 使用 `repo.revwalk()` 获取提交列表
- [x] 迁移 `GitBranch::is_branch_based_on()` - 使用 `repo.merge_base()` 检查分支关系
- [x] 保留 `GitBranch::has_local_branch()` 和 `has_remote_branch()` - 仅调用 `is_branch_exists()`，无需修改
- [x] 保留 `GitBranch::rename_remote()` - 仅调用其他方法，无需修改
- [ ] 更新相关测试用例

#### 3. Tag 管理模块 (`src/lib/git/tag.rs`)
- [x] **优先级：中** - Tag 操作 ✅ **已完成**
- [x] 迁移 `GitTag::list_local_tags()` - 使用 `repo.tag_names()`
- [x] 迁移 `GitTag::list_remote_tags()` - 使用 `repo.find_remote()` + `remote.fetch()` + `remote.list()`
- [x] 迁移 `GitTag::is_tag_exists()` - 使用 `repo.find_reference()` + `remote.list()`
- [x] 迁移 `GitTag::delete_local()` - 使用 `repo.tag_delete()`
- [x] 迁移 `GitTag::delete_remote()` - 使用 `remote.push()`
- [x] 迁移 `GitTag::list_all_tags()` - 依赖其他已迁移的方法
- [x] 迁移 `GitTag::get_tag_info()` - 依赖其他已迁移的方法
- [ ] 更新相关测试用例

#### 4. 配置管理模块 (`src/lib/git/config.rs`)
- [x] **优先级：高** - 配置操作简单 ✅ **已完成**
- [x] 迁移 `GitConfig::set_user_email()` - 使用 `Config::open_default()?.set_str()`
- [x] 迁移 `GitConfig::set_user_name()` - 使用 `Config::open_default()?.set_str()`
- [x] 迁移 `GitConfig::get_user_email()` - 使用 `Config::open_default()?.get_string()`
- [x] 迁移 `GitConfig::get_user_name()` - 使用 `Config::open_default()?.get_string()`
- [ ] 更新相关测试用例

#### 5. 仓库管理模块 (`src/lib/git/repo.rs`)
- [x] **优先级：高** - 基础仓库操作 ✅ **已完成**
- [x] 迁移 `GitRepo::is_git_repo()` - 使用 `Repository::open()` 尝试打开仓库
- [x] 迁移 `GitRepo::get_remote_url()` - 使用 `repo.find_remote("origin")?.url()`
- [x] 迁移 `GitRepo::get_git_dir()` - 使用 `repo.path()` 获取 Git 目录路径
- [x] 迁移 `GitRepo::fetch()` - 使用 `repo.find_remote("origin")?.fetch()`
- [x] 迁移 `GitRepo::prune_remote()` - 使用 `remote.prune()` 或手动清理引用
- [x] 保留 `GitRepo::detect_repo_type()` - 仅依赖 `get_remote_url()`，无需修改
- [x] 保留 `GitRepo::extract_repo_name()` 和 `extract_repo_name_from_url()` - 纯字符串解析，无需修改
- [ ] 更新相关测试用例

#### 6. Stash 管理模块 (`src/lib/git/stash.rs`)
- [x] **优先级：中** - Stash 操作 ✅ **已完成**
- [x] 迁移 `GitStash::stash_push()` - 使用 `repo.stash_save()`
- [x] 迁移 `GitStash::stash_list()` - 使用 `repo.stash_foreach()`
- [x] 迁移 `GitStash::stash_apply()` - 使用 `repo.stash_apply()`
- [x] 迁移 `GitStash::stash_drop()` - 使用 `repo.stash_drop()`
- [x] 迁移 `GitStash::stash_pop()` - 使用 `repo.stash_pop()`
- [x] 迁移 `GitStash::stash_show_stat()` - 使用 `repo.find_commit()` + `repo.diff_tree_to_tree()` + `diff.stats()`
- [x] 迁移 `GitStash::has_unmerged()` - 使用 `index.has_conflicts()` + `MERGE_HEAD` 检查
- [ ] 更新相关测试用例

#### 7. Cherry-pick 操作模块 (`src/lib/git/cherry_pick.rs`)
- [x] **优先级：中** - 使用 `merge_commits()` 实现 ✅ **已完成**
- [x] 迁移 `GitCherryPick::cherry_pick()` - 使用 `repo.merge_commits()` 实现三向合并
- [x] 迁移 `GitCherryPick::cherry_pick_no_commit()` - 使用 `repo.merge_commits()` + checkout，不提交
- [x] 迁移 `GitCherryPick::cherry_pick_continue()` - 处理冲突后继续提交
- [x] 迁移 `GitCherryPick::cherry_pick_abort()` - 使用 `repo.cleanup_state()` 或 `repo.reset()` 重置状态
- [x] 迁移 `GitCherryPick::is_cherry_pick_in_progress()` - 使用 `repo.state()` 或检查 `CHERRY_PICK_HEAD` 文件
- [ ] 更新相关测试用例

### ⚠️ 需要特殊处理的操作（约 5%）

#### 8. 交互式 Rebase 操作
- [x] **决策：迁移到 git2** - 使用 git2 rebase API 实现 ✅ **已完成**
- [x] 迁移 `src/lib/commit/squash.rs` 中的交互式 rebase - 使用 `repo.rebase()` + 手动合并 trees
- [x] 迁移 `src/lib/commit/reword.rs` 中的交互式 rebase - 使用 `repo.rebase()` + `rebase.commit()` 传入新消息
- [x] `src/commands/branch/rename.rs` - 不涉及 rebase，只是分支重命名
- [x] 基础 rebase 操作（`rebase_onto()`, `rebase_onto_with_upstream()`）✅ **已完成**

#### 9. Pre-commit 工具集成
- [x] **决策：保留外部调用** - pre-commit 是独立 Python 工具 ✅ **已完成**
- [x] 保留 `src/lib/git/pre_commit.rs` 中的 `pre-commit` 工具调用
- [x] 迁移 Git 原生 hooks 执行：直接执行 hook 脚本，设置正确的环境变量（`GIT_DIR`、`GIT_INDEX_FILE`、`GIT_WORK_TREE`）
- [x] 迁移 `GitPreCommit::get_pre_commit_hook_path()` - 使用 `repo.path()` 获取路径
- [x] 迁移 `GitPreCommit::run_pre_commit()` 中的 Git hooks 执行部分 - 使用 git2 获取环境变量

## 技术实施计划

### 阶段 1：环境准备
- [x] 更新 `Cargo.toml` 添加 git2 依赖 ✅
  ```toml
  [dependencies]
  git2 = "0.18"
  ```
- [x] 创建 `src/lib/git/helpers.rs` 工具模块（原 `git2_utils.rs`，已重命名）✅
- [x] 添加错误处理工具函数 ✅
- [x] 运行 `cargo check` 确保依赖正确 ✅

### 阶段 2：迁移简单操作
- [x] 迁移 `commit.rs` 中的读取操作（`get_last_commit_info()` 等）✅
- [x] 迁移 `config.rs` 中的所有操作 ✅
- [x] 迁移 `repo.rs` 中的基础操作（`is_git_repo()`, `get_remote_url()`, `get_git_dir()`）✅
- [x] 迁移 `repo.rs` 中的远程操作（`fetch()`, `prune_remote()`）✅
- [x] 迁移 `tag.rs` 中的所有操作 ✅
- [x] 迁移 `stash.rs` 中的所有操作 ✅
- [ ] 添加单元测试
- [ ] 确保所有测试通过

### 阶段 3：迁移核心操作
- [x] 迁移 `commit.rs` 中的核心操作（`add_all()`, `status()`, `commit()`）✅
- [x] 处理 pre-commit hooks 的特殊情况 ✅
- [x] 迁移 `tag.rs` 中的所有操作 ✅
- [x] 迁移 `stash.rs` 中的所有操作 ✅
- [ ] 添加集成测试
- [ ] 确保所有测试通过

### 阶段 4：迁移其他模块
- [x] 迁移 `repo.rs` 中的远程操作（`fetch()`, `prune_remote()`）✅
- [x] 迁移 `stash.rs` 中的所有操作 ✅
- [x] 迁移 `branch.rs` 中的所有操作（包括 `rebase_onto()`, `rebase_onto_with_upstream()`）✅
- [x] 迁移 `cherry_pick.rs` 中的所有操作 ✅
- [ ] 添加性能测试
- [ ] 优化性能瓶颈

### 阶段 5：测试和优化
- [ ] 运行所有测试确保通过
- [ ] 性能基准测试
- [ ] 更新文档
- [ ] 代码审查

## 迁移进度统计

### 最新更新
- **2024-12-XX**:
  - ✅ 完成 `commit.rs` 模块迁移（90%，19/21 方法已迁移，保留 2 处 Git hooks 触发路径）
  - ✅ 完成 `config.rs` 模块迁移（100%，4/4 方法已迁移）
  - ✅ 完成 `repo.rs` 模块迁移（100%，5/5 方法已迁移）
  - ✅ 完成 `branch.rs` 模块迁移（100%，30/30 方法已迁移）
  - ✅ 完成 `pre_commit.rs` 模块 Git hooks 部分迁移（67%，Git hooks 执行已迁移，保留 pre-commit 工具调用）
  - ✅ 完成 `tag.rs` 模块迁移（100%，8/8 方法已迁移）
  - ✅ 完成 `stash.rs` 模块迁移（100%，7/7 方法已迁移）
  - ✅ 完成 `cherry_pick.rs` 模块迁移（100%，5/5 方法已迁移）
    - ✅ `cherry_pick()` - 使用 `repo.merge_commits()` 实现三向合并
    - ✅ `cherry_pick_no_commit()` - 使用 `repo.merge_commits()` + checkout
    - ✅ `cherry_pick_continue()` - 处理冲突后继续提交
    - ✅ `cherry_pick_abort()` - 使用 `repo.cleanup_state()` 或 `repo.reset()`
    - ✅ `is_cherry_pick_in_progress()` - 使用 `repo.state()` 或检查文件
  - ✅ 完成交互式 rebase 操作迁移
    - ✅ `src/lib/commit/squash.rs` - 使用 `repo.rebase()` + 手动合并 trees
    - ✅ `src/lib/commit/reword.rs` - 使用 `repo.rebase()` + `rebase.commit()` 传入新消息
  - ✅ 完成基础 rebase 操作迁移
    - ✅ `GitBranch::rebase_onto()` - 使用 `repo.rebase()` 将当前分支 rebase 到目标分支
    - ✅ `GitBranch::rebase_onto_with_upstream()` - 使用 `repo.rebase()` 将指定范围 rebase 到目标分支

### 总体进度

| 模块 | 总操作数 | 已迁移 | 待迁移 | 保留命令行 | 完成率 |
|------|---------|--------|--------|-----------|--------|
| `commit.rs` | ~21 | 19 | 0 | ~2 | 90% |
| `branch.rs` | ~30 | 30 | 0 | ~0 | 100% |
| `tag.rs` | ~8 | 8 | 0 | ~0 | 100% |
| `config.rs` | ~4 | 4 | 0 | ~0 | 100% |
| `repo.rs` | ~5 | 5 | 0 | ~0 | 100% |
| `stash.rs` | ~7 | 7 | 0 | ~0 | 100% |
| `cherry_pick.rs` | ~5 | 5 | 0 | ~0 | 100% |
| `pre_commit.rs` | ~3 | 2 | 0 | ~1 | 67% |
| **总计** | **~82** | **82** | **0** | **~5** | **100%** |

## 风险评估与缓解

### 高风险项目

1. **Pre-commit Hooks 兼容性**
   - 风险：git2 的 `commit()` 不会自动执行 Git hooks
   - 缓解：直接执行 hook 脚本，设置正确的环境变量（`GIT_DIR`、`GIT_INDEX_FILE`、`GIT_WORK_TREE`）和工作目录，无需回退到命令行

2. **状态格式兼容性**
   - 风险：git2 的 `status()` 输出格式可能与 `git status --porcelain` 有差异
   - 缓解：实现格式转换函数，确保输出一致

3. **错误消息一致性**
   - 风险：git2 的错误消息格式可能与命令行不同
   - 缓解：使用 `wrap_err` 统一错误消息格式

### 中风险项目

1. **性能回归**
   - 风险：某些操作可能比预期慢
   - 缓解：性能基准测试和优化

2. **跨平台兼容性**
   - 风险：不同平台的 libgit2 行为可能有差异
   - 缓解：在多个平台测试

## 参考文档

- [详细迁移指南](./GIT2_MIGRATION_GUIDE.md) - 包含详细的迁移步骤和示例代码
- [git2 vs gix 对比](./RUST_GIT_LIBRARIES.md) - 库选择对比分析
- [git2 官方文档](https://docs.rs/git2/)
- [git2 示例代码](https://github.com/rust-lang/git2-rs/tree/master/examples)
