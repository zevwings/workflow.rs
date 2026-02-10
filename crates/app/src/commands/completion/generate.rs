//! Completion 生成命令
//!
//! 生成 Shell Completion 脚本并配置 shell。

use clap::CommandFactory;
use clap_complete::{generate, Shell};
use domain::get_completion_cache_shell_dir;
use prompt::{info, success};
use toolkit::{detect_shell, shell_from_string, shell_to_string};

use crate::bootstrap::get_completion_service;
use crate::commands::cli::Cli;

/// Completion 生成命令
pub struct CompletionGenerateCommand {
    shell_type: Option<String>,
    output_dir: Option<String>,
}

impl CompletionGenerateCommand {
    /// 创建新的 CompletionGenerateCommand 实例
    pub fn new(shell_type: Option<String>, output_dir: Option<String>) -> Self {
        Self {
            shell_type,
            output_dir,
        }
    }

    /// 运行生成命令
    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. 确定 shell 类型
        let shell =
            match &self.shell_type {
                Some(s) => shell_from_string(s)
                    .map_err(|e| format!("Failed to parse shell type: {}", e))?,
                None => detect_shell()
                    .map_err(|e| format!("Failed to auto-detect shell type: {}", e))?,
            };

        let shell_str = shell_to_string(&shell);
        info!("Detected shell type: {}", shell);

        // 2. 生成 completion 脚本内容
        let script_content = self.generate_completion_script(&shell)?;

        // 3. 调用 Service 保存并配置
        let service = get_completion_service();
        let result = service
            .save_and_configure(shell_str, &script_content, self.output_dir.as_deref())
            .map_err(|e| format!("Failed to save and configure completion: {}", e))?;

        // 4. 显示结果
        info!(
            "Generated completion script: {}",
            result.script_path.display()
        );

        if let Some(config_file) = &result.config_file {
            info!("Created completion config file: {}", config_file.display());
        }

        if result.config_added {
            success!("Added completion config to shell config file");
        } else {
            info!("Completion config already exists, skipping");
        }

        success!("Completion script generated and configured!");
        info!("Script location: {}", result.script_path.display());
        info!("Run the following command to apply the config:");
        info!("   {}", result.reload_hint);

        Ok(())
    }

    /// 生成 completion 脚本内容
    fn generate_completion_script(
        &self,
        shell: &Shell,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut cmd = Cli::command();
        let mut buffer = Vec::new();

        // 生成基础 completion 脚本
        generate(*shell, &mut cmd, "workflow", &mut buffer);

        // 根据 shell 类型添加额外功能
        match shell {
            Shell::Zsh | Shell::Bash => {
                // 添加动态补全支持
                self.append_dynamic_completion(&mut buffer, shell);
            }
            _ => {}
        }

        Ok(buffer)
    }

    /// 追加动态补全支持
    fn append_dynamic_completion(&self, buffer: &mut Vec<u8>, shell: &Shell) {
        let dynamic_code = match shell {
            Shell::Zsh => self.generate_zsh_dynamic_completion(),
            Shell::Bash => self.generate_bash_dynamic_completion(),
            _ => return,
        };

        buffer.extend_from_slice(b"\n\n");
        buffer.extend_from_slice(dynamic_code.as_bytes());
    }

    /// 生成 zsh 动态补全代码
    fn generate_zsh_dynamic_completion(&self) -> String {
        let cache_dir = get_completion_cache_shell_dir();
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

  # Try to use cached branches if valid
  if _workflow_is_cache_valid "$cache_file"; then
    local branches=(${{(f)"$(<$cache_file)"}})
    _describe 'branches' branches
    return
  fi

  # Get branches with timeout and error handling
  local branches
  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then
    # Cache the results
    echo "$branches" > "$cache_file" 2>/dev/null
    local branch_array=(${{(f)branches}})
    _describe 'branches' branch_array
  fi
}}

# Dynamic PR ID completion
_workflow_complete_pr_ids() {{
  _workflow_ensure_cache_dir
  local cache_file="$_WORKFLOW_CACHE_DIR/pr_ids"

  # Try to use cached PR IDs if valid
  if _workflow_is_cache_valid "$cache_file"; then
    local pr_ids=(${{(f)"$(<$cache_file)"}})
    _describe 'PR IDs' pr_ids
    return
  fi

  # Get recent PR IDs with timeout
  local pr_ids
  if command -v gh >/dev/null 2>&1 && pr_ids=$(timeout 3s gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null); then
    # Cache the results
    echo "$pr_ids" > "$cache_file" 2>/dev/null
    local pr_array=(${{(f)pr_ids}})
    _describe 'PR IDs' pr_array
  fi
}}
"#,
            cache_dir = cache_dir
        )
    }

    /// 生成 bash 动态补全代码
    fn generate_bash_dynamic_completion(&self) -> String {
        let cache_dir = get_completion_cache_shell_dir();
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

  # Use cached branches if valid
  if _workflow_is_cache_valid "$cache_file"; then
    cat "$cache_file" 2>/dev/null
    return
  fi

  # Get and cache branches
  local branches
  if branches=$(timeout 2s git branch --format='%(refname:short)' 2>/dev/null); then
    echo "$branches" | tee "$cache_file" 2>/dev/null
  fi
}}

# Get PR IDs for completion
_workflow_get_pr_ids() {{
  _workflow_ensure_cache_dir
  local cache_file="$_WORKFLOW_CACHE_DIR/pr_ids"

  # Use cached PR IDs if valid
  if _workflow_is_cache_valid "$cache_file"; then
    cat "$cache_file" 2>/dev/null
    return
  fi

  # Get and cache PR IDs
  if command -v gh >/dev/null 2>&1; then
    timeout 3s gh pr list --limit 20 --json number --jq '.[].number' 2>/dev/null | tee "$cache_file" 2>/dev/null
  fi
}}
"#,
            cache_dir = cache_dir
        )
    }
}
