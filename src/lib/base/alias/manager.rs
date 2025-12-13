//! 别名管理器
//!
//! 提供别名的加载、展开和管理功能。

use color_eyre::{eyre::WrapErr, Result};
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::base::settings::paths::Paths;
use crate::base::settings::Settings;

/// 别名管理器
///
/// 提供别名的加载、展开和管理功能。
pub struct AliasManager;

impl AliasManager {
    /// 加载别名配置
    ///
    /// 从 `workflow.toml` 配置文件中加载别名配置。
    ///
    /// # 返回
    ///
    /// 返回别名映射表（别名名称 -> 命令）。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取或解析失败，返回相应的错误信息。
    pub fn load() -> Result<HashMap<String, String>> {
        let settings = Settings::get();
        Ok(settings.aliases.clone())
    }

    /// 展开别名（支持嵌套）
    ///
    /// 递归展开别名，支持嵌套别名（别名引用别名）。
    /// 使用 `HashSet` 跟踪已访问的别名，防止循环引用。
    ///
    /// # 参数
    ///
    /// * `alias` - 要展开的别名名称
    /// * `visited` - 已访问的别名集合（用于检测循环）
    /// * `depth` - 当前展开深度（用于限制最大深度）
    ///
    /// # 返回
    ///
    /// 返回展开后的命令字符串。
    ///
    /// # 错误
    ///
    /// - 如果别名不存在，返回错误
    /// - 如果检测到循环引用，返回错误
    /// - 如果超过最大展开深度（10 层），返回错误
    pub fn expand(alias: &str, visited: &mut HashSet<String>, depth: usize) -> Result<String> {
        const MAX_DEPTH: usize = 10;

        // 检查深度限制
        if depth > MAX_DEPTH {
            return Err(color_eyre::eyre::eyre!(
                "Alias expansion depth exceeded maximum: {}",
                MAX_DEPTH
            ));
        }

        // 检查循环引用
        if visited.contains(alias) {
            return Err(color_eyre::eyre::eyre!(
                "Circular alias detected: {}",
                alias
            ));
        }

        // 加载别名配置
        let aliases = Self::load()?;

        // 检查别名是否存在
        let command = aliases
            .get(alias)
            .ok_or_else(|| color_eyre::eyre::eyre!("Alias not found: {}", alias))?;

        // 标记为已访问
        visited.insert(alias.to_string());

        // 检查命令是否包含其他别名（递归展开）
        let parts: Vec<&str> = command.split_whitespace().collect();
        if let Some(first_part) = parts.first() {
            if aliases.contains_key(*first_part) {
                // 递归展开嵌套别名
                let expanded = Self::expand(first_part, visited, depth + 1)?;
                // 将展开后的命令与剩余部分组合
                let mut result: Vec<&str> = expanded.split_whitespace().collect();
                result.extend_from_slice(&parts[1..]);
                return Ok(result.join(" "));
            }
        }

        Ok(command.clone())
    }

    /// 展开命令行参数（处理第一个参数是否为别名）
    ///
    /// 检查第一个参数是否是别名，如果是则展开为完整命令。
    /// 保留剩余参数不变。
    ///
    /// # 参数
    ///
    /// * `args` - 命令行参数向量（第一个元素通常是程序名）
    ///
    /// # 返回
    ///
    /// 返回展开后的命令行参数向量。
    ///
    /// # 错误
    ///
    /// 如果别名展开失败，返回相应的错误信息。
    pub fn expand_args(args: Vec<String>) -> Result<Vec<String>> {
        // 如果参数少于 2 个（只有程序名），直接返回
        if args.len() < 2 {
            return Ok(args);
        }

        // 获取第一个参数（命令名）
        let first_arg = &args[1];

        // 加载别名配置
        let aliases = Self::load()?;

        // 检查第一个参数是否是别名
        if aliases.contains_key(first_arg) {
            // 展开别名
            let mut visited = HashSet::new();
            let expanded = Self::expand(first_arg, &mut visited, 0)?;

            // 将展开后的命令分割为参数
            let mut expanded_args: Vec<String> =
                expanded.split_whitespace().map(|s| s.to_string()).collect();

            // 保留原始参数中的程序名和剩余参数
            let mut result = vec![args[0].clone()];
            result.append(&mut expanded_args);
            result.extend_from_slice(&args[2..]);

            Ok(result)
        } else {
            // 不是别名，直接返回原参数
            Ok(args)
        }
    }

