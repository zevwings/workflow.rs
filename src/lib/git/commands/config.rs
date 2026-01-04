//! Git 配置操作命令封装
//!
//! 提供配置相关的所有 Git 命令操作，包括：
//! - 配置读取（get_config）
//! - 配置设置（set_config）

use crate::git::commands::command::GitCommand;
use color_eyre::{eyre::WrapErr, Result};
use std::path::Path;

/// Git 配置命令操作
pub struct GitConfigCommand;

impl GitConfigCommand {
    /// 移除 Windows 路径的长路径前缀（\\?\）
    ///
    /// Git 命令不支持 Windows 的扩展路径前缀（\\?\），
    /// 因此在传递给 Git 命令之前需要移除该前缀。
    #[cfg(target_os = "windows")]
    fn remove_verbatim_prefix(path_str: &str) -> String {
        if path_str.starts_with("\\\\?\\") {
            path_str[4..].to_string()
        } else {
            path_str.to_string()
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn remove_verbatim_prefix(path_str: &str) -> String {
        path_str.to_string()
    }

    /// 获取配置值
    ///
    /// 使用 `git config --get <key>` 命令
    /// 当 `GIT_CONFIG` 环境变量被设置且 `global=false` 时，使用 `--file` 参数直接指定配置文件路径
    pub fn get_config(key: &str, global: bool, cwd: Option<&Path>) -> Result<Option<String>> {
        let mut args_vec = vec!["config".to_string(), "--get".to_string()];

        // 当 GIT_CONFIG 环境变量被设置且 global=false 时，使用 --file 参数
        if !global {
            if let Ok(git_config_path) = std::env::var("GIT_CONFIG") {
                args_vec.push("--file".to_string());
                args_vec.push(git_config_path);
            }
        } else if global {
            args_vec.push("--global".to_string());
        }

        args_vec.push(key.to_string());

        // 转换为 &[&str] 以便传递给 GitCommand
        let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        match GitCommand::run(args.as_slice(), cwd) {
            Ok(output) => {
                let value = output.trim().to_string();
                if value.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(value))
                }
            }
            Err(e) => {
                // 如果配置不存在，Git 会返回错误（退出码 1），这是正常的
                // Git 在配置不存在时返回退出码 1，我们可以通过检查错误类型来判断
                let error_str = format!("{}", e);
                if error_str.contains("not found")
                    || error_str.contains("no such key")
                    || error_str.contains("exited with code 1")
                {
                    Ok(None)
                } else {
                    Err(color_eyre::eyre::eyre!("{}", e))
                        .wrap_err_with(|| format!("Failed to get config: {}", key))
                }
            }
        }
    }

