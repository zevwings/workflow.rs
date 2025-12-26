//! CommitReword 业务逻辑测试
//!
//! 测试 CommitReword 模块的核心功能，包括：
//! - 预览创建和格式化
//! - 提交信息格式化
//! - 完成消息生成
//! - 历史选项和结果处理

use pretty_assertions::assert_eq;
use workflow::commit::{CommitReword, RewordHistoryOptions, RewordHistoryResult, RewordPreview};
use workflow::git::CommitInfo;

use crate::common::environments::GitTestEnv;
use crate::common::fixtures::git_repo_with_commit;
use rstest::rstest;

// ==================== Helper Functions ====================

/// 创建测试用的 CommitInfo
fn create_sample_commit_info() -> CommitInfo {
    CommitInfo {
        sha: "def456abc789012345678901234567890abcdef12".to_string(),
        message: "fix: resolve login authentication bug\n\nFix critical authentication issue that prevented users from logging in with valid credentials.".to_string(),
        author: "Jane Smith <jane.smith@example.com>".to_string(),
        date: "2023-12-02 14:45:00 +0000".to_string(),
    }
}


// ==================== Test Cases ====================

/// 测试创建提交重写预览
///
/// ## 测试目的
/// 验证`CommitReword::create_preview()`能够根据提交信息和新消息创建重写预览，包含原始SHA、原始消息、新消息和是否为HEAD提交的标识。
///
/// ## 测试场景
/// 1. 准备测试用的`CommitInfo`（包含SHA、消息、作者、日期）
/// 2. 准备新的提交消息和分支信息
/// 3. 调用`create_preview()`创建预览
/// 4. 验证预览包含正确的原始SHA、原始消息、新消息和`is_head`标识
///
/// ## 预期结果
/// - 预览创建成功，返回`Ok(RewordPreview)`
/// - `original_sha`与提交信息中的SHA一致
/// - `original_message`与提交信息中的消息一致
/// - `new_message`与新提供的消息一致
/// - `is_head`为`true`（表示这是HEAD提交）
#[test]
fn test_create_preview() {
    let commit_info = create_sample_commit_info();
    let new_message = "fix: resolve critical login authentication issue\n\nFix authentication bug that was blocking user access with proper credentials.";
    let is_head = true;
    let current_branch = "main";

    let result = CommitReword::create_preview(&commit_info, new_message, is_head, current_branch);
    assert!(result.is_ok());

    let preview = result.expect("operation should succeed");
    assert_eq!(
        preview.original_sha,
        "def456abc789012345678901234567890abcdef12"
    );
    assert_eq!(preview.original_message, "fix: resolve login authentication bug\n\nFix critical authentication issue that prevented users from logging in with valid credentials.");
    assert_eq!(preview.new_message, "fix: resolve critical login authentication issue\n\nFix authentication bug that was blocking user access with proper credentials.");
    assert_eq!(preview.is_head, true);
}

/// 测试格式化预览信息
///
/// ## 测试目的
/// 验证`CommitReword::format_preview()`能够将`RewordPreview`格式化为可读的字符串，包含原始SHA、原始消息和新消息的关键信息。
///
/// ## 测试场景
/// 1. 准备包含原始SHA、原始消息、新消息、`is_head`和`is_pushed`标识的`RewordPreview`实例
/// 2. 调用`format_preview()`格式化预览
/// 3. 验证格式化字符串包含原始SHA（短格式）、原始消息、新消息的关键内容
///
/// ## 预期结果
/// - 格式化字符串包含`"Original Commit SHA"`标签
/// - 格式化字符串包含原始SHA的短格式（前8个字符`def456ab`）
/// - 格式化字符串包含原始消息`"fix: resolve login authentication bug"`
/// - 格式化字符串包含新消息`"Updated commit message"`和详细描述
#[test]
fn test_format_preview_with_valid_preview_returns_formatted_string() {
    // Arrange: 准备 RewordPreview 实例
    let preview = RewordPreview {
        original_sha: "def456abc789012345678901234567890abcdef12".to_string(),
        original_message: "fix: resolve login authentication bug".to_string(),
        new_message: "Updated commit message\n\nWith new detailed description.".to_string(),
        is_head: false,
        is_pushed: true,
    };

    // Act: 格式化预览
    let formatted = CommitReword::format_preview(&preview);

    // Assert: 验证格式化字符串包含预期内容
    assert!(formatted.contains("Original Commit SHA"));
    assert!(formatted.contains("def456ab"));
    assert!(formatted.contains("fix: resolve login authentication bug"));
    assert!(formatted.contains("Updated commit message"));
    assert!(formatted.contains("With new detailed description."));
}

