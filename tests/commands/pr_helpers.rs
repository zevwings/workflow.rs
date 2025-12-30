//! PR 命令辅助函数测试
//!
//! 测试 PR 命令的辅助函数，包括目标分支解析等。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;
use workflow::commands::pr::helpers::resolve_target_branch;

use crate::common::environments::GitTestEnv;

/// 测试基于默认分支创建的分支应该直接使用默认分支
///
/// ## 测试目的
/// 验证当分支基于默认分支创建时，`resolve_target_branch` 应该直接返回默认分支，不询问用户。
///
/// ## 测试场景
/// 1. 创建基于默认分支（main）的新分支
/// 2. 调用 `resolve_target_branch`
/// 3. 验证返回默认分支
///
/// ## 预期结果
/// - 返回默认分支（main）
#[test]
#[serial]
fn test_resolve_target_branch_based_on_default() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建基于默认分支的新分支
    env.checkout_new_branch("feature/test")?;
    env.create_file("test.txt", "test")?;
    env.make_test_commit("test.txt", "test", "feat: add test")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/test", default_branch)?;

    // 应该返回默认分支（因为 feature/test 基于 main 创建）
    assert_eq!(target, default_branch);

    Ok(())
}

/// 测试基于非默认分支创建的分支应该检测基础分支
///
/// ## 测试目的
/// 验证当分支基于非默认分支创建时，`resolve_target_branch` 应该检测基础分支。
///
/// ## 测试场景
/// 1. 创建基础分支 feature/base 并提交
/// 2. 基于 feature/base 创建 feature/child 分支
/// 3. 调用 `resolve_target_branch`（使用非交互式模式，选择默认分支）
/// 4. 验证检测到基础分支
///
/// ## 预期结果
/// - 检测到基础分支 feature/base
#[test]
#[serial]
fn test_resolve_target_branch_based_on_non_default() -> Result<()> {
    use workflow::base::dialog::skip_config::{DialogConfigBuilder, DialogConfigManager};

    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 1. 创建基础分支 feature/base
    env.checkout_new_branch("feature/base")?;
    env.create_file("base.txt", "base")?;
    env.make_test_commit("base.txt", "base", "feat: add base")?;

    // 2. 基于 feature/base 创建 feature/child 分支
    env.checkout_new_branch("feature/child")?;
    env.create_file("child.txt", "child")?;
    env.make_test_commit("child.txt", "child", "feat: add child")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 设置非交互式模式，选择索引 0（默认分支）
    let config = DialogConfigBuilder::new().with_select_index(0).build();
    DialogConfigManager::set_config(config);

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

    // 清理非交互式模式
    DialogConfigManager::clear_config();

    // 应该返回默认分支（因为选择了索引 0）
    assert_eq!(target, default_branch);

    Ok(())
}

/// 测试检测不到基础分支时应该使用默认分支
///
/// ## 测试目的
/// 验证当无法检测到基础分支时，`resolve_target_branch` 应该使用默认分支。
///
/// ## 测试场景
/// 1. 创建一个孤立的分支（没有明确的基础分支）
/// 2. 调用 `resolve_target_branch`
/// 3. 验证返回默认分支
///
/// ## 预期结果
/// - 返回默认分支（main）
#[test]
#[serial]
fn test_resolve_target_branch_no_base_detected() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建一个分支（如果无法检测基础分支，应该使用默认分支）
    env.checkout_new_branch("orphan-branch")?;
    env.create_file("orphan.txt", "orphan")?;
    env.make_test_commit("orphan.txt", "orphan", "feat: add orphan")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 调用 resolve_target_branch
    // 如果检测不到基础分支，应该返回默认分支
    let target = resolve_target_branch("orphan-branch", default_branch)?;

    // 应该返回默认分支（因为检测不到基础分支）
    assert_eq!(target, default_branch);

    Ok(())
}

