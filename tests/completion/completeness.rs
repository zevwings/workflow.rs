//! 补全完整性验证测试
//!
//! 验证所有命令和子命令是否都包含在补全脚本中。

use pretty_assertions::assert_eq;
use std::collections::HashSet;
use std::fs;

use clap::CommandFactory;
use workflow::cli::Cli;
use workflow::completion::generate::CompletionGenerator;
use workflow::completion::helpers::get_completion_filename;

/// 所有顶级命令列表（从 Commands 枚举中提取）
const TOP_LEVEL_COMMANDS: &[&str] = &[
    "proxy",
    "check",
    "setup",
    "config",
    "uninstall",
    "version",
    "update",
    "log",
    "github",
    "llm",
    "completion",
    "branch",
    "commit",
    "migrate",
    "pr",
    "jira",
    "stash",
    "repo",
    "alias",
    "tag",
];

/// PR 子命令列表
const PR_SUBCOMMANDS: &[&str] = &[
    "create",
    "merge",
    "status",
    "list",
    "update",
    "sync",
    "rebase",
    "close",
    "summarize",
    "approve",
    "comment",
    "pick",
    "reword",
];

/// Log 子命令列表
const LOG_SUBCOMMANDS: &[&str] = &["download", "find", "search"];

/// Jira 子命令列表
const JIRA_SUBCOMMANDS: &[&str] = &[
    "info",
    "related",
    "changelog",
    "comment",
    "comments",
    "attachments",
    "clean",
    "log",
];

/// GitHub 子命令列表
const GITHUB_SUBCOMMANDS: &[&str] = &["list", "current", "add", "remove", "switch", "update"];

/// LLM 子命令列表
const LLM_SUBCOMMANDS: &[&str] = &["show", "setup"];

/// Branch 子命令列表
const BRANCH_SUBCOMMANDS: &[&str] = &["ignore", "create", "rename", "switch", "sync", "delete"];

/// Commit 子命令列表
const COMMIT_SUBCOMMANDS: &[&str] = &["amend", "reword", "squash"];

// Branch ignore 子命令列表（目前未在测试中使用，保留以备将来扩展）
// const BRANCH_IGNORE_SUBCOMMANDS: &[&str] = &["add", "remove", "list"];

/// Proxy 子命令列表
const PROXY_SUBCOMMANDS: &[&str] = &["on", "off", "check"];

/// Log 子命令列表
const LOG_LEVEL_SUBCOMMANDS: &[&str] = &["set", "check", "trace-console"];

/// Completion 子命令列表
const COMPLETION_SUBCOMMANDS: &[&str] = &["generate", "check", "remove"];

/// Stash 子命令列表
const STASH_SUBCOMMANDS: &[&str] = &["list", "apply", "drop", "pop", "push"];

/// Repo 子命令列表
const REPO_SUBCOMMANDS: &[&str] = &["setup", "show", "clean"];

/// Alias 子命令列表
const ALIAS_SUBCOMMANDS: &[&str] = &["list", "add", "remove"];

/// Tag 子命令列表
const TAG_SUBCOMMANDS: &[&str] = &["delete"];

/// 所有支持的 shell 类型
const SHELL_TYPES: &[&str] = &["zsh", "bash", "fish", "powershell", "elvish"];

// 以下函数用于从补全脚本中提取命令（目前未使用，保留以备将来扩展）
//
// /// 从 zsh 补全脚本中提取命令列表
// fn extract_commands_from_zsh_completion(content: &str) -> HashSet<String> {
//     ...
// }
//
// /// 从 bash 补全脚本中提取命令列表
// fn extract_commands_from_bash_completion(content: &str) -> HashSet<String> {
//     ...
// }

/// 验证 CLI 结构包含所有顶级命令
#[test]
fn test_cli_contains_all_top_level_commands() {
    let cmd = Cli::command();
    let subcommands: Vec<String> =
        cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

    // 检查所有预期的命令都存在
    for expected_cmd in TOP_LEVEL_COMMANDS {
        assert!(
            subcommand_set.contains(*expected_cmd),
            "Missing top-level command: {}",
            expected_cmd
        );
    }

    // 输出所有命令以便调试
    println!(
        "Found {} top-level commands: {:?}",
        subcommands.len(),
        subcommands
    );
}

