//! Git Tag 管理
//!
//! 本模块提供了 Git tag 相关的操作功能，包括：
//! - 列出所有 tag
//! - 删除本地和远程 tag
//! - 检查 tag 是否存在
//! - 获取 tag 信息

use color_eyre::{eyre::WrapErr, Result};
use regex::Regex;

use super::GitRepository;
use crate::base::logger::console::Logger;
use crate::git::commands::{tag::GitTagCommand, GitCommitCommand};

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
    /// 使用 GitCommand 列出所有本地 tag。
    ///
    /// # 返回
    ///
    /// 返回所有本地 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn list_local_tags() -> Result<Vec<String>> {
        let repo = GitRepository::open()?;
        GitTagCommand::list_local_tags(Some(repo.path()))
    }

    /// 列出所有远程 tag
    ///
    /// 使用 GitCommand 列出所有远程 tag。
    ///
    /// # 返回
    ///
    /// 返回所有远程 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn list_remote_tags() -> Result<Vec<String>> {
        let repo = GitRepository::open()?;
        GitTagCommand::list_remote_tags(None, Some(repo.path()))
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
        let repo = GitRepository::open()?;
        let repo_path = repo.path();

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
                GitTagCommand::get_tag_commit(&tag_name, Some(repo_path)).unwrap_or_default()
            } else if exists_remote {
                // 从远程获取 commit hash（使用 ls-remote）
                let mut repo_for_remote = GitRepository::open().ok();
                let remote_refs = repo_for_remote
                    .as_mut()
                    .and_then(|r| r.find_origin_remote().ok())
                    .and_then(|r| r.list().ok())
                    .unwrap_or_default();
                remote_refs
                    .iter()
                    .find(|(name, _)| name == &format!("refs/tags/{}", tag_name))
                    .map(|(_, sha)| sha.clone())
                    .unwrap_or_default()
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
    /// 使用 GitCommand 检查 tag 是否存在。
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
    /// 如果操作失败，返回相应的错误信息。
    pub fn is_tag_exists(tag_name: &str) -> Result<(bool, bool)> {
        let repo = GitRepository::open()?;
        GitTagCommand::tag_exists(tag_name, None, Some(repo.path()))
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
        let repo = GitRepository::open()?;
        let repo_path = repo.path();

        let (exists_local, exists_remote) = Self::is_tag_exists(tag_name)?;

        if !exists_local && !exists_remote {
            return Err(color_eyre::eyre::eyre!("Tag '{}' does not exist", tag_name));
        }

        // 获取 commit hash
        let commit_hash = if exists_local {
            GitTagCommand::get_tag_commit(tag_name, Some(repo_path))
                .wrap_err("Failed to get tag commit hash")?
        } else {
            // 从远程获取
            let mut repo_for_remote = GitRepository::open()?;
            let remote = repo_for_remote.find_origin_remote()?;
            let remote_refs = remote.list().wrap_err("Failed to list remote references")?;
            remote_refs
                .iter()
                .find(|(name, _)| name == &format!("refs/tags/{}", tag_name))
                .map(|(_, sha)| sha.clone())
                .ok_or_else(|| color_eyre::eyre::eyre!("Failed to get remote tag commit hash"))?
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
    /// 使用 GitCommand 删除本地 tag。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果 tag 不存在或删除失败，返回相应的错误信息。
    pub fn delete_local(tag_name: &str) -> Result<()> {
        let repo = GitRepository::open()?;
        GitTagCommand::delete_local(tag_name, Some(repo.path()))
    }

    /// 删除远程 tag
    ///
    /// 使用 GitCommand 删除远程 tag，通过推送空的 refspec 来实现。
    /// 这相当于 `git push origin :refs/tags/<tag_name>`。
    /// 包含超时和重试机制，提高网络操作的可靠性。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(tag_name: &str) -> Result<()> {
        use crate::base::resilience::{
            default_download_timeout, execute_with_timeout_and_retry, RetryConfig, TimeoutConfig,
        };

        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();
        let tag_name = tag_name.to_string();

        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            move || -> Result<()> {
                let repo = GitRepository::open()?;
                GitTagCommand::delete_remote(&tag_name, None, Some(repo.path()))
            },
            "Deleting remote tag",
        )?;
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
    /// 使用 GitCommand 创建 lightweight tag。
    /// 如果提供了 commit SHA，则在指定 commit 上创建 tag；否则在当前 HEAD 上创建。
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
        let repo = GitRepository::open()?;
        GitTagCommand::create_tag(tag_name, commit_sha, Some(repo.path()))
    }

    /// 推送 tag 到远程
    ///
    /// 使用 GitCommand 推送 tag 到远程仓库。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    /// 包含超时和重试机制，提高网络操作的可靠性。
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push(tag_name: &str) -> Result<()> {
        use crate::base::resilience::{
            default_download_timeout, execute_with_timeout_and_retry, RetryConfig, TimeoutConfig,
        };

        let timeout_config =
            TimeoutConfig::new(default_download_timeout()).with_platform_specific();
        let retry_config = RetryConfig::platform_default();
        let tag_name = tag_name.to_string();

        execute_with_timeout_and_retry(
            timeout_config,
            retry_config,
            move || -> Result<()> {
                let repo = GitRepository::open()?;
                GitTagCommand::push_tag(&tag_name, None, Some(repo.path()))
            },
            "Pushing tag to remote",
        )?;
        Ok(())
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
                GitRepository::open().ok().and_then(|r| r.head().ok()).unwrap_or_default()
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
        let repo = match GitRepository::open() {
            Ok(r) => r,
            Err(_) => return false,
        };

        // 使用 git merge-base 命令检查
        GitCommitCommand::is_ancestor(ancestor_sha, commit_sha, Some(repo.path()))
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
        let re = Regex::new(r"^v?([0-9]+\.[0-9]+\.[0-9]+)").ok()?;
        re.captures(tag_name)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
}
