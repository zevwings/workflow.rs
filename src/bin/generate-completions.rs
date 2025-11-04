//! 生成 shell completion 脚本
//!
//! 使用方法:
//!   cargo run --bin generate-completions -- <shell> <output_dir>
//!
//! 示例:
//!   cargo run --bin generate-completions -- zsh ~/.zsh/completions
//!   cargo run --bin generate-completions -- bash ~/.bash_completion.d

use anyhow::Result;
use clap_complete::{generate, shells::Shell};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("用法: generate-completions <shell> <output_dir>");
        eprintln!("支持的 shell: zsh, bash, fish, powershell, elvish");
        eprintln!("");
        eprintln!("示例:");
        eprintln!("  cargo run --bin generate-completions -- zsh ~/.zsh/completions");
        eprintln!("  cargo run --bin generate-completions -- bash ~/.bash_completion.d");
        std::process::exit(1);
    }

    let shell_name = &args[1];
    let output_dir = PathBuf::from(&args[2]);

    // 解析 shell 类型
    let shell = match shell_name.as_str() {
        "zsh" => Shell::Zsh,
        "bash" => Shell::Bash,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            eprintln!("不支持的 shell: {}", shell_name);
            eprintln!("支持的 shell: zsh, bash, fish, powershell, elvish");
            std::process::exit(1);
        }
    };

    // 创建输出目录
    std::fs::create_dir_all(&output_dir)?;

    // 生成 workflow 命令的 completion
    generate_workflow_completion(&shell, &output_dir)?;

    // 生成 pr 命令的 completion
    generate_pr_completion(&shell, &output_dir)?;

    // 生成 qk 命令的 completion
    generate_qk_completion(&shell, &output_dir)?;

    println!("✅ Shell completion 脚本已生成到: {}", output_dir.display());
    println!("");
    println!("请将以下内容添加到您的 shell 配置文件:");
    match shell_name.as_str() {
        "zsh" => {
            println!("  fpath=({} $fpath)", output_dir.display());
            println!("  autoload -Uz compinit && compinit");
        }
        "bash" => {
            println!("  source {}", output_dir.join("workflow.bash").display());
            println!("  或在 ~/.bashrc 中添加:");
            println!("    for f in {}/*.bash; do source \"$f\"; done", output_dir.display());
        }
        "fish" => {
            println!("  将生成的文件复制到 ~/.config/fish/completions/");
        }
        _ => {
            println!("  请参考相应 shell 的文档来加载 completion 文件");
        }
    }

    Ok(())
}

fn generate_workflow_completion(shell: &Shell, output_dir: &PathBuf) -> Result<()> {
    use clap::Command;

    let mut cmd = Command::new("workflow")
        .about("Workflow CLI tool")
        .subcommand(
            Command::new("proxy")
                .about("Manage proxy settings")
                .subcommand(Command::new("on").about("Turn proxy on"))
                .subcommand(Command::new("off").about("Turn proxy off"))
                .subcommand(Command::new("check").about("Check proxy status"))
        )
        .subcommand(
            Command::new("check")
                .about("Run checks")
                .subcommand(Command::new("git_status").about("Check git status"))
                .subcommand(Command::new("network").about("Check network connection"))
                .subcommand(Command::new("pre_commit").about("Run pre-commit checks"))
        )
        .subcommand(Command::new("setup").about("Initialize or update configuration"))
        .subcommand(Command::new("config").about("View current configuration"))
        .subcommand(Command::new("uninstall").about("Uninstall Workflow CLI configuration"));

    let mut buffer = Vec::new();
    generate(*shell, &mut cmd, "workflow", &mut buffer);

    let output_file = match shell {
        Shell::Zsh => output_dir.join("_workflow"),
        Shell::Bash => output_dir.join("workflow.bash"),
        Shell::Fish => output_dir.join("workflow.fish"),
        Shell::PowerShell => output_dir.join("_workflow.ps1"),
        Shell::Elvish => output_dir.join("workflow.elv"),
        _ => {
            eprintln!("不支持的 shell 类型");
            std::process::exit(1);
        }
    };
    std::fs::write(&output_file, buffer)?;
    println!("  生成: {}", output_file.display());

    Ok(())
}

fn generate_pr_completion(shell: &Shell, output_dir: &PathBuf) -> Result<()> {
    use clap::Command;

    let mut cmd = Command::new("pr")
        .about("Pull Request operations")
        .subcommand(
            Command::new("create")
                .about("Create a new Pull Request")
                .arg(clap::Arg::new("JIRA_TICKET").value_name("JIRA_TICKET"))
                .arg(clap::Arg::new("title").short('t').long("title"))
                .arg(clap::Arg::new("description").short('d').long("description"))
                .arg(clap::Arg::new("dry-run").long("dry-run"))
        )
        .subcommand(
            Command::new("merge")
                .about("Merge a Pull Request")
                .arg(clap::Arg::new("PR_ID").value_name("PR_ID"))
                .arg(clap::Arg::new("force").short('f').long("force"))
        )
        .subcommand(
            Command::new("status")
                .about("Show PR status information")
                .arg(clap::Arg::new("PR_ID_OR_BRANCH").value_name("PR_ID_OR_BRANCH"))
        )
        .subcommand(
            Command::new("list")
                .about("List PRs")
                .arg(clap::Arg::new("state").short('s').long("state"))
                .arg(clap::Arg::new("limit").short('l').long("limit"))
        )
        .subcommand(Command::new("update").about("Update code"));

    let mut buffer = Vec::new();
    generate(*shell, &mut cmd, "pr", &mut buffer);

    let output_file = match shell {
        Shell::Zsh => output_dir.join("_pr"),
        Shell::Bash => output_dir.join("pr.bash"),
        Shell::Fish => output_dir.join("pr.fish"),
        Shell::PowerShell => output_dir.join("_pr.ps1"),
        Shell::Elvish => output_dir.join("pr.elv"),
        _ => {
            eprintln!("不支持的 shell 类型");
            std::process::exit(1);
        }
    };
    std::fs::write(&output_file, buffer)?;
    println!("  生成: {}", output_file.display());

    Ok(())
}

fn generate_qk_completion(shell: &Shell, output_dir: &PathBuf) -> Result<()> {
    use clap::Command;

    let mut cmd = Command::new("qk")
        .about("Quick log operations")
        .arg(clap::Arg::new("JIRA_ID").value_name("JIRA_ID").required(true))
        .subcommand(Command::new("download").about("Download logs"))
        .subcommand(
            Command::new("find")
                .about("Find request by ID")
                .arg(clap::Arg::new("REQUEST_ID").value_name("REQUEST_ID"))
        )
        .subcommand(
            Command::new("search")
                .about("Search in logs")
                .arg(clap::Arg::new("SEARCH_TERM").value_name("SEARCH_TERM"))
        );

    let mut buffer = Vec::new();
    generate(*shell, &mut cmd, "qk", &mut buffer);

    let output_file = match shell {
        Shell::Zsh => output_dir.join("_qk"),
        Shell::Bash => output_dir.join("qk.bash"),
        Shell::Fish => output_dir.join("qk.fish"),
        Shell::PowerShell => output_dir.join("_qk.ps1"),
        Shell::Elvish => output_dir.join("qk.elv"),
        _ => {
            eprintln!("不支持的 shell 类型");
            std::process::exit(1);
        }
    };
    std::fs::write(&output_file, buffer)?;
    println!("  生成: {}", output_file.display());

    Ok(())
}

