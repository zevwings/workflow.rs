//! 安装命令实现
//!
//! 提供安装二进制文件和 shell completion 的功能。

use std::env;
use std::path::PathBuf;

#[cfg(unix)]
use std::process::Command;

#[cfg(windows)]
use std::fs;

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use color_eyre::{eyre::eyre, eyre::ContextCompat, eyre::WrapErr, Result};
use prompt::{br, info, print, success, warning};
use toolkit::{detect_shell, directory, shell_to_string};

use crate::cli::Cli;
use crate::registry::{get_completion_service, get_path_service};

/// 安装命令
pub struct InstallCommand {
    /// 是否只安装二进制文件
    binaries_only: bool,
    /// 是否只安装 completion
    completions_only: bool,
}

impl InstallCommand {
    /// 创建新的 InstallCommand 实例
    pub fn new(binaries_only: bool, completions_only: bool) -> Self {
        Self {
            binaries_only,
            completions_only,
        }
    }

    /// 运行安装命令
    pub fn run(&self) -> Result<()> {
        print!("Installing Workflow CLI...");
        br!();

        if self.completions_only {
            // 只安装 completion
            self.install_completions()?;
        } else if self.binaries_only {
            // 只安装二进制文件
            self.install_binaries()?;
        } else {
            // 默认安装全部
            self.install_binaries()?;
            br!();
            self.install_completions()?;
        }

        br!();
        success!("Installation completed!");

        Ok(())
    }

