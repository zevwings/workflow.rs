//! 代理管理器
//!
//! 负责协调系统代理读取器和配置生成器，提供高级代理管理功能。

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::base::shell::ShellConfigManager;
use crate::proxy::config_generator::ProxyConfigGenerator;
use crate::proxy::system_reader::SystemProxyReader;
use crate::proxy::{ProxyDisableResult, ProxyEnableResult, ProxyInfo, ProxyType};

/// 代理管理器
///
/// 协调系统代理读取器和配置生成器，提供代理启用、禁用和检查功能。
pub struct ProxyManager;

impl ProxyManager {
    /// 检查环境变量中的代理设置
    ///
    /// 从当前环境变量中读取代理配置（`http_proxy`、`https_proxy`、`all_proxy`）。
    ///
    /// # 返回
    ///
    /// 返回包含代理环境变量的 HashMap。
    pub fn check_env_proxy() -> HashMap<String, String> {
        ProxyType::all()
            .filter_map(|pt| {
                std::env::var(pt.env_key())
                    .ok()
                    .map(|value| (pt.env_key().to_string(), value))
            })
            .collect()
    }

    /// 检查代理设置是否匹配
    ///
    /// 检查环境变量中的代理设置是否与系统代理配置匹配。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果环境变量中的代理设置与系统配置匹配，否则返回 `false`。
    pub fn is_proxy_configured(proxy_info: &ProxyInfo) -> bool {
        let env_proxy = Self::check_env_proxy();

        ProxyType::all().all(|pt| {
            if let Some(expected) = proxy_info.get_proxy_url(pt) {
                env_proxy.get(pt.env_key()).map(|v| v.as_str()) == Some(expected.as_str())
            } else {
                true // 如果该类型代理未启用，认为配置正确
            }
        })
    }

    /// 检查系统代理是否启用
    ///
    /// 检查系统代理设置中是否有任何代理类型（HTTP、HTTPS、SOCKS）已启用。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果系统代理已启用，否则返回 `false`。
    fn is_system_proxy_enabled(proxy_info: &ProxyInfo) -> bool {
        ProxyType::all().any(|pt| {
            proxy_info
                .get_config(pt)
                .map(|config| config.enable)
                .unwrap_or(false)
        })
    }

    /// 确保代理已启用（如果系统代理已启用）
    ///
    /// 如果系统代理（VPN）已启用，但环境变量未设置，则自动在当前进程中设置环境变量。
    /// 这个方法主要用于在需要网络访问的命令执行前自动启用代理。
    ///
    /// # 返回
    ///
    /// 如果成功启用代理或代理已配置，返回 `Ok(())`；如果系统代理未启用，也返回 `Ok(())`（静默跳过）。
    ///
    /// # 错误
    ///
    /// 如果读取系统代理设置失败，返回相应的错误信息。
    pub fn ensure_proxy_enabled() -> Result<()> {
        // 1. 获取系统代理设置
        let proxy_info =
            SystemProxyReader::read().context("Failed to read system proxy settings")?;

        // 2. 检查系统代理是否启用
        if !Self::is_system_proxy_enabled(&proxy_info) {
            // 系统代理未启用，静默跳过
            return Ok(());
        }

        // 3. 检查环境变量是否已配置
        if Self::is_proxy_configured(&proxy_info) {
            // 环境变量已配置，无需操作
            return Ok(());
        }

        // 4. 系统代理已启用但环境变量未设置，在当前进程中设置环境变量
        let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
        for (key, value) in env_vars {
            std::env::set_var(&key, &value);
        }

        Ok(())
    }

    /// 开启代理
    ///
    /// 从系统设置读取代理配置，并根据 `temporary` 参数决定是否保存到 shell 配置文件。
    ///
    /// # 参数
    ///
    /// * `temporary` - 如果为 `true`，只在当前 shell 启用，不写入配置文件；默认为 `false`，写入配置文件
    ///
    /// # 返回
    ///
    /// 返回 `ProxyEnableResult`，包含代理命令字符串和 shell 配置文件路径。
    ///
    /// # 错误
    ///
    /// 如果读取系统代理设置失败或保存配置失败，返回相应的错误信息。
    pub fn enable(temporary: bool) -> Result<ProxyEnableResult> {
        // 1. 获取系统代理设置
        let proxy_info = SystemProxyReader::read()?;

        // 2. 检查代理是否已配置
        if Self::is_proxy_configured(&proxy_info) {
            return Ok(ProxyEnableResult {
                already_configured: true,
                proxy_command: None,
                shell_config_path: None,
            });
        }

        // 3. 生成代理命令
        let proxy_cmd = ProxyConfigGenerator::generate_command(&proxy_info);

        // 4. 根据 temporary 参数决定是否写入配置文件
        let env_vars = ProxyConfigGenerator::generate_env_vars(&proxy_info);
        let shell_config_path = if !temporary && !env_vars.is_empty() {
            // 默认行为：写入 shell 配置文件
            ShellConfigManager::set_env_vars(&env_vars)
                .context("Failed to save proxy settings to shell config")?;

            Some(ShellConfigManager::get_config_path().context("Failed to get shell config path")?)
        } else {
            // 临时模式：不写入配置文件
            None
        };

        Ok(ProxyEnableResult {
            already_configured: false,
            proxy_command: proxy_cmd,
            shell_config_path,
        })
    }

