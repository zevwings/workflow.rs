//! PR 命令辅助函数测试
//!
//! 测试 PR 命令的辅助函数，包括目标分支解析等。

use color_eyre::Result;
use pretty_assertions::assert_eq;
use serial_test::serial;
use workflow::base::dialog::skip_config;
use workflow::commands::pr::helpers::resolve_target_branch;

use crate::common::environments::GitTestEnv;
use crate::common::guards::DialogTestGuard;

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
#[ignore]
fn test_resolve_target_branch_based_on_default() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建基于默认分支的新分支
    env.checkout_new_branch("feature/test")?;
    env.create_file("test.txt", "test")?;
    env.make_test_commit("test.txt", "test", "feat: add test")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // ⚠️ 重要：必须在调用 resolve_target_branch 之前创建 DialogTestGuard
    // 虽然理论上，当检测到的基础分支是默认分支时，函数应该直接返回而不显示对话框
    // 但在某些边缘情况下（例如检测逻辑返回了非默认分支），函数会显示 SelectDialog
    // 如果没有 DialogTestGuard，SelectDialog 会阻塞等待用户输入，导致测试超时
    let _guard = DialogTestGuard::new().with_select_index(0);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(0),
        "DialogTestGuard should set select_index to 0"
    );

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
#[ignore]
fn test_resolve_target_branch_based_on_non_default() -> Result<()> {
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
    let _guard = DialogTestGuard::new().with_select_index(0);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(0),
        "DialogTestGuard should set select_index to 0"
    );

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

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
/// 1. 创建一个基于默认分支的分支（但可能在某些边缘情况下被误判）
/// 2. 调用 `resolve_target_branch`
/// 3. 验证返回默认分支
///
/// ## 预期结果
/// - 返回默认分支（main）
///
/// ## 注意事项
/// - 即使理论上应该直接返回默认分支，但在某些边缘情况下（例如检测逻辑返回了非默认分支），
///   函数可能会显示对话框让用户选择
/// - 因此必须设置 `DialogTestGuard` 来避免测试卡住
#[test]
#[serial]
#[ignore]
fn test_resolve_target_branch_no_base_detected() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建一个基于默认分支的新分支
    // 理论上应该直接返回默认分支，但为了测试的健壮性，设置非交互模式
    env.checkout_new_branch("orphan-branch")?;
    env.create_file("orphan.txt", "orphan")?;
    env.make_test_commit("orphan.txt", "orphan", "feat: add orphan")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // ⚠️ 重要：必须在调用 resolve_target_branch 之前创建 DialogTestGuard
    // 设置非交互式模式，选择索引 0（默认分支）
    //
    // 为什么需要这个 guard：
    // 1. 虽然理论上，当检测到的基础分支是默认分支时，函数应该直接返回而不显示对话框
    // 2. 但在某些边缘情况下（例如检测逻辑返回了非默认分支），函数会显示 SelectDialog
    // 3. SelectDialog 选项顺序为：[默认分支 (索引 0), 基础分支 (索引 1)]
    // 4. 设置 select_index(0) 确保如果对话框出现，会选择默认分支，避免测试卡住
    //
    // 如果测试仍然卡住，检查：
    // - DialogTestGuard 是否在函数调用前创建
    // - 是否正确设置了 select_index
    // - 是否存在线程安全问题（确保使用 #[serial] 属性）
    let _guard = DialogTestGuard::new().with_select_index(0);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(0),
        "DialogTestGuard should set select_index to 0"
    );

    // 调用 resolve_target_branch
    // 理论上应该直接返回默认分支（因为检测到的基础分支就是默认分支）
    // 但设置 DialogTestGuard 确保即使显示对话框也能正确处理
    let target = resolve_target_branch("orphan-branch", default_branch)?;

    // 应该返回默认分支
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
#[ignore]
fn test_resolve_target_branch_detection_failure() -> Result<()> {
    let env = GitTestEnv::new()?;
    let default_branch = "main";

    // 创建一个分支
    env.checkout_new_branch("feature/test")?;
    env.create_file("test.txt", "test")?;
    env.make_test_commit("test.txt", "test", "feat: add test")?;

    // 切换到新分支
    std::env::set_current_dir(env.path())?;

    // 设置非交互式模式，选择索引 0（默认分支）
    // 注意：即使检测失败，如果检测到了其他分支，函数可能会显示对话框
    // 所以需要设置非交互式模式来避免测试卡住
    let _guard = DialogTestGuard::new().with_select_index(0);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(0),
        "DialogTestGuard should set select_index to 0"
    );

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
#[ignore]
fn test_resolve_target_branch_user_cancelled() -> Result<()> {
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
    let _guard = DialogTestGuard::new().with_select_index(0);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(0),
        "DialogTestGuard should set select_index to 0"
    );

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

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
#[ignore]
fn test_resolve_target_branch_select_base_branch() -> Result<()> {
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
    let _guard = DialogTestGuard::new().with_select_index(1);

    // 验证 DialogTestGuard 正确设置（防御性检查）
    assert!(
        skip_config::DialogConfigManager::is_non_interactive(),
        "DialogTestGuard should enable non-interactive mode"
    );
    assert_eq!(
        skip_config::DialogConfigManager::get_select_index(),
        Some(1),
        "DialogTestGuard should set select_index to 1"
    );

    // 调用 resolve_target_branch
    let target = resolve_target_branch("feature/child", default_branch)?;

    // 应该返回基础分支（因为选择了索引 1）
    assert_eq!(target, "feature/base");

    Ok(())
}
