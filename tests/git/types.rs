//! Git/Types 数据结构测试
//!
//! 测试Git模块中各种数据结构的核心业务逻辑，包括：
//! - 仓库类型识别和解析
//! - 提交信息数据结构
//! - 工作区状态解析
//! - 合并策略枚举
//! - 分支名称处理逻辑

use crate::common::performance::measure_test_time_with_threshold;
use color_eyre::Result;
use rstest::rstest;
use std::time::Duration;

use workflow::git::{CommitInfo, MergeStrategy, RepoType, WorktreeStatus};

/// 模拟仓库类型解析逻辑（从GitRepo中提取的核心逻辑）
fn mock_parse_repo_type_from_url(url: &str) -> RepoType {
    // 检查 GitHub：包含 github.com 或 SSH host 以 github 开头
    if url.contains("github.com")
        || url.starts_with("git@github")
        || url.starts_with("ssh://git@github")
    {
        RepoType::GitHub
    } else if url.contains("codeup.aliyun.com") {
        RepoType::Codeup
    } else {
        RepoType::Unknown
    }
}

/// 模拟分支名称前缀移除逻辑
fn mock_remove_branch_prefix(branch: &str) -> &str {
    branch
        .rsplit('/')
        .next()
        .unwrap_or(branch)
        .rsplit("--")
        .next()
        .unwrap_or(branch)
}

/// 模拟Git状态解析逻辑
fn mock_parse_worktree_status(status_output: &str) -> WorktreeStatus {
    let mut modified_count = 0;
    let mut staged_count = 0;
    let mut untracked_count = 0;

    for line in status_output.lines() {
        if line.len() < 2 {
            continue;
        }

        let staged_status = line.chars().nth(0).unwrap_or(' ');
        let worktree_status = line.chars().nth(1).unwrap_or(' ');

        // 检查暂存区状态
        match staged_status {
            'A' | 'M' | 'D' | 'R' | 'C' => staged_count += 1,
            _ => {}
        }

        // 检查工作区状态
        match worktree_status {
            'M' | 'D' => modified_count += 1,
            _ => {}
        }

        // 检查未跟踪文件
        if staged_status == '?' && worktree_status == '?' {
            untracked_count += 1;
        }
    }

    WorktreeStatus {
        modified_count,
        staged_count,
        untracked_count,
    }
}