    /// 关闭代理
    ///
    /// 同时从 shell 配置文件和当前 shell 环境变量中移除代理设置。
    ///
    /// # 返回
    ///
    /// 返回 `ProxyDisableResult`，包含是否找到代理设置、shell 配置文件路径和 unset 命令。
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    pub fn disable() -> Result<ProxyDisableResult> {
        // 1. 收集当前代理设置
        let current_proxy = Self::collect_current_proxy()?;

        // 2. 检查是否有代理设置
        if current_proxy.is_empty() {
            return Ok(ProxyDisableResult {
                found_proxy: false,
                shell_config_path: None,
                unset_command: None,
                current_env_proxy: current_proxy.env_proxy,
            });
        }

        // 3. 从配置文件移除
        let shell_config_path = Self::remove_from_config_file(&current_proxy)?;

        // 4. 生成 unset 命令
        let unset_command = Self::generate_unset_command(&current_proxy.env_proxy);

        Ok(ProxyDisableResult {
            found_proxy: true,
            shell_config_path,
            unset_command,
            current_env_proxy: current_proxy.env_proxy,
        })
    }

    /// 收集当前代理设置（环境变量和配置文件）
    ///
    /// # 返回
    ///
    /// 返回 `CurrentProxyState`，包含环境变量和配置文件中的代理设置。
    fn collect_current_proxy() -> Result<CurrentProxyState> {
        let env_proxy = Self::check_env_proxy();
        let shell_config_env = ShellConfigManager::load_env_vars().unwrap_or_default();

        Ok(CurrentProxyState {
            env_proxy,
            shell_config_env,
        })
    }

    /// 从配置文件移除代理设置
    ///
    /// # 参数
    ///
    /// * `current_proxy` - 当前代理状态
    ///
    /// # 返回
    ///
    /// 如果从配置文件移除了代理设置，返回配置文件路径；否则返回 `None`。
    ///
    /// # 错误
    ///
    /// 如果读取或写入配置文件失败，返回相应的错误信息。
    fn remove_from_config_file(current_proxy: &CurrentProxyState) -> Result<Option<PathBuf>> {
        // 检查是否有配置需要移除
        let has_config =
            ProxyType::all().any(|pt| current_proxy.shell_config_env.contains_key(pt.env_key()));

        if !has_config {
            return Ok(None);
        }

        // 从配置块移除
        let mut env_vars = current_proxy.shell_config_env.clone();
        let proxy_keys = ProxyType::all_env_keys();

        let removed_from_block = proxy_keys.iter().any(|key| env_vars.remove(*key).is_some());

        if removed_from_block {
            ShellConfigManager::save_env_vars(&env_vars)
                .context("Failed to remove proxy settings from shell config")?;
        }

        // 从整个文件移除
        let has_in_file = ShellConfigManager::remove_env_vars(&proxy_keys)
            .context("Failed to remove proxy settings from shell config")?;

        if removed_from_block || has_in_file {
            Ok(Some(
                ShellConfigManager::get_config_path().context("Failed to get shell config path")?,
            ))
        } else {
            Ok(None)
        }
    }

    /// 生成 unset 命令
    ///
    /// # 参数
    ///
    /// * `env_proxy` - 环境变量中的代理设置
    ///
    /// # 返回
    ///
    /// 如果有代理环境变量，返回 unset 命令字符串；否则返回 `None`。
    fn generate_unset_command(env_proxy: &HashMap<String, String>) -> Option<String> {
        let unset_cmds: Vec<String> = ProxyType::all()
            .filter(|pt| env_proxy.contains_key(pt.env_key()))
            .map(|pt| {
                let mut cmd = String::from("unset ");
                cmd.push_str(pt.env_key());
                cmd
            })
            .collect();

        if unset_cmds.is_empty() {
            None
        } else {
            Some(unset_cmds.join(" && "))
        }
    }
}

/// 当前代理状态
///
/// 包含环境变量和配置文件中的代理设置。
struct CurrentProxyState {
    /// 当前环境变量中的代理设置
    env_proxy: HashMap<String, String>,
    /// Shell 配置文件中的代理设置
    shell_config_env: HashMap<String, String>,
}

impl CurrentProxyState {
    /// 检查是否有任何代理设置
    fn is_empty(&self) -> bool {
        let proxy_keys = ProxyType::all_env_keys();
        let has_in_config = proxy_keys
            .iter()
            .any(|key| self.shell_config_env.contains_key(*key));
        let has_in_env = proxy_keys
            .iter()
            .any(|key| self.env_proxy.contains_key(*key));

        !has_in_config && !has_in_env
    }
}
