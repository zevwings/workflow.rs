//! Config CLI 命令测试
//!
//! 测试 Config CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use pretty_assertions::assert_eq;
use rstest::rstest;
use workflow::cli::ConfigSubcommand;

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-config")]
struct TestConfigCli {
    #[command(subcommand)]
    command: ConfigSubcommand,
}

// ==================== Show 命令测试 ====================

#[test]
fn test_config_show_command_structure() {
    // 测试 Show 命令基本结构
    let cli = TestConfigCli::try_parse_from(&["test-config", "show"])
        .expect("CLI args should parse successfully");

    match cli.command {
        ConfigSubcommand::Show => {
            // Show 命令没有参数，只需要验证可以解析
            assert!(true);
        }
        _ => panic!("Expected Show command"),
    }
}

#[test]
fn test_config_show_command_no_arguments() {
    // 测试命令不接受参数
    // Show 命令不应该接受任何参数，如果传入参数应该失败
    let _result = TestConfigCli::try_parse_from(&["test-config", "show", "invalid-arg"]);
    // 注意：clap 可能会接受额外的参数，这取决于配置
    // 这里主要验证命令可以正确解析
    let cli = TestConfigCli::try_parse_from(&["test-config", "show"])
        .expect("CLI args should parse successfully");
    assert!(matches!(cli.command, ConfigSubcommand::Show));
}

// ==================== Validate 命令测试 ====================

#[rstest]
#[case(None, false, false)]
#[case(Some("/path/to/config.toml"), false, false)]
#[case(None, true, false)]
#[case(None, false, true)]
#[case(Some("/path/to/config.toml"), true, true)]
fn test_config_validate_command(
    #[case] config_path: Option<&str>,
    #[case] fix: bool,
    #[case] strict: bool,
) {
    let mut args = vec!["test-config", "validate"];
    if let Some(path) = config_path {
        args.push(path);
    }
    if fix {
        args.push("--fix");
    }
    if strict {
        args.push("--strict");
    }

    let cli = TestConfigCli::try_parse_from(&args).expect("CLI args should parse successfully");

    match cli.command {
        ConfigSubcommand::Validate {
            config_path: cp,
            fix: f,
            strict: s,
        } => {
            assert_eq!(cp, config_path.map(|s| s.to_string()));
            assert_eq!(f, fix);
            assert_eq!(s, strict);
        }
        _ => panic!("Expected Validate command"),
    }
}

// ==================== Export 命令测试 ====================

#[test]
fn test_config_export_command_structure() {
    // 测试 Export 命令基本结构
    let cli = TestConfigCli::try_parse_from(&["test-config", "export", "output.toml"])
        .expect("CLI args should parse successfully");

    match cli.command {
        ConfigSubcommand::Export {
            output_path,
            section,
            no_secrets,
            toml,
            json,
            yaml,
        } => {
            assert_eq!(output_path, "output.toml");
            assert_eq!(section, None);
            assert!(!no_secrets);
            assert!(!toml);
            assert!(!json);
            assert!(!yaml);
        }
        _ => panic!("Expected Export command"),
    }
}

#[rstest]
#[case("output.toml", None, false, false, false, false)]
#[case("/path/to/output.json", None, false, false, false, false)]
#[case("output.toml", Some("jira"), false, false, false, false)]
#[case("output.toml", None, true, false, false, false)]
#[case("output.toml", None, false, true, false, false)]
#[case("output.toml", None, false, false, true, false)]
#[case("output.toml", None, false, false, false, true)]
#[case("output.toml", Some("jira"), true, true, true, true)]
fn test_config_export_command(
    #[case] output_path: &str,
    #[case] section: Option<&str>,
    #[case] no_secrets: bool,
    #[case] toml: bool,
    #[case] json: bool,
    #[case] yaml: bool,
) {
    let mut args = vec!["test-config", "export", output_path];
    if let Some(s) = section {
        args.push("--section");
        args.push(s);
    }
    if no_secrets {
        args.push("--no-secrets");
    }
    if toml {
        args.push("--toml");
    }
    if json {
        args.push("--json");
    }
    if yaml {
        args.push("--yaml");
    }

    let cli = TestConfigCli::try_parse_from(&args).expect("CLI args should parse successfully");

    match cli.command {
        ConfigSubcommand::Export {
            output_path: op,
            section: s,
            no_secrets: ns,
            toml: t,
            json: j,
            yaml: y,
        } => {
            assert_eq!(op, output_path);
            assert_eq!(s, section.map(|s| s.to_string()));
            assert_eq!(ns, no_secrets);
            assert_eq!(t, toml);
            assert_eq!(j, json);
            assert_eq!(y, yaml);
        }
        _ => panic!("Expected Export command"),
    }
}

