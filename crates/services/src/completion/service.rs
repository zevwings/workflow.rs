//! Shell Completion 服务实现
//!
//! 实现 `CompletionService` trait，负责 Completion 脚本的保存、配置和管理。

use std::fs;
use std::path::PathBuf;

use domain::{
    errors::ServiceError, get_all_completion_filenames, get_completion_filename,
    get_shell_source_path, CompletionCheckResult, CompletionGenerateResult, CompletionRemoveResult,
    CompletionService, ShellCompletionStatus, COMPLETIONS_FILE,
};
use toolkit::{
    add_source, completion_dir, completion_dir_shell_path, completion_file_shell_path,
    completion_source_shell_path, config_file_path, detect_shell, directory, file, has_source,
    reload_hint, remove_source, shell_from_string, shell_to_string, supported_shells, workflow_dir,
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
        let workflow_dir = workflow_dir().map_err(|e| ServiceError::Other(e.to_string()))?;
        let config_file = workflow_dir.join(COMPLETIONS_FILE);

        let completions_path = completion_dir_shell_path();
        let config_content = match shell_str.to_lowercase().as_str() {
            "zsh" => {
                format!(
                    "# Workflow CLI completions\n\
                     # Zsh completion setup\n\
                     \n\
                     fpath=({} $fpath)\n\
                     autoload -Uz compinit\n\
                     compinit\n",
                    completions_path
                )
            }
            "bash" => {
                format!(
                    "# Workflow CLI completions\n\
                     # Bash completion setup\n\
                     \n\
                     for f in {}/*.bash; do\n\
                         [[ -f \"$f\" ]] && source \"$f\"\n\
                     done\n",
                    completions_path
                )
            }
            _ => return Ok(config_file),
        };

        let config_content = config_content.as_str();

        file::write_string(&config_file, config_content).map_err(|e| {
            ServiceError::Other(format!(
                "Failed to write completion config file: {}: {}",
                config_file.display(),
                e
            ))
        })?;

        Ok(config_file)
    }

    /// 配置 shell 配置文件
    fn configure_shell(&self, shell_str: &str) -> Result<bool, ServiceError> {
        let shell = shell_from_string(shell_str)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid shell type: {}", e)))?;

        match shell_str.to_lowercase().as_str() {
            "zsh" | "bash" => {
                // 添加 source 语句到 shell 配置文件
                let source_path = completion_source_shell_path();
                let added = add_source(&shell, &source_path, Some("Workflow CLI completions"))
                    .map_err(|e| ServiceError::Other(e.to_string()))?;

                Ok(added)
            }
            "fish" | "powershell" | "pwsh" | "elvish" => {
                // 直接在各自配置文件中添加 source 语句
                let filename = get_completion_filename(shell_str);
                let source_path = completion_file_shell_path(&filename);

                let added = add_source(&shell, &source_path, Some("Workflow CLI completions"))
                    .map_err(|e| ServiceError::Other(e.to_string()))?;

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

        let removed =
            remove_source(&shell, &source_path).map_err(|e| ServiceError::Other(e.to_string()))?;

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
        let shell_enum = shell_from_string(shell)
            .map_err(|e| ServiceError::InvalidInput(format!("Invalid shell type: {}", e)))?;
        let shell_str = shell_to_string(&shell_enum);

        // 2. 确定输出目录
        let output_path = match output_dir {
            Some(dir) => PathBuf::from(dir),
            None => completion_dir().map_err(|e| ServiceError::Other(e.to_string()))?,
        };

        // 3. 确保输出目录存在
        directory::ensure_exists(&output_path).map_err(|e| ServiceError::Other(e.to_string()))?;

        // 4. 写入脚本文件
        let filename = get_completion_filename(shell_str);
        let script_path = output_path.join(&filename);

        file::write_bytes(&script_path, script_content).map_err(|e| {
            ServiceError::Other(format!(
                "Failed to write completion script: {}: {}",
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
        let comp_dir = completion_dir().ok();

        // 检查各个 shell 的状态
        let mut shell_statuses = Vec::new();

        for shell in supported_shells() {
            let shell_str = shell_to_string(&shell);
            let source_path = get_shell_source_path(shell_str);

            // 检查是否已配置
            let is_configured = has_source(&shell, &source_path).unwrap_or(false);

            // 检查脚本文件是否存在
            let script_exists = if let Some(ref dir) = comp_dir {
                let filename = get_completion_filename(shell_str);
                dir.join(&filename).exists()
            } else {
                false
            };

            // 获取配置文件路径
            let config_path = config_file_path(&shell);

            // 是否为当前 shell
            let is_current = current_shell_str.as_ref().is_some_and(|c| c == shell_str);

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
            completion_dir: comp_dir,
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
            let shell = detect_shell().map_err(|e| ServiceError::Other(e.to_string()))?;
            let shell_str = shell_to_string(&shell);

            match self.remove_shell_config(shell_str) {
                Ok(true) => removed_configs.push(shell_str.to_string()),
                Ok(false) => {} // 未配置，跳过
                Err(e) => failures.push((shell_str.to_string(), e.to_string())),
            }
        }

        // 2. 移除 completion 脚本文件
        if let Ok(comp_dir) = completion_dir() {
            if comp_dir.exists() {
                let filenames = get_all_completion_filenames();

                for filename in &filenames {
                    let file_path = comp_dir.join(filename);
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
        let removed_config_file = if let Ok(wf_dir) = workflow_dir() {
            let config_file = wf_dir.join(COMPLETIONS_FILE);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use tempfile::tempdir;
    use toolkit::shell::{config_file_path, shell_from_string};

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    struct EnvGuard {
        original_home: Option<OsString>,
        original_disable_icloud: Option<OsString>,
    }

    impl EnvGuard {
        fn new(home_dir: &PathBuf) -> Self {
            let original_home = env::var_os("HOME");
            let original_disable_icloud = env::var_os("WORKFLOW_DISABLE_ICLOUD");
            env::set_var("HOME", home_dir);
            env::set_var("WORKFLOW_DISABLE_ICLOUD", "1");
            Self {
                original_home,
                original_disable_icloud,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original_home {
                Some(value) => env::set_var("HOME", value),
                None => env::remove_var("HOME"),
            }
            match &self.original_disable_icloud {
                Some(value) => env::set_var("WORKFLOW_DISABLE_ICLOUD", value),
                None => env::remove_var("WORKFLOW_DISABLE_ICLOUD"),
            }
        }
    }

    fn with_test_home<F: FnOnce(&tempfile::TempDir)>(f: F) {
        let _lock = ENV_LOCK.lock().expect("lock env");
        let temp = tempdir().expect("tempdir");
        let _guard = EnvGuard::new(&temp.path().to_path_buf());
        f(&temp);
    }

    #[test]
    fn test_save_and_configure_rejects_invalid_shell() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let err = service.save_and_configure("invalid-shell", b"", None).unwrap_err();
            assert!(matches!(err, ServiceError::InvalidInput(_)));
        });
    }

    #[test]
    fn test_save_and_configure_zsh_creates_script_and_config() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.save_and_configure("zsh", b"content", None).unwrap();

            assert_eq!(result.shell, "zsh");
            assert!(result.script_path.exists());
            assert_eq!(fs::read(&result.script_path).unwrap(), b"content");

            let config_file = result.config_file.expect("config file should exist");
            assert!(config_file.exists());
            let config_content = fs::read_to_string(&config_file).unwrap();
            assert!(config_content.contains("Workflow CLI completions"));
            assert!(config_content.contains(&completion_dir_shell_path()));
            assert!(result.config_added);
        });
    }

    #[test]
    fn test_save_and_configure_fish_writes_source_to_config() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.save_and_configure("fish", b"content", None).unwrap();

            assert_eq!(result.shell, "fish");
            assert!(result.script_path.exists());
            assert!(result.config_file.is_none());
            assert!(result.config_added);

            let shell = shell_from_string("fish").expect("fish shell");
            let config_path = config_file_path(&shell).expect("config path");
            let config_content = fs::read_to_string(config_path).unwrap();
            let filename = get_completion_filename("fish");
            let expected = completion_file_shell_path(&filename);
            assert!(config_content.contains(&expected));
        });
    }

    #[test]
    fn test_check_status_marks_existing_script() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let _ = service.save_and_configure("zsh", b"content", None).unwrap();

            let status = service.check_status().unwrap();
            let zsh_status =
                status.shell_statuses.iter().find(|s| s.shell == "zsh").expect("zsh status");
            assert!(zsh_status.script_exists);
            assert!(zsh_status.is_configured);
        });
    }

    #[test]
    fn test_remove_all_removes_scripts_and_config() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.save_and_configure("zsh", b"content", None).unwrap();
            let script_path = result.script_path.clone();
            let config_path = result.config_file.clone().expect("config file");

            let remove_result = service.remove(true).unwrap();
            assert!(remove_result.failures.is_empty());
            assert!(remove_result.removed_configs.contains(&"zsh".to_string()));
            assert!(remove_result.removed_files.iter().any(|path| path == &script_path));
            assert_eq!(remove_result.removed_config_file, Some(config_path.clone()));
            assert!(!script_path.exists());
            assert!(!config_path.exists());
        });
    }

    #[test]
    fn test_save_and_configure_bash_creates_script_and_config() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.save_and_configure("bash", b"bash content", None).unwrap();

            assert_eq!(result.shell, "bash");
            assert!(result.script_path.exists());
            assert_eq!(fs::read(&result.script_path).unwrap(), b"bash content");

            let config_file = result.config_file.expect("config file should exist");
            assert!(config_file.exists());
            let config_content = fs::read_to_string(&config_file).unwrap();
            assert!(config_content.contains("Workflow CLI completions"));
            assert!(config_content.contains("Bash completion setup"));
            assert!(result.config_added);
        });
    }

    #[test]
    fn test_save_and_configure_powershell_writes_source_to_config() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.save_and_configure("powershell", b"pwsh content", None).unwrap();

            assert_eq!(result.shell, "powershell");
            assert!(result.script_path.exists());
            assert!(result.config_file.is_none());
            assert!(result.config_added);

            let shell = shell_from_string("powershell").expect("powershell shell");
            let config_path = config_file_path(&shell).expect("config path");
            let config_content = fs::read_to_string(config_path).unwrap();
            let filename = get_completion_filename("powershell");
            let expected = completion_file_shell_path(&filename);
            assert!(config_content.contains(&expected));
        });
    }

    #[test]
    fn test_save_and_configure_idempotent() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result1 = service.save_and_configure("zsh", b"content1", None).unwrap();
            let script_path = result1.script_path.clone();

            let result2 = service.save_and_configure("zsh", b"content2", None).unwrap();
            assert_eq!(result2.script_path, script_path);
            assert_eq!(fs::read(&script_path).unwrap(), b"content2");
        });
    }

    #[test]
    fn test_save_and_configure_with_custom_output_dir() {
        with_test_home(|temp| {
            let custom_dir = temp.path().join("custom_completions");

            let service = CompletionServiceImpl::new();
            let result = service
                .save_and_configure("zsh", b"content", Some(custom_dir.to_str().unwrap()))
                .unwrap();

            assert!(result.script_path.starts_with(&custom_dir));
            assert!(result.script_path.exists());
        });
    }

    #[test]
    fn test_check_status_when_no_scripts_exist() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let status = service.check_status().unwrap();

            assert!(status.shell_statuses.iter().all(|s| !s.script_exists));
        });
    }

    #[test]
    fn test_remove_when_nothing_exists() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            let result = service.remove(true).unwrap();

            assert!(result.removed_configs.is_empty());
            assert!(result.removed_files.is_empty());
            assert!(result.removed_config_file.is_none());
            assert!(result.failures.is_empty());
        });
    }

    #[test]
    fn test_remove_current_shell_only() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            service.save_and_configure("zsh", b"zsh content", None).unwrap();
            service.save_and_configure("fish", b"fish content", None).unwrap();

            let result = service.remove(false).unwrap();
            // 应该只移除当前 shell（如果当前 shell 是 zsh 或 fish）
            assert!(!result.failures.is_empty() || result.removed_configs.len() <= 1);
        });
    }

    #[test]
    fn test_check_status_detects_multiple_shells() {
        with_test_home(|_| {
            let service = CompletionServiceImpl::new();
            service.save_and_configure("zsh", b"zsh", None).unwrap();
            service.save_and_configure("bash", b"bash", None).unwrap();

            let status = service.check_status().unwrap();
            let zsh_status =
                status.shell_statuses.iter().find(|s| s.shell == "zsh").expect("zsh status");
            let bash_status =
                status.shell_statuses.iter().find(|s| s.shell == "bash").expect("bash status");

            assert!(zsh_status.script_exists);
            assert!(bash_status.script_exists);
        });
    }
}