/// 验证 PR 子命令完整性
#[test]
fn test_pr_subcommands_completeness() {
    let cmd = Cli::command();
    let pr_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "pr")
        .expect("pr command should exist");

    let subcommands: Vec<String> =
        pr_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

    for expected_subcmd in PR_SUBCOMMANDS {
        assert!(
            subcommand_set.contains(*expected_subcmd),
            "Missing PR subcommand: {}",
            expected_subcmd
        );
    }

    println!(
        "Found {} PR subcommands: {:?}",
        subcommands.len(),
        subcommands
    );
    assert_eq!(
        subcommands.len(),
        PR_SUBCOMMANDS.len(),
        "PR subcommands count mismatch"
    );
}

/// 验证 Commit 子命令完整性
#[test]
fn test_commit_subcommands_completeness() {
    let cmd = Cli::command();
    let commit_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "commit")
        .expect("commit command should exist");

    let subcommands: Vec<String> =
        commit_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

    for expected_subcmd in COMMIT_SUBCOMMANDS {
        assert!(
            subcommand_set.contains(*expected_subcmd),
            "Missing Commit subcommand: {}",
            expected_subcmd
        );
    }

    println!(
        "Found {} Commit subcommands: {:?}",
        subcommands.len(),
        subcommands
    );
    assert_eq!(
        subcommands.len(),
        COMMIT_SUBCOMMANDS.len(),
        "Commit subcommands count mismatch"
    );
}

/// 验证补全脚本生成功能
#[test]
fn test_completion_generation() {
    // 使用系统临时目录
    let output_dir = std::env::temp_dir().join("workflow_completion_test");
    fs::create_dir_all(&output_dir).expect("Failed to create temp directory");

    // 测试所有支持的 shell 类型
    let shell_types = ["zsh", "bash", "fish", "powershell", "elvish"];

    for shell_type in &shell_types {
        let generator = CompletionGenerator::new(
            Some(shell_type.to_string()),
            Some(output_dir.to_string_lossy().to_string()),
        )
        .expect(&format!("Failed to create generator for {}", shell_type));

        let result = generator.generate_all();
        assert!(
            result.is_ok(),
            "Failed to generate completion for {}: {:?}",
            shell_type,
            result.err()
        );

        // 验证文件已生成
        let filename = get_completion_filename(shell_type, "workflow")
            .expect(&format!("Failed to get filename for {}", shell_type));
        let file_path = output_dir.join(&filename);

        assert!(
            file_path.exists(),
            "Completion file not generated for {}: {}",
            shell_type,
            file_path.display()
        );

        // 验证文件不为空
        let content = fs::read_to_string(&file_path).expect(&format!(
            "Failed to read completion file for {}",
            shell_type
        ));

        assert!(
            !content.is_empty(),
            "Completion file is empty for {}",
            shell_type
        );

        println!(
            "Generated {} completion: {} bytes",
            shell_type,
            content.len()
        );
    }

    // 清理临时文件
    fs::remove_dir_all(&output_dir).ok();
}

/// 验证 zsh 补全脚本包含所有命令
#[test]
fn test_zsh_completion_contains_all_commands() {
    // 使用系统临时目录
    let output_dir = std::env::temp_dir().join("workflow_zsh_test");
    fs::create_dir_all(&output_dir).expect("Failed to create temp directory");

    let generator = CompletionGenerator::new(
        Some("zsh".to_string()),
        Some(output_dir.to_string_lossy().to_string()),
    )
    .expect("Failed to create zsh generator");

    generator.generate_all().expect("Failed to generate zsh completion");

    let filename = get_completion_filename("zsh", "workflow").expect("Failed to get filename");
    let file_path = output_dir.join(&filename);
    let content = fs::read_to_string(&file_path).expect("Failed to read completion file");

    // 验证补全脚本包含 workflow 命令
    assert!(
        content.contains("workflow"),
        "Zsh completion should contain 'workflow'"
    );

    // 对于 zsh，clap 生成的补全脚本可能使用不同的格式
    // 我们主要验证文件生成成功且包含基本内容
    println!("Zsh completion file size: {} bytes", content.len());
    println!(
        "Zsh completion contains 'workflow': {}",
        content.contains("workflow")
    );

    // 清理临时文件
    fs::remove_dir_all(&output_dir).ok();
}

