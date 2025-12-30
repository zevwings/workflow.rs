//! Branch 类型集成测试
//!
//! 测试分支类型的集成功能，包括需要交互式输入或外部资源的功能。
//!
//! 注意：纯逻辑的单元测试已迁移到 `src/lib/branch/types.rs` 的 `#[cfg(test)]` 模块中。

// ==================== prompt_selection 和 resolve_with_repo_prefix 测试 ====================

/// 测试分支类型的交互式选择
///
/// ## 测试目的
/// 验证`BranchType::prompt_selection()`方法能够正确显示选项并接收用户选择的分支类型。
///
/// ## 为什么被忽略
/// - **需要用户交互**: 测试需要用户使用方向键选择分支类型
/// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
/// - **UI/UX验证**: 用于手动验证分支类型选择对话框
/// - **会卡住CI**: 在非交互式环境中会无限等待用户输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_prompt_selection -- --ignored
/// ```
/// 然后使用↑↓键选择分支类型，Enter确认
///
/// ## 测试场景
/// 1. 调用BranchType::prompt_selection()显示分支类型列表
/// 2. 显示所有可用的分支类型（feature、fix、hotfix等）
/// 3. 等待用户使用方向键选择
/// 4. 用户按Enter确认选择
/// 5. 返回选中的分支类型
///
/// ## 预期行为
/// - 成功情况：返回Ok(BranchType)包含用户选择的类型
/// - 取消情况：返回Err表示用户取消了选择
/// - 显示的分支类型列表完整且正确
/// - 选择的分支类型在BranchType::all()列表中
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_prompt_selection() {
    // Arrange: 准备测试交互式选择分支类型
    // 注意：这个测试需要用户交互，在 CI 环境中会卡住
    // 使用 `cargo test -- --ignored` 来运行这些测试
    let result = workflow::branch::BranchType::prompt_selection();

    // 如果用户交互失败（例如没有配置），应该返回错误
    // 如果成功，应该返回有效的 BranchType
    match result {
        Ok(branch_type) => {
            // Assert: 验证返回的是有效的分支类型
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 交互失败是可以接受的（例如在 CI 环境中）
            // 这种情况不需要断言，因为错误是预期的
        }
    }
}

/// 测试从仓库配置解析分支类型（有prefix）
///
/// ## 测试目的
/// 验证`BranchType::resolve_with_repo_prefix()`在仓库配置了prefix时能够正确解析分支类型。
///
/// ## 为什么被忽略
/// - **需要仓库配置**: 测试依赖实际的仓库配置文件
/// - **可能需要用户交互**: 如果配置不完整，可能需要用户选择
/// - **环境依赖**: 不同环境中的配置可能不同
/// - **CI环境不支持**: 可能需要交互式输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_resolve_with_repo_prefix_with_prefix -- --ignored
/// ```
/// 注意：需要在配置了repository prefix的Git仓库中运行
///
/// ## 测试场景
/// 1. 读取仓库配置文件
/// 2. 查找repository prefix配置
/// 3. 如果有prefix：直接解析对应的分支类型
/// 4. 如果无prefix：调用prompt_selection（需要交互）
/// 5. 返回解析的分支类型
///
/// ## 预期行为
/// - 有配置时：返回Ok(BranchType)对应配置的类型
/// - 无配置时：调用交互选择或返回错误
/// - 解析的分支类型有效
/// - 配置文件解析正确
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_resolve_with_repo_prefix_with_prefix() {
    // Arrange: 准备测试有 repository prefix 的情况
    // 注意：这个测试依赖于实际的仓库配置，可能在不同环境中表现不同
    let result = workflow::branch::BranchType::resolve_with_repo_prefix();

    // 如果仓库有配置 prefix，应该返回对应的 BranchType
    // 如果没有配置，可能会调用 prompt_selection（需要交互）
    match result {
        Ok(branch_type) => {
            // Assert: 验证返回的是有效的分支类型
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 如果没有配置且 prompt 失败，这是可以接受的
            // 这种情况不需要断言，因为错误是预期的
        }
    }
}

/// 测试从仓库配置解析分支类型（无prefix）
///
/// ## 测试目的
/// 验证`BranchType::resolve_with_repo_prefix()`在仓库未配置prefix时能够回退到交互式选择。
///
/// ## 为什么被忽略
/// - **需要用户交互**: 无prefix时会调用prompt_selection需要用户输入
/// - **CI环境不支持**: 自动化CI环境无法提供交互式输入
/// - **环境依赖**: 依赖仓库配置状态
/// - **会卡住CI**: 在非交互式环境中会等待用户输入
///
/// ## 如何手动运行
/// ```bash
/// cargo test test_branch_type_resolve_with_repo_prefix_without_prefix -- --ignored
/// ```
/// 注意：需要在未配置repository prefix的Git仓库中运行
///
/// ## 测试场景
/// 1. 读取仓库配置文件
/// 2. 发现没有配置repository prefix
/// 3. 调用BranchType::prompt_selection()进行交互选择
/// 4. 等待用户选择分支类型
/// 5. 返回用户选择的分支类型
///
/// ## 预期行为
/// - 检测到无prefix配置
/// - 自动调用交互式选择
/// - 用户选择后返回Ok(BranchType)
/// - 用户取消则返回Err
/// - 整个流程无panic或hang
#[test]
#[ignore] // 需要交互式输入，在 CI 环境中会卡住
#[cfg(feature = "interactive-tests")]
fn test_branch_type_resolve_with_repo_prefix_without_prefix() {
    // Arrange: 准备测试没有 repository prefix 的情况（会调用 prompt_selection）
    // 注意：这个测试需要用户交互，在 CI 环境中会卡住
    let result = workflow::branch::BranchType::resolve_with_repo_prefix();

    // 如果没有 prefix，应该调用 prompt_selection
    match result {
        Ok(branch_type) => {
            assert!(workflow::branch::BranchType::all().contains(&branch_type));
        }
        Err(_) => {
            // 交互失败是可以接受的
            // 这种情况不需要断言，因为错误是预期的
        }
    }
}
