# 测试需求文档

本文档列出了需要新增的测试用例，以完善测试覆盖。

## 📋 测试覆盖情况概览

### ✅ 已有测试的模块

- `tests/cli/branch.rs` - Branch 命令测试（完整）
- `tests/cli/pr.rs` - PR 命令测试（完整）
- `tests/cli/github.rs` - GitHub 命令测试（完整）
- `tests/cli/llm.rs` - LLM 命令测试（完整）
- `tests/cli/log.rs` - Log 命令测试（完整）
- `tests/cli/proxy.rs` - Proxy 命令测试（完整）
- `tests/cli/jira.rs` - Jira 命令测试（部分）

### ❌ 需要新增/补充的测试

## 1. Jira 命令测试补充 (`tests/cli/jira.rs`)

### 1.1 Changelog 命令测试

需要测试 `JiraSubcommand::Changelog` 命令的所有参数组合：

- [ ] `test_jira_changelog_command_structure` - 测试 Changelog 命令基本结构
- [ ] `test_jira_changelog_command_with_jira_id` - 测试带 JIRA ID 的情况
- [ ] `test_jira_changelog_command_without_id` - 测试不带 JIRA ID（交互式输入）
- [ ] `test_jira_changelog_command_with_field_filter` - 测试 `--field` 参数
- [ ] `test_jira_changelog_command_output_formats` - 测试输出格式（table, json, yaml, markdown）
- [ ] `test_jira_changelog_command_all_flags` - 测试所有标志组合
- [ ] `test_jira_changelog_command_short_flags` - 测试短标志（如果有）

### 1.2 Comments 命令测试

需要测试 `JiraSubcommand::Comments` 命令的所有参数组合：

- [ ] `test_jira_comments_command_structure` - 测试 Comments 命令基本结构
- [ ] `test_jira_comments_command_with_jira_id` - 测试带 JIRA ID 的情况
- [ ] `test_jira_comments_command_without_id` - 测试不带 JIRA ID（交互式输入）
- [ ] `test_jira_comments_command_with_limit` - 测试 `--limit` 参数
- [ ] `test_jira_comments_command_with_offset` - 测试 `--offset` 参数
- [ ] `test_jira_comments_command_with_author` - 测试 `--author` 参数
- [ ] `test_jira_comments_command_with_since` - 测试 `--since` 参数
- [ ] `test_jira_comments_command_output_formats` - 测试输出格式（table, json, yaml, markdown）
- [ ] `test_jira_comments_command_all_filters` - 测试所有过滤参数组合
- [ ] `test_jira_comments_command_pagination` - 测试分页参数组合（limit + offset）

### 1.3 Info 命令测试补充

需要补充 Info 命令的输出格式测试：

- [ ] `test_jira_info_command_output_formats` - 测试输出格式（table, json, yaml, markdown）
- [ ] `test_jira_info_command_format_flags_combination` - 测试格式标志的组合

## 2. Config 命令测试（新建 `tests/cli/config.rs`）

### 2.1 Show 命令测试

需要测试 `ConfigSubcommand::Show` 命令：

- [ ] `test_config_show_command_structure` - 测试 Show 命令基本结构
- [ ] `test_config_show_command_no_arguments` - 测试命令不接受参数

### 2.2 Validate 命令测试

需要测试 `ConfigSubcommand::Validate` 命令的所有参数组合：

- [ ] `test_config_validate_command_structure` - 测试 Validate 命令基本结构
- [ ] `test_config_validate_command_with_config_path` - 测试指定配置文件路径
- [ ] `test_config_validate_command_without_config_path` - 测试使用默认配置文件路径
- [ ] `test_config_validate_command_with_fix_flag` - 测试 `--fix` 标志
- [ ] `test_config_validate_command_with_strict_flag` - 测试 `--strict` 标志
- [ ] `test_config_validate_command_all_flags` - 测试所有标志组合
- [ ] `test_config_validate_command_invalid_path` - 测试无效配置文件路径的错误处理

### 2.3 Export 命令测试

需要测试 `ConfigSubcommand::Export` 命令的所有参数组合：

- [ ] `test_config_export_command_structure` - 测试 Export 命令基本结构
- [ ] `test_config_export_command_with_output_path` - 测试指定输出路径
- [ ] `test_config_export_command_with_section` - 测试 `--section` 参数
- [ ] `test_config_export_command_with_no_secrets` - 测试 `--no-secrets` 标志
- [ ] `test_config_export_command_output_formats` - 测试输出格式（toml, json, yaml）
- [ ] `test_config_export_command_all_flags` - 测试所有标志组合
- [ ] `test_config_export_command_invalid_section` - 测试无效 section 的错误处理

