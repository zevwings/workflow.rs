//! Git Tag 管理
//!
//! 本模块提供了 Git tag 相关的操作功能，包括：
//! - 列出所有 tag
//! - 删除本地和远程 tag
//! - 检查 tag 是否存在
//! - 获取 tag 信息

use color_eyre::{eyre::WrapErr, Result};

use super::helpers::{check_ref_exists, check_success, cmd_read, cmd_run};

/// Tag 信息
#[derive(Debug, Clone)]
pub struct TagInfo {
    /// Tag 名称
    pub name: String,
    /// Tag 指向的 commit hash
    pub commit_hash: String,
    /// Tag 是否在本地存在
    pub exists_local: bool,
    /// Tag 是否在远程存在
    pub exists_remote: bool,
}

/// Git Tag 管理
///
/// 提供 tag 相关的操作功能，包括：
/// - 列出所有 tag
/// - 删除本地和远程 tag
/// - 检查 tag 是否存在
pub struct GitTag;

impl GitTag {
    /// 列出所有本地 tag
    ///
    /// 使用 `git tag` 列出所有本地 tag。
    ///
    /// # 返回
    ///
    /// 返回所有本地 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn list_local_tags() -> Result<Vec<String>> {
        let output = cmd_read(&["tag", "-l"]).wrap_err("Failed to list local tags")?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut tags: Vec<String> = output.lines().map(|s| s.trim().to_string()).collect();
        tags.sort();
        Ok(tags)
    }

    /// 列出所有远程 tag
    ///
    /// 使用 `git ls-remote --tags` 列出所有远程 tag。
    ///
    /// # 返回
    ///
    /// 返回所有远程 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn list_remote_tags() -> Result<Vec<String>> {
        let output =
            cmd_read(&["ls-remote", "--tags", "origin"]).wrap_err("Failed to list remote tags")?;

        if output.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut tags = Vec::new();
        for line in output.lines() {
            // 格式：<commit_hash>	refs/tags/<tag_name>
            // 或者：<commit_hash>	refs/tags/<tag_name>^{} (peeled tag)
            if let Some(tag_part) = line.split_whitespace().nth(1) {
                if let Some(tag_name) = tag_part.strip_prefix("refs/tags/") {
                    // 移除 ^{} 后缀（peeled tag 引用）
                    let tag_name = tag_name.strip_suffix("^{}").unwrap_or(tag_name);
                    if !tags.contains(&tag_name.to_string()) {
                        tags.push(tag_name.to_string());
                    }
                }
            }
        }

        tags.sort();
        Ok(tags)
    }

    /// 列出所有 tag（本地和远程）
    ///
    /// 返回所有 tag 的详细信息，包括本地和远程存在状态。
    ///
    /// # 返回
    ///
    /// 返回所有 tag 的信息列表。
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn list_all_tags() -> Result<Vec<TagInfo>> {
        let local_tags = Self::list_local_tags()?;
        let remote_tags = Self::list_remote_tags()?;

        // 合并本地和远程 tag，去重
        let all_tag_names: std::collections::HashSet<String> =
            local_tags.iter().chain(remote_tags.iter()).cloned().collect();

        let mut tags = Vec::new();
        for tag_name in all_tag_names {
            let exists_local = local_tags.contains(&tag_name);
            let exists_remote = remote_tags.contains(&tag_name);

            // 获取 tag 指向的 commit hash
            let commit_hash = if exists_local {
                cmd_read(&["rev-parse", &tag_name]).unwrap_or_else(|_| String::new())
            } else if exists_remote {
                // 从远程获取 commit hash
                let output = cmd_read(&["ls-remote", "origin", &format!("refs/tags/{}", tag_name)])
                    .unwrap_or_else(|_| String::new());
                output.split_whitespace().next().unwrap_or("").to_string()
            } else {
                String::new()
            };

            tags.push(TagInfo {
                name: tag_name,
                commit_hash,
                exists_local,
                exists_remote,
            });
        }

        // 按名称排序
        tags.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(tags)
    }

    /// 检查 tag 是否存在（本地或远程）
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要检查的 tag 名称
    ///
    /// # 返回
    ///
    /// 返回元组 `(本地存在, 远程存在)`：
    /// - `(true, true)` - tag 在本地和远程都存在
    /// - `(true, false)` - tag 只在本地存在
    /// - `(false, true)` - tag 只在远程存在
    /// - `(false, false)` - tag 不存在
    ///
    /// # 错误
    ///
    /// 如果 Git 命令执行失败，返回相应的错误信息。
    pub fn is_tag_exists(tag_name: &str) -> Result<(bool, bool)> {
        // 检查本地 tag
        let exists_local = check_ref_exists(&format!("refs/tags/{}", tag_name));

        // 检查远程 tag
        let exists_remote = check_success(&[
            "ls-remote",
            "--exit-code",
            "origin",
            &format!("refs/tags/{}", tag_name),
        ]);

        Ok((exists_local, exists_remote))
    }

    /// 获取 tag 信息
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    ///
    /// # 返回
    ///
    /// 返回 tag 信息，如果 tag 不存在则返回错误。
    ///
    /// # 错误
    ///
    /// 如果 tag 不存在或 Git 命令执行失败，返回相应的错误信息。
    pub fn get_tag_info(tag_name: &str) -> Result<TagInfo> {
        let (exists_local, exists_remote) = Self::is_tag_exists(tag_name)?;

        if !exists_local && !exists_remote {
            return Err(color_eyre::eyre::eyre!("Tag '{}' does not exist", tag_name));
        }

        // 获取 commit hash
        let commit_hash = if exists_local {
            cmd_read(&["rev-parse", tag_name]).wrap_err("Failed to get tag commit hash")?
        } else {
            // 从远程获取
            let output = cmd_read(&["ls-remote", "origin", &format!("refs/tags/{}", tag_name)])
                .wrap_err("Failed to get remote tag commit hash")?;
            output.split_whitespace().next().unwrap_or("").to_string()
        };

        Ok(TagInfo {
            name: tag_name.to_string(),
            commit_hash,
            exists_local,
            exists_remote,
        })
    }

    /// 删除本地 tag
    ///
    /// 使用 `git tag -d` 删除本地 tag。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果 tag 不存在或删除失败，返回相应的错误信息。
    pub fn delete_local(tag_name: &str) -> Result<()> {
        cmd_run(&["tag", "-d", tag_name])
            .wrap_err_with(|| format!("Failed to delete local tag: {}", tag_name))
    }

    /// 删除远程 tag
    ///
    /// 使用 `git push origin --delete` 删除远程 tag。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(tag_name: &str) -> Result<()> {
        // 尝试使用 --delete 方式
        let result = cmd_run(&["push", "origin", "--delete", tag_name]);

        if result.is_err() {
            // 回退到使用 :refs/tags/ 方式
            cmd_run(&["push", "origin", &format!(":refs/tags/{}", tag_name)])
                .wrap_err_with(|| format!("Failed to delete remote tag: {}", tag_name))?;
        }

        Ok(())
    }

    /// 删除本地和远程 tag
    ///
    /// 同时删除本地和远程 tag。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_both(tag_name: &str) -> Result<()> {
        let (exists_local, exists_remote) = Self::is_tag_exists(tag_name)?;

        // 删除本地 tag（如果存在）
        if exists_local {
            if let Err(e) = Self::delete_local(tag_name) {
                // 记录错误但继续删除远程 tag
                use crate::base::logger::console::Logger;
                Logger::print_warning(format!("Failed to delete local tag: {}", e));
            }
        }

        // 删除远程 tag（如果存在）
        if exists_remote {
            Self::delete_remote(tag_name)?;
        }

        Ok(())
    }
}
