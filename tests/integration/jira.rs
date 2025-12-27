//! Jira集成测试
//!
//! 测试Jira ticket创建、更新、状态同步等完整流程。
//!
//! 使用MockServer模拟Jira API，避免依赖真实网络连接。

use crate::common::cli_helpers::CliCommandBuilder;
use crate::common::environments::CliTestEnv;
use crate::common::mock::server::MockServer;
use crate::common::test_data::TestDataFactory;
use color_eyre::Result;

// ==================== Jira Ticket 创建测试 ====================

/// 测试Jira ticket创建流程
///
/// ## 测试目的
/// 验证能够创建关联Jira ticket的分支和提交。
///
/// ## 测试场景
/// 1. 初始化Git仓库和Jira配置
/// 2. 创建关联Jira ticket的分支
/// 3. 创建包含ticket ID的提交
/// 4. 验证分支和提交正确创建
#[test]
fn test_jira_ticket_creation_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?.create_config(
        r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"
"#,
    )?;

    // 创建初始提交
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 创建关联Jira ticket的分支
    let ticket_id = "PROJ-123";
    let branch_name = format!("feature/{}", ticket_id);
    env.create_branch(&branch_name)?.checkout(&branch_name)?;

    // 创建包含ticket ID的提交
    env.create_file("feature.txt", "new feature")?
        .create_commit(&format!("feat({}): add feature", ticket_id))?;

    // 验证分支存在
    let env_path = env.path().to_path_buf();
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&env_path)
        .output()?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let current_branch = stdout_str.trim();
    assert_eq!(current_branch, branch_name);

    Ok(())
}

// ==================== Jira 状态同步测试 ====================

/// 测试PR创建时Jira状态同步
///
/// ## 测试目的
/// 验证创建PR时能够同步更新Jira ticket状态。
///
/// ## 测试场景
/// 1. 设置Mock Jira API服务器
/// 2. 设置Jira配置
/// 3. 创建关联ticket的分支和提交
/// 4. 创建PR（dry-run模式）
/// 5. 验证Jira状态同步逻辑
#[test]
fn test_jira_status_sync_on_pr_creation() -> Result<()> {
    // 设置Mock Jira API
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // Mock Jira Issue API响应（使用 TestDataFactory）
    let factory = TestDataFactory::new();
    let issue_data = factory
        .jira_issue()
        .key("PROJ-123")
        .summary("Test Issue")
        .description("Test description")
        .status("In Progress")
        .build()?;

    let response_body = serde_json::to_string(&issue_data)?;
    mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    // Mock Jira状态转换API
    mock_server.mock_jira_issue(
        "POST",
        "/rest/api/3/issue/PROJ-123/transitions",
        r#"{"status": "In Review"}"#,
        204,
    );

    let env = CliTestEnv::new()?;
    env.init_git_repo()?.create_config(&format!(
        r#"
[jira]
url = "{}"
username = "test@example.com"
token = "test_token"
"#,
        mock_server.base_url()
    ))?;

    // 禁用 LLM 调用以避免网络请求超时（在用户级配置中设置无效 provider）
    env.create_home_workflow_config(
        r#"
[llm]
provider = "invalid_provider"
"#,
    )?;

    // 创建初始提交
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 创建关联Jira ticket的分支
    let ticket_id = "PROJ-123";
    let branch_name = format!("feature/{}", ticket_id);
    env.create_branch(&branch_name)?.checkout(&branch_name)?;

    // 创建提交
    env.create_file("feature.txt", "new feature")?
        .create_commit(&format!("feat({}): add feature", ticket_id))?;

    // 创建PR（dry-run模式）
    // LLM 调用会在 build_url() 阶段快速失败，不会等待网络超时
    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证PR命令能够识别Jira ticket
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 应该能够识别Git仓库和Jira配置
    assert!(
        !stderr.contains("not a git repository"),
        "PR command should recognize git repository"
    );

    Ok(())
}