    /// 添加别名
    ///
    /// 将新别名添加到配置文件中。
    ///
    /// # 参数
    ///
    /// * `name` - 别名名称
    /// * `command` - 别名对应的命令
    ///
    /// # 错误
    ///
    /// 如果配置文件读写失败，返回相应的错误信息。
    pub fn add(name: &str, command: &str) -> Result<()> {
        let config_path = Paths::workflow_config()?;
        let mut settings = Settings::get().clone();

        // 添加别名
        settings.aliases.insert(name.to_string(), command.to_string());

        // 保存配置
        let toml_content =
            toml::to_string_pretty(&settings).wrap_err("Failed to serialize settings to TOML")?;
        fs::write(&config_path, toml_content)
            .wrap_err(format!("Failed to write config file: {:?}", config_path))?;

        // 设置文件权限（仅 Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
                .wrap_err("Failed to set config file permissions")?;
        }

        Ok(())
    }

    /// 删除别名
    ///
    /// 从配置文件中删除指定的别名。
    ///
    /// # 参数
    ///
    /// * `name` - 要删除的别名名称
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果别名存在并被删除，`false` 如果别名不存在。
    ///
    /// # 错误
    ///
    /// 如果配置文件读写失败，返回相应的错误信息。
    pub fn remove(name: &str) -> Result<bool> {
        let config_path = Paths::workflow_config()?;
        let mut settings = Settings::get().clone();

        // 检查别名是否存在
        if !settings.aliases.contains_key(name) {
            return Ok(false);
        }

        // 删除别名
        settings.aliases.remove(name);

        // 保存配置
        let toml_content =
            toml::to_string_pretty(&settings).wrap_err("Failed to serialize settings to TOML")?;
        fs::write(&config_path, toml_content)
            .wrap_err(format!("Failed to write config file: {:?}", config_path))?;

        // 设置文件权限（仅 Unix）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
                .wrap_err("Failed to set config file permissions")?;
        }

        Ok(true)
    }

    /// 列出所有别名
    ///
    /// 返回所有已定义的别名映射表。
    ///
    /// # 返回
    ///
    /// 返回别名映射表（别名名称 -> 命令）。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取失败，返回相应的错误信息。
    pub fn list() -> Result<HashMap<String, String>> {
        Self::load()
    }

    /// 检查别名是否存在
    ///
    /// # 参数
    ///
    /// * `name` - 别名名称
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果别名存在，`false` 如果不存在。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取失败，返回相应的错误信息。
    pub fn exists(name: &str) -> Result<bool> {
        let aliases = Self::load()?;
        Ok(aliases.contains_key(name))
    }

    /// 检查循环别名
    ///
    /// 检查添加新别名 `name` 指向 `target` 是否会导致循环引用。
    ///
    /// # 参数
    ///
    /// * `name` - 新别名名称
    /// * `target` - 新别名指向的命令
    ///
    /// # 返回
    ///
    /// 返回 `true` 如果会导致循环引用，`false` 如果不会。
    ///
    /// # 错误
    ///
    /// 如果配置文件读取失败，返回相应的错误信息。
    pub fn check_circular(name: &str, target: &str) -> Result<bool> {
        let aliases = Self::load()?;

        // 检查 target 的第一个词是否是别名
        let parts: Vec<&str> = target.split_whitespace().collect();
        if let Some(first_part) = parts.first() {
            // 如果第一个词是 name 本身，直接返回循环
            if *first_part == name {
                return Ok(true);
            }

            // 如果第一个词是已存在的别名，检查是否会形成循环
            if aliases.contains_key(*first_part) {
                let mut visited = HashSet::new();
                visited.insert(name.to_string());

                // 尝试展开这个别名，看是否会回到 name
                if let Ok(expanded) = Self::expand(first_part, &mut visited, 0) {
                    let expanded_parts: Vec<&str> = expanded.split_whitespace().collect();
                    if let Some(expanded_first) = expanded_parts.first() {
                        if *expanded_first == name {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_expand_basic_alias() {
        // 这个测试需要实际的配置文件，暂时跳过
        // 在实际集成测试中测试
    }

    #[test]
    fn test_circular_detection() {
        // 这个测试需要实际的配置文件，暂时跳过
        // 在实际集成测试中测试
    }
}