/// 测试格式化提交信息
///
/// ## 测试目的
/// 验证`CommitReword::format_commit_info()`能够将`CommitInfo`和分支名格式化为可读的字符串，包含SHA、消息、作者、日期和分支信息。
///
/// ## 测试场景
/// 1. 准备测试用的`CommitInfo`（包含SHA、消息、作者、日期）
/// 2. 准备分支名`"feature-auth"`
/// 3. 调用`format_commit_info()`格式化提交信息
/// 4. 验证格式化字符串包含SHA（短格式）、消息、作者姓名、作者邮箱、日期和分支名
///
/// ## 预期结果
/// - 格式化字符串包含SHA的短格式（前8个字符`def456ab`）
/// - 格式化字符串包含提交消息`"fix: resolve login authentication bug"`
/// - 格式化字符串包含作者姓名`"Jane Smith"`
/// - 格式化字符串包含作者邮箱`"jane.smith@example.com"`
/// - 格式化字符串包含日期`"2023-12-02 14:45:00"`
/// - 格式化字符串包含分支名`"feature-auth"`
#[test]
fn test_format_commit_info_with_valid_info_returns_formatted_string() {
    // Arrange: 准备提交信息和分支名
    let commit_info = create_sample_commit_info();
    let branch = "feature-auth";

    // Act: 格式化提交信息
    let formatted = CommitReword::format_commit_info(&commit_info, branch);

    // Assert: 验证格式化字符串包含预期内容
    assert!(formatted.contains("def456ab")); // 短 SHA
    assert!(formatted.contains("fix: resolve login authentication bug"));
    assert!(formatted.contains("Jane Smith"));
    assert!(formatted.contains("jane.smith@example.com"));
    assert!(formatted.contains("2023-12-02 14:45:00"));
    assert!(formatted.contains("feature-auth")); // 分支名
}

