//! Shell Completion 服务实现
//!
//! 实现 `CompletionService` trait，负责 Completion 脚本的保存、配置和管理。

use std::fs;
use std::path::PathBuf;

use domain::{
    errors::ServiceError,
    get_all_completion_filenames, get_completion_filename, get_shell_source_path,
    CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult, CompletionService,
    ShellCompletionStatus, COMPLETIONS_FILE, COMPLETIONS_SOURCE_PATH,
};
use toolkit::{
    config_file_path, detect_shell, reload_hint, shell_from_string, shell_to_string,
    supported_shells, DirectoryWalker, FileWriter, Paths, ShellConfigManager,
};

/// Shell Completion 服务实现
pub struct CompletionServiceImpl;

impl CompletionServiceImpl {
    /// 创建新的 CompletionServiceImpl 实例
    pub fn new() -> Self {
        Self
    }

    /// 创建 workflow completion 配置文件（用于 zsh/bash）
    fn create_completion_config_file(&self, shell_str: &str) -> Result<PathBuf, ServiceError> {
        let workflow_dir = Paths::workflow_dir().map_err(|e| {
            ServiceError::Other(format!("获取 workflow 目录失败: {}", e))
        })?;
        let config_file = workflow_dir.join(COMPLETIONS_FILE);

        let config_content = match shell_str.to_lowercase().as_str() {
            "zsh" => {
                "# Workflow CLI completions\n\
                 # Zsh completion setup\n\
                 \n\
                 fpath=($HOME/.workflow/completions $fpath)\n\
                 autoload -Uz compinit\n\
                 compinit\n"
            }
            "bash" => {
                "# Workflow CLI completions\n\
                 # Bash completion setup\n\
                 \n\
                 for f in $HOME/.workflow/completions/*.bash; do\n\
                     [[ -f \"$f\" ]] && source \"$f\"\n\
                 done\n"
            }
            _ => return Ok(config_file),
        };

        FileWriter::new(&config_file)
            .write_str(config_content)
            .map_err(|e| {
                ServiceError::Other(format!(
                    "写入 completion 配置文件失败: {}: {}",
                    config_file.display(),
                    e
                ))
            })?;

        Ok(config_file)
    }

    /// 配置 shell 配置文件
    fn configure_shell(&self, shell_str: &str) -> Result<bool, ServiceError> {
        let shell = shell_from_string(shell_str).map_err(|e| {
            ServiceError::InvalidInput(format!("无效的 shell 类型: {}", e))
        })?;

        match shell_str.to_lowercase().as_str() {
            "zsh" | "bash" => {
                // 添加 source 语句到 shell 配置文件
                let added = ShellConfigManager::add_source(
                    &shell,
                    COMPLETIONS_SOURCE_PATH,
                    Some("Workflow CLI completions"),
                )
                .map_err(|e| ServiceError::Other(format!("添加 completion source 失败: {}", e)))?;

                Ok(added)
            }
            "fish" | "powershell" | "pwsh" | "elvish" => {
                // 直接在各自配置文件中添加 source 语句
                let filename = get_completion_filename(shell_str);
                let source_path = format!("$HOME/.workflow/completions/{}", filename);

                let added = ShellConfigManager::add_source(
                    &shell,
                    &source_path,
                    Some("Workflow CLI completions"),
                )
                .map_err(|e| ServiceError::Other(format!("添加 completion source 失败: {}", e)))?;

                Ok(added)
            }
            _ => Ok(false),
        }
    }

    /// 移除单个 shell 的配置
    fn remove_shell_config(&self, shell_str: &str) -> Result<bool, ServiceError> {
        let shell = match shell_from_string(shell_str) {
            Ok(s) => s,
            Err(_) => return Ok(false),
        };

        let source_path = get_shell_source_path(shell_str);

        let removed = ShellConfigManager::remove_source(&shell, &source_path)
            .map_err(|e| ServiceError::Other(format!("移除 {} 配置失败: {}", shell_str, e)))?;

        Ok(removed)
    }
}

