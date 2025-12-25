//! PR 表格显示模块测试
//!
//! 测试 PullRequestRow 结构体的创建、字段访问和表格显示功能。

use pretty_assertions::assert_eq;
use workflow::pr::table::PullRequestRow;

// ==================== PullRequestRow 结构体创建测试 ====================

/// 测试创建PullRequestRow结构体
#[test]
fn test_pull_request_row_creation() {
    // Arrange: 准备测试创建 PullRequestRow 结构体
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/new-feature".to_string(),
        title: "Add new feature".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(row.number, "123");
    assert_eq!(row.state, "open");
    assert_eq!(row.branch, "feature/new-feature");
    assert_eq!(row.title, "Add new feature");
    assert_eq!(row.author, "alice");
    assert_eq!(row.url, "https://github.com/owner/repo/pull/123");
}

/// 测试使用空字符串创建PullRequestRow
#[test]
fn test_pull_request_row_with_empty_fields() {
    // Arrange: 准备测试使用空字符串创建 PullRequestRow
    let row = PullRequestRow {
        number: String::new(),
        state: String::new(),
        branch: String::new(),
        title: String::new(),
        author: String::new(),
        url: String::new(),
    };

    assert_eq!(row.number, "");
    assert_eq!(row.state, "");
    assert_eq!(row.branch, "");
    assert_eq!(row.title, "");
    assert_eq!(row.author, "");
    assert_eq!(row.url, "");
}

/// 测试使用长字符串创建PullRequestRow
#[test]
fn test_pull_request_row_with_long_strings() {
    // Arrange: 准备测试使用长字符串创建 PullRequestRow
    let long_title = "A".repeat(200);
    let long_branch = "feature/".to_string() + &"x".repeat(100);
    let long_url = "https://github.com/".to_string() + &"owner/".repeat(20) + "repo/pull/123";

    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: long_branch.clone(),
        title: long_title.clone(),
        author: "alice".to_string(),
        url: long_url.clone(),
    };

    assert_eq!(row.title, long_title);
    assert_eq!(row.branch, long_branch);
    assert_eq!(row.url, long_url);
}

/// 测试使用特殊字符创建PullRequestRow
#[test]
fn test_pull_request_row_with_special_characters() {
    // Arrange: 准备测试使用特殊字符创建 PullRequestRow
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/test-123_abc".to_string(),
        title: "Fix: Bug #123 & Issue #456".to_string(),
        author: "user@example.com".to_string(),
        url: "https://github.com/owner/repo/pull/123?query=test&param=value".to_string(),
    };

    assert_eq!(row.branch, "feature/test-123_abc");
    assert_eq!(row.title, "Fix: Bug #123 & Issue #456");
    assert_eq!(row.author, "user@example.com");
    assert_eq!(
        row.url,
        "https://github.com/owner/repo/pull/123?query=test&param=value"
    );
}

/// 测试使用Unicode字符创建PullRequestRow
#[test]
fn test_pull_request_row_with_unicode() {
    // Arrange: 准备测试使用 Unicode 字符创建 PullRequestRow
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/测试功能".to_string(),
        title: "添加新功能：支持中文标题".to_string(),
        author: "张三".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(row.branch, "feature/测试功能");
    assert_eq!(row.title, "添加新功能：支持中文标题");
    assert_eq!(row.author, "张三");
}

// ==================== PullRequestRow 字段访问测试 ====================

/// 测试PullRequestRow的字段访问（读取和修改）
#[test]
fn test_pull_request_row_field_access() {
    // Arrange: 准备测试字段访问
    let mut row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/test".to_string(),
        title: "Test PR".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    // Arrange: 准备测试读取字段
    assert_eq!(row.number, "123");
    assert_eq!(row.state, "open");

    // Arrange: 准备测试修改字段
    row.state = "closed".to_string();
    row.title = "Updated PR".to_string();

    assert_eq!(row.state, "closed");
    assert_eq!(row.title, "Updated PR");
}

// ==================== PullRequestRow 不同状态测试 ====================

/// 测试open状态的PR
#[test]
fn test_pull_request_row_open_state() {
    // Arrange: 准备测试 open 状态的 PR
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/new".to_string(),
        title: "New feature".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(row.state, "open");
}

/// 测试closed状态的PR
#[test]
fn test_pull_request_row_closed_state() {
    // Arrange: 准备测试 closed 状态的 PR
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "closed".to_string(),
        branch: "feature/new".to_string(),
        title: "New feature".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(row.state, "closed");
}

/// 测试merged状态的PR
#[test]
fn test_pull_request_row_merged_state() {
    // Arrange: 准备测试 merged 状态的 PR
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "merged".to_string(),
        branch: "feature/new".to_string(),
        title: "New feature".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert_eq!(row.state, "merged");
}

