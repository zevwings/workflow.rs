//! Git Tag 管理
//!
//! 本模块提供了 Git tag 相关的操作功能，包括：
//! - 列出所有 tag
//! - 删除本地和远程 tag
//! - 检查 tag 是否存在
//! - 获取 tag 信息

use color_eyre::{eyre::WrapErr, Result};

use super::GitCommand;

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
        let output = GitCommand::new(["tag", "-l"]).read().wrap_err("Failed to list local tags")?;

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
        let output = GitCommand::new(["ls-remote", "--tags", "origin"])
            .read()
            .wrap_err("Failed to list remote tags")?;

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
                GitCommand::new(["rev-parse", &tag_name])
                    .read()
                    .unwrap_or_else(|_| String::new())
            } else if exists_remote {
                // 从远程获取 commit hash
                let output =
                    GitCommand::new(["ls-remote", "origin", &format!("refs/tags/{}", tag_name)])
                        .read()
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
        let exists_local =
            GitCommand::new(["rev-parse", "--verify", &format!("refs/tags/{}", tag_name)])
                .quiet_success();

        // 检查远程 tag
        let exists_remote = GitCommand::new([
            "ls-remote",
            "--exit-code",
            "origin",
            &format!("refs/tags/{}", tag_name),
        ])
        .quiet_success();

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
            GitCommand::new(["rev-parse", tag_name])
                .read()
                .wrap_err("Failed to get tag commit hash")?
        } else {
            // 从远程获取
            let output =
                GitCommand::new(["ls-remote", "origin", &format!("refs/tags/{}", tag_name)])
                    .read()
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
        GitCommand::new(["tag", "-d", tag_name])
            .run()
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
        let result = GitCommand::new(["push", "origin", "--delete", tag_name]).run();

        if result.is_err() {
            // 回退到使用 :refs/tags/ 方式
            GitCommand::new(["push", "origin", &format!(":refs/tags/{}", tag_name)])
                .run()
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

    /// 创建 tag（基于指定的 commit SHA）
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    /// * `commit_sha` - 可选的 commit SHA，如果不提供则使用当前 HEAD
    ///
    /// # 错误
    ///
    /// 如果 tag 创建失败，返回相应的错误信息。
    pub fn create(tag_name: &str, commit_sha: Option<&str>) -> Result<()> {
        let mut cmd = GitCommand::new(["tag", tag_name]);
        if let Some(sha) = commit_sha {
            cmd = GitCommand::new(["tag", tag_name, sha]);
        }
        cmd.run().wrap_err_with(|| format!("Failed to create tag: {}", tag_name))
    }

    /// 推送 tag 到远程
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push(tag_name: &str) -> Result<()> {
        GitCommand::new(["push", "origin", tag_name])
            .run()
            .wrap_err_with(|| format!("Failed to push tag: {}", tag_name))
    }

    /// 创建并推送 tag
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    /// * `commit_sha` - 可选的 commit SHA，如果不提供则使用当前 HEAD
    ///
    /// # 错误
    ///
    /// 如果创建或推送失败，返回相应的错误信息。
    pub fn create_and_push(tag_name: &str, commit_sha: Option<&str>) -> Result<()> {
        // 检查 tag 是否已存在
        let (exists_local, exists_remote) = Self::is_tag_exists(tag_name)?;

        if exists_local || exists_remote {
            // 获取现有 tag 的 commit SHA
            let existing_tag_info = Self::get_tag_info(tag_name)?;
            let target_sha = commit_sha.map(|s| s.to_string()).unwrap_or_else(|| {
                GitCommand::new(["rev-parse", "HEAD"]).read().unwrap_or_default()
            });

            if existing_tag_info.commit_hash == target_sha {
                // Tag 已存在且指向正确的 commit
                return Ok(());
            } else {
                // Tag 已存在但指向不同的 commit，需要删除后重新创建
                if exists_local {
                    Self::delete_local(tag_name)?;
                }
                if exists_remote {
                    Self::delete_remote(tag_name)?;
                }
            }
        }

        // 创建 tag
        Self::create(tag_name, commit_sha)?;

        // 推送 tag
        Self::push(tag_name)?;

        Ok(())
    }

    /// 列出所有 alpha tag
    ///
    /// 查找所有匹配 `*.alpha-*` 格式的 tag。
    ///
    /// # 返回
    ///
    /// 返回所有 alpha tag 名称的列表（已排序）。
    pub fn list_alpha_tags() -> Result<Vec<String>> {
        let all_tags = Self::list_local_tags()?;
        let alpha_tags: Vec<String> =
            all_tags.into_iter().filter(|tag| tag.contains(".alpha-")).collect();
        Ok(alpha_tags)
    }

    /// 检查 commit 是否在指定 commit 的祖先中
    ///
    /// # 参数
    ///
    /// * `commit_sha` - 要检查的 commit SHA
    /// * `ancestor_sha` - 祖先 commit SHA
    ///
    /// # 返回
    ///
    /// 如果 `commit_sha` 是 `ancestor_sha` 的祖先，返回 `true`。
    pub fn is_ancestor(commit_sha: &str, ancestor_sha: &str) -> bool {
        GitCommand::new(["merge-base", "--is-ancestor", commit_sha, ancestor_sha]).quiet_success()
    }

    /// 提取 tag 的版本号
    ///
    /// 从 tag 名称中提取版本号（例如：`v1.6.0.alpha-001` -> `1.6.0`）。
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    ///
    /// # 返回
    ///
    /// 返回版本号字符串，如果无法提取则返回 `None`。
    pub fn extract_version(tag_name: &str) -> Option<String> {
        use regex::Regex;
        let re = Regex::new(r"^v?([0-9]+\.[0-9]+\.[0-9]+)").ok()?;
        re.captures(tag_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}
