//! Git/Types 数据结构测试
//!
//! 测试Git模块中各种数据结构的核心业务逻辑，包括：
//! - 仓库类型识别和解析
//! - 提交信息数据结构
//! - 工作区状态解析
//! - 合并策略枚举
//! - 分支名称处理逻辑

use rstest::rstest;

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

    #[test]
    fn test_repo_type_equality() {
        assert_eq!(RepoType::GitHub, RepoType::GitHub);
        assert_eq!(RepoType::Codeup, RepoType::Codeup);
        assert_eq!(RepoType::Unknown, RepoType::Unknown);

        assert_ne!(RepoType::GitHub, RepoType::Codeup);
        assert_ne!(RepoType::GitHub, RepoType::Unknown);
        assert_ne!(RepoType::Codeup, RepoType::Unknown);
    }

    #[test]
    fn test_repo_type_debug() {
        // 测试Debug trait的实现
        let github = RepoType::GitHub;
        let debug_str = format!("{:?}", github);
        assert_eq!(debug_str, "GitHub");

        let codeup = RepoType::Codeup;
        let debug_str = format!("{:?}", codeup);
        assert_eq!(debug_str, "Codeup");

        let unknown = RepoType::Unknown;
        let debug_str = format!("{:?}", unknown);
        assert_eq!(debug_str, "Unknown");
    }

    #[test]
    fn test_repo_type_clone() {
        let original = RepoType::GitHub;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        // 验证克隆后的值是独立的
        let another = RepoType::Codeup;
        assert_ne!(cloned, another);
    }

    #[test]
    fn test_repo_type_copy() {
        let original = RepoType::GitHub;
        let copied = original; // Copy trait 允许这样赋值
        assert_eq!(original, copied);

        // 验证原始值仍然可用（Copy语义）
        assert_eq!(original, RepoType::GitHub);
    }

    // ==================== 仓库类型解析测试 ====================

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

    #[test]
    fn test_github_url_variations() {
        let github_urls = vec![
            "https://github.com/user/repo",
            "https://github.com/user/repo.git",
            "git@github.com:user/repo.git",
            "ssh://git@github.com:22/user/repo.git",
            "git@github-work:user/repo.git",
            "git@github123:user/repo.git",
        ];

        for url in github_urls {
            assert_eq!(
                mock_parse_repo_type_from_url(url),
                RepoType::GitHub,
                "Failed for URL: {}",
                url
            );
        }
    }

    #[test]
    fn test_codeup_url_variations() {
        let codeup_urls = vec![
            "https://codeup.aliyun.com/user/repo",
            "https://codeup.aliyun.com/user/repo.git",
            "git@codeup.aliyun.com:user/repo.git",
            "ssh://git@codeup.aliyun.com/user/repo.git",
        ];

        for url in codeup_urls {
            assert_eq!(
                mock_parse_repo_type_from_url(url),
                RepoType::Codeup,
                "Failed for URL: {}",
                url
            );
        }
    }

    #[test]
    fn test_unknown_repo_types() {
        let unknown_urls = vec![
            "https://gitlab.com/user/repo.git",
            "git@bitbucket.org:user/repo.git",
            "https://git.example.com/repo.git",
            "file:///local/path/to/repo",
            "invalid-url",
            "ftp://example.com/repo",
        ];

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

    #[test]
    fn test_complex_branch_names() {
        // 测试复杂的分支名称处理
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

    #[test]
    fn test_edge_case_branch_names() {
        // 测试边界情况
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

    #[test]
    fn test_merge_strategy_debug() {
        assert_eq!(format!("{:?}", MergeStrategy::Merge), "Merge");
        assert_eq!(format!("{:?}", MergeStrategy::Squash), "Squash");
        assert_eq!(
            format!("{:?}", MergeStrategy::FastForwardOnly),
            "FastForwardOnly"
        );
    }

    #[test]
    fn test_merge_strategy_clone() {
        let original = MergeStrategy::Merge;
        let cloned = original.clone();

        // 由于是枚举，我们通过Debug输出来验证
        assert_eq!(format!("{:?}", original), format!("{:?}", cloned));
    }

    #[test]
    fn test_merge_strategy_copy() {
        let original = MergeStrategy::Squash;
        let copied = original; // Copy trait

        // 验证Copy语义
        assert_eq!(format!("{:?}", original), format!("{:?}", copied));
        assert_eq!(format!("{:?}", original), "Squash");
    }

    // ==================== CommitInfo 结构体测试 ====================

    #[test]
    fn test_commit_info_creation() {
        let commit = CommitInfo {
            sha: "abc123def456".to_string(),
            message: "Initial commit".to_string(),
            author: "John Doe".to_string(),
            date: "2024-01-01".to_string(),
        };

        assert_eq!(commit.sha, "abc123def456");
        assert_eq!(commit.message, "Initial commit");
        assert_eq!(commit.author, "John Doe");
        assert_eq!(commit.date, "2024-01-01");
    }

    #[test]
    fn test_commit_info_clone() {
        let original = CommitInfo {
            sha: "abc123".to_string(),
            message: "Test commit".to_string(),
            author: "Alice".to_string(),
            date: "2024-12-17".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original.sha, cloned.sha);
        assert_eq!(original.message, cloned.message);
        assert_eq!(original.author, cloned.author);
        assert_eq!(original.date, cloned.date);

        // 验证是深拷贝（修改克隆不影响原始）
        // 由于String是堆分配的，这里验证地址不同
        assert_ne!(original.sha.as_ptr(), cloned.sha.as_ptr());
    }

    #[test]
    fn test_commit_info_debug() {
        let commit = CommitInfo {
            sha: "abc123".to_string(),
            message: "Test".to_string(),
            author: "Dev".to_string(),
            date: "2024-01-01".to_string(),
        };

        let debug_str = format!("{:?}", commit);
        assert!(debug_str.contains("abc123"));
        assert!(debug_str.contains("Test"));
        assert!(debug_str.contains("Dev"));
        assert!(debug_str.contains("2024-01-01"));
    }

    #[test]
    fn test_parse_commit_info_valid() {
        let output = "abc123def456\nJohn Doe\n2024-01-01 10:30:00\nInitial commit";
        let commit = mock_parse_commit_info(output).unwrap();

        assert_eq!(commit.sha, "abc123def456");
        assert_eq!(commit.author, "John Doe");
        assert_eq!(commit.date, "2024-01-01 10:30:00");
        assert_eq!(commit.message, "Initial commit");
    }

    #[test]
    fn test_parse_commit_info_invalid() {
        // 测试不完整的输出
        let invalid_outputs = vec![
            "",
            "abc123",
            "abc123\nJohn Doe",
            "abc123\nJohn Doe\n2024-01-01",
        ];

        for output in invalid_outputs {
            assert!(mock_parse_commit_info(output).is_none());
        }
    }

    #[test]
    fn test_parse_commit_info_with_multiline_message() {
        let output = "abc123\nJohn Doe\n2024-01-01\nFirst line of commit message";
        let commit = mock_parse_commit_info(output).unwrap();

        assert_eq!(commit.message, "First line of commit message");
    }

    // ==================== WorktreeStatus 结构体测试 ====================

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

    #[test]
    fn test_parse_worktree_status_empty() {
        let status = mock_parse_worktree_status("");
        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    }

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

    #[test]
    fn test_parse_worktree_status_only_staged() {
        let output = "A  new_file.txt\nM  modified_file.txt\nD  deleted_file.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 3);
        assert_eq!(status.untracked_count, 0);
    }

    #[test]
    fn test_parse_worktree_status_only_modified() {
        let output = " M file1.txt\n M file2.txt\n D file3.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 3);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
    }

    #[test]
    fn test_parse_worktree_status_only_untracked() {
        let output = "?? file1.txt\n?? file2.txt\n?? dir/file3.txt";
        let status = mock_parse_worktree_status(output);

        assert_eq!(status.modified_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 3);
    }

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

    #[test]
    fn test_performance_characteristics() {
        use std::time::Instant;

        let start = Instant::now();

        // 测试大量URL解析的性能
        for i in 0..1000 {
            let url = format!("https://github.com/user{}/repo{}.git", i, i);
            let _ = mock_parse_repo_type_from_url(&url);
        }

        let duration = start.elapsed();
        // 1000次URL解析应该很快完成
        assert!(duration.as_millis() < 100);
    }

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
