//! CLI 参数检查测试
//!
//! 验证 CLI 参数是否遵循规范：
//! - 是否应该使用已封装的参数（JiraIdArg、OutputFormatArgs、DryRunArgs）
//! - 参数命名是否一致
//! - 是否使用 #[command(flatten)] 复用参数组

use std::fs;
use std::path::Path;

/// 检查是否应该使用 JiraIdArg 但使用了自定义参数
#[test]
fn test_jira_id_arg_usage() {
    let cli_dir = Path::new("src/lib/cli");

    // 读取所有 CLI 文件
    let files = vec![
        "pr.rs",
        "jira.rs",
        "log.rs",
        "branch.rs",
        "commit.rs",
    ];

    let mut issues = Vec::new();

    for file in files {
        let file_path = cli_dir.join(file);
        if !file_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

        // 检查是否有 jira 相关参数但没有使用 JiraIdArg
        if content.contains("jira") && !content.contains("use.*JiraIdArg") {
            // 检查是否定义了 jira 相关参数
            let has_jira_arg = content.contains("#[arg") &&
                (content.contains("jira_id") || content.contains("jira_ticket") ||
                 content.contains("JIRA_ID") || content.contains("JIRA_TICKET"));

            // 检查是否使用了 JiraIdArg
            let uses_jira_id_arg = content.contains("JiraIdArg") ||
                content.contains("use.*args::JiraIdArg") ||
                content.contains("use super::args::JiraIdArg");

            if has_jira_arg && !uses_jira_id_arg {
                // 检查是否是自定义定义（不是使用 JiraIdArg）
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    if (line.contains("jira_id") || line.contains("jira_ticket")) &&
                       line.contains("Option<String>") &&
                       !line.contains("JiraIdArg") {
                        issues.push(format!(
                            "{}:{} - Should use JiraIdArg instead of custom jira parameter: {}",
                            file, i + 1, line.trim()
                        ));
                    }
                }
            }
        }
    }

    if !issues.is_empty() {
        eprintln!("\n⚠️  Found {} issue(s) with JiraIdArg usage:\n", issues.len());
        for issue in &issues {
            eprintln!("  {}", issue);
        }
        eprintln!("\n💡  Fix: Use JiraIdArg from src/lib/cli/args.rs with #[command(flatten)]");
        eprintln!("   Example:");
        eprintln!("     use super::args::JiraIdArg;");
        eprintln!("     #[command(flatten)]");
        eprintln!("     jira_id: JiraIdArg,");
    }

    println!("JiraIdArg usage check completed. Found {} potential issue(s)", issues.len());
}

/// 检查是否应该使用 OutputFormatArgs 但使用了自定义参数
#[test]
fn test_output_format_args_usage() {
    let cli_dir = Path::new("src/lib/cli");

    let files = vec![
        "jira.rs",
        "pr.rs",
        "branch.rs",
    ];

    let mut issues = Vec::new();

    for file in files {
        let file_path = cli_dir.join(file);
        if !file_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

        // 检查是否有输出格式相关参数但没有使用 OutputFormatArgs
        let has_format_args = (content.contains("json") || content.contains("yaml") ||
                               content.contains("table") || content.contains("markdown")) &&
                              content.contains("#[arg");

        let uses_output_format_args = content.contains("OutputFormatArgs") ||
            content.contains("use.*args::OutputFormatArgs") ||
            content.contains("use super::args::OutputFormatArgs");

        if has_format_args && !uses_output_format_args {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if (line.contains("json") || line.contains("yaml") ||
                    line.contains("table") || line.contains("markdown")) &&
                   line.contains("#[arg") &&
                   !line.contains("OutputFormatArgs") {
                    issues.push(format!(
                        "{}:{} - Should use OutputFormatArgs instead of custom format parameter: {}",
                        file, i + 1, line.trim()
                    ));
                }
            }
        }
    }

    if !issues.is_empty() {
        eprintln!("\n⚠️  Found {} issue(s) with OutputFormatArgs usage:\n", issues.len());
        for issue in &issues {
            eprintln!("  {}", issue);
        }
        eprintln!("\n💡  Fix: Use OutputFormatArgs from src/lib/cli/args.rs with #[command(flatten)]");
    }

    println!("OutputFormatArgs usage check completed. Found {} potential issue(s)", issues.len());
}