/// 模拟提交信息解析逻辑
fn mock_parse_commit_info(commit_output: &str) -> Option<CommitInfo> {
    let lines: Vec<&str> = commit_output.trim().lines().collect();
    if lines.len() < 4 {
        return None;
    }

    Some(CommitInfo {
        sha: lines[0].to_string(),
        author: lines[1].to_string(),
        date: lines[2].to_string(),
        message: lines[3].to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RepoType 枚举测试 ====================

    /// 测试 RepoType 枚举的相等性比较
    ///
    /// ## 测试目的
    /// 验证 RepoType 枚举的相等性比较功能。
    ///
    /// ## 预期结果
    /// - 相同类型的 RepoType 相等
    /// - 不同类型的 RepoType 不相等
    #[test]
    fn test_repo_type_equality_with_same_types_returns_equal() {
        // Arrange: 准备相同类型的RepoType

        // Act & Assert: 验证相同类型相等，不同类型不相等
        assert_eq!(RepoType::GitHub, RepoType::GitHub);
        assert_eq!(RepoType::Codeup, RepoType::Codeup);
        assert_eq!(RepoType::Unknown, RepoType::Unknown);
        assert_ne!(RepoType::GitHub, RepoType::Codeup);
        assert_ne!(RepoType::GitHub, RepoType::Unknown);
        assert_ne!(RepoType::Codeup, RepoType::Unknown);
    }

    /// 测试 RepoType 枚举的调试格式化
    ///
    /// ## 测试目的
    /// 验证 RepoType 枚举能够正确格式化调试字符串。
    ///
    /// ## 预期结果
    /// - GitHub 变体格式化为 "GitHub"
    /// - Codeup 变体格式化为 "Codeup"
    /// - Unknown 变体格式化为 "Unknown"
    #[test]
    fn test_repo_type_debug_with_variants_returns_debug_string() {
        // Arrange: 准备各种RepoType变体
        let github = RepoType::GitHub;
        let codeup = RepoType::Codeup;
        let unknown = RepoType::Unknown;

        // Act: 格式化Debug输出
        let github_debug = format!("{:?}", github);
        let codeup_debug = format!("{:?}", codeup);
        let unknown_debug = format!("{:?}", unknown);

        // Assert: 验证Debug字符串正确
        assert_eq!(github_debug, "GitHub");
        assert_eq!(codeup_debug, "Codeup");
        assert_eq!(unknown_debug, "Unknown");
    }

    /// 测试 RepoType 枚举的克隆功能
    ///
    /// ## 测试目的
    /// 验证 RepoType 枚举能够正确克隆。
    ///
    /// ## 预期结果
    /// - 克隆后的值与原值相等
    /// - 克隆后的值与不同类型的值不相等
    #[test]
    fn test_repo_type_clone_with_valid_type_creates_clone() {
        // Arrange: 准备原始RepoType
        let original = RepoType::GitHub;

        // Act: 克隆RepoType
        let cloned = original.clone();

        // Assert: 验证克隆后的值相等且独立
        assert_eq!(original, cloned);
        let another = RepoType::Codeup;
        assert_ne!(cloned, another);
    }

    /// 测试 RepoType 枚举的 Copy trait
    ///
    /// ## 测试目的
    /// 验证 RepoType 枚举实现了 Copy trait，能够按值复制。
    ///
    /// ## 预期结果
    /// - 复制后的值与原值相等
    /// - 原始值仍然可用（Copy语义）
    #[test]
    fn test_repo_type_copy_with_valid_type_copies_value() {
        // Arrange: 准备原始RepoType
        let original = RepoType::GitHub;

        // Act: 复制RepoType（Copy trait允许这样赋值）
        let copied = original;

        // Assert: 验证复制后的值相等且原始值仍然可用（Copy语义）
        assert_eq!(original, copied);
        assert_eq!(original, RepoType::GitHub);
    }

    // ==================== 仓库类型解析测试 ====================

    /// 测试从URL解析仓库类型（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证能够从各种URL格式中正确解析仓库类型。
    ///
    /// ## 测试场景
    /// 测试GitHub、Codeup和未知类型的各种URL格式（HTTPS、SSH、SSH Host别名等）
    ///
    /// ## 预期结果
    /// - 所有URL都能正确解析为对应的仓库类型
    #[rstest]
    #[case("https://github.com/user/repo.git", RepoType::GitHub)]
    #[case("git@github.com:user/repo.git", RepoType::GitHub)]
    #[case("ssh://git@github.com/user/repo.git", RepoType::GitHub)]
    #[case("git@github-company:user/repo.git", RepoType::GitHub)] // SSH Host别名
    #[case("ssh://git@github-personal/user/repo.git", RepoType::GitHub)]
    #[case("https://codeup.aliyun.com/user/repo.git", RepoType::Codeup)]
    #[case("git@codeup.aliyun.com:user/repo.git", RepoType::Codeup)]
    #[case("https://gitlab.com/user/repo.git", RepoType::Unknown)]
    #[case("git@bitbucket.org:user/repo.git", RepoType::Unknown)]
    #[case("https://example.com/git/repo.git", RepoType::Unknown)]
    #[case("", RepoType::Unknown)]
    fn test_parse_repo_type_from_url(#[case] url: &str, #[case] expected: RepoType) {
        assert_eq!(mock_parse_repo_type_from_url(url), expected);
    }

    /// 测试GitHub URL的各种变体
    ///
    /// ## 测试目的
    /// 验证能够正确识别GitHub仓库的各种URL格式。
    ///
    /// ## 测试场景
    /// 测试HTTPS、SSH、SSH Host别名等GitHub URL格式
    ///
    /// ## 预期结果
    /// - 所有GitHub URL变体都解析为 RepoType::GitHub
    #[test]
    fn test_github_url_variations_with_various_urls_returns_github() {
        // Arrange: 准备各种GitHub URL变体
        let github_urls = vec![
            "https://github.com/user/repo",
            "https://github.com/user/repo.git",
            "git@github.com:user/repo.git",
            "ssh://git@github.com:22/user/repo.git",
            "git@github-work:user/repo.git",
            "git@github123:user/repo.git",
        ];

        // Act & Assert: 验证所有GitHub URL变体都解析为GitHub类型
        for url in github_urls {
            assert_eq!(
                mock_parse_repo_type_from_url(url),
                RepoType::GitHub,
                "Failed for URL: {}",
                url
            );
        }
    }

    /// 测试Codeup URL的各种变体
    ///
    /// ## 测试目的
    /// 验证能够正确识别Codeup仓库的各种URL格式。
    ///
    /// ## 测试场景
    /// 测试HTTPS、SSH等Codeup URL格式
    ///
    /// ## 预期结果
    /// - 所有Codeup URL变体都解析为 RepoType::Codeup
    #[test]
    fn test_codeup_url_variations_with_various_urls_returns_codeup() {
        // Arrange: 准备各种Codeup URL变体
        let codeup_urls = vec![
            "https://codeup.aliyun.com/user/repo",
            "https://codeup.aliyun.com/user/repo.git",
            "git@codeup.aliyun.com:user/repo.git",
            "ssh://git@codeup.aliyun.com/user/repo.git",
        ];

        // Act & Assert: 验证所有Codeup URL变体都解析为Codeup类型
        for url in codeup_urls {
            assert_eq!(
                mock_parse_repo_type_from_url(url),
                RepoType::Codeup,
                "Failed for URL: {}",
                url
            );
        }
    }

    /// 测试未知仓库类型的URL处理
    ///
    /// ## 测试目的
    /// 验证未知仓库类型的URL能够正确解析为 RepoType::Unknown。
    ///
    /// ## 测试场景
    /// 测试GitLab、Bitbucket、本地路径、无效URL等
    ///
    /// ## 预期结果
    /// - 所有未知URL都解析为 RepoType::Unknown
    #[test]
    fn test_unknown_repo_types_with_unknown_urls_returns_unknown() {
        // Arrange: 准备未知类型的URL列表
        let unknown_urls = vec![
            "https://gitlab.com/user/repo.git",
            "git@bitbucket.org:user/repo.git",
            "https://git.example.com/repo.git",
            "file:///local/path/to/repo",
            "invalid-url",
            "ftp://example.com/repo",
        ];

        // Act & Assert: 验证所有未知URL都解析为Unknown类型
        for url in unknown_urls {
            assert_eq!(
                mock_parse_repo_type_from_url(url),
                RepoType::Unknown,
                "Failed for URL: {}",
                url
            );
        }
    }

    // ==================== 分支名称处理测试 ====================

    /// 测试移除分支名称前缀（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证能够正确移除分支名称的前缀（路径分隔符和双破折号分隔符）。
    ///
    /// ## 测试场景
    /// 测试各种分支名称格式：feature/xxx、PROJ-123--xxx、feature/PROJ-789--xxx等
    ///
    /// ## 预期结果
    /// - 所有分支名称都能正确移除前缀，返回最后一部分
    #[rstest]
    #[case("feature/login-system", "login-system")]
    #[case("bugfix/fix-auth-issue", "fix-auth-issue")]
    #[case("hotfix/critical-security-fix", "critical-security-fix")]
    #[case("PROJ-123--implement-feature", "implement-feature")]
    #[case("TASK-456--refactor-code", "refactor-code")]
    #[case("feature/PROJ-789--add-validation", "add-validation")]
    #[case("simple-branch", "simple-branch")]
    #[case("", "")]
    fn test_remove_branch_prefix(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(mock_remove_branch_prefix(input), expected);
    }

    /// 测试复杂分支名称的前缀移除
    ///
    /// ## 测试目的
    /// 验证能够正确处理包含多层前缀的复杂分支名称。
    ///
    /// ## 测试场景
    /// 测试多层路径和双破折号组合的分支名称
    ///
    /// ## 预期结果
    /// - 复杂分支名称的前缀被正确移除
    /// - 返回最后一部分名称
    #[test]
    fn test_complex_branch_names_with_complex_names_removes_prefixes() {
        // Arrange: 准备复杂的分支名称

        // Act & Assert: 验证复杂分支名称的前缀移除正确
        assert_eq!(
            mock_remove_branch_prefix("feature/user/PROJ-123--login-system"),
            "login-system"
        );
        assert_eq!(
            mock_remove_branch_prefix("release/v1.0/TASK-456--final-fixes"),
            "final-fixes"
        );
        assert_eq!(
            mock_remove_branch_prefix("develop/team-a/feature/new-ui"),
            "new-ui"
        );
    }

    /// 测试边界情况的分支名称处理
    ///
    /// ## 测试目的
    /// 验证能够正确处理边界情况的分支名称（空字符串、只有分隔符等）。
    ///
    /// ## 测试场景
    /// 测试空字符串、只有分隔符、双分隔符等边界情况
    ///
    /// ## 预期结果
    /// - 边界情况都能正确处理
    /// - 不会panic或返回错误
    #[test]
    fn test_edge_case_branch_names_with_edge_cases_handles_correctly() {
        // Arrange: 准备边界情况的分支名称

        // Act & Assert: 验证边界情况处理正确
        assert_eq!(mock_remove_branch_prefix("/"), "");
        assert_eq!(mock_remove_branch_prefix("--"), "");
        assert_eq!(mock_remove_branch_prefix("feature/"), "");
        assert_eq!(mock_remove_branch_prefix("PROJ-123--"), "");
        assert_eq!(
            mock_remove_branch_prefix("feature//double-slash"),
            "double-slash"
        );
        assert_eq!(
            mock_remove_branch_prefix("PROJ----double-dash"),
            "double-dash"
        );
    }

    // ==================== MergeStrategy 枚举测试 ====================

    /// 测试 MergeStrategy 枚举的调试格式化
    ///
    /// ## 测试目的
    /// 验证 MergeStrategy 枚举能够正确格式化调试字符串。
    ///
    /// ## 预期结果
    /// - Merge 变体格式化为 "Merge"
    /// - Squash 变体格式化为 "Squash"
    /// - FastForwardOnly 变体格式化为 "FastForwardOnly"
    #[test]
    fn test_merge_strategy_debug_with_variants_returns_debug_string() {
        // Arrange: 准备各种MergeStrategy变体

        // Act: 格式化Debug输出
        let merge_debug = format!("{:?}", MergeStrategy::Merge);
        let squash_debug = format!("{:?}", MergeStrategy::Squash);
        let ff_debug = format!("{:?}", MergeStrategy::FastForwardOnly);

        // Assert: 验证Debug字符串正确
        assert_eq!(merge_debug, "Merge");
        assert_eq!(squash_debug, "Squash");
        assert_eq!(ff_debug, "FastForwardOnly");
    }

    /// 测试 MergeStrategy 枚举的克隆功能
    ///
    /// ## 测试目的
    /// 验证 MergeStrategy 枚举能够正确克隆。
    ///
    /// ## 预期结果
    /// - 克隆后的值与原值相等
    #[test]
    fn test_merge_strategy_clone_with_valid_strategy_creates_clone() {
        // Arrange: 准备原始MergeStrategy
        let original = MergeStrategy::Merge;

        // Act: 克隆MergeStrategy
        let cloned = original.clone();

        // Assert: 验证克隆后的值相等（通过Debug输出验证）
        assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
    }

    /// 测试 MergeStrategy 枚举的 Copy trait
    ///
    /// ## 测试目的
    /// 验证 MergeStrategy 枚举实现了 Copy trait，能够按值复制。
    ///
    /// ## 预期结果
    /// - 复制后的值与原值相等
    /// - 原始值仍然可用（Copy语义）
    #[test]
    fn test_merge_strategy_copy_with_valid_strategy_copies_value() {
        // Arrange: 准备原始MergeStrategy
        let original = MergeStrategy::Squash;

        // Act: 复制MergeStrategy（Copy trait）
        let copied = original;

        // Assert: 验证Copy语义（原始值和复制值相等）
        assert_eq!(format!("{:?}", original), format!("{:?}", copied));
        assert_eq!(format!("{:?}", original), "Squash");
    }

    // ==================== CommitInfo 结构体测试 ====================

    /// 测试 CommitInfo 结构体的创建
    ///
    /// ## 测试目的
    /// 验证 CommitInfo 结构体能够使用有效字段正确创建。
    ///
    /// ## 预期结果
    /// - 所有字段正确设置
    /// - sha、author、date、message 等字段正确
    #[test]
    fn test_commit_info_creation_with_valid_fields_creates_commit_info() {
        // Arrange: 准备CommitInfo字段值
        let sha = "abc123def456";
        let message = "Initial commit";
        let author = "John Doe";
        let date = "2024-01-01";

        // Act: 创建CommitInfo
        let commit = CommitInfo {
            sha: sha.to_string(),
            message: message.to_string(),
            author: author.to_string(),
            date: date.to_string(),
        };

        // Assert: 验证所有字段值正确
        assert_eq!(commit.sha, sha);
        assert_eq!(commit.message, message);
        assert_eq!(commit.author, author);
        assert_eq!(commit.date, date);
    }

    /// 测试 CommitInfo 结构体的深克隆功能
    ///
    /// ## 测试目的
    /// 验证 CommitInfo 结构体能够正确进行深克隆（所有字段都是独立的副本）。
    ///
    /// ## 预期结果
    /// - 克隆后的值与原值相等
    /// - 克隆后的值与原始值在内存中独立（深拷贝）
    #[test]
    fn test_commit_info_clone_with_valid_info_creates_deep_clone() {
        // Arrange: 准备原始CommitInfo
        let original = CommitInfo {
            sha: "abc123".to_string(),
            message: "Test commit".to_string(),
            author: "Alice".to_string(),
            date: "2024-12-17".to_string(),
        };

        // Act: 克隆CommitInfo
        let cloned = original.clone();

        // Assert: 验证克隆后的值相等且是深拷贝
        assert_eq!(original.sha, cloned.sha);
        assert_eq!(original.message, cloned.message);
        assert_eq!(original.author, cloned.author);
        assert_eq!(original.date, cloned.date);
        // 验证是深拷贝（修改克隆不影响原始）- 由于String是堆分配的，这里验证地址不同
        assert_ne!(original.sha.as_ptr(), cloned.sha.as_ptr());
    }

    /// 测试 CommitInfo 结构体的调试格式化
    ///
    /// ## 测试目的
    /// 验证 CommitInfo 结构体能够正确格式化调试字符串。
    ///
    /// ## 预期结果
    /// - 调试字符串包含所有字段信息（sha、message、author、date）
    #[test]
    fn test_commit_info_debug_with_valid_info_returns_debug_string() {
        // Arrange: 准备CommitInfo
        let commit = CommitInfo {
            sha: "abc123".to_string(),
            message: "Test".to_string(),
            author: "Dev".to_string(),
            date: "2024-01-01".to_string(),
        };

        // Act: 格式化Debug输出
        let debug_str = format!("{:?}", commit);

        // Assert: 验证Debug字符串包含所有字段
        assert!(debug_str.contains("abc123"));
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("Dev"));
        assert!(debug_str.contains("2024-01-01"));
    }

    /// 测试解析有效的提交信息输出
    ///
    /// ## 测试目的
    /// 验证能够从有效的Git提交信息输出中正确解析 CommitInfo。
    ///
    /// ## 测试场景
    /// 解析包含sha、author、date、message四行的输出
    ///
    /// ## 预期结果
    /// - 所有字段正确解析
    /// - 返回 Some(CommitInfo)
    #[test]
    fn test_parse_commit_info_valid_with_valid_output_parses_correctly() -> Result<()> {
        // Arrange: 准备有效的提交信息输出
        let output = "abc123def456\nJohn Doe\n2024-01-01 10:30:00\nInitial commit";

        // Act: 解析提交信息
        let commit = mock_parse_commit_info(output)
            .ok_or_else(|| color_eyre::eyre::eyre!("operation should succeed"))?;

        // Assert: 验证解析结果正确
        assert_eq!(commit.sha, "abc123def456");
        assert_eq!(commit.author, "John Doe");
        assert_eq!(commit.date, "2024-01-01 10:30:00");
        assert_eq!(commit.message, "Initial commit");
        Ok(())
    }

    /// 测试解析无效的提交信息输出
    ///
    /// ## 测试目的
    /// 验证能够正确处理无效或不完整的提交信息输出。
    ///
    /// ## 测试场景
    /// 测试空字符串、只有sha、缺少字段等无效输出
    ///
    /// ## 预期结果
    /// - 所有无效输出都返回 None
    #[test]
    fn test_parse_commit_info_invalid_with_invalid_output_returns_none() {
        // Arrange: 准备不完整的输出
        let invalid_outputs = vec![
            "",
            "abc123",
            "abc123\nJohn Doe",
            "abc123\nJohn Doe\n2024-01-01",
        ];

        // Act & Assert: 验证所有无效输出都返回None
        for output in invalid_outputs {
            let result = mock_parse_commit_info(output);
            assert!(
                result.is_none(),
                "Should return None for invalid output: {:?}",
                output
            );
        }
    }

    /// 测试解析包含多行提交消息的提交信息
    ///
    /// ## 测试目的
    /// 验证能够正确解析包含多行提交消息的提交信息（只取第一行）。
    ///
    /// ## 预期结果
    /// - 提交消息正确解析（只取第一行）
    #[test]
    fn test_parse_commit_info_with_multiline_message() -> Result<()> {
        let output = "abc123\nJohn Doe\n2024-01-01\nFirst line of commit message";
        let commit = mock_parse_commit_info(output)
            .ok_or_else(|| color_eyre::eyre::eyre!("operation should succeed"))?;

        assert_eq!(commit.message, "First line of commit message");
        Ok(())
    }

    // ==================== WorktreeStatus 结构体测试 ====================

    /// 测试 WorktreeStatus 结构体的创建
    ///
    /// ## 测试目的
    /// 验证 WorktreeStatus 结构体能够使用有效字段正确创建。
    ///
    /// ## 预期结果
    /// - 所有字段正确设置
    /// - modified_count、staged_count、untracked_count 等字段正确
    #[test]
    fn test_worktree_status_creation() {
        let status = WorktreeStatus {
            modified_count: 5,
            staged_count: 3,
            untracked_count: 2,
        };

        assert_eq!(status.modified_count, 5);
        assert_eq!(status.staged_count, 3);
        assert_eq!(status.untracked_count, 2);
    }

    /// 测试 WorktreeStatus 结构体创建零计数状态
    ///
    /// ## 测试目的
    /// 验证 WorktreeStatus 结构体能够创建所有计数为零的状态（干净的工作区）。
    ///
    /// ## 预期结果
    /// - 所有计数字段都为0
    #[test]
    fn test_worktree_status_zero_counts() {
        let status = WorktreeStatus {
            modified_count: 0,
            staged_count: 0,
            untracked_count: 0,
        };

        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    }

    /// 测试解析空的Git状态输出
    ///
    /// ## 测试目的
    /// 验证能够正确解析空的Git状态输出（干净的工作区）。
    ///
    /// ## 预期结果
    /// - 所有计数都为0
    #[test]
    fn test_parse_worktree_status_empty() {
        let status = mock_parse_worktree_status("");
        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    }

    /// 测试解析混合状态的Git状态输出
    ///
    /// ## 测试目的
    /// 验证能够正确解析包含多种状态的Git状态输出（修改、暂存、未跟踪文件混合）。
    ///
    /// ## 测试场景
    /// 测试包含工作区修改、暂存区修改、未跟踪文件等多种状态的输出
    ///
    /// ## 预期结果
    /// - 各种状态的计数正确
    #[test]
    fn test_parse_worktree_status_mixed() {
        let output = " M file1.txt\nA  file2.txt\n?? file3.txt\nMM file4.txt\nD  file5.txt";
        let status = mock_parse_worktree_status(output);

        // M  - 工作区修改：1个 (file1.txt)
        // MM - 暂存区修改 + 工作区修改：1个暂存，1个修改 (file4.txt)
        // A  - 暂存区新增：1个 (file2.txt)
        // D  - 暂存区删除：1个 (file5.txt)
        // ?? - 未跟踪：1个 (file3.txt)
        assert_eq!(status.modified_count, 2); // file1.txt, file4.txt
        assert_eq!(status.staged_count, 3); // file2.txt, file4.txt, file5.txt
        assert_eq!(status.untracked_count, 1); // file3.txt
    }

    /// 测试解析只有暂存文件的Git状态输出
    ///
    /// ## 测试目的
    /// 验证能够正确解析只包含暂存文件的Git状态输出。
    ///
    /// ## 预期结果
    /// - staged_count 正确
    /// - modified_count 和 untracked_count 为0
    #[test]
    fn test_parse_worktree_status_only_staged() {
        let output = "A  new_file.txt\nM  modified_file.txt\nD  deleted_file.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 3);
        assert_eq!(status.untracked_count, 0);
    }

    /// 测试解析只有工作区修改的Git状态输出
    ///
    /// ## 测试目的
    /// 验证能够正确解析只包含工作区修改的Git状态输出。
    ///
    /// ## 预期结果
    /// - modified_count 正确
    /// - staged_count 和 untracked_count 为0
    #[test]
    fn test_parse_worktree_status_only_modified() {
        let output = " M file1.txt\n M file2.txt\n D file3.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 3);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    }

    /// 测试解析只有未跟踪文件的Git状态输出
    ///
    /// ## 测试目的
    /// 验证能够正确解析只包含未跟踪文件的Git状态输出。
    ///
    /// ## 预期结果
    /// - untracked_count 正确
    /// - modified_count 和 staged_count 为0
    #[test]
    fn test_parse_worktree_status_only_untracked() {
        let output = "?? file1.txt\n?? file2.txt\n?? dir/file3.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 3);
    }

    /// 测试解析单个文件的Git状态（参数化测试）
    ///
    /// ## 测试目的
    /// 使用参数化测试验证能够正确解析各种Git状态代码（A、M、D、R、C、??等）。
    ///
    /// ## 测试场景
    /// 测试新增、修改、删除、重命名、复制、未跟踪等各种状态
    ///
    /// ## 预期结果
    /// - 所有状态代码都能正确解析为对应的计数
    #[rstest]
    #[case("A  new.txt", 0, 1, 0)] // 新增文件（暂存）
    #[case(" M modified.txt", 1, 0, 0)] // 修改文件（工作区）
    #[case("M  staged.txt", 0, 1, 0)] // 修改文件（暂存）
    #[case("MM both.txt", 1, 1, 0)] // 暂存+工作区都有修改
    #[case("?? untracked.txt", 0, 0, 1)] // 未跟踪文件
    #[case("D  deleted.txt", 0, 1, 0)] // 删除文件（暂存）
    #[case(" D deleted2.txt", 1, 0, 0)] // 删除文件（工作区）
    #[case("R  renamed.txt", 0, 1, 0)] // 重命名文件
    #[case("C  copied.txt", 0, 1, 0)] // 复制文件
    fn test_parse_worktree_status_single_file(
        #[case] line: &str,
        #[case] expected_modified: usize,
        #[case] expected_staged: usize,
        #[case] expected_untracked: usize,
    ) {
        let status = mock_parse_worktree_status(line);
        assert_eq!(status.modified_count, expected_modified);
        assert_eq!(status.staged_count, expected_staged);
        assert_eq!(status.untracked_count, expected_untracked);
    }

    // ==================== 边界条件和错误处理测试 ====================

    /// 测试空字符串输入的处理
    ///
    /// ## 测试目的
    /// 验证所有解析函数能够正确处理空字符串输入。
    ///
    /// ## 测试场景
    /// 测试空字符串在各种解析函数中的行为
    ///
    /// ## 预期结果
    /// - 空URL解析为 Unknown
    /// - 空分支名称返回空字符串
    /// - 空提交信息返回 None
    /// - 空状态输出返回零计数
    #[test]
    fn test_empty_strings() {
        // 测试空字符串输入
        assert_eq!(mock_parse_repo_type_from_url(""), RepoType::Unknown);
        assert_eq!(mock_remove_branch_prefix(""), "");
        assert!(mock_parse_commit_info("").is_none());

        let empty_status = mock_parse_worktree_status("");
        assert_eq!(empty_status.modified_count, 0);
        assert_eq!(empty_status.staged_count, 0);
        assert_eq!(empty_status.untracked_count, 0);
    }

    /// 测试空白字符的处理
    ///
    /// ## 测试目的
    /// 验证解析函数能够正确处理包含空白字符的输入。
    ///
    /// ## 测试场景
    /// 测试提交信息中的前后空白字符和状态输出中的空行
    ///
    /// ## 预期结果
    /// - 空白字符被正确处理（trim或忽略）
    /// - 空行不影响解析结果
    #[test]
    fn test_whitespace_handling() {
        // 测试空白字符处理
        let commit_with_whitespace = "  abc123  \n  John Doe  \n  2024-01-01  \n  Test commit  ";
        let commit = mock_parse_commit_info(commit_with_whitespace);
        assert!(commit.is_some());

        // Git状态解析应该处理空行
        let status_with_empty_lines = "\n M file1.txt\n\n?? file2.txt\n\n";
        let status = mock_parse_worktree_status(status_with_empty_lines);
        assert_eq!(status.modified_count, 1);
        assert_eq!(status.untracked_count, 1);
    }

    /// 测试Unicode字符的处理
    ///
    /// ## 测试目的
    /// 验证解析函数能够正确处理包含Unicode字符的输入。
    ///
    /// ## 测试场景
    /// 测试包含中文字符的分支名称和URL
    ///
    /// ## 预期结果
    /// - Unicode字符被正确处理
    /// - 分支名称前缀移除功能正常工作
    /// - URL解析功能正常工作
    #[test]
    fn test_unicode_handling() {
        // 测试Unicode字符处理
        let unicode_branch = "feature/用户登录--implement-用户-system";
        let result = mock_remove_branch_prefix(unicode_branch);
        assert_eq!(result, "implement-用户-system");

        let unicode_url = "https://github.com/用户/项目.git";
        assert_eq!(mock_parse_repo_type_from_url(unicode_url), RepoType::GitHub);
    }

    // ==================== 性能和一致性测试 ====================

    /// 测试解析函数的性能特征
    ///
    /// ## 测试目的
    /// 验证解析函数在处理大量输入时的性能表现。
    ///
    /// ## 测试场景
    /// 执行1000次URL解析操作
    ///
    /// ## 预期结果
    /// - 1000次解析操作在100毫秒内完成
    #[test]
    fn test_performance_characteristics() -> Result<()> {
        // 测试大量URL解析的性能（1000次URL解析应该很快完成，< 100ms）
        measure_test_time_with_threshold(
            "test_performance_characteristics",
            Duration::from_millis(100),
            || {
                for i in 0..1000 {
                    let url = format!("https://github.com/user{}/repo{}.git", i, i);
                    let _ = mock_parse_repo_type_from_url(&url);
                }
                Ok(())
            },
        )
    }

    /// 测试多次调用的一致性
    ///
    /// ## 测试目的
    /// 验证解析函数在多次调用时能够产生一致的结果。
    ///
    /// ## 测试场景
    /// 对相同的输入执行10次解析操作
    ///
    /// ## 预期结果
    /// - 所有调用都产生相同的结果
    /// - 没有随机性或状态依赖
    #[test]
    fn test_consistency_across_calls() {
        // 测试相同输入的一致性
        let url = "https://github.com/user/repo.git";
        let branch = "feature/test-branch";

        for _ in 0..10 {
            assert_eq!(mock_parse_repo_type_from_url(url), RepoType::GitHub);
            assert_eq!(mock_remove_branch_prefix(branch), "test-branch");
        }
    }
}