impl Default for CompletionServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionService for CompletionServiceImpl {
    fn save_and_configure(
        &self,
        shell: &str,
        script_content: &[u8],
        output_dir: Option<&str>,
    ) -> Result<CompletionGenerateResult, ServiceError> {
        // 1. 解析 shell 类型
        let shell_enum = shell_from_string(shell).map_err(|e| {
            ServiceError::InvalidInput(format!("无效的 shell 类型: {}", e))
        })?;
        let shell_str = shell_to_string(&shell_enum);

        // 2. 确定输出目录
        let output_path = match output_dir {
            Some(dir) => PathBuf::from(dir),
            None => Paths::completion_dir().map_err(|e| {
                ServiceError::Other(format!("获取 completion 目录失败: {}", e))
            })?,
        };

        // 3. 确保输出目录存在
        DirectoryWalker::new(&output_path)
            .ensure_exists()
            .map_err(|e| {
                ServiceError::Other(format!("创建输出目录失败: {}", e))
            })?;

        // 4. 写入脚本文件
        let filename = get_completion_filename(shell_str);
        let script_path = output_path.join(&filename);

        FileWriter::new(&script_path)
            .write_bytes(script_content)
            .map_err(|e| {
                ServiceError::Other(format!(
                    "写入 completion 脚本失败: {}: {}",
                    script_path.display(),
                    e
                ))
            })?;

        // 5. 创建配置文件（仅 zsh/bash）
        let config_file = match shell_str {
            "zsh" | "bash" => Some(self.create_completion_config_file(shell_str)?),
            _ => None,
        };

        // 6. 配置 shell
        let config_added = self.configure_shell(shell_str)?;

        // 7. 获取重载提示
        let reload = reload_hint(&shell_enum).to_string();

        Ok(CompletionGenerateResult {
            shell: shell_str.to_string(),
            script_path,
            config_file,
            config_added,
            reload_hint: reload,
        })
    }

    fn check_status(&self) -> Result<CompletionCheckResult, ServiceError> {
        // 获取当前 shell
        let current_shell = detect_shell().ok();
        let current_shell_str = current_shell.as_ref().map(|s| shell_to_string(s).to_string());

        // 获取 completion 目录
        let completion_dir = Paths::completion_dir().ok();

        // 检查各个 shell 的状态
        let mut shell_statuses = Vec::new();

        for shell in supported_shells() {
            let shell_str = shell_to_string(&shell);
            let source_path = get_shell_source_path(shell_str);

            // 检查是否已配置
            let is_configured = ShellConfigManager::has_source(&shell, &source_path).unwrap_or(false);

            // 检查脚本文件是否存在
            let script_exists = if let Some(ref dir) = completion_dir {
                let filename = get_completion_filename(shell_str);
                dir.join(&filename).exists()
            } else {
                false
            };

            // 获取配置文件路径
            let config_path = config_file_path(&shell);

            // 是否为当前 shell
            let is_current = current_shell_str
                .as_ref()
                .map_or(false, |c| c == shell_str);

            shell_statuses.push(ShellCompletionStatus {
                shell: shell_str.to_string(),
                is_configured,
                script_exists,
                config_file: config_path,
                is_current,
            });
        }

        Ok(CompletionCheckResult {
            current_shell: current_shell_str,
            completion_dir,
            shell_statuses,
        })
    }

    fn remove(&self, remove_all: bool) -> Result<CompletionRemoveResult, ServiceError> {
        let mut removed_configs = Vec::new();
        let mut removed_files = Vec::new();
        let mut failures = Vec::new();

        // 1. 移除 shell 配置
        if remove_all {
            // 移除所有 shell 的配置
            for shell in supported_shells() {
                let shell_str = shell_to_string(&shell);
                match self.remove_shell_config(shell_str) {
                    Ok(true) => removed_configs.push(shell_str.to_string()),
                    Ok(false) => {} // 未配置，跳过
                    Err(e) => failures.push((shell_str.to_string(), e.to_string())),
                }
            }
        } else {
            // 只移除当前 shell 的配置
            let shell = detect_shell().map_err(|e| {
                ServiceError::Other(format!("自动检测 shell 类型失败: {}", e))
            })?;
            let shell_str = shell_to_string(&shell);

            match self.remove_shell_config(shell_str) {
                Ok(true) => removed_configs.push(shell_str.to_string()),
                Ok(false) => {} // 未配置，跳过
                Err(e) => failures.push((shell_str.to_string(), e.to_string())),
            }
        }

        // 2. 移除 completion 脚本文件
        if let Ok(completion_dir) = Paths::completion_dir() {
            if completion_dir.exists() {
                let filenames = get_all_completion_filenames();

                for filename in &filenames {
                    let file_path = completion_dir.join(filename);
                    if file_path.exists() {
                        match fs::remove_file(&file_path) {
                            Ok(_) => removed_files.push(file_path),
                            Err(e) => {
                                failures.push((file_path.display().to_string(), e.to_string()))
                            }
                        }
                    }
                }
            }
        }

        // 3. 移除 completion 配置文件
        let removed_config_file = if let Ok(workflow_dir) = Paths::workflow_dir() {
            let config_file = workflow_dir.join(COMPLETIONS_FILE);
            if config_file.exists() {
                match fs::remove_file(&config_file) {
                    Ok(_) => Some(config_file),
                    Err(e) => {
                        failures.push((config_file.display().to_string(), e.to_string()));
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(CompletionRemoveResult {
            removed_configs,
            removed_files,
            removed_config_file,
            failures,
        })
    }
}