/// 检查是否应该使用 DryRunArgs 但使用了自定义参数
#[test]
fn test_dry_run_args_usage() {
    let cli_dir = Path::new("src/lib/cli");

    let files = vec![
        "pr.rs",
        "branch.rs",
        "jira.rs",
        "config.rs",
        "tag.rs",
    ];

    let mut issues = Vec::new();

    for file in files {
        let file_path = cli_dir.join(file);
        if !file_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

        // 检查是否有 dry-run 相关参数但没有使用 DryRunArgs
        let has_dry_run = (content.contains("dry") && content.contains("run")) ||
                         content.contains("dry_run") ||
                         content.contains("dry-run");

        let uses_dry_run_args = content.contains("DryRunArgs") ||
            content.contains("use.*args::DryRunArgs") ||
            content.contains("use super::args::DryRunArgs");

        if has_dry_run && !uses_dry_run_args {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if (line.contains("dry") || line.contains("dry_run")) &&
                   line.contains("#[arg") &&
                   !line.contains("DryRunArgs") {
                    issues.push(format!(
                        "{}:{} - Should use DryRunArgs instead of custom dry-run parameter: {}",
                        file, i + 1, line.trim()
                    ));
                }
            }
        }
    }

    if !issues.is_empty() {
        eprintln!("\n⚠️  Found {} issue(s) with DryRunArgs usage:\n", issues.len());
        for issue in &issues {
            eprintln!("  {}", issue);
        }
        eprintln!("\n💡  Fix: Use DryRunArgs from src/lib/cli/args.rs with #[command(flatten)]");
    }

    println!("DryRunArgs usage check completed. Found {} potential issue(s)", issues.len());
}

/// 检查参数命名一致性
#[test]
fn test_argument_naming_consistency() {
    let cli_dir = Path::new("src/lib/cli");

    let mut issues = Vec::new();

    // 检查 JIRA 相关参数的命名一致性
    for file in ["pr.rs", "jira.rs", "log.rs", "branch.rs"].iter() {
        let file_path = cli_dir.join(file);
        if !file_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

        // 检查是否混用了不同的命名
        let has_jira_id = content.contains("jira_id") || content.contains("JIRA_ID");
        let has_jira_ticket = content.contains("jira_ticket") || content.contains("JIRA_TICKET");

        if has_jira_id && has_jira_ticket {
            issues.push(format!(
                "{} - Inconsistent JIRA parameter naming: found both jira_id and jira_ticket",
                file
            ));
        }
    }

    if !issues.is_empty() {
        eprintln!("\n⚠️  Found {} naming consistency issue(s):\n", issues.len());
        for issue in &issues {
            eprintln!("  {}", issue);
        }
        eprintln!("\n💡  Fix: Use consistent naming (prefer jira_id/JIRA_ID)");
        eprintln!("   Or use JiraIdArg from src/lib/cli/args.rs for consistency");
    }

    println!("Argument naming consistency check completed. Found {} potential issue(s)", issues.len());
}

/// 检查是否使用了 #[command(flatten)] 复用参数组
#[test]
fn test_flatten_attribute_usage() {
    let cli_dir = Path::new("src/lib/cli");

    let mut issues = Vec::new();

    for file in ["pr.rs", "jira.rs", "branch.rs", "log.rs"].iter() {
        let file_path = cli_dir.join(file);
        if !file_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read {}", file_path.display()));

        // 检查是否使用了共用参数但没有使用 flatten
        let uses_common_args = content.contains("JiraIdArg") ||
                              content.contains("OutputFormatArgs") ||
                              content.contains("DryRunArgs");

        if uses_common_args {
            let lines: Vec<&str> = content.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                // 检查是否定义了共用参数但没有使用 flatten
                if (line.contains("JiraIdArg") ||
                    line.contains("OutputFormatArgs") ||
                    line.contains("DryRunArgs")) &&
                   !line.contains("#[command(flatten)]") &&
                   i > 0 && !lines[i-1].contains("#[command(flatten)]") {
                    issues.push(format!(
                        "{}:{} - Should use #[command(flatten)] for common argument: {}",
                        file, i + 1, line.trim()
                    ));
                }
            }
        }
    }

    if !issues.is_empty() {
        eprintln!("\n⚠️  Found {} issue(s) with #[command(flatten)] usage:\n", issues.len());
        for issue in &issues {
            eprintln!("  {}", issue);
        }
        eprintln!("\n💡  Fix: Add #[command(flatten)] attribute before common argument");
        eprintln!("   Example:");
        eprintln!("     #[command(flatten)]");
        eprintln!("     jira_id: JiraIdArg,");
    }

    println!("Flatten attribute usage check completed. Found {} potential issue(s)", issues.len());
}

/// 运行所有参数检查
#[test]
fn test_all_argument_checks() {
    println!("\n=== Running CLI Argument Checks ===\n");

    test_jira_id_arg_usage();
    println!();

    test_output_format_args_usage();
    println!();

    test_dry_run_args_usage();
    println!();

    test_argument_naming_consistency();
    println!();

    test_flatten_attribute_usage();
    println!();

    println!("=== All Checks Completed ===\n");
    println!("Note: These checks are informational and do not fail the test.");
    println!("Review the output above and fix any issues found.");
}
