use crate::base::util::Clipboard;
use crate::{br, debug, info, success, warning, ProxyManager, SystemProxyReader};
use color_eyre::{eyre::WrapErr, Result};

/// 代理检查命令
pub struct ProxyCommand;

impl ProxyCommand {
    /// 检查代理状态和配置
    pub fn check() -> Result<()> {
        // 1. 获取系统代理设置
        debug!("Reading system proxy settings...");
        let proxy_info =
            SystemProxyReader::read().wrap_err("Failed to read system proxy settings")?;

        // 2. 检查环境变量中的代理设置
        let env_proxy = ProxyManager::check_env_proxy();
        let shell_config_env =
            crate::base::shell::ShellConfigManager::load_env_vars().unwrap_or_default();

        // 3. 显示系统代理设置
        success!("System proxy settings:");
        for proxy_type in crate::ProxyType::all() {
            if let Some(config) = proxy_info.get_config(proxy_type) {
                if config.enable {
                    if let (Some(addr), Some(port)) = (&config.address, &config.port) {
                        let type_name = match proxy_type {
                            crate::ProxyType::Http => "HTTP",
                            crate::ProxyType::Https => "HTTPS",
                            crate::ProxyType::Socks => "SOCKS",
                        };
                        info!("  {}: {}:{}", type_name, addr, port);
                    }
                }
            }
        }

        // 4. 显示当前环境变量设置
        br!();
        info!("Current environment variables:");

        // 合并显示：先显示环境变量，再显示配置文件中的（但不在环境变量中的）
        let mut has_any_proxy = false;
        let mut has_env_proxy = false;
        let mut has_config_proxy = false;

        for (key, value) in &env_proxy {
            if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                info!("  {}={} (current session)", key, value);
                has_any_proxy = true;
                has_env_proxy = true;
            }
        }

        for (key, value) in &shell_config_env {
            if (key == "http_proxy" || key == "https_proxy" || key == "all_proxy")
                && !env_proxy.contains_key(key)
            {
                info!("  {}={} (in shell config)", key, value);
                has_any_proxy = true;
                has_config_proxy = true;
            }
        }

        if !has_any_proxy {
            warning!("  No proxy environment variables set");
        } else {
            br!();
            info!("Proxy configuration status:");
            if has_env_proxy {
                info!("  Current shell: Enabled");
            } else {
                info!("  Current shell: Disabled");
            }
            if has_config_proxy {
                info!("  Shell config file: Enabled (will be loaded in new shells)");
            } else {
                info!("  Shell config file: Disabled");
            }
        }

        // 5. 检查代理是否已正确配置
        if ProxyManager::is_proxy_configured(&proxy_info) {
            br!();
            success!("  Proxy is configured correctly");
        } else {
            br!();
            warning!("  Proxy is not configured correctly");
            info!("Run 'workflow proxy on' to enable proxy");
            info!("Or check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 开启代理（设置环境变量）
    ///
    /// # 参数
    ///
    /// * `temporary` - 如果为 `true`，只在当前 shell 启用，不写入配置文件；默认为 `false`，写入配置文件
    pub fn on(temporary: bool) -> Result<()> {
        debug!("Reading system proxy settings...");
        let result = ProxyManager::enable(temporary).wrap_err("Failed to enable proxy")?;

        if result.already_configured {
            success!("Proxy is already configured correctly");
            return Ok(());
        }

        if let Some(ref proxy_cmd) = result.proxy_command {
            success!("Proxy command generated:");
            info!("{}", proxy_cmd);

            if temporary {
                // 临时模式
                info!("Mode: Temporary (current shell only)");
                info!("The proxy will NOT be saved to shell config file");
            } else {
                // 默认模式（持久化）
                if let Some(ref shell_config_path) = result.shell_config_path {
                    success!("Proxy settings saved to {:?}", shell_config_path);
                    info!("Mode: Persistent (saved to config file)");
                    info!("The proxy will be enabled when you start a new shell");
                    info!("Or run: source {:?}", shell_config_path);
                }
            }

            // 复制到剪贴板
            Clipboard::copy(proxy_cmd).wrap_err("Failed to copy proxy command to clipboard")?;

            success!("Proxy command copied to clipboard");
            info!(
                "Run this command to enable proxy in current shell: {}",
                proxy_cmd
            );
            if !temporary {
                info!("Or use 'eval $(workflow proxy on)' to enable in current shell");
            }
        } else {
            warning!("No proxy configuration found in system settings");
            info!("Check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 关闭代理（取消环境变量）
    ///
    /// 同时从 shell 配置文件和当前 shell 环境变量中移除代理设置。
    pub fn off() -> Result<()> {
        let result = ProxyManager::disable().wrap_err("Failed to disable proxy")?;

        if !result.found_proxy {
            success!("Proxy is already off (no proxy environment variables set)");
            return Ok(());
        }

        // 显示从配置文件移除的结果
        if let Some(ref shell_config_path) = result.shell_config_path {
            success!("Proxy settings removed from {:?}", shell_config_path);
            info!("The proxy will be disabled when you start a new shell");
            info!("Or run: source {:?}", shell_config_path);
        }

        if let Some(ref unset_cmd) = result.unset_command {
            info!("Current proxy environment variables:");
            for (key, value) in &result.current_env_proxy {
                info!("  {}={}", key, value);
            }

            success!("Proxy unset command generated:");
            info!("{}", unset_cmd);

            // 复制到剪贴板
            Clipboard::copy(unset_cmd).wrap_err("Failed to copy unset command to clipboard")?;

            success!("Proxy unset command copied to clipboard");
            info!(
                "Run this command to disable proxy in current shell: {}",
                unset_cmd
            );
        }

        Ok(())
    }
}