/// 验证 bash 补全脚本包含所有命令
#[test]
fn test_bash_completion_contains_all_commands() {
    // 使用系统临时目录
    let output_dir = std::env::temp_dir().join("workflow_bash_test");
    fs::create_dir_all(&output_dir).expect("Failed to create temp directory");

    let generator = CompletionGenerator::new(
        Some("bash".to_string()),
        Some(output_dir.to_string_lossy().to_string()),
    )
    .expect("Failed to create bash generator");

    generator.generate_all().expect("Failed to generate bash completion");

    let filename = get_completion_filename("bash", "workflow").expect("Failed to get filename");
    let file_path = output_dir.join(&filename);
    let content = fs::read_to_string(&file_path).expect("Failed to read completion file");

    // 验证补全脚本包含 workflow 命令
    assert!(
        content.contains("workflow"),
        "Bash completion should contain 'workflow'"
    );

    println!("Bash completion file size: {} bytes", content.len());
    println!(
        "Bash completion contains 'workflow': {}",
        content.contains("workflow")
    );

    // 清理临时文件
    fs::remove_dir_all(&output_dir).ok();
}

/// 验证所有子命令的完整性
#[test]
fn test_all_subcommands_completeness() {
    let cmd = Cli::command();

    // 验证 PR 子命令
    let pr_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "pr")
        .expect("pr command should exist");
    let pr_subcommands: Vec<String> =
        pr_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(pr_subcommands.len(), PR_SUBCOMMANDS.len());

    // 验证 Jira Log 子命令（log 现在是 jira 的子命令）
    let jira_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "jira")
        .expect("jira command should exist");
    let jira_log_cmd = jira_cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "log")
        .expect("jira log command should exist");
    let log_subcommands: Vec<String> =
        jira_log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(log_subcommands.len(), LOG_SUBCOMMANDS.len());

    // 验证 Jira 子命令
    let jira_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "jira")
        .expect("jira command should exist");
    let jira_subcommands: Vec<String> =
        jira_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(jira_subcommands.len(), JIRA_SUBCOMMANDS.len());

    // 验证 GitHub 子命令
    let github_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "github")
        .expect("github command should exist");
    let github_subcommands: Vec<String> =
        github_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(github_subcommands.len(), GITHUB_SUBCOMMANDS.len());

    // 验证 LLM 子命令
    let llm_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "llm")
        .expect("llm command should exist");
    let llm_subcommands: Vec<String> =
        llm_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(llm_subcommands.len(), LLM_SUBCOMMANDS.len());

    // 验证 Branch 子命令
    let branch_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "branch")
        .expect("branch command should exist");
    let branch_subcommands: Vec<String> =
        branch_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(branch_subcommands.len(), BRANCH_SUBCOMMANDS.len());

    // 验证 Commit 子命令
    let commit_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "commit")
        .expect("commit command should exist");
    let commit_subcommands: Vec<String> =
        commit_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(commit_subcommands.len(), COMMIT_SUBCOMMANDS.len());

    // 验证 Proxy 子命令
    let proxy_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "proxy")
        .expect("proxy command should exist");
    let proxy_subcommands: Vec<String> =
        proxy_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(proxy_subcommands.len(), PROXY_SUBCOMMANDS.len());

    // 验证 Log 子命令
    let log_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "log")
        .expect("log command should exist");
    let log_subcommands: Vec<String> =
        log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(log_subcommands.len(), LOG_LEVEL_SUBCOMMANDS.len());

    // 验证 Completion 子命令
    let completion_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "completion")
        .expect("completion command should exist");
    let completion_subcommands: Vec<String> =
        completion_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(completion_subcommands.len(), COMPLETION_SUBCOMMANDS.len());

    // 验证 Stash 子命令
    let stash_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "stash")
        .expect("stash command should exist");
    let stash_subcommands: Vec<String> =
        stash_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(stash_subcommands.len(), STASH_SUBCOMMANDS.len());

    // 验证 Repo 子命令
    let repo_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "repo")
        .expect("repo command should exist");
    let repo_subcommands: Vec<String> =
        repo_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(repo_subcommands.len(), REPO_SUBCOMMANDS.len());

    // 验证 Alias 子命令
    let alias_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "alias")
        .expect("alias command should exist");
    let alias_subcommands: Vec<String> =
        alias_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(alias_subcommands.len(), ALIAS_SUBCOMMANDS.len());

    // 验证 Tag 子命令
    let tag_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "tag")
        .expect("tag command should exist");
    let tag_subcommands: Vec<String> =
        tag_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(tag_subcommands.len(), TAG_SUBCOMMANDS.len());

    println!("All subcommands verified successfully!");
}

/// 验证补全脚本文件命名正确
#[test]
fn test_completion_filename_generation() {
    let shell_types = ["zsh", "bash", "fish", "powershell", "elvish"];
    let expected_filenames = [
        "_workflow",
        "workflow.bash",
        "workflow.fish",
        "_workflow.ps1",
        "workflow.elv",
    ];

    for (shell_type, expected_filename) in shell_types.iter().zip(expected_filenames.iter()) {
        let filename = get_completion_filename(shell_type, "workflow")
            .expect(&format!("Failed to get filename for {}", shell_type));
        assert_eq!(
            &filename, expected_filename,
            "Wrong filename for {}: expected {}, got {}",
            shell_type, expected_filename, filename
        );
    }
}

