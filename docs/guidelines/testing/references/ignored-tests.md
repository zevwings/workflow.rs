# 被忽略测试规范

> 本文档定义被忽略测试的文档规范。

---

对于使用 `#[ignore]` 标记的测试，必须添加完整的文档注释。

## 统一文档格式

所有被忽略的测试都应该包含以下5个部分的文档注释：

```rust
/// 测试标题（简短描述测试内容）
///
/// ## 测试目的
/// 验证/测试...（说明测试验证什么功能）
///
/// ## 为什么被忽略
/// - **主要原因**: ...
/// - **次要原因**: ...
///
/// ## 如何手动运行
/// \`\`\`bash
/// cargo test test_function_name -- --ignored
/// \`\`\`
///
/// ## 测试场景
/// 1. ...
///
/// ## 预期行为
/// - ...
#[test]
#[ignore] // 简短原因
fn test_function_name() {}
```

## 常见忽略原因

1. **用户交互测试** - 需要用户交互，CI不支持
2. **网络请求测试** - 需要网络连接和API密钥
3. **时间相关测试** - 涉及真实时间延迟
4. **修改系统配置** - 会修改系统文件，有安全风险
5. **外部服务测试** - 需要连接实际服务

更多测试类型模板和使用示例，请参考具体测试代码。

---

## 改进被忽略的测试

根据测试代码审查，**约40-50%的被忽略测试可以移除 `#[ignore]` 标记**。

### 可以移除 `#[ignore]` 的测试分类

#### 1. 网络请求测试 - 应使用Mock（约15-20个）

**现状**: 这些测试需要真实的GitHub/Jira/LLM API，但项目中已有完善的MockServer基础设施。

**改进方案**: 使用 `MockServer` 和 `GitTestEnv` 替代真实网络请求。

**步骤**:
1. 识别需要真实API的测试
2. 使用 `MockServer` 创建Mock端点
3. 使用 `TestDataFactory` 生成测试数据
4. 移除 `#[ignore]` 标记

**示例**:
```rust
// 之前
#[test]
#[ignore] // 需要真实GitHub API
fn test_create_pr() {
    // 调用真实API...
}

// 之后
#[test]
fn test_create_pr() -> color_eyre::Result<()> {
    let mut mock_server = MockServer::new();
    mock_server.setup_github_base_url();
    mock_server.setup_github_create_pr_success("owner", "repo", 123);
    // 测试代码...
    Ok(())
}
```

**具体测试**:
- `tests/commands/branch_sync.rs` - 4个测试（已有Mock版本，可以删除ignore版本）
- `tests/integration/jira.rs` - 4个测试（应改为使用MockServer）
- `tests/cli/basic_cli.rs` - 4个测试（应改为使用MockServer + GitTestEnv）

**LLM API测试**（需要特殊处理）:
- `tests/base/llm/client.rs` - 3个测试
- **建议**: 保留ignore，但可以添加环境变量控制，如 `ENABLE_LLM_TESTS=1` 时运行

#### 2. Git仓库测试 - 应使用GitTestEnv（约5-8个）

**改进方案**: 使用 `GitTestEnv` 创建临时Git仓库。

**步骤**:
1. 识别需要真实Git仓库的测试
2. 使用 `GitTestEnv::new()` 创建临时仓库
3. 移除 `#[ignore]` 标记

**示例**:
```rust
// 之前
#[test]
#[ignore] // 需要真实Git仓库
fn test_branch_operations() {
    // 依赖真实Git仓库...
}

// 之后
#[test]
fn test_branch_operations() -> color_eyre::Result<()> {
    let env = GitTestEnv::new()?;
    env.create_branch("feature/test")?;
    // 测试代码...
    Ok(())
}
```

#### 3. 可以添加环境变量控制的测试（约3-5个）

**改进方案**: 添加环境变量检测，仅在特定条件下运行。

```rust
#[test]
#[ignore] // 默认忽略，但可以通过环境变量启用
fn test_something() {
    if std::env::var("ENABLE_NETWORK_TESTS").is_err() {
        eprintln!("Skipping test. Set ENABLE_NETWORK_TESTS=1 to run.");
        return;
    }
    // 测试代码
}
```

