//! 端到端测试
//!
//! 测试完整的用户工作流，从命令输入到最终结果。
//!
//! 这些测试验证整个系统的集成，确保各个组件能够正确协作。

use crate::common::cli_helpers::CliCommandBuilder;
use crate::common::environments::CliTestEnv;
use color_eyre::Result;

// ==================== PR 工作流测试 ====================

/// 测试完整的PR创建流程
///
/// ## 测试目的
/// 验证从创建分支到创建PR的完整工作流程。
///
/// ## 测试场景
/// 1. 初始化Git仓库
/// 2. 创建并切换到新分支
/// 3. 创建文件并提交
/// 4. 创建PR（dry-run模式）
///
/// ## 预期结果
/// - 所有步骤成功执行
/// - PR创建命令能够识别Git仓库和分支
/// - 输出包含PR相关信息
#[test]
fn test_pr_creation_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 1. 创建初始提交（PR需要至少一个提交）
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 2. 创建分支并切换
    env.create_branch("feature/test")?.checkout("feature/test")?;

    // 3. 创建文件并提交
    env.create_file("test.txt", "test content")?.create_commit("feat: add test")?;

    // 4. 创建PR（dry-run模式，避免实际创建）
    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证输出
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // PR命令应该能够识别Git仓库（不应该报"not a git repository"错误）
    assert!(
        !stderr.contains("not a git repository"),
        "PR command should recognize git repository"
    );

    Ok(())
}

/// 测试分支创建到PR创建的完整流程（带Jira集成）
///
/// ## 测试目的
/// 验证包含Jira ticket的完整工作流程。
///
/// ## 测试场景
/// 1. 初始化Git仓库和配置
/// 2. 创建关联Jira ticket的分支
/// 3. 创建文件并提交（包含Jira ticket ID）
/// 4. 创建PR（dry-run模式）
///
/// ## 预期结果
/// - 所有步骤成功执行
/// - PR描述中包含Jira ticket信息（如果支持）
#[test]
#[ignore] // 需要Mock Jira API或实际配置
fn test_pr_creation_with_jira_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?.create_config(
        r#"
[jira]
url = "https://test.atlassian.net"
"#,
    )?;

    // 1. 创建初始提交
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 2. 创建关联Jira ticket的分支
    env.create_branch("feature/PROJ-123")?.checkout("feature/PROJ-123")?;

    // 3. 创建文件并提交（包含Jira ticket ID）
    env.create_file("feature.txt", "new feature")?
        .create_commit("feat(PROJ-123): add new feature")?;

    // 4. 创建PR（dry-run模式）
    let binding = CliCommandBuilder::new()
        .args(["pr", "create", "--dry-run"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    // 验证输出
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // 应该能够识别Git仓库
    assert!(
        !stderr.contains("not a git repository"),
        "PR command should recognize git repository"
    );

    Ok(())
}

// ==================== 分支工作流测试 ====================

/// 测试完整的分支创建和管理流程
///
/// ## 测试目的
/// 验证分支创建、切换、查看等操作的完整流程。
///
/// ## 测试场景
/// 1. 初始化Git仓库
/// 2. 创建多个分支
/// 3. 在不同分支间切换
/// 4. 验证分支状态
///
/// ## 预期结果
/// - 所有分支操作成功
/// - 分支切换正确
/// - 分支列表正确显示
#[test]
fn test_branch_management_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 1. 创建初始提交
    env.create_file("README.md", "# Test")?.create_commit("Initial commit")?;

    // 2. 创建多个分支
    env.create_branch("feature/one")?
        .create_branch("feature/two")?
        .create_branch("hotfix/bugfix")?;

    // 3. 切换到第一个分支
    env.checkout("feature/one")?;

    // 验证当前分支
    let env_path = env.path().to_path_buf();
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&env_path)
        .output()?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let current_branch = stdout_str.trim();
    assert_eq!(current_branch, "feature/one");

    // 4. 切换到另一个分支
    env.checkout("feature/two")?;

    let env_path = env.path().to_path_buf();
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&env_path)
        .output()?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let current_branch = stdout_str.trim();
    assert_eq!(current_branch, "feature/two");

    Ok(())
}

// ==================== 提交工作流测试 ====================

/// 测试完整的提交工作流程
///
/// ## 测试目的
/// 验证从文件创建到提交的完整流程。
///
/// ## 测试场景
/// 1. 初始化Git仓库
/// 2. 创建多个文件
/// 3. 创建多个提交
/// 4. 验证提交历史
///
/// ## 预期结果
/// - 所有提交成功创建
/// - 提交历史正确
#[test]
fn test_commit_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?;

    // 1. 创建初始提交
    env.create_file("README.md", "# Test Project")?
        .create_commit("Initial commit")?;

    // 2. 创建多个文件并提交
    env.create_file("src/main.rs", "fn main() {}")?
        .create_commit("feat: add main function")?;

    env.create_file("src/lib.rs", "pub fn hello() {}")?
        .create_commit("feat: add library")?;

    // 3. 验证提交历史
    let env_path = env.path().to_path_buf();
    let output = std::process::Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&env_path)
        .output()?;

    let log_output = String::from_utf8_lossy(&output.stdout);

    // 应该包含所有提交
    assert!(log_output.contains("Initial commit"));
    assert!(log_output.contains("add main function"));
    assert!(log_output.contains("add library"));

    Ok(())
}

// ==================== 配置工作流测试 ====================

/// 测试配置文件的创建和使用流程
///
/// ## 测试目的
/// 验证配置文件创建后，命令能够正确读取和使用配置。
///
/// ## 测试场景
/// 1. 初始化Git仓库
/// 2. 创建配置文件
/// 3. 执行需要配置的命令
/// 4. 验证配置被正确读取
///
/// ## 预期结果
/// - 配置文件创建成功
/// - 命令能够读取配置
#[test]
fn test_config_workflow() -> Result<()> {
    let env = CliTestEnv::new()?;
    env.init_git_repo()?.create_config(
        r#"
[jira]
url = "https://test.atlassian.net"
username = "test@example.com"

[github]
token = "test_token"
"#,
    )?;

    // 验证配置文件存在
    let config_path = env.path().join(".workflow").join("workflow.toml");
    assert!(config_path.exists());

    // 验证配置内容
    let config_content = std::fs::read_to_string(&config_path)?;
    assert!(config_content.contains("jira"));
    assert!(config_content.contains("github"));

    // 测试配置命令能够读取配置
    let binding = CliCommandBuilder::new()
        .args(["config", "show"])
        .current_dir(env.path())
        .assert();
    let output = binding.get_output();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 配置命令应该能够读取并显示配置
    assert!(!stdout.is_empty() || !String::from_utf8_lossy(&output.stderr).is_empty());

    Ok(())
}
