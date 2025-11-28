//! Completion 脚本生成工具
//!
//! 提供生成各种 shell 的 completion 脚本文件的功能。

use crate::log_debug;
use crate::log_success;
use anyhow::{Context, Result};
use clap::Command;
use clap_complete::{generate, shells::Shell as ClapShell};
use std::fs;
use std::path::PathBuf;

use crate::completion::helpers::get_completion_filename;

/// Completion 脚本生成器
///
/// 提供生成各种 shell 的 completion 脚本文件的功能。
/// 支持 workflow 命令及其所有子命令的 completion 生成。
pub struct CompletionGenerator {
    shell: ClapShell,
    output_dir: PathBuf,
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
    /// let generator = CompletionGenerator::new(
    ///     Some("zsh".to_string()),
    ///     Some("/path/to/completions".to_string()),
    /// )?;
    /// generator.generate_all()?;
    /// ```
    pub fn new(shell_type: Option<String>, output_dir: Option<String>) -> Result<Self> {
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
                anyhow::bail!("Unsupported shell type: {}. Supported shell types: zsh, bash, fish, powershell, elvish", shell);
            }
        };

        // 解析输出目录
        let output = output_dir.map(PathBuf::from).unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(&home).join(".workflow/completions")
        });

        Ok(Self {
            shell: clap_shell,
            output_dir: output,
        })
    }

    /// 生成所有 completion 脚本文件
    ///
    /// 为所有命令生成 completion 脚本：
    /// - `workflow` 命令及其所有子命令（包括 `pr`（create、merge、approve、comment、close、status、list、update、sync、summarize）、`log`、`jira`、`github`、`llm`、`proxy`、`log-level`、`branch` 等）
    pub fn generate_all(&self) -> Result<()> {
        log_debug!("Generating shell completion scripts...");
        log_debug!("Shell type: {}", self.shell);
        log_debug!("Output directory: {}", self.output_dir.display());

        // 创建输出目录
        fs::create_dir_all(&self.output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {} (shell: {})",
                self.output_dir.display(),
                self.shell
            )
        })?;

        // 生成 completion 脚本
        self.generate_workflow()?;

        log_success!(
            "  Shell completion scripts generated to: {}",
            self.output_dir.display()
        );
        Ok(())
    }

    /// 生成 workflow 命令的 completion
    ///
    /// 使用实际的 CLI 结构体自动生成补全脚本，确保补全脚本与实际命令结构保持同步。
    /// 这样就不需要手动维护两套命令定义，避免了不同步的问题。
    fn generate_workflow(&self) -> Result<()> {
        // 使用实际的 CLI 结构体生成补全脚本，而不是手动构建
        // 这样可以确保补全脚本与实际命令结构保持同步
        let mut cmd = crate::cli::get_cli_command();

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

        let shell_type_str = self.shell.to_string();
        let filename = get_completion_filename(&shell_type_str, command_name)?;
        let output_file = self.output_dir.join(&filename);

        fs::write(&output_file, buffer).with_context(|| {
            format!(
                "Failed to write completion file: {} (command: {}, shell: {})",
                output_file.display(),
                command_name,
                self.shell
            )
        })?;
        log_success!("  Generated: {}", output_file.display());

        Ok(())
    }
}