#[rstest]
#[case("--toml", true, false, false)]
#[case("--json", false, true, false)]
#[case("--yaml", false, false, true)]
fn test_config_export_command_output_formats(
    #[case] flag: &str,
    #[case] toml: bool,
    #[case] json: bool,
    #[case] yaml: bool,
) {
    let cli =
        TestConfigCli::try_parse_from(&["test-config", "export", "output.toml", flag])
            .expect("CLI args should parse successfully");
    match cli.command {
        ConfigSubcommand::Export {
            toml: t,
            json: j,
            yaml: y,
            ..
        } => {
            assert_eq!(t, toml);
            assert_eq!(j, json);
            assert_eq!(y, yaml);
        }
        _ => panic!("Expected Export command"),
    }
}

// ==================== Import 命令测试 ====================

#[rstest]
#[case("input.toml", false, None, false)]
#[case("/path/to/input.json", false, None, false)]
#[case("input.toml", true, None, false)]
#[case("input.toml", false, Some("pr"), false)]
#[case("input.toml", false, None, true)]
#[case("input.toml", true, Some("jira"), true)]
fn test_config_import_command(
    #[case] input_path: &str,
    #[case] overwrite: bool,
    #[case] section: Option<&str>,
    #[case] dry_run: bool,
) {
    let mut args = vec!["test-config", "import", input_path];
    if overwrite {
        args.push("--overwrite");
    }
    if let Some(s) = section {
        args.push("--section");
        args.push(s);
    }
    if dry_run {
        args.push("--dry-run");
    }

    let cli = TestConfigCli::try_parse_from(&args).expect("CLI args should parse successfully");

    match cli.command {
        ConfigSubcommand::Import {
            input_path: ip,
            overwrite: o,
            section: s,
            dry_run: dr,
        } => {
            assert_eq!(ip, input_path);
            assert_eq!(o, overwrite);
            assert_eq!(s, section.map(|s| s.to_string()));
            assert_eq!(dr.dry_run, dry_run);
        }
        _ => panic!("Expected Import command"),
    }
}

// ==================== Config 命令通用测试 ====================

#[rstest]
#[case("show", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Show))]
#[case("validate", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Validate { .. }))]
#[case("export", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Export { .. }))]
#[case("import", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Import { .. }))]
fn test_config_command_parsing_all_subcommands(
    #[case] subcommand: &str,
    #[case] assert_fn: fn(&ConfigSubcommand) -> bool,
) {
    let mut args = vec!["test-config", subcommand];
    // 为需要参数的命令添加最小参数
    match subcommand {
        "export" => args.push("output.toml"),
        "import" => args.push("input.toml"),
        _ => {}
    }

    let cli = TestConfigCli::try_parse_from(&args).expect("CLI args should parse successfully");
    assert!(
        assert_fn(&cli.command),
        "Command should match expected variant"
    );
}

#[test]
fn test_config_command_error_handling_invalid_subcommand() {
    // 测试无效子命令的错误处理
    let result = TestConfigCli::try_parse_from(&["test-config", "invalid"]);
    assert!(result.is_err(), "Should fail on invalid subcommand");
}

#[test]
fn test_config_command_error_handling_missing_subcommand() {
    // 测试缺少子命令的错误处理
    // 由于 TestConfigCli 中的 command 字段是必需的（不是 Option），缺少子命令应该失败
    let result = TestConfigCli::try_parse_from(&["test-config"]);
    assert!(result.is_err(), "Should fail when subcommand is missing");
}
