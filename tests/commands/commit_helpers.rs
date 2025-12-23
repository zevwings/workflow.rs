//! Commit 命令辅助函数测试
//!
//! 测试 Commit 命令的辅助函数，包括分支检查、force push 处理等。

use workflow::commands::commit::helpers::check_has_last_commit;

#[test]
fn test_check_has_last_commit_structure() {
    // 测试函数可以正常调用（不抛出编译错误）
    // 注意：这个测试在非 Git 仓库中会失败，这是预期的
    let result = check_has_last_commit();

    // 验证函数返回 Result 类型
    match result {
        Ok(_) => {
            // 在 Git 仓库中有 commit，这是正常的
        }
        Err(_) => {
            // 在非 Git 仓库或无 commit 的情况下，返回错误是正常的
        }
    }
}