### 2.4 Import 命令测试

需要测试 `ConfigSubcommand::Import` 命令的所有参数组合：

- [ ] `test_config_import_command_structure` - 测试 Import 命令基本结构
- [ ] `test_config_import_command_with_input_path` - 测试指定输入文件路径
- [ ] `test_config_import_command_with_overwrite` - 测试 `--overwrite` 标志
- [ ] `test_config_import_command_with_section` - 测试 `--section` 参数
- [ ] `test_config_import_command_with_dry_run` - 测试 `--dry-run` 标志
- [ ] `test_config_import_command_all_flags` - 测试所有标志组合
- [ ] `test_config_import_command_invalid_path` - 测试无效输入文件路径的错误处理
- [ ] `test_config_import_command_invalid_section` - 测试无效 section 的错误处理

### 2.5 Config 命令通用测试

- [ ] `test_config_command_parsing_all_subcommands` - 测试所有子命令都可以正确解析
- [ ] `test_config_command_error_handling_invalid_subcommand` - 测试无效子命令的错误处理
- [ ] `test_config_command_error_handling_missing_subcommand` - 测试缺少子命令的错误处理

## 3. 其他潜在测试需求

### 3.1 集成测试

- [ ] 测试命令之间的交互（例如：config export -> config import）
- [ ] 测试配置验证在导出/导入流程中的作用

### 3.2 边界情况测试

- [ ] 测试空配置文件
- [ ] 测试无效格式的配置文件
- [ ] 测试超大配置文件
- [ ] 测试特殊字符处理

### 3.3 错误处理测试

- [ ] 测试文件权限错误
- [ ] 测试磁盘空间不足
- [ ] 测试网络错误（对于需要网络访问的命令）

## 📝 测试编写指南

### 测试文件结构

每个测试文件应遵循以下结构：

```rust
//! [模块名] CLI 命令测试
//!
//! 测试 [模块名] CLI 命令的参数解析、命令执行流程和错误处理。

use clap::Parser;
use workflow::cli::[SubcommandEnum];

// 创建一个测试用的 CLI 结构来测试参数解析
#[derive(Parser)]
#[command(name = "test-[module]")]
struct Test[Cli]Cli {
    #[command(subcommand)]
    command: [SubcommandEnum],
}

// ==================== 命令结构测试 ====================

#[test]
fn test_[command]_command_structure() {
    // 测试基本结构
}

#[test]
fn test_[command]_command_with_[parameter]() {
    // 测试带参数的情况
}

// ==================== 错误处理测试 ====================

#[test]
fn test_[command]_error_handling_invalid_[scenario]() {
    // 测试错误处理
}
```

### 测试命名规范

- 使用 `test_` 前缀
- 使用下划线分隔单词
- 描述性命名：`test_[module]_[command]_[scenario]`

### 测试覆盖目标

每个命令应该测试：

1. **基本结构** - 命令可以正确解析
2. **所有参数** - 每个参数的各种组合
3. **可选参数** - 参数存在和不存在的情况
4. **标志组合** - 多个标志的组合使用
5. **错误处理** - 无效输入、缺失参数等
6. **边界情况** - 空值、特殊字符等

## 🎯 优先级

### 高优先级（立即实现）

1. **Config 命令测试** - 新建 `tests/cli/config.rs`，这是全新的测试文件
2. **Jira Changelog 命令测试** - 补充到 `tests/cli/jira.rs`
3. **Jira Comments 命令测试** - 补充到 `tests/cli/jira.rs`

### 中优先级（后续实现）

1. **Jira Info 命令输出格式测试** - 补充现有测试
2. **集成测试** - 测试命令之间的交互

### 低优先级（可选）

1. **边界情况测试** - 特殊场景测试
2. **性能测试** - 大文件处理等

## 📊 测试覆盖率目标

- **CLI 命令参数解析**: 100%
- **命令枚举变体**: 100%
- **错误处理**: 80%+
- **边界情况**: 60%+

## 🔗 相关文件

- `src/lib/cli/jira.rs` - Jira 命令定义
- `src/lib/cli/config.rs` - Config 命令定义
- `src/commands/jira/changelog.rs` - Changelog 命令实现
- `src/commands/jira/comments.rs` - Comments 命令实现
- `src/commands/config/validate.rs` - Validate 命令实现
- `src/commands/config/export.rs` - Export 命令实现
- `src/commands/config/import.rs` - Import 命令实现