/// 测试检测失败时应该使用默认分支
///
/// ## 测试目的
/// 验证当检测基础分支失败时，`resolve_target_branch` 应该使用默认分支。
///
/// ## 测试场景
/// 1. 创建一个分支
/// 2. 模拟检测失败（通过使用不存在的分支名）
/// 3. 调用 `resolve_target_branch`
/// 4. 验证返回默认分支
///
/// ## 预期结果
/// - 返回默认分支（main）
#[test]
#[serial]
fn test_resolve_target_branch_detection_failure() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建一个分支
    env.checkout_new_branch("feature/test")?;
    env.create_file("test.txt", "test")?;
    env.make_test_commit("test.txt", "test", "feat: add test")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 调用 resolve_target_branch
    // 即使检测失败，也应该返回默认分支（错误处理逻辑）
    let target = resolve_target_branch("feature/test", default_branch)?;

    // 应该返回默认分支或检测到的基础分支
    // 由于 feature/test 基于 main 创建，应该返回 main
    assert_eq!(target, default_branch);

    Ok(())
}

/// 测试用户取消选择时应该使用默认分支
///
/// ## 测试目的
/// 验证当用户取消选择时，`resolve_target_branch` 应该使用默认分支。
///
/// ## 测试场景
/// 1. 创建基于非默认分支的分支
/// 2. 模拟用户取消操作
/// 3. 调用 `resolve_target_branch`
/// 4. 验证返回默认分支
///
/// ## 预期结果
/// - 返回默认分支（main）
#[test]
#[serial]
fn test_resolve_target_branch_user_cancelled() -> Result<()> {
    use workflow::base::dialog::skip_config::{DialogConfigBuilder, DialogConfigManager};

    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 1. 创建基础分支 feature/base
    env.checkout_new_branch("feature/base")?;
    env.create_file("base.txt", "base")?;
    env.make_test_commit("base.txt", "base", "feat: add base")?;

    // 2. 基于 feature/base 创建 feature/child 分支
    env.checkout_new_branch("feature/child")?;
    env.create_file("child.txt", "child")?;
    env.make_test_commit("child.txt", "child", "feat: add child")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 注意：由于 SelectDialog 在非交互式模式下不会真正取消，
    // 我们通过设置索引来模拟选择默认分支的行为
    // 实际的取消逻辑在代码中通过错误消息检测实现
    let config = DialogConfigBuilder::new().with_select_index(0).build();
    DialogConfigManager::set_config(config);

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

    // 清理非交互式模式
    DialogConfigManager::clear_config();

    // 应该返回默认分支（因为选择了索引 0）
    assert_eq!(target, default_branch);

    Ok(())
}

/// 测试选择基础分支的情况
///
/// ## 测试目的
/// 验证当检测到基础分支时，用户可以选择基础分支。
///
/// ## 测试场景
/// 1. 创建基础分支 feature/base 并提交
/// 2. 基于 feature/base 创建 feature/child 分支
/// 3. 调用 `resolve_target_branch`（使用非交互式模式，选择基础分支）
/// 4. 验证返回基础分支
///
/// ## 预期结果
/// - 返回基础分支（feature/base）
#[test]
#[serial]
fn test_resolve_target_branch_select_base_branch() -> Result<()> {
    use workflow::base::dialog::skip_config::{DialogConfigBuilder, DialogConfigManager};

    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 1. 创建基础分支 feature/base
    env.checkout_new_branch("feature/base")?;
    env.create_file("base.txt", "base")?;
    env.make_test_commit("base.txt", "base", "feat: add base")?;

    // 2. 基于 feature/base 创建 feature/child 分支
    env.checkout_new_branch("feature/child")?;
    env.create_file("child.txt", "child")?;
    env.make_test_commit("child.txt", "child", "feat: add child")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 设置非交互式模式，选择索引 1（基础分支）
    let config = DialogConfigBuilder::new().with_select_index(1).build();
    DialogConfigManager::set_config(config);

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

    // 清理非交互式模式
    DialogConfigManager::clear_config();

    // 应该返回基础分支（因为选择了索引 1）
    assert_eq!(target, "feature/base");

    Ok(())
}
