//! Config CLI 命令测试
//!
//! 测试 Config CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use color_eyre::Result;
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

// ==================== Show Command Tests ====================

/// 测试 Config Show 命令解析
///
/// ## 测试目的
/// 验证 ConfigSubcommand::Show 能够正确解析命令行参数。
///
/// ## 测试场景
/// 1. 准备有效的 Show 命令输入
/// 2. 解析命令行参数
/// 3. 验证 Show 命令可以正确解析
///
/// ## 预期结果
/// - Show 命令正确解析（没有参数）
#[test]
fn test_config_show_command_with_valid_input_parses_successfully() -> Result<()> {
    // Arrange: 准备有效的 Show 命令输入
    let args = &["test-config", "show"];

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(args)?;

    // Assert: 验证 Show 命令可以正确解析（没有参数）
    match cli.command {
        ConfigSubcommand::Show => {
            assert!(true);
        }
        _ => return Err(color_eyre::eyre::eyre!("Expected Show command")),
    }

    Ok(())
}

/// 测试 Config Show 命令解析（不带额外参数）
///
/// ## 测试目的
/// 验证 ConfigSubcommand::Show 能够正确解析不带额外参数的命令行参数。
///
/// ## 测试场景
/// 1. 准备不带额外参数的 Show 命令输入
/// 2. 解析命令行参数
/// 3. 验证命令可以正确解析
///
/// ## 预期结果
/// - Show 命令正确解析
#[test]
fn test_config_show_command_with_no_arguments_parses_successfully() -> Result<()> {
    // Arrange: 准备不带额外参数的 Show 命令输入
    // 注意：clap 可能会接受额外的参数，这取决于配置
    let args = &["test-config", "show"];

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(args)?;

    // Assert: 验证命令可以正确解析
    assert!(matches!(cli.command, ConfigSubcommand::Show));

    Ok(())
}

// ==================== Validate Command Tests ====================

/// 测试 Config Validate 命令解析（各种选项）
///
/// ## 测试目的
/// 使用参数化测试验证 ConfigSubcommand::Validate 能够正确解析各种选项组合。
///
/// ## 测试场景
/// 1. 准备各种选项组合的命令行参数（config_path、fix、strict）
/// 2. 解析命令行参数
/// 3. 验证参数解析正确
///
/// ## 预期结果
/// - 所有选项组合都能正确解析
#[rstest]
#[case(None, false, false)]
#[case(Some("/path/to/config.toml"), false, false)]
#[case(None, true, false)]
#[case(None, false, true)]
#[case(Some("/path/to/config.toml"), true, true)]
fn test_config_validate_command_with_various_options_parses_correctly(
    #[case] config_path: Option<&str>,
    #[case] fix: bool,
    #[case] strict: bool,
) -> Result<()> {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(&args)?;

    // Assert: 验证参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Validate command")),
    }

    Ok(())
}

// ==================== Export Command Tests ====================

/// 测试 Config Export 命令解析（基本结构）
///
/// ## 测试目的
/// 验证 ConfigSubcommand::Export 能够正确解析基本的命令行参数。
///
/// ## 测试场景
/// 1. 准备基本的 Export 命令输入
/// 2. 解析命令行参数
/// 3. 验证基本结构解析正确
///
/// ## 预期结果
/// - Export 命令基本结构解析正确，默认值正确
#[test]
fn test_config_export_command_with_basic_structure_parses_correctly() -> Result<()> {
    // Arrange: 准备基本的 Export 命令输入
    let args = &["test-config", "export", "output.toml"];

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(args)?;

    // Assert: 验证基本结构解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Export command")),
    }

    Ok(())
}

/// 测试 Config Export 命令解析（各种选项）
///
/// ## 测试目的
/// 使用参数化测试验证 ConfigSubcommand::Export 能够正确解析各种选项组合。
///
/// ## 测试场景
/// 1. 准备各种选项组合的命令行参数（output_path、section、no_secrets、toml、json、yaml）
/// 2. 解析命令行参数
/// 3. 验证参数解析正确
///
/// ## 预期结果
/// - 所有选项组合都能正确解析
#[rstest]
#[case("output.toml", None, false, false, false, false)]
#[case("/path/to/output.json", None, false, false, false, false)]
#[case("output.toml", Some("jira"), false, false, false, false)]
#[case("output.toml", None, true, false, false, false)]
#[case("output.toml", None, false, true, false, false)]
#[case("output.toml", None, false, false, true, false)]
#[case("output.toml", None, false, false, false, true)]
#[case("output.toml", Some("jira"), true, true, true, true)]
fn test_config_export_command_with_various_options_parses_correctly(
    #[case] output_path: &str,
    #[case] section: Option<&str>,
    #[case] no_secrets: bool,
    #[case] toml: bool,
    #[case] json: bool,
    #[case] yaml: bool,
) -> Result<()> {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(&args)?;

    // Assert: 验证参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Export command")),
    }

    Ok(())
}