// ==================== PullRequestRow 集合操作测试 ====================

/// 测试创建PullRequestRow向量
#[test]
fn test_pull_request_row_vector() {
    // Arrange: 准备测试创建 PullRequestRow 向量
    let rows = vec![
        PullRequestRow {
            number: "123".to_string(),
            state: "open".to_string(),
            branch: "feature/one".to_string(),
            title: "PR One".to_string(),
            author: "alice".to_string(),
            url: "https://github.com/owner/repo/pull/123".to_string(),
        },
        PullRequestRow {
            number: "456".to_string(),
            state: "closed".to_string(),
            branch: "feature/two".to_string(),
            title: "PR Two".to_string(),
            author: "bob".to_string(),
            url: "https://github.com/owner/repo/pull/456".to_string(),
        },
        PullRequestRow {
            number: "789".to_string(),
            state: "merged".to_string(),
            branch: "feature/three".to_string(),
            title: "PR Three".to_string(),
            author: "charlie".to_string(),
            url: "https://github.com/owner/repo/pull/789".to_string(),
        },
    ];

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].number, "123");
    assert_eq!(rows[1].number, "456");
    assert_eq!(rows[2].number, "789");
}

/// 测试空的PullRequestRow向量
#[test]
fn test_pull_request_row_empty_vector() {
    // Arrange: 准备测试空的 PullRequestRow 向量
    let rows: Vec<PullRequestRow> = vec![];

    assert_eq!(rows.len(), 0);
}

// ==================== PullRequestRow 边界条件测试 ====================

/// 测试包含空白字符的字段
#[test]
fn test_pull_request_row_with_whitespace() {
    // Arrange: 准备测试包含空白字符的字段
    let row = PullRequestRow {
        number: "  123  ".to_string(),
        state: "  open  ".to_string(),
        branch: "  feature/test  ".to_string(),
        title: "  Test PR  ".to_string(),
        author: "  alice  ".to_string(),
        url: "  https://github.com/owner/repo/pull/123  ".to_string(),
    };

    // Assert: 验证字段包含空白字符
    assert!(row.number.starts_with(' '));
    assert!(row.number.ends_with(' '));
    assert!(row.title.contains("Test PR"));
}

/// 测试包含换行符的字段
#[test]
fn test_pull_request_row_with_newlines() {
    // Arrange: 准备测试包含换行符的字段（虽然在实际使用中可能不常见）
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/test".to_string(),
        title: "Line 1\nLine 2".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    assert!(row.title.contains('\n'));
}

// ==================== PullRequestRow 实际使用场景测试 ====================

/// 测试模拟从GitHub PR创建PullRequestRow
#[test]
fn test_pull_request_row_from_github_pr() {
    // Arrange: 准备测试模拟从 GitHub PR 创建 PullRequestRow
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/PROJ-123-add-feature".to_string(),
        title: "Add new feature for PROJ-123".to_string(),
        author: "github-user".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    // Assert: 验证字段符合预期格式
    assert!(row.number.parse::<u32>().is_ok());
    assert!(row.url.starts_with(workflow::git::github::BASE));
    assert!(row.url.contains("/pull/"));
}

// ==================== PullRequestRow 结构体特性测试 ====================

/// 测试手动克隆PullRequestRow
#[test]
fn test_pull_request_row_manual_clone() {
    // Arrange: 准备测试手动克隆 PullRequestRow（因为结构体没有实现 Clone trait）
    let row1 = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/test".to_string(),
        title: "Test PR".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    let row2 = PullRequestRow {
        number: row1.number.clone(),
        state: row1.state.clone(),
        branch: row1.branch.clone(),
        title: row1.title.clone(),
        author: row1.author.clone(),
        url: row1.url.clone(),
    };

    assert_eq!(row1.number, row2.number);
    assert_eq!(row1.state, row2.state);
    assert_eq!(row1.branch, row2.branch);
    assert_eq!(row1.title, row2.title);
    assert_eq!(row1.author, row2.author);
    assert_eq!(row1.url, row2.url);
}

/// 测试创建后访问字段
#[test]
fn test_pull_request_row_field_access_after_creation() {
    // Arrange: 准备测试创建后访问字段
    let row = PullRequestRow {
        number: "123".to_string(),
        state: "open".to_string(),
        branch: "feature/test".to_string(),
        title: "Test PR".to_string(),
        author: "alice".to_string(),
        url: "https://github.com/owner/repo/pull/123".to_string(),
    };

    // Assert: 验证所有字段都可以访问
    assert_eq!(row.number, "123");
    assert_eq!(row.state, "open");
    assert_eq!(row.branch, "feature/test");
    assert_eq!(row.title, "Test PR");
    assert_eq!(row.author, "alice");
    assert_eq!(row.url, "https://github.com/owner/repo/pull/123");
}
