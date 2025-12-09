//! 代理配置生成器
//!
//! 负责生成代理命令和环境变量。

use std::collections::HashMap;

use crate::proxy::{ProxyInfo, ProxyType};

/// 代理配置生成器
///
/// 提供生成代理命令和环境变量的功能。
pub struct ProxyConfigGenerator;

impl ProxyConfigGenerator {
    /// 生成代理键值对
    ///
    /// 提取公共逻辑，生成代理环境变量的键值对列表。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回包含代理环境变量的键值对列表（键：`http_proxy`、`https_proxy`、`all_proxy`）。
    fn generate_proxy_pairs(proxy_info: &ProxyInfo) -> Vec<(String, String)> {
        ProxyType::all()
            .filter_map(|pt| {
                proxy_info
                    .get_proxy_url(pt)
                    .map(|url| (pt.env_key().to_string(), url))
            })
            .collect()
    }

    /// 生成代理命令字符串
    ///
    /// 根据代理配置生成 `export` 命令字符串，用于设置环境变量。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回 `export` 命令字符串（如 `export http_proxy=... https_proxy=...`）。
    /// 如果没有启用的代理，返回 `None`。
    pub fn generate_command(proxy_info: &ProxyInfo) -> Option<String> {
        let pairs = Self::generate_proxy_pairs(proxy_info);
        if pairs.is_empty() {
            return None;
        }

        let mut cmd = String::from("export ");
        let parts: Vec<String> = pairs
            .iter()
            .map(|(key, value)| {
                let mut part = String::from(key);
                part.push('=');
                part.push_str(value);
                part
            })
            .collect();
        cmd.push_str(&parts.join(" "));
        Some(cmd)
    }

    /// 生成环境变量 HashMap
    ///
    /// 根据代理配置生成环境变量 HashMap，用于保存到 shell 配置文件。
    ///
    /// # 参数
    ///
    /// * `proxy_info` - 代理信息结构体
    ///
    /// # 返回
    ///
    /// 返回包含代理环境变量的 HashMap（键：`http_proxy`、`https_proxy`、`all_proxy`）。
    pub fn generate_env_vars(proxy_info: &ProxyInfo) -> HashMap<String, String> {
        Self::generate_proxy_pairs(proxy_info).into_iter().collect()
    }
}