/// 测试 Config Export 命令输出格式标志解析
///
/// ## 测试目的
/// 使用参数化测试验证 ConfigSubcommand::Export 能够正确解析输出格式标志。
///
/// ## 测试场景
/// 1. 准备带输出格式标志的命令输入
/// 2. 解析命令行参数
/// 3. 验证输出格式标志解析正确
///
/// ## 预期结果
/// - 输出格式标志（--toml、--json、--yaml）都能正确解析
#[rstest]
#[case("--toml", true, false, false)]
#[case("--json", false, true, false)]
#[case("--yaml", false, false, true)]
fn test_config_export_command_with_output_format_flags_parses_correctly(
    #[case] flag: &str,
    #[case] toml: bool,
    #[case] json: bool,
    #[case] yaml: bool,
) -> Result<()> {
    // Arrange: 准备带输出格式标志的命令输入
    let args = &["test-config", "export", "output.toml", flag];

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(args)?;

    // Assert: 验证输出格式标志解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Export command")),
    }

    Ok(())
}

// ==================== Import Command Tests ====================

/// 测试 Config Import 命令解析（各种选项）
///
/// ## 测试目的
/// 使用参数化测试验证 ConfigSubcommand::Import 能够正确解析各种选项组合。
///
/// ## 测试场景
/// 1. 准备各种选项组合的命令行参数（input_path、overwrite、section、dry_run）
/// 2. 解析命令行参数
/// 3. 验证参数解析正确
///
/// ## 预期结果
/// - 所有选项组合都能正确解析
#[rstest]
#[case("input.toml", false, None, false)]
#[case("/path/to/input.json", false, None, false)]
#[case("input.toml", true, None, false)]
#[case("input.toml", false, Some("pr"), false)]
#[case("input.toml", false, None, true)]
#[case("input.toml", true, Some("jira"), true)]
fn test_config_import_command_with_various_options_parses_correctly(
    #[case] input_path: &str,
    #[case] overwrite: bool,
    #[case] section: Option<&str>,
    #[case] dry_run: bool,
) -> Result<()> {
    // Arrange: 准备命令行参数
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

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(&args)?;

    // Assert: 验证参数解析正确
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
        _ => return Err(color_eyre::eyre::eyre!("Expected Import command")),
    }

    Ok(())
}

// ==================== Common Command Tests ====================

/// 测试 Config 命令的所有子命令解析
///
/// ## 测试目的
/// 使用参数化测试验证 ConfigSubcommand 的所有子命令都能够正确解析。
///
/// ## 测试场景
/// 1. 准备所有子命令的输入（为需要参数的命令添加最小参数）
/// 2. 解析命令行参数
/// 3. 验证所有子命令都可以正确解析
///
/// ## 预期结果
/// - 所有子命令都可以正确解析
#[rstest]
#[case("show", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Show))]
#[case("validate", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Validate { .. }))]
#[case("export", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Export { .. }))]
#[case("import", |cmd: &ConfigSubcommand| matches!(cmd, ConfigSubcommand::Import { .. }))]
fn test_config_command_with_all_subcommands_parses_successfully(
    #[case] subcommand: &str,
    #[case] assert_fn: fn(&ConfigSubcommand) -> bool,
) -> Result<()> {
    // Arrange: 准备所有子命令的输入
    let mut args = vec!["test-config", subcommand];
    // 为需要参数的命令添加最小参数
    match subcommand {
        "export" => args.push("output.toml"),
        "import" => args.push("input.toml"),
        _ => {}
    }

    // Act: 解析命令行参数
    let cli = TestConfigCli::try_parse_from(&args)?;

    // Assert: 验证所有子命令都可以正确解析
    assert!(
        assert_fn(&cli.command),
        "Command should match expected variant"
    );

    Ok(())
}

// ==================== Error Handling Tests ====================

/// 测试 Config 命令无效子命令错误处理
///
/// ## 测试目的
/// 验证 ConfigSubcommand 对无效子命令返回错误。
///
/// ## 测试场景
/// 1. 准备无效子命令的输入
/// 2. 尝试解析无效子命令
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 无效子命令返回解析错误
#[test]
fn test_config_command_with_invalid_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备无效子命令的输入
    let args = &["test-config", "invalid"];

    // Act: 尝试解析无效子命令
    let result = TestConfigCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail on invalid subcommand");

    Ok(())
}

/// 测试 Config 命令缺少子命令错误处理
///
/// ## 测试目的
/// 验证 ConfigSubcommand 在缺少子命令时返回错误。
///
/// ## 测试场景
/// 1. 准备缺少子命令的输入
/// 2. 尝试解析缺少子命令的参数
/// 3. 验证返回错误
///
/// ## 预期结果
/// - 缺少子命令时返回解析错误
#[test]
fn test_config_command_with_missing_subcommand_returns_error() -> Result<()> {
    // Arrange: 准备缺少子命令的输入
    // 注意：由于 TestConfigCli 中的 command 字段是必需的（不是 Option），缺少子命令应该失败
    let args = &["test-config"];

    // Act: 尝试解析缺少子命令的参数
    let result = TestConfigCli::try_parse_from(args);

    // Assert: 验证返回错误
    assert!(result.is_err(), "Should fail when subcommand is missing");

    Ok(())
}
