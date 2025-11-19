use crate::{
    log_break, log_debug, log_info, log_message, log_success, log_warning, Clipboard, Proxy,
};
use anyhow::{Context, Result};

/// 代理检查命令
pub struct ProxyCommand;

impl ProxyCommand {
    /// 检查代理状态和配置
    pub fn check() -> Result<()> {
        // 1. 获取系统代理设置
        log_debug!("Reading system proxy settings...");
        let proxy_info =
            Proxy::get_system_proxy().context("Failed to read system proxy settings")?;

        // 2. 检查环境变量中的代理设置
        let env_proxy = Proxy::check_env_proxy();
        let shell_config_env = crate::EnvFile::load().unwrap_or_default();

        // 3. 显示系统代理设置
        log_success!("System proxy settings:");
        if proxy_info.http_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.http_proxy, &proxy_info.http_port) {
                log_info!("  HTTP: {}:{}", addr, port);
            }
        }
        if proxy_info.https_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.https_proxy, &proxy_info.https_port) {
                log_info!("  HTTPS: {}:{}", addr, port);
            }
        }
        if proxy_info.socks_enable {
            if let (Some(addr), Some(port)) = (&proxy_info.socks_proxy, &proxy_info.socks_port) {
                log_info!("  SOCKS: {}:{}", addr, port);
            }
        }

        // 4. 显示当前环境变量设置
        log_break!();
        log_message!("Current environment variables:");

        // 合并显示：先显示环境变量，再显示配置文件中的（但不在环境变量中的）
        let mut has_any_proxy = false;

        for (key, value) in &env_proxy {
            if key == "http_proxy" || key == "https_proxy" || key == "all_proxy" {
                log_info!("  {}={} (current session)", key, value);
                has_any_proxy = true;
            }
        }

        for (key, value) in &shell_config_env {
            if (key == "http_proxy" || key == "https_proxy" || key == "all_proxy")
                && !env_proxy.contains_key(key)
            {
                log_info!("  {}={} (in shell config)", key, value);
                has_any_proxy = true;
            }
        }

        if !has_any_proxy {
            log_warning!("  No proxy environment variables set");
        }

        // 5. 检查代理是否已正确配置
        if Proxy::is_proxy_configured(&proxy_info) {
            log_break!();
            log_success!("  Proxy is configured correctly");
        } else {
            log_break!();
            log_warning!("  Proxy is not configured correctly");
            log_message!("Run 'workflow proxy on' to enable proxy");
            log_message!("Or check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 开启代理（设置环境变量）
    pub fn on() -> Result<()> {
        log_debug!("Reading system proxy settings...");
        let result = Proxy::enable_proxy().context("Failed to enable proxy")?;

        if result.already_configured {
            log_success!("Proxy is already configured correctly");
            return Ok(());
        }

        if let Some(ref proxy_cmd) = result.proxy_command {
            log_success!("Proxy command generated:");
            log_message!("{}", proxy_cmd);

            if let Some(ref shell_config_path) = result.shell_config_path {
                log_success!("Proxy settings saved to {:?}", shell_config_path);
                log_message!("The proxy will be enabled when you start a new shell");
                log_message!("Or run: source {:?}", shell_config_path);
            }

            // 复制到剪贴板
            Clipboard::copy(proxy_cmd).context("Failed to copy proxy command to clipboard")?;

            log_success!("Proxy command copied to clipboard");
            log_message!("You can also run it manually: {}", proxy_cmd);
        } else {
            log_warning!("No proxy configuration found in system settings");
            log_message!("Check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 关闭代理（取消环境变量）
    pub fn off() -> Result<()> {
        let result = Proxy::disable_proxy().context("Failed to disable proxy")?;

        if !result.found_proxy {
            log_success!("Proxy is already off (no proxy environment variables set)");
            return Ok(());
        }

        if let Some(ref shell_config_path) = result.shell_config_path {
            log_success!("Proxy settings removed from {:?}", shell_config_path);
            log_message!("The proxy will be disabled when you start a new shell");
            log_message!("Or run: source {:?}", shell_config_path);
        }

        if let Some(ref unset_cmd) = result.unset_command {
            log_message!("Current proxy environment variables:");
            for (key, value) in &result.current_env_proxy {
                log_info!("  {}={}", key, value);
            }

            log_success!("Proxy unset command generated:");
            log_message!("{}", unset_cmd);

            // 复制到剪贴板
            Clipboard::copy(unset_cmd).context("Failed to copy unset command to clipboard")?;

            log_success!("Proxy unset command copied to clipboard");
            log_message!(
                "Run this command to disable proxy in current shell: {}",
                unset_cmd
            );
        }

        Ok(())
    }
}
