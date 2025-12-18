//! Completion 脚本生成工具
//!
//! 提供生成各种 shell 的 completion 脚本文件的功能。

use std::path::PathBuf;

use clap::{Command, CommandFactory};
use clap_complete::{generate, shells::Shell as ClapShell};
use color_eyre::{eyre::WrapErr, Result};

use super::helpers::get_completion_filename;
use crate::base::alias::AliasManager;
use crate::base::settings::paths::Paths;
use crate::base::util::directory::DirectoryWalker;
use crate::base::util::file::FileWriter;

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