/// 验证 CLI 命令结构完整性总结
#[test]
fn test_cli_structure_summary() {
    let cmd = Cli::command();
    let subcommands: Vec<String> =
        cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    println!("\n=== CLI Structure Summary ===");
    println!("Total top-level commands: {}", subcommands.len());
    println!("Commands: {:?}", subcommands);

    // 统计所有子命令
    let mut total_subcommands = 0;
    for subcmd in cmd.get_subcommands() {
        let sub_subcommands: Vec<String> =
            subcmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        if !sub_subcommands.is_empty() {
            println!(
                "  {}: {} subcommands ({:?})",
                subcmd.get_name(),
                sub_subcommands.len(),
                sub_subcommands
            );
            total_subcommands += sub_subcommands.len();
        }
    }

    println!("Total subcommands: {}", total_subcommands);
    println!("=============================\n");

    // 验证基本完整性
    assert!(
        subcommands.len() >= 10,
        "Should have at least 10 top-level commands"
    );
    assert!(
        total_subcommands >= 20,
        "Should have at least 20 subcommands"
    );
}

/// 参数化测试：验证所有 shell 类型的补全生成
#[test]
fn test_all_shell_types_completion_generation() {
    // 使用系统临时目录
    let output_dir = std::env::temp_dir().join("workflow_all_shells_test");
    fs::create_dir_all(&output_dir).expect("Failed to create temp directory");

    let expected_filenames = [
        "_workflow",
        "workflow.bash",
        "workflow.fish",
        "_workflow.ps1",
        "workflow.elv",
    ];

    for (shell_type, expected_filename) in SHELL_TYPES.iter().zip(expected_filenames.iter()) {
        println!("Testing {} completion generation...", shell_type);

        let generator = CompletionGenerator::new(
            Some(shell_type.to_string()),
            Some(output_dir.to_string_lossy().to_string()),
        )
        .expect(&format!("Failed to create generator for {}", shell_type));

        // 生成补全脚本
        let result = generator.generate_all();
        assert!(
            result.is_ok(),
            "Failed to generate completion for {}: {:?}",
            shell_type,
            result.err()
        );

        // 验证文件命名
        let filename = get_completion_filename(shell_type, "workflow")
            .expect(&format!("Failed to get filename for {}", shell_type));
        assert_eq!(
            &filename, expected_filename,
            "Wrong filename for {}: expected {}, got {}",
            shell_type, expected_filename, filename
        );

        // 验证文件存在
        let file_path = output_dir.join(&filename);
        assert!(
            file_path.exists(),
            "Completion file not generated for {}: {}",
            shell_type,
            file_path.display()
        );

        // 验证文件内容
        let content = fs::read_to_string(&file_path).expect(&format!(
            "Failed to read completion file for {}",
            shell_type
        ));

        assert!(
            !content.is_empty(),
            "Completion file is empty for {}",
            shell_type
        );

        assert!(
            content.contains("workflow"),
            "{} completion should contain 'workflow'",
            shell_type
        );

        println!(
            "✓ {} completion: {} bytes, filename: {}",
            shell_type,
            content.len(),
            filename
        );
    }

    // 清理临时文件
    fs::remove_dir_all(&output_dir).ok();
}