### 移除ignore标记的步骤

1. **分析测试**: 确定为什么被忽略
2. **选择改进方案**: Mock、GitTestEnv、环境变量控制等
3. **实施改进**: 修改测试代码
4. **验证测试**: 确保测试通过
5. **移除ignore**: 删除 `#[ignore]` 标记
6. **更新文档**: 更新测试文档注释

### 应保留 `#[ignore]` 的测试

以下类型的测试应保留 `#[ignore]` 标记：

#### 1. 用户交互测试（约15-20个）

**原因**: 这些测试需要真实的用户输入，无法在CI中自动化运行。

**具体测试**:
- `tests/base/dialog/form_builder.rs` - 8个
- `tests/base/dialog/confirm.rs` - 2个
- `tests/base/dialog/multi_select.rs` - 1个
- `tests/base/dialog/select.rs` - 1个
- `tests/branch/types.rs` - 3个
- `tests/jira/logs.rs` - 1个
- `tests/jira/status.rs` - 2个

**建议**: ✅ **保留ignore** - 这些测试确实需要用户交互

#### 2. 性能测试（约8个）

**原因**: 性能测试不应该在每次CI中运行，只在需要时运行。

**具体测试**:
- `tests/integration/performance.rs` - 8个

**建议**: ✅ **保留ignore** - 性能测试应该单独运行

#### 3. 系统配置修改测试（约3个）

**原因**: 这些测试会修改系统配置文件，有安全风险。

**具体测试**:
- `tests/base/shell/config.rs` - 3个（修改用户的shell配置文件）

**建议**: ✅ **保留ignore** - 修改系统文件有安全风险

#### 4. 浏览器测试（约2个）

**原因**: 实际打开浏览器会阻塞测试。

**具体测试**:
- `tests/base/system/browser.rs` - 2个

**建议**: ✅ **保留ignore** - 浏览器测试不适合CI

#### 5. 超时/重试测试（约10个）

**原因**: 超时行为在单元测试中难以可靠重现。

**具体测试**:
- `tests/base/http/retry.rs` - 10个

**建议**: ⚠️ **考虑改进** - 可以使用Mock服务器模拟超时，而不是真实超时

### 改进优先级

#### 🔴 高优先级（立即改进）

1. **删除重复的ignore测试**（`tests/commands/branch_sync.rs`）
   - 已有Mock版本，删除ignore版本
   - **影响**: 4个测试

2. **将Jira集成测试改为使用Mock**（`tests/integration/jira.rs`）
   - 使用MockServer替代真实API
   - **影响**: 4个测试

3. **将CLI测试改为使用Mock**（`tests/cli/basic_cli.rs`）
   - 使用MockServer + GitTestEnv
   - **影响**: 4个测试

#### 🟡 中优先级（计划改进）

1. **添加环境变量控制**（LLM测试）
   - 允许通过环境变量启用测试
   - **影响**: 3个测试

2. **改进超时测试**（`tests/base/http/retry.rs`）
   - 使用Mock服务器模拟超时
   - **影响**: 10个测试

#### 🟢 低优先级（保持现状）

1. **用户交互测试** - 保留ignore
2. **性能测试** - 保留ignore
3. **系统配置测试** - 保留ignore
4. **浏览器测试** - 保留ignore

### 改进检查清单

- [ ] 测试已分析，确定改进方案
- [ ] 已实施改进（使用Mock、GitTestEnv等）
- [ ] 测试可以正常运行
- [ ] 测试通过
- [ ] 已移除 `#[ignore]` 标记
- [ ] 已更新测试文档注释

### 预期改进效果

**改进前**:
- **被忽略测试**: 65个
- **CI测试覆盖率**: 不完整

**改进后（高优先级完成后）**:
- **被忽略测试**: ~55个（减少10个）
- **CI测试覆盖率**: 提升约15%

**改进后（全部完成后）**:
- **被忽略测试**: ~35个（减少30个）
- **CI测试覆盖率**: 提升约45%

---

**最后更新**: 2025-01-27