    /// 设置配置值
    ///
    /// 使用 `git config <scope> <key> <value>` 命令
    /// 当 `GIT_CONFIG` 环境变量被设置且 `global=false` 时，使用 `--file` 参数直接指定配置文件路径
    pub fn set_config(key: &str, value: &str, global: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args_vec = vec!["config".to_string()];

        // 当 GIT_CONFIG 环境变量被设置且 global=false 时，使用 --file 参数
        // 这样可以确保 Git 命令能够正确写入配置，即使配置文件是空的
        if !global {
            if let Ok(git_config_path) = std::env::var("GIT_CONFIG") {
                args_vec.push("--file".to_string());
                args_vec.push(git_config_path);
            }
        } else if global {
            args_vec.push("--global".to_string());
        }

        args_vec.push(key.to_string());
        args_vec.push(value.to_string());

        // 转换为 &[&str] 以便传递给 GitCommand
        let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        GitCommand::execute(args.as_slice(), cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to set config: {} = {}", key, value))
    }

    /// 删除配置项
    ///
    /// 使用 `git config --unset <key>` 命令
    ///
    /// 注意：如果配置项不存在，Git 会返回退出代码 5，这是正常的，不会返回错误。
    /// 当 `GIT_CONFIG` 环境变量被设置且 `global=false` 时，使用 `--file` 参数直接指定配置文件路径
    pub fn unset_config(key: &str, global: bool, cwd: Option<&Path>) -> Result<()> {
        let mut args_vec = vec!["config".to_string(), "--unset".to_string()];

        // 当 GIT_CONFIG 环境变量被设置且 global=false 时，使用 --file 参数
        if !global {
            if let Ok(git_config_path) = std::env::var("GIT_CONFIG") {
                args_vec.push("--file".to_string());
                args_vec.push(git_config_path);
            }
        } else if global {
            args_vec.push("--global".to_string());
        }

        args_vec.push(key.to_string());

        // 转换为 &[&str] 以便传递给 GitCommand
        let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        match GitCommand::execute(args.as_slice(), cwd) {
            Ok(()) => Ok(()),
            Err(e) => {
                // Git 在配置项不存在时返回退出代码 5，这是正常的
                // 检查错误信息或退出代码
                let error_str = format!("{}", e);
                if error_str.contains("exited with code 5") {
                    // 配置项不存在，这是正常的，返回成功
                    Ok(())
                } else {
                    Err(color_eyre::eyre::eyre!("{}", e))
                        .wrap_err_with(|| format!("Failed to unset config: {}", key))
                }
            }
        }
    }

    /// 获取用户邮箱
    ///
    /// 使用 `git config user.email` 命令
    pub fn get_user_email(global: bool, cwd: Option<&Path>) -> Result<Option<String>> {
        Self::get_config("user.email", global, cwd)
    }

    /// 获取用户名称
    ///
    /// 使用 `git config user.name` 命令
    pub fn get_user_name(global: bool, cwd: Option<&Path>) -> Result<Option<String>> {
        Self::get_config("user.name", global, cwd)
    }

    /// 设置用户邮箱和名称
    ///
    /// # 返回
    ///
    /// 返回设置后的 `(email, name)` 元组
    pub fn set_user(
        email: &str,
        name: &str,
        global: bool,
        cwd: Option<&Path>,
    ) -> Result<(String, String)> {
        Self::set_config("user.email", email, global, cwd)?;
        Self::set_config("user.name", name, global, cwd)?;
        Ok((email.to_string(), name.to_string()))
    }

    /// 列出所有配置项
    ///
    /// 使用 `git config --list` 命令
    ///
    /// 注意：此方法使用平台相关的超时时间（Windows 120秒，其他平台 60秒），
    /// 因为读取全局配置可能需要较长时间，特别是在配置项很多或系统较慢的情况下。
    /// 当 `GIT_CONFIG` 环境变量被设置且 `global=false` 时，使用 `--file` 参数直接指定配置文件路径
    pub fn list_config(global: bool, cwd: Option<&Path>) -> Result<Vec<(String, String)>> {
        let mut args_vec = vec!["config".to_string(), "--list".to_string()];

        // 当 GIT_CONFIG 环境变量被设置且 global=false 时，使用 --file 参数
        if !global {
            if let Ok(git_config_path) = std::env::var("GIT_CONFIG") {
                args_vec.push("--file".to_string());
                args_vec.push(git_config_path);
            }
        } else if global {
            args_vec.push("--global".to_string());
        }

        // 转换为 &[&str] 以便传递给 GitCommand
        let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();

        // 使用平台相关的超时时间（Windows 120秒，其他平台 60秒）
        // 因为读取全局配置可能需要较长时间，特别是在配置项很多或系统较慢的情况下
        #[cfg(target_os = "windows")]
        let timeout = std::time::Duration::from_secs(180);
        #[cfg(not(target_os = "windows"))]
        let timeout = std::time::Duration::from_secs(120);

        let output = GitCommand::run_with_timeout(args.as_slice(), cwd, timeout)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

        let mut configs = Vec::new();
        for line in output.lines() {
            if let Some((key, value)) = line.split_once('=') {
                configs.push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(configs)
    }

    /// 获取本地配置值
    ///
    /// 使用 `git config --file <path>/.git/config --get <key>` 命令（本地配置）
    /// 直接指定 `.git/config` 文件路径，确保从仓库的本地配置文件读取，
    /// 即使 GIT_CONFIG 环境变量被设置也能正常工作。
    pub fn get_local(key: &str, cwd: Option<&Path>) -> Result<String> {
        // 确定仓库路径
        let repo_path =
            cwd.ok_or_else(|| color_eyre::eyre::eyre!("cwd is required for get_local"))?;

        // 构建 .git/config 文件路径
        let config_path = repo_path.join(".git").join("config");

        // 在 Windows 上规范化路径，将短路径格式（8.3格式）转换为长路径格式
        // 这样可以避免 "The filename, directory name, or volume label syntax is incorrect" 错误
        let config_path_str = if cfg!(target_os = "windows") && config_path.exists() {
            let canonical_path = config_path
                .canonicalize()
                .map_err(|e| {
                    color_eyre::eyre::eyre!(
                        "Failed to canonicalize config path: {}: {}",
                        config_path.display(),
                        e
                    )
                })?;
            // 移除 Windows 长路径前缀（\\?\），因为 Git 命令不支持该前缀
            Self::remove_verbatim_prefix(&canonical_path.to_string_lossy())
        } else {
            config_path.to_string_lossy().to_string()
        };

        let args = vec!["config", "--file", &config_path_str, "--get", key];

        match GitCommand::run(&args, cwd) {
            Ok(output) => {
                let value = output.trim().to_string();
                if value.is_empty() {
                    Err(color_eyre::eyre::eyre!("Config key '{}' not found", key))
                } else {
                    Ok(value)
                }
            }
            Err(e) => {
                // 如果配置不存在，Git 会返回错误（退出码 1），这是正常的
                let error_str = format!("{}", e);
                if error_str.contains("not found")
                    || error_str.contains("no such key")
                    || error_str.contains("exited with code 1")
                {
                    Err(color_eyre::eyre::eyre!("Config key '{}' not found", key))
                } else {
                    Err(color_eyre::eyre::eyre!("{}", e))
                        .wrap_err_with(|| format!("Failed to get local config: {}", key))
                }
            }
        }
    }

    /// 设置本地配置值
    ///
    /// 使用 `git config --file <path>/.git/config <key> <value>` 命令（本地配置）
    /// 直接指定 `.git/config` 文件路径，确保写入仓库的本地配置文件，
    /// 即使 GIT_CONFIG 环境变量被设置也能正常工作。
    pub fn set_local(key: &str, value: &str, cwd: Option<&Path>) -> Result<()> {
        // 确定仓库路径
        let repo_path =
            cwd.ok_or_else(|| color_eyre::eyre::eyre!("cwd is required for set_local"))?;

        // 构建 .git/config 文件路径
        let config_path = repo_path.join(".git").join("config");

        // 在 Windows 上规范化路径，将短路径格式（8.3格式）转换为长路径格式
        // 这样可以避免 "The filename, directory name, or volume label syntax is incorrect" 错误
        let config_path_str = if cfg!(target_os = "windows") {
            // 规范化 .git 目录路径（通常已存在）
            let git_dir = repo_path.join(".git");
            if git_dir.exists() {
                let canonical_git_dir = git_dir
                    .canonicalize()
                    .map_err(|e| {
                        color_eyre::eyre::eyre!(
                            "Failed to canonicalize .git directory: {}: {}",
                            git_dir.display(),
                            e
                        )
                    })?;
                let canonical_config_path = canonical_git_dir.join("config");
                // 移除 Windows 长路径前缀（\\?\），因为 Git 命令不支持该前缀
                Self::remove_verbatim_prefix(&canonical_config_path.to_string_lossy())
            } else {
                // 如果 .git 目录不存在，使用原始路径（这种情况不应该发生，但作为后备）
                config_path.to_string_lossy().to_string()
            }
        } else {
            config_path.to_string_lossy().to_string()
        };

        let args = vec!["config", "--file", &config_path_str, key, value];

        GitCommand::execute(&args, cwd)
            .map_err(|e| color_eyre::eyre::eyre!("{}", e))
            .wrap_err_with(|| format!("Failed to set local config: {} = {}", key, value))
    }
}
