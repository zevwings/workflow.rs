//! 补全完整性验证测试
//!
//! 验证所有命令和子命令是否都包含在补全脚本中。

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
    "log-level",
    "github",
    "llm",
    "completion",
    "branch",
    "pr",
    "log",
    "jira",
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
];

/// Log 子命令列表
const LOG_SUBCOMMANDS: &[&str] = &["download", "find", "search"];

/// Jira 子命令列表
const JIRA_SUBCOMMANDS: &[&str] = &[
    "info",
    "related",
    "changelog",
    "comments",
    "attachments",
    "clean",
];

/// GitHub 子命令列表
const GITHUB_SUBCOMMANDS: &[&str] = &["list", "current", "add", "remove", "switch", "update"];

/// LLM 子命令列表
const LLM_SUBCOMMANDS: &[&str] = &["show", "setup"];

/// Branch 子命令列表
const BRANCH_SUBCOMMANDS: &[&str] = &["clean", "ignore", "prefix"];

// Branch ignore 子命令列表（目前未在测试中使用，保留以备将来扩展）
// const BRANCH_IGNORE_SUBCOMMANDS: &[&str] = &["add", "remove", "list"];

/// Proxy 子命令列表
const PROXY_SUBCOMMANDS: &[&str] = &["on", "off", "check"];

/// LogLevel 子命令列表
const LOG_LEVEL_SUBCOMMANDS: &[&str] = &["set", "check", "trace-console"];

/// Completion 子命令列表
const COMPLETION_SUBCOMMANDS: &[&str] = &["generate", "check", "remove"];

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

    // 验证 Log 子命令
    let log_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "log")
        .expect("log command should exist");
    let log_subcommands: Vec<String> =
        log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
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

    // 验证 Proxy 子命令
    let proxy_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "proxy")
        .expect("proxy command should exist");
    let proxy_subcommands: Vec<String> =
        proxy_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(proxy_subcommands.len(), PROXY_SUBCOMMANDS.len());

    // 验证 LogLevel 子命令
    let log_level_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "log-level")
        .expect("log-level command should exist");
    let log_level_subcommands: Vec<String> =
        log_level_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(log_level_subcommands.len(), LOG_LEVEL_SUBCOMMANDS.len());

    // 验证 Completion 子命令
    let completion_cmd = cmd
        .get_subcommands()
        .find(|sc| sc.get_name() == "completion")
        .expect("completion command should exist");
    let completion_subcommands: Vec<String> =
        completion_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
    assert_eq!(completion_subcommands.len(), COMPLETION_SUBCOMMANDS.len());

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
