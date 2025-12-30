//! Completion 脚本生成工具
//!
//! 提供生成各种 shell 的 completion 脚本文件的功能。

use std::path::PathBuf;

use clap::{Command, CommandFactory};
use clap_complete::{generate, shells::Shell as ClapShell};
use color_eyre::{eyre::WrapErr, Result};

use super::helpers::get_completion_filename;
use crate::base::alias::AliasManager;
use crate::base::fs::{DirectoryWalker, FileWriter};
use crate::base::settings::paths::Paths;

/// 生成结果
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// 生成的消息列表
    pub messages: Vec<String>,
}

/// Completion 脚本生成器
///
/// 提供生成各种 shell 的 completion 脚本文件的功能。
/// 支持 workflow 命令及其所有子命令的 completion 生成。
/// 包含动态补全和性能优化功能。
pub struct CompletionGenerator {
    shell: ClapShell,
    output_dir: PathBuf,
    enable_dynamic_completion: bool,
    enable_performance_optimization: bool,
}

impl CompletionGenerator {
    /// 创建新的 CompletionGenerator 实例
    ///
    /// # 参数
    ///
    /// * `shell_type` - Shell 类型字符串（"zsh", "bash", "fish", "powershell", "elvish"），如果为 None 则自动检测
    /// * `output_dir` - 输出目录路径，如果为 None 则使用默认目录 `~/.workflow/completions`
    ///
    /// # 返回
    ///
    /// 返回 `CompletionGenerator` 实例，如果 shell 类型不支持则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use workflow::completion::generate::CompletionGenerator;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let generator = CompletionGenerator::new(
    ///     Some("zsh".to_string()),
    ///     Some("/path/to/completions".to_string()),
    /// )?;
    /// generator.generate_all()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(shell_type: Option<String>, output_dir: Option<String>) -> Result<Self> {
        Self::with_options(shell_type, output_dir, true, true)
    }

    /// 创建带自定义选项的 CompletionGenerator 实例
    ///
    /// # 参数
    ///
    /// * `shell_type` - Shell 类型字符串
    /// * `output_dir` - 输出目录路径
    /// * `enable_dynamic_completion` - 是否启用动态补全（分支名、PR ID 等）
    /// * `enable_performance_optimization` - 是否启用性能优化（缓存、延迟加载等）
    pub fn with_options(
        shell_type: Option<String>,
        output_dir: Option<String>,
        enable_dynamic_completion: bool,
        enable_performance_optimization: bool,
    ) -> Result<Self> {
        // 解析 shell 类型
        let shell = shell_type.as_deref().unwrap_or_else(|| {
            let shell_env = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            if shell_env.contains("zsh") {
                "zsh"
            } else if shell_env.contains("bash") {
                "bash"
            } else {
                "zsh" // 默认
            }
        });

        let clap_shell = match shell {
            "zsh" => ClapShell::Zsh,
            "bash" => ClapShell::Bash,
            "fish" => ClapShell::Fish,
            "powershell" => ClapShell::PowerShell,
            "elvish" => ClapShell::Elvish,
            _ => {
                color_eyre::eyre::bail!("Unsupported shell type: {}. Supported shell types: zsh, bash, fish, powershell, elvish", shell);
            }
        };

        // 解析输出目录
        let output = output_dir.map(PathBuf::from).unwrap_or_else(|| {
            Paths::completion_dir().unwrap_or_else(|_| PathBuf::from("~/.workflow/completions"))
        });

        Ok(Self {
            shell: clap_shell,
            output_dir: output,
            enable_dynamic_completion,
            enable_performance_optimization,
        })
    }

    /// 生成所有 completion 脚本文件
    ///
    /// 为所有命令生成 completion 脚本：
    /// - `workflow` 命令及其所有子命令（包括 `pr`（create、merge、approve、comment、close、status、list、update、sync、rebase、pick、summarize）、`log`（set、check）、`jira`（info、related、changelog、comments、attachments、clean）、`config`（show、validate、export、import）、`github`、`llm`、`proxy`、`branch`（ignore、create、rename、switch、sync、delete）、`repo`（setup、show、clean）、`migrate`（cleanup）等）
    ///
    /// # 返回
    ///
    /// 返回 `GenerateResult`，包含生成的消息。
    pub fn generate_all(&self) -> Result<GenerateResult> {
        crate::trace_debug!("Generating shell completion scripts...");
        crate::trace_debug!("Shell type: {}", self.shell);
        crate::trace_debug!("Output directory: {}", self.output_dir.display());

        // 创建输出目录
        DirectoryWalker::new(&self.output_dir).ensure_exists()?;

        // 生成 completion 脚本
        self.generate_workflow()?;

        Ok(GenerateResult {
            messages: vec![format!(
                "  Shell completion scripts generated to: {}",
                self.output_dir.display()
            )],
        })
    }

    /// 生成 workflow 命令的 completion
    ///
    /// 使用实际的 CLI 结构体自动生成补全脚本，确保补全脚本与实际命令结构保持同步。
    /// 这样就不需要手动维护两套命令定义，避免了不同步的问题。
    fn generate_workflow(&self) -> Result<()> {
        // 使用实际的 CLI 结构体生成补全脚本，而不是手动构建
        // 这样可以确保补全脚本与实际命令结构保持同步
        let mut cmd = crate::cli::Cli::command();

        self.generate_completion(&mut cmd, "workflow")
    }

    /// 生成单个命令的 completion（通用方法）
    ///
    /// # 参数
    ///
    /// * `cmd` - clap Command 实例
    /// * `command_name` - 命令名称（"workflow"）
    fn generate_completion(&self, cmd: &mut Command, command_name: &str) -> Result<()> {
        let mut buffer = Vec::new();
        generate(self.shell, cmd, command_name, &mut buffer);

        // 添加别名补全支持
        self.append_alias_completion(&mut buffer, command_name)?;

        // 添加动态补全支持
        if self.enable_dynamic_completion {
            self.append_dynamic_completion(&mut buffer, command_name)?;
        }

        let shell_type_str = self.shell.to_string();
        let filename = get_completion_filename(&shell_type_str, command_name)?;
        let output_file = self.output_dir.join(&filename);

        FileWriter::new(&output_file).write_bytes(&buffer).wrap_err_with(|| {
            format!(
                "Failed to write completion file: {} (command: {}, shell: {})",
                output_file.display(),
                command_name,
                self.shell
            )
        })?;

        Ok(())
    }

    /// 追加别名补全支持到补全脚本
    ///
    /// 为 zsh 和 bash 添加自定义补全函数，支持别名展开后的补全。
    /// 在生成的补全脚本后追加别名补全逻辑。
    fn append_alias_completion(&self, buffer: &mut Vec<u8>, command_name: &str) -> Result<()> {
        // 加载别名配置
        let aliases = match AliasManager::load() {
            Ok(aliases) => aliases,
            Err(_) => {
                // 如果加载失败（配置文件不存在等），跳过别名补全
                return Ok(());
            }
        };

        if aliases.is_empty() {
            return Ok(());
        }

        // 根据 shell 类型生成不同的别名补全代码
        let alias_completion = match self.shell {
            ClapShell::Zsh => self.generate_zsh_alias_completion(&aliases, command_name),
            ClapShell::Bash => self.generate_bash_alias_completion(&aliases, command_name),
            _ => {
                // 其他 shell 暂不支持别名补全
                return Ok(());
            }
        };

        buffer.extend_from_slice(b"\n\n");
        buffer.extend_from_slice(alias_completion.as_bytes());

        Ok(())
    }

    /// 生成 zsh 别名补全函数
    ///
    /// 在 zsh 补全脚本中添加别名展开支持。
    /// 当检测到第一个参数是别名时，展开别名并基于展开后的命令提供补全。
    ///
    /// 注意：这个函数在补全脚本的末尾追加，会包装原始的 `_workflow` 函数。
    /// 使用函数别名来保存原始函数，避免无限递归。
    fn generate_zsh_alias_completion(
        &self,
        aliases: &std::collections::HashMap<String, String>,
        _command_name: &str,
    ) -> String {
        let mut code = String::from("\n# Alias completion support\n");
        code.push_str("# This code is appended after the clap-generated completion script\n");
        code.push_str("# It wraps the original _workflow function to handle alias expansion\n\n");
        code.push_str("# Save the original _workflow function before we override it\n");
        code.push_str("functions[_workflow_orig]=$functions[_workflow]\n\n");
        code.push_str("# Override _workflow to handle aliases\n");
        code.push_str("_workflow() {\n");
        code.push_str("  # Check if first argument (after command name) is an alias\n");
        code.push_str("  if [[ ${#words[@]} -ge 2 ]]; then\n");
        code.push_str("    local first_arg=${words[2]}\n");
        code.push_str("    case \"$first_arg\" in\n");

        // 为每个别名生成 case 分支
        for (alias, command) in aliases {
            code.push_str(&format!("      {})\n", alias));
            code.push_str("        # Expand alias and rebuild words array\n");
            // 将别名展开为命令，并分割为多个词
            let command_parts: Vec<&str> = command.split_whitespace().collect();
            if command_parts.len() == 1 {
                code.push_str(&format!("        words[2]=\"{}\"\n", command_parts[0]));
            } else {
                // 多个词：需要重新构建 words 数组
                code.push_str("        # Replace alias with expanded command parts\n");
                // 使用 ${words[3,-1]} 获取从索引 3 开始到最后一个元素的所有元素
                code.push_str(&format!(
                    "        words=(${{words[1]}} {} ${{words[3,-1]}})\n",
                    command_parts.join(" ")
                ));
                code.push_str(&format!(
                    "        CURRENT=$((CURRENT + {} - 1))\n",
                    command_parts.len()
                ));
            }
            code.push_str("        ;;\n");
        }

        code.push_str("    esac\n");
        code.push_str("  fi\n");
        code.push_str("  # Call original completion function with (possibly expanded) words\n");
        code.push_str("  _workflow_orig \"$@\"\n");
        code.push_str("}\n");

        code
    }

    /// 生成 bash 别名补全函数
    ///
    /// 在 bash 补全脚本中添加别名展开支持。
    /// 当检测到第一个参数是别名时，展开别名并基于展开后的命令提供补全。
    ///
    /// 注意：这个函数在补全脚本的末尾追加，会包装原始的 `_workflow` 函数。
    /// 使用函数别名来保存原始函数，避免无限递归。
    fn generate_bash_alias_completion(
        &self,
        aliases: &std::collections::HashMap<String, String>,
        _command_name: &str,
    ) -> String {
        let mut code = String::from("\n# Alias completion support\n");
        code.push_str("# This code is appended after the clap-generated completion script\n");
        code.push_str("# It wraps the original _workflow function to handle alias expansion\n\n");
        code.push_str("# Save the original _workflow function before we override it\n");
        code.push_str("# Use eval to capture the function definition\n");
        code.push_str("eval \"_workflow_orig() { $(declare -f _workflow | sed '1d;$d') }\"\n\n");
        code.push_str("# Override _workflow to handle aliases\n");
        code.push_str("_workflow() {\n");
        code.push_str("  # Check if first argument (after command name) is an alias\n");
        code.push_str("  if [[ ${#COMP_WORDS[@]} -ge 2 ]]; then\n");
        code.push_str("    local first_arg=${COMP_WORDS[1]}\n");
        code.push_str("    case \"$first_arg\" in\n");

        // 为每个别名生成 case 分支
        for (alias, command) in aliases {
            code.push_str(&format!("      {})\n", alias));
            code.push_str("        # Expand alias and rebuild COMP_WORDS array\n");
            // 将别名展开为命令，并分割为多个词
            let command_parts: Vec<&str> = command.split_whitespace().collect();
            if command_parts.len() == 1 {
                code.push_str(&format!("        COMP_WORDS[1]=\"{}\"\n", command_parts[0]));
            } else {
                // 多个词：需要重新构建 COMP_WORDS 数组
                code.push_str("        # Rebuild COMP_WORDS with expanded command\n");
                code.push_str("        local new_words=(\"${COMP_WORDS[0]}\"");
                for part in &command_parts {
                    code.push_str(&format!(" \"{}\"", part));
                }
                code.push_str(" \"${COMP_WORDS[@]:2}\")\n");
                code.push_str("        COMP_WORDS=(\"${new_words[@]}\")\n");
                code.push_str(&format!(
                    "        COMP_CWORD=$((COMP_CWORD + {} - 1))\n",
                    command_parts.len()
                ));
            }
            code.push_str("        break\n");
            code.push_str("        ;;\n");
        }

        code.push_str("    esac\n");
        code.push_str("  fi\n");
        code.push_str(
            "  # Call original completion function with (possibly expanded) COMP_WORDS\n",
        );
        code.push_str("  _workflow_orig \"$@\"\n");
        code.push_str("}\n");

        code
    }

    /// 追加动态补全支持到补全脚本
    ///
    /// 为支持的 shell 添加动态补全功能，包括：
    /// - 分支名补全
    /// - PR ID 补全
    /// - JIRA ticket ID 补全
    /// - 性能优化（缓存、超时控制）
    fn append_dynamic_completion(&self, buffer: &mut Vec<u8>, command_name: &str) -> Result<()> {
        let dynamic_completion = match self.shell {
            ClapShell::Zsh => self.generate_zsh_dynamic_completion(command_name),
            ClapShell::Bash => self.generate_bash_dynamic_completion(command_name),
            _ => {
                // 其他 shell 暂不支持动态补全
                return Ok(());
            }
        };

        buffer.extend_from_slice(b"\n\n");
        buffer.extend_from_slice(dynamic_completion.as_bytes());

        Ok(())
    }

    /// 生成 zsh 动态补全函数
    ///
    /// 添加动态值补全支持，包括分支名、PR ID 等。
    /// 包含性能优化：缓存、超时控制、错误处理。
    fn generate_zsh_dynamic_completion(&self, _command_name: &str) -> String {
        let mut code = String::from("\n# Dynamic completion support\n");
        code.push_str("# This provides dynamic completion for branch names, PR IDs, etc.\n");
        code.push_str(
            "# Includes performance optimizations: caching, timeouts, error handling\n\n",
        );

        if self.enable_performance_optimization {
            code.push_str("# Performance optimization: cache directory\n");
            code.push_str(
                "typeset -g _WORKFLOW_CACHE_DIR=\"${HOME}/.workflow/.completion_cache\"\n",
            );
            code.push_str("typeset -g _WORKFLOW_CACHE_TTL=300  # 5 minutes\n\n");

            code.push_str("# Ensure cache directory exists\n");
            code.push_str("_workflow_ensure_cache_dir() {\n");
            code.push_str("  [[ ! -d \"$_WORKFLOW_CACHE_DIR\" ]] && mkdir -p \"$_WORKFLOW_CACHE_DIR\" 2>/dev/null\n");
            code.push_str("}\n\n");

            code.push_str("# Check if cache file is valid (not expired)\n");
            code.push_str("_workflow_is_cache_valid() {\n");
            code.push_str("  local cache_file=\"$1\"\n");
            code.push_str("  [[ -f \"$cache_file\" ]] || return 1\n");
            code.push_str("  local cache_time=$(stat -f %m \"$cache_file\" 2>/dev/null || stat -c %Y \"$cache_file\" 2>/dev/null)\n");
            code.push_str("  local current_time=$(date +%s)\n");
            code.push_str("  (( current_time - cache_time < _WORKFLOW_CACHE_TTL ))\n");
            code.push_str("}\n\n");
        }

        // 分支名补全
        code.push_str("# Dynamic branch name completion\n");
        code.push_str("_workflow_complete_branches() {\n");
        if self.enable_performance_optimization {
            code.push_str("  _workflow_ensure_cache_dir\n");
            code.push_str("  local cache_file=\"$_WORKFLOW_CACHE_DIR/branches\"\n");
            code.push_str("  \n");
            code.push_str("  # Try to use cached branches if valid\n");
            code.push_str("  if _workflow_is_cache_valid \"$cache_file\"; then\n");
            code.push_str("    local branches=(${(f)\"$(<$cache_file)\"})\n");
            code.push_str("    _describe 'branches' branches\n");
            code.push_str("    return\n");
            code.push_str("  fi\n");
            code.push_str("  \n");
        }
        code.push_str("  # Get branches with timeout and error handling\n");
        code.push_str("  local branches\n");
        if self.enable_performance_optimization {
            code.push_str("  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then\n");
            code.push_str("    # Cache the results\n");
            code.push_str("    echo \"$branches\" > \"$cache_file\" 2>/dev/null\n");
        } else {
            code.push_str(
                "  if branches=$(git branch --format='%(refname:short)' 2>/dev/null); then\n",
            );
        }
        code.push_str("    local branch_array=(${(f)branches})\n");
        code.push_str("    _describe 'branches' branch_array\n");
        code.push_str("  fi\n");
        code.push_str("}\n\n");

        // PR ID 补全
        code.push_str("# Dynamic PR ID completion\n");
        code.push_str("_workflow_complete_pr_ids() {\n");
        if self.enable_performance_optimization {
            code.push_str("  _workflow_ensure_cache_dir\n");
            code.push_str("  local cache_file=\"$_WORKFLOW_CACHE_DIR/pr_ids\"\n");
            code.push_str("  \n");
            code.push_str("  # Try to use cached PR IDs if valid\n");
            code.push_str("  if _workflow_is_cache_valid \"$cache_file\"; then\n");
            code.push_str("    local pr_ids=(${(f)\"$(<$cache_file)\"})\n");
            code.push_str("    _describe 'PR IDs' pr_ids\n");
            code.push_str("    return\n");
            code.push_str("  fi\n");
            code.push_str("  \n");
        }
        code.push_str("  # Get recent PR IDs with timeout\n");
        code.push_str("  local pr_ids\n");
        if self.enable_performance_optimization {
            code.push_str("  if command -v gh >/dev/null 2>&1 && pr_ids=$(timeout 3s gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null); then\n");
            code.push_str("    # Cache the results\n");
            code.push_str("    echo \"$pr_ids\" > \"$cache_file\" 2>/dev/null\n");
        } else {
            code.push_str("  if command -v gh >/dev/null 2>&1 && pr_ids=$(gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null); then\n");
        }
        code.push_str("    local pr_array=(${(f)pr_ids})\n");
        code.push_str("    _describe 'PR IDs' pr_array\n");
        code.push_str("  fi\n");
        code.push_str("}\n\n");

        // JIRA ticket 补全
        code.push_str("# Dynamic JIRA ticket completion\n");
        code.push_str("_workflow_complete_jira_tickets() {\n");
        code.push_str("  # Extract JIRA tickets from recent git commits\n");
        code.push_str("  local jira_tickets\n");
        if self.enable_performance_optimization {
            code.push_str("  if jira_tickets=$(timeout 2s git log --oneline -20 | grep -oE '[A-Z]+-[0-9]+' | sort -u 2>/dev/null); then\n");
        } else {
            code.push_str("  if jira_tickets=$(git log --oneline -20 | grep -oE '[A-Z]+-[0-9]+' | sort -u 2>/dev/null); then\n");
        }
        code.push_str("    local jira_array=(${(f)jira_tickets})\n");
        code.push_str("    _describe 'JIRA tickets' jira_array\n");
        code.push_str("  fi\n");
        code.push_str("}\n\n");

        // 集成到主补全函数
        code.push_str("# Enhanced workflow completion with dynamic values\n");
        code.push_str("# This wraps the existing _workflow function to add dynamic completion\n");
        code.push_str("if (( $+functions[_workflow_orig] )); then\n");
        code.push_str("  # Save the current _workflow function\n");
        code.push_str("  functions[_workflow_base]=$functions[_workflow]\n");
        code.push_str("  \n");
        code.push_str("  # Override with dynamic completion support\n");
        code.push_str("  _workflow() {\n");
        code.push_str("    # Check context for dynamic completion\n");
        code.push_str("    local context state line\n");
        code.push_str("    local -A opt_args\n");
        code.push_str("    \n");
        code.push_str("    # Analyze current command context\n");
        code.push_str("    if [[ ${#words[@]} -ge 3 ]]; then\n");
        code.push_str("      case \"${words[2]}\" in\n");
        code.push_str("        branch)\n");
        code.push_str("          case \"${words[3]}\" in\n");
        code.push_str("            switch|delete|rename)\n");
        code.push_str("              _workflow_complete_branches\n");
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("        pr)\n");
        code.push_str("          case \"${words[3]}\" in\n");
        code.push_str("            close|merge|approve|comment|status|update|sync|rebase)\n");
        code.push_str("              _workflow_complete_pr_ids\n");
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("        jira)\n");
        code.push_str("          case \"${words[3]}\" in\n");
        code.push_str("            info|related|changelog|comment|comments|attachments)\n");
        code.push_str("              _workflow_complete_jira_tickets\n");
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("      esac\n");
        code.push_str("    fi\n");
        code.push_str("    \n");
        code.push_str("    # Fall back to base completion\n");
        code.push_str("    _workflow_base \"$@\"\n");
        code.push_str("  }\n");
        code.push_str("fi\n");

        code
    }

    /// 生成 bash 动态补全函数
    ///
    /// 为 bash 添加动态补全支持，功能与 zsh 版本类似。
    fn generate_bash_dynamic_completion(&self, _command_name: &str) -> String {
        let mut code = String::from("\n# Dynamic completion support for bash\n");
        code.push_str("# This provides dynamic completion for branch names, PR IDs, etc.\n\n");

        if self.enable_performance_optimization {
            code.push_str("# Performance optimization: cache settings\n");
            code.push_str("_WORKFLOW_CACHE_DIR=\"${HOME}/.workflow/.completion_cache\"\n");
            code.push_str("_WORKFLOW_CACHE_TTL=300  # 5 minutes\n\n");

            code.push_str("# Ensure cache directory exists\n");
            code.push_str("_workflow_ensure_cache_dir() {\n");
            code.push_str("  [[ ! -d \"$_WORKFLOW_CACHE_DIR\" ]] && mkdir -p \"$_WORKFLOW_CACHE_DIR\" 2>/dev/null\n");
            code.push_str("}\n\n");

            code.push_str("# Check if cache file is valid\n");
            code.push_str("_workflow_is_cache_valid() {\n");
            code.push_str("  local cache_file=\"$1\"\n");
            code.push_str("  [[ -f \"$cache_file\" ]] || return 1\n");
            code.push_str("  local cache_time=$(stat -c %Y \"$cache_file\" 2>/dev/null)\n");
            code.push_str("  local current_time=$(date +%s)\n");
            code.push_str("  (( current_time - cache_time < _WORKFLOW_CACHE_TTL ))\n");
            code.push_str("}\n\n");
        }

        // 分支名补全
        code.push_str("# Get branch names for completion\n");
        code.push_str("_workflow_get_branches() {\n");
        if self.enable_performance_optimization {
            code.push_str("  _workflow_ensure_cache_dir\n");
            code.push_str("  local cache_file=\"$_WORKFLOW_CACHE_DIR/branches\"\n");
            code.push_str("  \n");
            code.push_str("  # Use cached branches if valid\n");
            code.push_str("  if _workflow_is_cache_valid \"$cache_file\"; then\n");
            code.push_str("    cat \"$cache_file\" 2>/dev/null\n");
            code.push_str("    return\n");
            code.push_str("  fi\n");
            code.push_str("  \n");
            code.push_str("  # Get and cache branches\n");
            code.push_str("  local branches\n");
            code.push_str("  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then\n");
            code.push_str("    echo \"$branches\" | tee \"$cache_file\" 2>/dev/null\n");
            code.push_str("  fi\n");
        } else {
            code.push_str("  git branch --format='%(refname:short)' 2>/dev/null\n");
        }
        code.push_str("}\n\n");

        // PR ID 补全
        code.push_str("# Get PR IDs for completion\n");
        code.push_str("_workflow_get_pr_ids() {\n");
        if self.enable_performance_optimization {
            code.push_str("  _workflow_ensure_cache_dir\n");
            code.push_str("  local cache_file=\"$_WORKFLOW_CACHE_DIR/pr_ids\"\n");
            code.push_str("  \n");
            code.push_str("  # Use cached PR IDs if valid\n");
            code.push_str("  if _workflow_is_cache_valid \"$cache_file\"; then\n");
            code.push_str("    cat \"$cache_file\" 2>/dev/null\n");
            code.push_str("    return\n");
            code.push_str("  fi\n");
            code.push_str("  \n");
            code.push_str("  # Get and cache PR IDs\n");
            code.push_str("  if command -v gh >/dev/null 2>&1; then\n");
            code.push_str("    timeout 3s gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null | tee \"$cache_file\" 2>/dev/null\n");
            code.push_str("  fi\n");
        } else {
            code.push_str("  if command -v gh >/dev/null 2>&1; then\n");
            code.push_str(
                "    gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null\n",
            );
            code.push_str("  fi\n");
        }
        code.push_str("}\n\n");

        // JIRA ticket 补全
        code.push_str("# Get JIRA tickets for completion\n");
        code.push_str("_workflow_get_jira_tickets() {\n");
        if self.enable_performance_optimization {
            code.push_str("  timeout 2s git log --oneline -20 | grep -oE '[A-Z]+-[0-9]+' | sort -u 2>/dev/null\n");
        } else {
            code.push_str(
                "  git log --oneline -20 | grep -oE '[A-Z]+-[0-9]+' | sort -u 2>/dev/null\n",
            );
        }
        code.push_str("}\n\n");

        // 增强的主补全函数
        code.push_str("# Enhanced workflow completion function\n");
        code.push_str("if declare -F _workflow_orig >/dev/null; then\n");
        code.push_str("  # Save the original function\n");
        code.push_str("  eval \"_workflow_base() { $(declare -f _workflow | sed '1d;$d') }\"\n");
        code.push_str("  \n");
        code.push_str("  # Override with dynamic completion\n");
        code.push_str("  _workflow() {\n");
        code.push_str("    local cur=\"${COMP_WORDS[COMP_CWORD]}\"\n");
        code.push_str("    \n");
        code.push_str("    # Check for dynamic completion contexts\n");
        code.push_str("    if [[ ${#COMP_WORDS[@]} -ge 3 ]]; then\n");
        code.push_str("      case \"${COMP_WORDS[1]}\" in\n");
        code.push_str("        branch)\n");
        code.push_str("          case \"${COMP_WORDS[2]}\" in\n");
        code.push_str("            switch|delete|rename)\n");
        code.push_str(
            "              COMPREPLY=($(compgen -W \"$(_workflow_get_branches)\" -- \"$cur\"))\n",
        );
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("        pr)\n");
        code.push_str("          case \"${COMP_WORDS[2]}\" in\n");
        code.push_str("            close|merge|approve|comment|status|update|sync|rebase)\n");
        code.push_str(
            "              COMPREPLY=($(compgen -W \"$(_workflow_get_pr_ids)\" -- \"$cur\"))\n",
        );
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("        jira)\n");
        code.push_str("          case \"${COMP_WORDS[2]}\" in\n");
        code.push_str("            info|related|changelog|comment|comments|attachments)\n");
        code.push_str("              COMPREPLY=($(compgen -W \"$(_workflow_get_jira_tickets)\" -- \"$cur\"))\n");
        code.push_str("              return\n");
        code.push_str("              ;;\n");
        code.push_str("          esac\n");
        code.push_str("          ;;\n");
        code.push_str("      esac\n");
        code.push_str("    fi\n");
        code.push_str("    \n");
        code.push_str("    # Fall back to base completion\n");
        code.push_str("    _workflow_base \"$@\"\n");
        code.push_str("  }\n");
        code.push_str("fi\n");

        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use crate::completion::helpers::get_completion_filename;
    use color_eyre::Result;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    use std::fs;

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
    const BRANCH_SUBCOMMANDS: &[&str] = &[
        "ignore", "create", "rename", "switch", "sync", "delete", "push",
    ];

    /// Commit 子命令列表
    const COMMIT_SUBCOMMANDS: &[&str] = &["amend", "reword", "squash"];

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

    // ==================== CLI Command Structure Tests ====================

    /// 测试 CLI 包含所有顶级命令
    ///
    /// ## 测试目的
    /// 验证 CLI 命令结构包含所有预期的顶级命令。
    ///
    /// ## 测试场景
    /// 1. 获取 CLI 命令结构
    /// 2. 检查所有预期的命令是否存在
    /// 3. 验证所有预期的命令都存在
    ///
    /// ## 预期结果
    /// - 所有预期的顶级命令都存在
    #[test]
    fn test_cli_contains_all_top_level_commands_returns_true() {
        // Arrange: 准备 CLI 命令结构
        let cmd = Cli::command();
        let subcommands: Vec<String> =
            cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();

        // Act: 检查所有预期的命令是否存在
        let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

        // Assert: 验证所有预期的命令都存在
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

    // ==================== Subcommand Completeness Tests ====================

    /// 测试 PR 子命令完整性
    ///
    /// ## 测试目的
    /// 验证 PR 命令包含所有预期的子命令。
    ///
    /// ## 测试场景
    /// 1. 获取 PR 命令结构
    /// 2. 获取 PR 子命令列表
    /// 3. 验证所有预期的子命令都存在且数量匹配
    ///
    /// ## 预期结果
    /// - 所有预期的 PR 子命令都存在且数量匹配
    #[test]
    fn test_pr_subcommands_completeness_with_all_subcommands_returns_true() -> Result<()> {
        // Arrange: 准备 PR 命令结构
        let cmd = Cli::command();
        let pr_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "pr")
            .ok_or_else(|| color_eyre::eyre::eyre!("pr command should exist"))?;

        // Act: 获取 PR 子命令列表
        let subcommands: Vec<String> =
            pr_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

        // Assert: 验证所有预期的子命令都存在且数量匹配
        for expected_subcmd in PR_SUBCOMMANDS {
            assert!(
                subcommand_set.contains(*expected_subcmd),
                "Missing PR subcommand: {}",
                expected_subcmd
            );
        }
        assert_eq!(
            subcommands.len(),
            PR_SUBCOMMANDS.len(),
            "PR subcommands count mismatch"
        );
        println!(
            "Found {} PR subcommands: {:?}",
            subcommands.len(),
            subcommands
        );
        Ok(())
    }

    /// 测试 Commit 子命令完整性
    ///
    /// ## 测试目的
    /// 验证 Commit 命令包含所有预期的子命令。
    ///
    /// ## 测试场景
    /// 1. 获取 Commit 命令结构
    /// 2. 获取 Commit 子命令列表
    /// 3. 验证所有预期的子命令都存在且数量匹配
    ///
    /// ## 预期结果
    /// - 所有预期的 Commit 子命令都存在且数量匹配
    #[test]
    fn test_commit_subcommands_completeness_with_all_subcommands_returns_true() -> Result<()> {
        // Arrange: 准备 Commit 命令结构
        let cmd = Cli::command();
        let commit_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "commit")
            .ok_or_else(|| color_eyre::eyre::eyre!("commit command should exist"))?;

        // Act: 获取 Commit 子命令列表
        let subcommands: Vec<String> =
            commit_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        let subcommand_set: HashSet<String> = subcommands.iter().cloned().collect();

        // Assert: 验证所有预期的子命令都存在且数量匹配
        for expected_subcmd in COMMIT_SUBCOMMANDS {
            assert!(
                subcommand_set.contains(*expected_subcmd),
                "Missing Commit subcommand: {}",
                expected_subcmd
            );
        }
        assert_eq!(
            subcommands.len(),
            COMMIT_SUBCOMMANDS.len(),
            "Commit subcommands count mismatch"
        );
        println!(
            "Found {} Commit subcommands: {:?}",
            subcommands.len(),
            subcommands
        );
        Ok(())
    }

    // ==================== Completion Generation Tests ====================

    /// 测试为所有 shell 类型生成补全文件
    ///
    /// ## 测试目的
    /// 验证 CompletionGenerator 能够为所有支持的 shell 类型生成补全脚本文件。
    ///
    /// ## 测试场景
    /// 1. 准备临时输出目录和 shell 类型列表
    /// 2. 为每个 shell 类型生成补全脚本
    /// 3. 验证文件已生成且不为空
    ///
    /// ## 预期结果
    /// - 所有 shell 类型的补全文件都已生成且不为空
    #[test]
    fn test_completion_generation_with_all_shells_generates_files() -> Result<()> {
        // Arrange: 准备临时输出目录和 shell 类型列表
        let output_dir = std::env::temp_dir().join("workflow_completion_test");
        fs::create_dir_all(&output_dir).map_err(|e| {
            color_eyre::eyre::eyre!(
                "{}: {}",
                crate::base::constants::errors::file_operations::CREATE_TEMP_DIR_FAILED,
                e
            )
        })?;
        let shell_types = ["zsh", "bash", "fish", "powershell", "elvish"];

        // Act: 为每个 shell 类型生成补全脚本
        for shell_type in &shell_types {
            let generator = CompletionGenerator::new(
                Some(shell_type.to_string()),
                Some(output_dir.to_string_lossy().to_string()),
            )
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to create generator for {}: {}", shell_type, e)
            })?;

            generator.generate_all().map_err(|e| {
                color_eyre::eyre::eyre!("Failed to generate completion for {}: {}", shell_type, e)
            })?;

            // Assert: 验证文件已生成且不为空
            let filename = get_completion_filename(shell_type, "workflow").map_err(|e| {
                color_eyre::eyre::eyre!("Failed to get filename for {}: {}", shell_type, e)
            })?;
            let file_path = output_dir.join(&filename);
            assert!(
                file_path.exists(),
                "Completion file not generated for {}: {}",
                shell_type,
                file_path.display()
            );
            let content = fs::read_to_string(&file_path).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to read completion file for {}: {}", shell_type, e)
            })?;
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
        Ok(())
    }

    /// 测试 zsh 补全脚本包含所有命令
    ///
    /// ## 测试目的
    /// 验证生成的 zsh 补全脚本包含 workflow 命令。
    ///
    /// ## 测试场景
    /// 1. 生成 zsh 补全脚本
    /// 2. 读取补全脚本内容
    /// 3. 验证包含 workflow 命令
    ///
    /// ## 预期结果
    /// - zsh 补全脚本包含 "workflow" 命令
    #[test]
    fn test_zsh_completion_contains_all_commands_with_valid_content_returns_true() -> Result<()> {
        // Arrange: 准备临时输出目录
        let output_dir = std::env::temp_dir().join("workflow_zsh_test");
        fs::create_dir_all(&output_dir).map_err(|e| {
            color_eyre::eyre::eyre!(
                "{}: {}",
                crate::base::constants::errors::file_operations::CREATE_TEMP_DIR_FAILED,
                e
            )
        })?;

        // Act: 生成 zsh 补全脚本
        let generator = CompletionGenerator::new(
            Some("zsh".to_string()),
            Some(output_dir.to_string_lossy().to_string()),
        )
        .map_err(|e| {
            color_eyre::eyre::eyre!(
                "{}: {}",
                crate::base::constants::errors::generator_creation::CREATE_ZSH_GENERATOR_FAILED,
                e
            )
        })?;
        generator
            .generate_all()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to generate zsh completion: {}", e))?;

        // Assert: 验证补全脚本包含 workflow 命令
        let filename = get_completion_filename("zsh", "workflow")
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get filename: {}", e))?;
        let file_path = output_dir.join(&filename);
        let content = fs::read_to_string(&file_path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to read completion file: {}", e))?;
        assert!(
            content.contains("workflow"),
            "Zsh completion should contain 'workflow'"
        );
        println!("Zsh completion file size: {} bytes", content.len());
        println!(
            "Zsh completion contains 'workflow': {}",
            content.contains("workflow")
        );
        Ok(())
    }

    /// 测试 bash 补全脚本包含所有命令
    ///
    /// ## 测试目的
    /// 验证生成的 bash 补全脚本包含 workflow 命令。
    ///
    /// ## 测试场景
    /// 1. 生成 bash 补全脚本
    /// 2. 读取补全脚本内容
    /// 3. 验证包含 workflow 命令
    ///
    /// ## 预期结果
    /// - bash 补全脚本包含 "workflow" 命令
    #[test]
    fn test_bash_completion_contains_all_commands_with_valid_content_returns_true() -> Result<()> {
        // Arrange: 准备临时输出目录
        let output_dir = std::env::temp_dir().join("workflow_bash_test");
        fs::create_dir_all(&output_dir).map_err(|e| {
            color_eyre::eyre::eyre!(
                "{}: {}",
                crate::base::constants::errors::file_operations::CREATE_TEMP_DIR_FAILED,
                e
            )
        })?;

        // Act: 生成 bash 补全脚本
        let generator = CompletionGenerator::new(
            Some("bash".to_string()),
            Some(output_dir.to_string_lossy().to_string()),
        )
        .map_err(|e| color_eyre::eyre::eyre!("Failed to create bash generator: {}", e))?;
        generator
            .generate_all()
            .map_err(|e| color_eyre::eyre::eyre!("Failed to generate bash completion: {}", e))?;

        // Assert: 验证补全脚本包含 workflow 命令
        let filename = get_completion_filename("bash", "workflow")
            .map_err(|e| color_eyre::eyre::eyre!("Failed to get filename: {}", e))?;
        let file_path = output_dir.join(&filename);
        let content = fs::read_to_string(&file_path)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to read completion file: {}", e))?;
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
        Ok(())
    }

    /// 测试所有子命令的完整性
    ///
    /// ## 测试目的
    /// 验证所有带子命令的命令都包含预期的子命令。
    ///
    /// ## 测试场景
    /// 1. 获取 CLI 命令结构
    /// 2. 验证所有带子命令的命令（PR、Jira、GitHub、LLM、Branch、Commit、Proxy、Log、Completion、Stash、Repo、Alias、Tag）
    /// 3. 验证所有子命令数量匹配
    ///
    /// ## 预期结果
    /// - 所有带子命令的命令都包含预期的子命令
    #[test]
    fn test_all_subcommands_completeness_with_all_commands_returns_true() -> Result<()> {
        // Arrange: 准备 CLI 命令结构
        let cmd = Cli::command();

        // Act & Assert: 验证所有子命令的完整性
        // 验证 PR 子命令
        let pr_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "pr")
            .ok_or_else(|| color_eyre::eyre::eyre!("pr command should exist"))?;
        let pr_subcommands: Vec<String> =
            pr_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(pr_subcommands.len(), PR_SUBCOMMANDS.len());

        // 验证 Jira Log 子命令（log 现在是 jira 的子命令）
        let jira_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "jira")
            .ok_or_else(|| color_eyre::eyre::eyre!("jira command should exist"))?;
        let jira_log_cmd = jira_cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "log")
            .ok_or_else(|| color_eyre::eyre::eyre!("jira log command should exist"))?;
        let log_subcommands: Vec<String> =
            jira_log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(log_subcommands.len(), LOG_SUBCOMMANDS.len());

        // 验证 Jira 子命令
        let jira_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "jira")
            .ok_or_else(|| color_eyre::eyre::eyre!("jira command should exist"))?;
        let jira_subcommands: Vec<String> =
            jira_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(jira_subcommands.len(), JIRA_SUBCOMMANDS.len());

        // 验证 GitHub 子命令
        let github_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "github")
            .ok_or_else(|| color_eyre::eyre::eyre!("github command should exist"))?;
        let github_subcommands: Vec<String> =
            github_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(github_subcommands.len(), GITHUB_SUBCOMMANDS.len());

        // 验证 LLM 子命令
        let llm_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "llm")
            .ok_or_else(|| color_eyre::eyre::eyre!("llm command should exist"))?;
        let llm_subcommands: Vec<String> =
            llm_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(llm_subcommands.len(), LLM_SUBCOMMANDS.len());

        // 验证 Branch 子命令
        let branch_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "branch")
            .ok_or_else(|| color_eyre::eyre::eyre!("branch command should exist"))?;
        let branch_subcommands: Vec<String> =
            branch_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(branch_subcommands.len(), BRANCH_SUBCOMMANDS.len());

        // 验证 Commit 子命令
        let commit_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "commit")
            .ok_or_else(|| color_eyre::eyre::eyre!("commit command should exist"))?;
        let commit_subcommands: Vec<String> =
            commit_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(commit_subcommands.len(), COMMIT_SUBCOMMANDS.len());

        // 验证 Proxy 子命令
        let proxy_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "proxy")
            .ok_or_else(|| color_eyre::eyre::eyre!("proxy command should exist"))?;
        let proxy_subcommands: Vec<String> =
            proxy_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(proxy_subcommands.len(), PROXY_SUBCOMMANDS.len());

        // 验证 Log 子命令
        let log_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "log")
            .ok_or_else(|| color_eyre::eyre::eyre!("log command should exist"))?;
        let log_subcommands: Vec<String> =
            log_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(log_subcommands.len(), LOG_LEVEL_SUBCOMMANDS.len());

        // 验证 Completion 子命令
        let completion_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "completion")
            .ok_or_else(|| color_eyre::eyre::eyre!("completion command should exist"))?;
        let completion_subcommands: Vec<String> =
            completion_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(completion_subcommands.len(), COMPLETION_SUBCOMMANDS.len());

        // 验证 Stash 子命令
        let stash_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "stash")
            .ok_or_else(|| color_eyre::eyre::eyre!("stash command should exist"))?;
        let stash_subcommands: Vec<String> =
            stash_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(stash_subcommands.len(), STASH_SUBCOMMANDS.len());

        // 验证 Repo 子命令
        let repo_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "repo")
            .ok_or_else(|| color_eyre::eyre::eyre!("repo command should exist"))?;
        let repo_subcommands: Vec<String> =
            repo_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(repo_subcommands.len(), REPO_SUBCOMMANDS.len());

        // 验证 Alias 子命令
        let alias_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "alias")
            .ok_or_else(|| color_eyre::eyre::eyre!("alias command should exist"))?;
        let alias_subcommands: Vec<String> =
            alias_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(alias_subcommands.len(), ALIAS_SUBCOMMANDS.len());

        // 验证 Tag 子命令
        let tag_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "tag")
            .ok_or_else(|| color_eyre::eyre::eyre!("tag command should exist"))?;
        let tag_subcommands: Vec<String> =
            tag_cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        assert_eq!(tag_subcommands.len(), TAG_SUBCOMMANDS.len());

        println!("All subcommands verified successfully!");
        Ok(())
    }

    /// 测试补全文件名生成（所有 shell 类型）
    ///
    /// ## 测试目的
    /// 验证 get_completion_filename() 为所有 shell 类型生成正确的文件名。
    ///
    /// ## 测试场景
    /// 1. 准备 shell 类型和预期文件名
    /// 2. 为每个 shell 类型生成文件名
    /// 3. 验证文件名正确
    ///
    /// ## 预期结果
    /// - 所有 shell 类型的文件名都正确
    #[test]
    fn test_completion_filename_generation_with_all_shells_returns_correct_filenames() -> Result<()>
    {
        // Arrange: 准备 shell 类型和预期文件名
        let shell_types = ["zsh", "bash", "fish", "powershell", "elvish"];
        let expected_filenames = [
            "_workflow",
            "workflow.bash",
            "workflow.fish",
            "_workflow.ps1",
            "workflow.elv",
        ];

        // Act & Assert: 验证每个 shell 类型的文件名生成正确
        for (shell_type, expected_filename) in shell_types.iter().zip(expected_filenames.iter()) {
            let filename = get_completion_filename(shell_type, "workflow").map_err(|e| {
                color_eyre::eyre::eyre!("Failed to get filename for {}: {}", shell_type, e)
            })?;
            assert_eq!(
                &filename, expected_filename,
                "Wrong filename for {}: expected {}, got {}",
                shell_type, expected_filename, filename
            );
        }
        Ok(())
    }

    /// 测试 CLI 结构摘要
    ///
    /// ## 测试目的
    /// 验证 CLI 命令结构的完整性并输出摘要信息。
    ///
    /// ## 测试场景
    /// 1. 获取 CLI 命令结构
    /// 2. 统计所有命令和子命令
    /// 3. 验证基本完整性并输出摘要
    ///
    /// ## 预期结果
    /// - CLI 结构完整，至少包含10个顶级命令和20个子命令
    #[test]
    fn test_cli_structure_summary_with_all_commands_returns_summary() {
        // Arrange: 准备 CLI 命令结构
        let cmd = Cli::command();

        // Act: 统计所有命令和子命令
        let subcommands: Vec<String> =
            cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
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

        // Assert: 验证基本完整性并输出摘要
        println!("\n=== CLI Structure Summary ===");
        println!("Total top-level commands: {}", subcommands.len());
        println!("Commands: {:?}", subcommands);
        println!("Total subcommands: {}", total_subcommands);
        println!("=============================\n");
        assert!(
            subcommands.len() >= 10,
            "Should have at least 10 top-level commands"
        );
        assert!(
            total_subcommands >= 20,
            "Should have at least 20 subcommands"
        );
    }

    // ==================== Parameterized Shell Completion Tests ====================

    /// 测试所有 shell 类型的补全生成（参数化）
    ///
    /// ## 测试目的
    /// 验证 CompletionGenerator 能够为所有支持的 shell 类型生成补全脚本并验证文件名和内容。
    ///
    /// ## 测试场景
    /// 1. 准备临时输出目录和预期文件名
    /// 2. 为每个 shell 类型生成补全脚本
    /// 3. 验证文件名正确、文件存在、内容不为空且包含 workflow 命令
    ///
    /// ## 预期结果
    /// - 所有 shell 类型的补全文件都正确生成，文件名和内容都正确
    #[test]
    fn test_all_shell_types_completion_generation_with_all_shells_generates_files() -> Result<()> {
        // Arrange: 准备临时输出目录和 shell 类型列表
        let output_dir = std::env::temp_dir().join("workflow_all_shells_test");
        fs::create_dir_all(&output_dir).map_err(|e| {
            color_eyre::eyre::eyre!(
                "{}: {}",
                crate::base::constants::errors::file_operations::CREATE_TEMP_DIR_FAILED,
                e
            )
        })?;
        let expected_filenames = [
            "_workflow",
            "workflow.bash",
            "workflow.fish",
            "_workflow.ps1",
            "workflow.elv",
        ];

        // Act & Assert: 为每个 shell 类型生成并验证补全脚本
        for (shell_type, expected_filename) in SHELL_TYPES.iter().zip(expected_filenames.iter()) {
            println!("Testing {} completion generation...", shell_type);

            let generator = CompletionGenerator::new(
                Some(shell_type.to_string()),
                Some(output_dir.to_string_lossy().to_string()),
            )
            .map_err(|e| {
                color_eyre::eyre::eyre!("Failed to create generator for {}: {}", shell_type, e)
            })?;
            let result = generator.generate_all();
            assert!(
                result.is_ok(),
                "Failed to generate completion for {}: {:?}",
                shell_type,
                result.err()
            );

            let filename = get_completion_filename(shell_type, "workflow").map_err(|e| {
                color_eyre::eyre::eyre!("Failed to get filename for {}: {}", shell_type, e)
            })?;
            assert_eq!(
                &filename, expected_filename,
                "Wrong filename for {}: expected {}, got {}",
                shell_type, expected_filename, filename
            );

            let file_path = output_dir.join(&filename);
            assert!(
                file_path.exists(),
                "Completion file not generated for {}: {}",
                shell_type,
                file_path.display()
            );

            let content = fs::read_to_string(&file_path).map_err(|e| {
                color_eyre::eyre::eyre!("Failed to read completion file for {}: {}", shell_type, e)
            })?;
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
        Ok(())
    }

    /// 测试所有带子命令的命令完整性
    ///
    /// ## 测试目的
    /// 验证所有带子命令的命令都包含预期的子命令且数量匹配。
    ///
    /// ## 测试场景
    /// 1. 准备 CLI 命令结构和命令列表
    /// 2. 验证所有带子命令的命令
    /// 3. 验证每个命令的子命令都存在且数量匹配
    ///
    /// ## 预期结果
    /// - 所有带子命令的命令都包含预期的子命令且数量匹配
    #[test]
    fn test_all_commands_with_subcommands_with_all_commands_returns_true() -> Result<()> {
        // Arrange: 准备 CLI 命令结构和命令列表
        let cmd = Cli::command();
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

        // Act & Assert: 验证所有带子命令的命令完整性
        for (cmd_name, expected_subcommands) in &commands_with_subcommands {
            println!("Testing {} subcommands...", cmd_name);

            let subcommand = cmd
                .get_subcommands()
                .find(|sc| sc.get_name() == *cmd_name)
                .ok_or_else(|| color_eyre::eyre::eyre!("{} command should exist", cmd_name))?;
            let actual_subcommands: Vec<String> =
                subcommand.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
            let subcommand_set: HashSet<String> = actual_subcommands.iter().cloned().collect();

            for expected_subcmd in *expected_subcommands {
                assert!(
                    subcommand_set.contains(*expected_subcmd),
                    "Missing {} subcommand: {}",
                    cmd_name,
                    expected_subcmd
                );
            }
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
        Ok(())
    }

    /// 测试嵌套子命令完整性
    ///
    /// ## 测试目的
    /// 验证嵌套子命令（Jira Log 和 Branch Ignore）都包含预期的子命令。
    ///
    /// ## 测试场景
    /// 1. 获取 CLI 命令结构
    /// 2. 验证 Jira Log 嵌套子命令
    /// 3. 验证 Branch Ignore 嵌套子命令
    /// 4. 验证所有预期的子命令都存在且数量匹配
    ///
    /// ## 预期结果
    /// - 所有嵌套子命令都包含预期的子命令且数量匹配
    #[test]
    fn test_nested_subcommands_with_jira_log_and_branch_ignore_returns_true() -> Result<()> {
        // Arrange: 准备 CLI 命令结构
        let cmd = Cli::command();
        const BRANCH_IGNORE_SUBCOMMANDS: &[&str] = &["add", "remove", "list"];

        // Act & Assert: 验证 Jira Log 嵌套子命令
        let jira_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "jira")
            .ok_or_else(|| color_eyre::eyre::eyre!("jira command should exist"))?;
        let jira_log_cmd = jira_cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "log")
            .ok_or_else(|| color_eyre::eyre::eyre!("jira log command should exist"))?;
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

        // Act & Assert: 验证 Branch Ignore 嵌套子命令
        let branch_cmd = cmd
            .get_subcommands()
            .find(|sc| sc.get_name() == "branch")
            .ok_or_else(|| color_eyre::eyre::eyre!("branch command should exist"))?;
        let branch_ignore_cmd =
            branch_cmd
                .get_subcommands()
                .find(|sc| sc.get_name() == "ignore")
                .ok_or_else(|| color_eyre::eyre::eyre!("branch ignore command should exist"))?;
        let ignore_subcommands: Vec<String> = branch_ignore_cmd
            .get_subcommands()
            .map(|sc| sc.get_name().to_string())
            .collect();
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
        Ok(())
    }

    /// 测试顶级命令与常量同步
    ///
    /// ## 测试目的
    /// 验证 CLI 中的顶级命令与 TOP_LEVEL_COMMANDS 常量保持同步。
    ///
    /// ## 测试场景
    /// 1. 获取 CLI 命令结构
    /// 2. 获取实际命令列表
    /// 3. 验证所有命令都在常量列表中且数量匹配
    ///
    /// ## 预期结果
    /// - CLI 中的顶级命令与常量列表完全同步
    #[test]
    fn test_top_level_commands_sync_with_constants_returns_true() {
        // Arrange: 准备 CLI 命令结构和常量列表
        let cmd = Cli::command();

        // Act: 获取实际命令列表
        let actual_commands: Vec<String> =
            cmd.get_subcommands().map(|sc| sc.get_name().to_string()).collect();
        let expected_set: HashSet<&str> = TOP_LEVEL_COMMANDS.iter().copied().collect();
        let actual_set: HashSet<String> = actual_commands.iter().cloned().collect();

        // Assert: 验证所有命令都在常量列表中且数量匹配
        for actual_cmd in &actual_commands {
            assert!(
                expected_set.contains(actual_cmd.as_str()),
                "Command '{}' is missing from TOP_LEVEL_COMMANDS constant",
                actual_cmd
            );
        }
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
}