    /// 安装二进制文件到系统目录
    ///
    /// 在当前可执行文件所在目录查找 workflow 二进制文件，
    /// 并将其复制到系统二进制目录（通常是 /usr/local/bin）。
    fn install_binaries(&self) -> Result<()> {
        let install_dir = get_path_service().get_binary_install_dir()?;
        info!("Installing binaries to {}...", install_dir.display());

        // 创建安装目录（Windows 需要）
        let install_path = PathBuf::from(&install_dir);
        directory::ensure_exists(&install_path)?;

        // 获取当前可执行文件所在目录
        let current_exe = env::current_exe().wrap_err("Failed to get current executable path")?;
        let current_dir =
            current_exe.parent().wrap_err("Failed to get parent directory of executable")?;

        toolkit::log_debug!("Current directory: {}", current_dir.display());
        toolkit::log_debug!("Install directory: {}", install_dir.display());

        let bin_name = get_path_service().get_binary_name()?;

        let source = current_dir.join(&bin_name);
        let target = install_path.join(&bin_name);

        if !source.exists() {
            warning!("Binary file {} does not exist, skipping", source.display());
            return Err(eyre!("Binary file {} does not exist", source.display()));
        }

        info!("  Installing {} -> {}", bin_name, target.display());

        // Unix: 使用 sudo 复制文件
        // Windows: 直接复制文件
        #[cfg(unix)]
        {
            let status = Command::new("sudo")
                .arg("cp")
                .arg(&source)
                .arg(&target)
                .status()
                .wrap_err_with(|| {
                    format!(
                        "Failed to copy {} to {}",
                        source.display(),
                        target.display()
                    )
                })?;

            if !status.success() {
                return Err(eyre!("Failed to install {}", bin_name));
            }

            // 设置执行权限
            Command::new("sudo")
                .arg("chmod")
                .arg("+x")
                .arg(&target)
                .status()
                .wrap_err_with(|| {
                    format!(
                        "Failed to set executable permission for {}",
                        target.display()
                    )
                })?;
        }

        #[cfg(windows)]
        {
            fs::copy(&source, &target).wrap_err_with(|| {
                format!(
                    "Failed to copy {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
        }

        success!("  {} installed", bin_name);

        Ok(())
    }

    /// 安装 shell completion 脚本
    ///
    /// 自动检测当前 shell 类型并安装相应的 completion 脚本。
    fn install_completions(&self) -> Result<()> {
        info!("Installing shell completion scripts...");

        // 检测 shell 类型
        let shell = detect_shell().wrap_err("Failed to detect shell type")?;
        let shell_str = shell_to_string(&shell);

        toolkit::log_debug!("Detected shell: {}", shell);

        // 生成 completion 脚本内容
        let script_content = self.generate_completion_script(&shell)?;

        // 调用 Service 保存并配置
        let service = get_completion_service();
        let result = service
            .save_and_configure(shell_str, &script_content, None)
            .wrap_err("Failed to save and configure completion")?;

        // 显示结果
        success!(
            "  Generated completion script: {}",
            result.script_path.display()
        );

        if let Some(config_file) = &result.config_file {
            success!(
                "  Created completion config file: {}",
                config_file.display()
            );
        }

        if result.config_added {
            success!("  Added completion config to shell config file");
        } else {
            info!("  Completion config already exists, skipped");
        }

        success!("Shell completion installation complete");
        br!();
        info!("Please run the following command to reload configuration:");
        info!("  {}", result.reload_hint);

        Ok(())
    }

    /// 生成 completion 脚本内容
    fn generate_completion_script(&self, shell: &Shell) -> Result<Vec<u8>> {
        let mut cmd = Cli::command();
        let mut buffer = Vec::new();

        // 生成基础 completion 脚本
        generate(*shell, &mut cmd, "workflow", &mut buffer);

        // 根据 shell 类型添加额外功能
        match shell {
            Shell::Zsh | Shell::Bash => {
                self.append_dynamic_completion(&mut buffer, shell);
            }
            _ => {}
        }

        Ok(buffer)
    }

    /// 追加动态补全支持
    fn append_dynamic_completion(&self, buffer: &mut Vec<u8>, shell: &Shell) {
        let dynamic_code = match shell {
            Shell::Zsh => Self::generate_zsh_dynamic_completion(),
            Shell::Bash => Self::generate_bash_dynamic_completion(),
            _ => return,
        };

        buffer.extend_from_slice(b"\n\n");
        buffer.extend_from_slice(dynamic_code.as_bytes());
    }

    /// 生成 zsh 动态补全代码
    fn generate_zsh_dynamic_completion() -> String {
        let path_service = get_path_service();
        let cache_dir = path_service.get_completion_cache_shell_dir();
        format!(
            r#"
# Dynamic completion support
# This provides dynamic completion for branch names, PR IDs, etc.

# Performance optimization: cache directory
typeset -g _WORKFLOW_CACHE_DIR="{cache_dir}"
typeset -g _WORKFLOW_CACHE_TTL=300  # 5 minutes

# Ensure cache directory exists
_workflow_ensure_cache_dir() {{
  [[ ! -d "$_WORKFLOW_CACHE_DIR" ]] && mkdir -p "$_WORKFLOW_CACHE_DIR" 2>/dev/null
}}

# Check if cache file is valid (not expired)
_workflow_is_cache_valid() {{
  local cache_file="$1"
  [[ -f "$cache_file" ]] || return 1
  local cache_time=$(stat -f %m "$cache_file" 2>/dev/null || stat -c %Y "$cache_file" 2>/dev/null)
  local current_time=$(date +%s)
  (( current_time - cache_time < _WORKFLOW_CACHE_TTL ))
}}

# Dynamic branch name completion
_workflow_complete_branches() {{
  _workflow_ensure_cache_dir
  local cache_file="$_WORKFLOW_CACHE_DIR/branches"

  if _workflow_is_cache_valid "$cache_file"; then
    local branches=(${{(f)"$(<$cache_file)"}})
    _describe 'branches' branches
    return
  fi

  local branches
  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then
    echo "$branches" > "$cache_file" 2>/dev/null
    local branch_array=(${{(f)branches}})
    _describe 'branches' branch_array
  fi
}}
"#,
            cache_dir = cache_dir
        )
    }

    /// 生成 bash 动态补全代码
    fn generate_bash_dynamic_completion() -> String {
        let path_service = get_path_service();
        let cache_dir = path_service.get_completion_cache_shell_dir();
        format!(
            r#"
# Dynamic completion support for bash
# This provides dynamic completion for branch names, PR IDs, etc.

# Performance optimization: cache settings
_WORKFLOW_CACHE_DIR="{cache_dir}"
_WORKFLOW_CACHE_TTL=300  # 5 minutes

# Ensure cache directory exists
_workflow_ensure_cache_dir() {{
  [[ ! -d "$_WORKFLOW_CACHE_DIR" ]] && mkdir -p "$_WORKFLOW_CACHE_DIR" 2>/dev/null
}}

# Check if cache file is valid
_workflow_is_cache_valid() {{
  local cache_file="$1"
  [[ -f "$cache_file" ]] || return 1
  local cache_time=$(stat -c %Y "$cache_file" 2>/dev/null)
  local current_time=$(date +%s)
  (( current_time - cache_time < _WORKFLOW_CACHE_TTL ))
}}

# Get branch names for completion
_workflow_get_branches() {{
  _workflow_ensure_cache_dir
  local cache_file="$_WORKFLOW_CACHE_DIR/branches"

  if _workflow_is_cache_valid "$cache_file"; then
    cat "$cache_file" 2>/dev/null
    return
  fi

  local branches
  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then
    echo "$branches" | tee "$cache_file" 2>/dev/null
  fi
}}
"#,
            cache_dir = cache_dir
        )
    }
}
