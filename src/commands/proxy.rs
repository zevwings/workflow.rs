use crate::{log_break, log_info, log_success, log_warning, Clipboard, EnvFile, Proxy};
use anyhow::{Context, Result};

/// 代理检查命令
pub struct ProxyCommand;

impl ProxyCommand {
    /// 检查代理状态和配置
    pub fn check() -> Result<()> {
        // 1. 获取系统代理设置
        log_info!("Reading system proxy settings...");
        let proxy_info =
            Proxy::get_system_proxy().context("Failed to read system proxy settings")?;

        // 2. 检查环境变量中的代理设置
        let env_proxy = Proxy::check_env_proxy();
        let shell_config_env = EnvFile::load().unwrap_or_default();

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
        log_info!("Current environment variables:");

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
            log_info!("  Run 'workflow proxy on' to enable proxy");
            log_info!("  Or check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 开启代理（设置环境变量）
    pub fn on() -> Result<()> {
        // 1. 获取系统代理设置
        log_info!("Reading system proxy settings...");
        let proxy_info =
            Proxy::get_system_proxy().context("Failed to read system proxy settings")?;

        // 2. 检查代理是否已配置
        if Proxy::is_proxy_configured(&proxy_info) {
            log_success!("Proxy is already configured correctly");
            return Ok(());
        }

        // 3. 生成代理命令
        if let Some(proxy_cmd) = Proxy::generate_proxy_command(&proxy_info) {
            log_success!("Proxy command generated:");
            log_info!("  {}", proxy_cmd);

            // 4. 生成环境变量 HashMap 并写入到 ~/.zshrc
            let env_vars = Proxy::generate_env_vars(&proxy_info);
            if !env_vars.is_empty() {
                EnvFile::set_multiple(&env_vars)
                    .context("Failed to save proxy settings to ~/.zshrc")?;

                let shell_config_path =
                    EnvFile::get_shell_config_path().context("Failed to get shell config path")?;
                log_success!("Proxy settings saved to {:?}", shell_config_path);
                log_info!("  The proxy will be enabled when you start a new shell");
                log_info!("  Or run: source {:?}", shell_config_path);
            }

            // 5. 复制到剪贴板
            Clipboard::copy(&proxy_cmd).context("Failed to copy proxy command to clipboard")?;

            log_success!("Proxy command copied to clipboard");
            log_info!("  You can also run it manually: {}", proxy_cmd);
        } else {
            log_warning!("No proxy configuration found in system settings");
            log_info!("  Check macOS System Preferences > Network > Advanced > Proxies");
        }

        Ok(())
    }

    /// 关闭代理（取消环境变量）
    pub fn off() -> Result<()> {
        // 检查当前代理环境变量（包括环境变量和配置文件）
        let env_proxy = Proxy::check_env_proxy();
        let shell_config_env = EnvFile::load().unwrap_or_default();

        // 检查配置文件中的代理设置
        let has_http_proxy = shell_config_env.contains_key("http_proxy");
        let has_https_proxy = shell_config_env.contains_key("https_proxy");
        let has_all_proxy = shell_config_env.contains_key("all_proxy");
        let has_env_http_proxy = env_proxy.contains_key("http_proxy");
        let has_env_https_proxy = env_proxy.contains_key("https_proxy");
        let has_env_all_proxy = env_proxy.contains_key("all_proxy");

        if !has_http_proxy
            && !has_https_proxy
            && !has_all_proxy
            && !has_env_http_proxy
            && !has_env_https_proxy
            && !has_env_all_proxy
        {
            log_success!("Proxy is already off (no proxy environment variables set)");
            return Ok(());
        }

        // 从 ~/.zshrc 中移除代理配置（包括配置块内外）
        // 1. 从配置块内移除（通过 EnvFile API）
        let mut env_vars = shell_config_env;
        let mut removed_from_block = false;

        if env_vars.remove("http_proxy").is_some() {
            removed_from_block = true;
        }
        if env_vars.remove("https_proxy").is_some() {
            removed_from_block = true;
        }
        if env_vars.remove("all_proxy").is_some() {
            removed_from_block = true;
        }

        if removed_from_block {
            // 保存更新后的配置（移除了代理配置）
            EnvFile::save(&env_vars).context("Failed to remove proxy settings from ~/.zshrc")?;
        }

        // 2. 从整个文件中移除所有代理相关的 export 语句（包括配置块外）
        let proxy_keys = ["http_proxy", "https_proxy", "all_proxy"];
        EnvFile::remove_from_file(&proxy_keys)
            .context("Failed to remove proxy settings from ~/.zshrc")?;

        let shell_config_path =
            EnvFile::get_shell_config_path().context("Failed to get shell config path")?;
        log_success!("Proxy settings removed from {:?}", shell_config_path);
        log_info!("  The proxy will be disabled when you start a new shell");
        log_info!("  Or run: source {:?}", shell_config_path);

        // 生成 unset 命令（用于当前 shell 会话）
        let mut unset_cmds = Vec::new();
        if has_env_http_proxy {
            unset_cmds.push("unset http_proxy".to_string());
        }
        if has_env_https_proxy {
            unset_cmds.push("unset https_proxy".to_string());
        }
        if has_env_all_proxy {
            unset_cmds.push("unset all_proxy".to_string());
        }

        if !unset_cmds.is_empty() {
            let unset_cmd = unset_cmds.join(" && ");

            log_info!("Current proxy environment variables:");
            for (key, value) in &env_proxy {
                log_info!("  {}={}", key, value);
            }

            log_success!("Proxy unset command generated:");
            log_info!("  {}", unset_cmd);

            // 复制到剪贴板
            Clipboard::copy(&unset_cmd).context("Failed to copy unset command to clipboard")?;

            log_success!("Proxy unset command copied to clipboard");
            log_info!(
                "  Run this command to disable proxy in current shell: {}",
                unset_cmd
            );
        }

        Ok(())
    }
}