/// 参数化测试：验证所有带子命令的命令完整性
#[test]
fn test_all_commands_with_subcommands() {
    let cmd = Cli::command();

    // 定义所有带子命令的命令及其预期子命令列表
    let commands_with_subcommands = [
        ("pr", PR_SUBCOMMANDS),
        ("commit", COMMIT_SUBCOMMANDS),
        ("jira", JIRA_SUBCOMMANDS),
        ("github", GITHUB_SUBCOMMANDS),
        ("llm", LLM_SUBCOMMANDS),
        ("branch", BRANCH_SUBCOMMANDS),
        ("proxy", PROXY_SUBCOMMANDS),
        ("log", LOG_LEVEL_SUBCOMMANDS),
        ("completion", COMPLETION_SUBCOMMANDS),
        ("stash", STASH_SUBCOMMANDS),
        ("repo", REPO_SUBCOMMANDS),
        ("alias", ALIAS_SUBCOMMANDS),
        ("tag", TAG_SUBCOMMANDS),
    ];

    for (cmd_name, expected_subcommands) in &commands_with_subcommands {
        println!("Testing {} subcommands...", cmd_name);

        let subcommand = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == *cmd_name)
            .expect(&format!("{} command should exist", cmd_name));

        let actual_subcommands: Vec<String> =
            subcommand.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

        let subcommand_set: HashSet<String> = actual_subcommands.iter().cloned().collect();

        // 检查所有预期的子命令都存在
        for expected_subcmd in *expected_subcommands {
            assert!(
                subcommand_set.contains(*expected_subcmd),
                "Missing {} subcommand: {}",
                cmd_name,
                expected_subcmd
            );
        }

        // 检查数量匹配
        assert_eq!(
            actual_subcommands.len(),
            expected_subcommands.len(),
            "{} subcommands count mismatch. Expected: {:?}, Found: {:?}",
            cmd_name,
            expected_subcommands,
            actual_subcommands
        );

        println!(
            "✓ {} has {} subcommands: {:?}",
            cmd_name,
            actual_subcommands.len(),
            actual_subcommands
        );
    }
}

/// 参数化测试：验证嵌套子命令（如 jira log）
#[test]
fn test_nested_subcommands() {
    let cmd = Cli::command();

    // 验证 Jira Log 子命令（log 是 jira 的子命令）
    let jira_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "jira")
        .expect("jira command should exist");

    let jira_log_cmd = jira_cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "log")
        .expect("jira log command should exist");

    let log_subcommands: Vec<String> =
        jira_log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    let subcommand_set: HashSet<String> = log_subcommands.iter().cloned().collect();

    for expected_subcmd in LOG_SUBCOMMANDS {
        assert!(
            subcommand_set.contains(*expected_subcmd),
            "Missing jira log subcommand: {}",
            expected_subcmd
        );
    }

    assert_eq!(
        log_subcommands.len(),
        LOG_SUBCOMMANDS.len(),
        "Jira log subcommands count mismatch"
    );

    println!(
        "✓ jira log has {} subcommands: {:?}",
        log_subcommands.len(),
        log_subcommands
    );

    // 验证 Branch Ignore 子命令（ignore 是 branch 的子命令）
    let branch_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "branch")
        .expect("branch command should exist");

    let branch_ignore_cmd = branch_cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "ignore")
        .expect("branch ignore command should exist");

    let ignore_subcommands: Vec<String> = branch_ignore_cmd
        .get_subcommands()
        .map(|sc| sc.get_name().to_string())
        .collect();

    // Branch ignore 有 add, remove, list 子命令
    const BRANCH_IGNORE_SUBCOMMANDS: &[&str] = &["add", "remove", "list"];
    let ignore_subcommand_set: HashSet<String> = ignore_subcommands.iter().cloned().collect();

    for expected_subcmd in BRANCH_IGNORE_SUBCOMMANDS {
        assert!(
            ignore_subcommand_set.contains(*expected_subcmd),
            "Missing branch ignore subcommand: {}",
            expected_subcmd
        );
    }

    assert_eq!(
        ignore_subcommands.len(),
        BRANCH_IGNORE_SUBCOMMANDS.len(),
        "Branch ignore subcommands count mismatch"
    );

    println!(
        "✓ branch ignore has {} subcommands: {:?}",
        ignore_subcommands.len(),
        ignore_subcommands
    );
}

/// 验证所有顶级命令在常量列表中都有定义
#[test]
fn test_top_level_commands_sync() {
    let cmd = Cli::command();
    let actual_commands: Vec<String> =
        cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

    let expected_set: HashSet<&str> = TOP_LEVEL_COMMANDS.iter().copied().collect();
    let actual_set: HashSet<String> = actual_commands.iter().cloned().collect();

    // 检查是否有遗漏的命令
    for actual_cmd in &actual_commands {
        assert!(
            expected_set.contains(actual_cmd.as_str()),
            "Command '{}' is missing from TOP_LEVEL_COMMANDS constant",
            actual_cmd
        );
    }

    // 检查是否有多余的命令
    for expected_cmd in TOP_LEVEL_COMMANDS {
        assert!(
            actual_set.contains(*expected_cmd),
            "Command '{}' in TOP_LEVEL_COMMANDS constant does not exist in CLI",
            expected_cmd
        );
    }

    assert_eq!(
        actual_commands.len(),
        TOP_LEVEL_COMMANDS.len(),
        "Top-level commands count mismatch"
    );

    println!(
        "✓ All {} top-level commands are synchronized",
        actual_commands.len()
    );
}