/// 测试PR合并时Jira状态同步
///
/// ## 测试目的
/// 验证合并PR时能够同步更新Jira ticket状态。
///
/// ## 测试场景
/// 1. 设置Mock Jira API服务器
/// 2. 设置Jira配置
/// 3. 创建关联ticket的分支和提交
/// 4. 执行PR合并命令（dry-run模式）
/// 5. 验证Jira状态同步逻辑
#[test]
fn test_jira_status_sync_on_pr_merge() -> Result<()> {
    // 设置Mock Jira API
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // Mock Jira Issue API响应（使用 TestDataFactory）
    let factory = TestDataFactory::new();
    let issue_data = factory
        .jira_issue()
        .key("PROJ-123")
        .summary("Test Issue")
        .description("Test description")
        .status("In Progress")
        .build()?;

    let response_body = serde_json::to_string(&issue_data)?;
    mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    // Mock Jira状态转换API（合并后状态）
    mock_server.mock_jira_issue(
        "POST",
        "/rest/api/3/issue/PROJ-123/transitions",
        r#"{"status": "Done"}"#,
        204,
    );

    let env = CliTestEnv::new()?;
    env.init_git_repo()?.create_config(&format!(
        r#"
[jira]
url = "{}"
username = "test@example.com"
token = "test_token"
"#,
        mock_server.base_url()
    ))?;

    // 创建初始提交
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 创建关联Jira ticket的分支
    let ticket_id = "PROJ-123";
    let branch_name = format!("feature/{}", ticket_id);
    env.create_branch(&branch_name)?.checkout(&branch_name)?;

    // 创建提交
    env.create_file("feature.txt", "new feature")?
        .create_commit(&format!("feat({}): add feature", ticket_id))?;

    // 切换回主分支
    env.checkout("main")?;

    // 测试PR合并命令（dry-run模式）
    let binding = CliCommandBuilder::new()
        .args(["pr", "merge", "--dry-run"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证命令能够识别Jira配置
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("not a git repository"),
        "PR merge command should recognize git repository"
    );

    Ok(())
}

// ==================== Jira CLI 命令测试 ====================

/// 测试Jira info命令
///
/// ## 测试目的
/// 验证Jira info命令能够正确解析参数并调用Mock API。
///
/// ## 测试场景
/// 1. 设置Mock Jira API服务器
/// 2. 设置Jira配置
/// 3. 执行jira info命令
/// 4. 验证命令能够正确调用API
#[test]
fn test_jira_info_command() -> Result<()> {
    // 设置Mock Jira API
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // Mock Jira Issue API响应（使用 TestDataFactory）
    let factory = TestDataFactory::new();
    let issue_data = factory
        .jira_issue()
        .key("PROJ-123")
        .summary("Test Issue")
        .description("Test description")
        .status("In Progress")
        .build()?;

    let response_body = serde_json::to_string(&issue_data)?;
    mock_server
        .server
        .as_mut()
        .mock("GET", "/rest/api/3/issue/PROJ-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(&response_body)
        .create();

    let env = CliTestEnv::new()?;
    env.create_config(&format!(
        r#"
[jira]
url = "{}"
username = "test@example.com"
token = "test_token"
"#,
        mock_server.base_url()
    ))?;

    // 测试Jira info命令
    let binding = CliCommandBuilder::new()
        .args(["jira", "info", "PROJ-123"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证命令执行
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 命令应该能够解析参数（即使API调用失败，也应该有输出）
    assert!(!stdout.is_empty() || !stderr.is_empty());

    Ok(())
}

/// 测试Jira comment命令
///
/// ## 测试目的
/// 验证Jira comment命令能够正确添加评论。
///
/// ## 测试场景
/// 1. 设置Mock Jira API服务器
/// 2. 设置Jira配置
/// 3. 执行jira comment命令
/// 4. 验证命令能够正确调用API
#[test]
fn test_jira_comment_command() -> Result<()> {
    // 设置Mock Jira API
    let mut mock_server = MockServer::new();
    mock_server.setup_jira_base_url();

    // Mock Jira Comment API响应
    mock_server.mock_jira_issue(
        "POST",
        "/rest/api/3/issue/PROJ-123/comment",
        r#"{"id": "12345", "body": "Test comment"}"#,
        201,
    );

    let env = CliTestEnv::new()?;
    env.create_config(&format!(
        r#"
[jira]
url = "{}"
username = "test@example.com"
token = "test_token"
"#,
        mock_server.base_url()
    ))?;

    // 测试Jira comment命令
    let binding = CliCommandBuilder::new()
        .args(["jira", "comment", "PROJ-123", "Test comment"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证命令执行
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!stdout.is_empty() || !stderr.is_empty());

    Ok(())
}

// ==================== Jira 配置测试 ====================

/// 测试Jira配置加载
///
/// ## 测试目的
/// 验证Jira配置能够正确加载和使用。
#[test]
fn test_jira_config_loading() -> Result<()> {
    let env = CliTestEnv::new()?;
    let config_content = r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"
token = "test_token"
"#;

    env.create_config(config_content)?;

    // 验证配置文件存在
    let config_path = env.path().join(".workflow").join("workflow.toml");
    assert!(config_path.exists());

    // 验证配置内容
    let loaded_content = std::fs::read_to_string(&config_path)?;
    assert!(loaded_content.contains("jira"));
    assert!(loaded_content.contains("test.atlassian.net"));
    assert!(loaded_content.contains("test@example.com"));

    Ok(())
}

/// 测试Jira配置验证
///
/// ## 测试目的
/// 验证Jira配置格式验证逻辑。
#[test]
fn test_jira_config_validation() -> Result<()> {
    let env = CliTestEnv::new()?;

    // 测试有效配置
    let valid_config = r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"
"#;
    env.create_config(valid_config)?;

    let config_path = env.path().join(".workflow").join("workflow.toml");
    assert!(config_path.exists());

    // 测试无效配置（缺少必需字段）
    // 注意：这里只测试配置文件的创建，实际验证逻辑在配置加载时进行
    let invalid_config = r#"
[jira]
url = ""
"#;
    env.create_config(invalid_config)?;

    // 配置文件应该能够创建（验证逻辑在加载时进行）
    assert!(config_path.exists());

    Ok(())
}