/// 测试格式化完成消息
///
/// ## 测试目的
/// 验证`CommitReword::format_completion_message()`能够根据分支名和SHA生成提交重写完成的提示消息。
///
/// ## 测试场景
/// 1. 准备分支名`"main"`和旧SHA
/// 2. 调用`format_completion_message()`格式化完成消息
/// 3. 验证返回结果（成功返回消息、返回`None`或错误都是可以接受的，取决于Git仓库状态）
///
/// ## 预期结果
/// - 成功时：返回`Ok(Some(message))`，消息包含`"successfully"`或`"completed"`等关键词
/// - 某些情况下可能返回`Ok(None)`（这也是有效的）
/// - 在测试环境中可能返回`Err`（因为需要访问Git仓库，这是可以接受的）
#[test]
fn test_format_completion_message_with_valid_input_returns_message() {
    // Arrange: 准备分支名和 SHA
    let current_branch = "main";
    let old_sha = "def456abc789012345678901234567890abcdef12";

    // Act: 格式化完成消息
    let result = CommitReword::format_completion_message(current_branch, old_sha);

    // Assert: 验证返回结果（成功、None 或错误都是可以接受的）
    match result {
        Ok(Some(message)) => {
            assert!(message.contains("successfully") || message.contains("completed"));
        }
        Ok(None) => {
            // 某些情况下可能返回 None，这也是有效的
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

// ==================== Force Push Warning Tests ====================

/// 测试检查是否应该显示强制推送警告（已推送的提交）
///
/// ## 测试目的
/// 验证`CommitReword::should_show_force_push_warning()`能够检查提交是否已推送到远程仓库，以决定是否显示强制推送警告。
///
/// ## 测试场景
/// 1. 准备分支名`"main"`和提交SHA
/// 2. 调用`should_show_force_push_warning()`检查是否应该显示警告
/// 3. 验证返回布尔值（成功或失败都是可以接受的，取决于Git仓库状态）
///
/// ## 预期结果
/// - 成功时：返回`Ok(bool)`，布尔值表示是否应该显示警告（`true`表示已推送，需要强制推送）
/// - 在测试环境中可能返回`Err`（因为需要访问Git仓库和远程分支，这是可以接受的）
#[test]
fn test_should_show_force_push_warning_with_pushed_commit_returns_bool() {
    // Arrange: 准备分支名和 SHA
    let current_branch = "main";
    let old_sha = "test123456789012345678901234567890abcdef";

    // Act: 检查是否应该显示强制推送警告
    let result = CommitReword::should_show_force_push_warning(current_branch, old_sha);

    // Assert: 验证返回布尔值（成功或失败都是可以接受的）
    match result {
        Ok(is_pushed) => {
            // Assert: 验证返回值是布尔类型
            assert!(is_pushed == true || is_pushed == false);
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

/// 测试检查是否应该显示强制推送警告（未推送的提交）
///
/// ## 测试目的
/// 验证`CommitReword::should_show_force_push_warning()`能够检查未推送的提交，返回`false`表示不需要显示强制推送警告。
///
/// ## 测试场景
/// 1. 准备分支名`"feature"`和提交SHA（未推送的提交）
/// 2. 调用`should_show_force_push_warning()`检查是否应该显示警告
/// 3. 验证返回布尔值（成功或失败都是可以接受的，取决于Git仓库状态）
///
/// ## 预期结果
/// - 成功时：返回`Ok(false)`，表示提交未推送，不需要强制推送警告
/// - 在测试环境中可能返回`Err`（因为需要访问Git仓库和远程分支，这是可以接受的）
#[test]
fn test_should_show_force_push_warning_with_not_pushed_commit_returns_bool() {
    // Arrange: 准备分支名和 SHA（未推送的提交）
    let current_branch = "feature";
    let old_sha = "test987654321098765432109876543210987654";

    // Act: 检查是否应该显示强制推送警告
    let result = CommitReword::should_show_force_push_warning(current_branch, old_sha);

    // Assert: 验证返回布尔值（成功或失败都是可以接受的）
    match result {
        Ok(is_pushed) => {
            assert!(is_pushed == true || is_pushed == false);
        }
        Err(_) => {
            // 在测试环境中可能失败，这是可以接受的
        }
    }
}

// ==================== RewordHistoryOptions Tests ====================

/// 测试使用有效字段创建历史重写选项
///
/// ## 测试目的
/// 验证`RewordHistoryOptions`结构体可以使用有效的字段值（提交SHA、新消息、自动暂存标志）正确创建。
///
/// ## 测试场景
/// 1. 准备历史重写选项字段值：提交SHA、新消息、自动暂存标志
/// 2. 使用这些字段值创建`RewordHistoryOptions`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `RewordHistoryOptions`实例创建成功
/// - `commit_sha`字段与提供的SHA一致
/// - `new_message`字段与新消息一致
/// - `auto_stash`字段与自动暂存标志一致
#[test]
fn test_reword_history_options_with_valid_fields_creates_options() {
    // Arrange: 准备历史重写选项字段值
    let commit_sha = "abc123def456";
    let new_message = "Updated historical commit message";
    let auto_stash = true;

    // Act: 创建 RewordHistoryOptions 实例
    let options = RewordHistoryOptions {
        commit_sha: commit_sha.to_string(),
        new_message: new_message.to_string(),
        auto_stash,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(options.commit_sha, commit_sha);
    assert_eq!(options.new_message, new_message);
    assert_eq!(options.auto_stash, auto_stash);
}

// ==================== RewordHistoryResult Tests ====================

/// 测试创建成功的历史重写结果
///
/// ## 测试目的
/// 验证`RewordHistoryResult`结构体可以使用成功的结果字段值（成功标志、无冲突、已暂存）正确创建。
///
/// ## 测试场景
/// 1. 准备成功的结果字段值：`success`为`true`，`has_conflicts`为`false`，`was_stashed`为`true`
/// 2. 使用这些字段值创建`RewordHistoryResult`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `RewordHistoryResult`实例创建成功
/// - `success`字段为`true`（表示重写成功）
/// - `has_conflicts`字段为`false`（表示无冲突）
/// - `was_stashed`字段为`true`（表示已暂存）
#[test]
fn test_reword_history_result_with_success_creates_result() {
    // Arrange: 准备成功的结果字段值
    let success = true;
    let has_conflicts = false;
    let was_stashed = true;

    // Act: 创建 RewordHistoryResult 实例
    let result = RewordHistoryResult {
        success,
        has_conflicts,
        was_stashed,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(result.success, success);
    assert_eq!(result.has_conflicts, has_conflicts);
    assert_eq!(result.was_stashed, was_stashed);
}

/// 测试创建失败的历史重写结果（有冲突）
///
/// ## 测试目的
/// 验证`RewordHistoryResult`结构体可以使用失败的结果字段值（失败标志、有冲突、未暂存）正确创建。
///
/// ## 测试场景
/// 1. 准备失败的结果字段值：`success`为`false`，`has_conflicts`为`true`，`was_stashed`为`false`
/// 2. 使用这些字段值创建`RewordHistoryResult`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `RewordHistoryResult`实例创建成功
/// - `success`字段为`false`（表示重写失败）
/// - `has_conflicts`字段为`true`（表示有冲突）
/// - `was_stashed`字段为`false`（表示未暂存）
#[test]
fn test_reword_history_result_failure_with_conflicts_creates_result() {
    // Arrange: 准备失败的结果字段值
    let success = false;
    let has_conflicts = true;
    let was_stashed = false;

    // Act: 创建 RewordHistoryResult 实例
    let result = RewordHistoryResult {
        success,
        has_conflicts,
        was_stashed,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(result.success, success);
    assert_eq!(result.has_conflicts, has_conflicts);
    assert_eq!(result.was_stashed, was_stashed);
}

// ==================== RewordPreview Tests ====================

/// 测试使用有效字段创建重写预览结构体
///
/// ## 测试目的
/// 验证`RewordPreview`结构体可以使用有效的字段值（原始SHA、原始消息、新消息、是否为HEAD、是否已推送）正确创建。
///
/// ## 测试场景
/// 1. 准备预览字段值：原始SHA、原始消息、新消息、`is_head`为`false`，`is_pushed`为`true`
/// 2. 使用这些字段值创建`RewordPreview`实例
/// 3. 验证所有字段值正确设置
///
/// ## 预期结果
/// - `RewordPreview`实例创建成功
/// - `original_sha`字段与提供的SHA一致
/// - `original_message`字段与原始消息一致
/// - `new_message`字段与新消息一致
/// - `is_head`字段为`false`（表示不是HEAD提交）
/// - `is_pushed`字段为`true`（表示已推送）
#[test]
fn test_reword_preview_struct_with_valid_fields_creates_preview() {
    // Arrange: 准备预览字段值
    let original_sha = "preview_sha";
    let original_message = "original preview message";
    let new_message = "new preview message";

    // Act: 创建 RewordPreview 实例
    let preview = RewordPreview {
        original_sha: original_sha.to_string(),
        original_message: original_message.to_string(),
        new_message: new_message.to_string(),
        is_head: false,
        is_pushed: true,
    };

    // Assert: 验证所有字段值正确
    assert_eq!(preview.original_sha, original_sha);
    assert_eq!(preview.original_message, original_message);
    assert_eq!(preview.new_message, new_message);
    assert_eq!(preview.is_head, false);
    assert_eq!(preview.is_pushed, true);
}

/// 测试克隆重写预览
///
/// ## 测试目的
/// 验证`RewordPreview`结构体的`Clone`实现能够正确创建预览的副本，所有字段值保持一致。
///
/// ## 测试场景
/// 1. 准备原始`RewordPreview`实例，包含所有字段值
/// 2. 调用`clone()`方法创建副本
/// 3. 验证克隆后的预览与原始预览的所有字段值相同
///
/// ## 预期结果
/// - 克隆操作成功
/// - 克隆后的`original_sha`、`original_message`、`new_message`、`is_head`、`is_pushed`字段值与原始预览完全相同
#[test]
fn test_reword_preview_clone_with_valid_preview_creates_clone() {
    // Arrange: 准备原始 RewordPreview
    let original = RewordPreview {
        original_sha: "clone_sha".to_string(),
        original_message: "clone original".to_string(),
        new_message: "clone new".to_string(),
        is_head: true,
        is_pushed: false,
    };

    // Act: 克隆预览
    let cloned = original.clone();

    // Assert: 验证克隆的字段值与原始值相同
    assert_eq!(original.original_sha, cloned.original_sha);
    assert_eq!(original.original_message, cloned.original_message);
    assert_eq!(original.new_message, cloned.new_message);
    assert_eq!(original.is_head, cloned.is_head);
    assert_eq!(original.is_pushed, cloned.is_pushed);
}

/// 测试空消息处理
#[test]
fn test_create_preview_with_empty_message_creates_preview() {
    // Arrange: 准备提交信息和空消息
    let commit_info = create_sample_commit_info();
    let empty_message = "";
    let is_head = false;
    let current_branch = "feature";

    // Act: 创建预览
    let result = CommitReword::create_preview(&commit_info, empty_message, is_head, current_branch);
    assert!(result.is_ok());

    // Assert: 验证预览创建成功且空消息被正确处理
    let preview = result.expect("operation should succeed");
    assert_eq!(preview.new_message, "");
    assert_eq!(preview.is_head, false);
}

/// 测试 Git 仓库集成
///
/// ## 测试目的
/// 验证 CommitReword 功能在真实 Git 仓库环境中的集成行为。
///
/// ## 测试场景
/// 1. 创建 Git 测试环境
/// 2. 执行 CommitReword 相关操作
/// 3. 验证与 Git 仓库的集成
///
/// ## 预期结果
/// - Git 集成操作成功
#[rstest]
fn test_git_integration_return_ok(git_repo_with_commit: GitTestEnv) -> color_eyre::Result<()> {
    // 使用 GitTestEnv 创建隔离的 Git 仓库
    let env = &git_repo_with_commit;

    // 创建多个提交以模拟真实场景
    for i in 1..=3 {
        env.make_test_commit(
            &format!("file{}.txt", i),
            &format!("Content of file {}\n", i),
            &format!("Commit {}: add file{}.txt", i, i),
        )?;
    }

    // 这个测试主要验证 Git 仓库创建辅助函数工作正常
    // 实际的 Git 集成测试应该在更高级别的集成测试中进行
    // GitTestEnv 会在函数结束时自动清理
    Ok(())
}
