//! Git Tag 管理
//!
//! 本模块提供了 Git tag 相关的操作功能，包括：
//! - 列出所有 tag
//! - 删除本地和远程 tag
//! - 检查 tag 是否存在
//! - 获取 tag 信息

use color_eyre::{eyre::WrapErr, Result};

use super::GitAuth;
use crate::git::helpers::open_repo;
use git2::{Oid, PushOptions};

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
    /// 使用 git2 库列出所有本地 tag。
    ///
    /// # 返回
    ///
    /// 返回所有本地 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn list_local_tags() -> Result<Vec<String>> {
        let repo = crate::git::helpers::open_repo()?;

        let mut tags = Vec::new();
        repo.tag_foreach(|_id, name| {
            if let Ok(name_str) = std::str::from_utf8(name) {
                // 移除 "refs/tags/" 前缀
                let tag_name = name_str.strip_prefix("refs/tags/").unwrap_or(name_str);
                tags.push(tag_name.to_string());
            }
            true
        })
        .wrap_err("Failed to iterate tags")?;

        tags.sort();
        Ok(tags)
    }

    /// 列出所有远程 tag
    ///
    /// 使用 git2 库列出所有远程 tag。
    ///
    /// # 返回
    ///
    /// 返回所有远程 tag 名称的列表（已排序）。
    ///
    /// # 错误
    ///
    /// 如果操作失败，返回相应的错误信息。
    pub fn list_remote_tags() -> Result<Vec<String>> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 连接远程并获取引用列表
        remote
            .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
            .wrap_err("Failed to connect to remote")?;

        let remote_refs = remote.list().wrap_err("Failed to list remote references")?;

        let mut tags = Vec::new();
        for remote_ref in remote_refs {
            let ref_name = remote_ref.name();
            // 提取 tag 名称（移除 refs/tags/ 前缀）
            if let Some(tag_ref) = ref_name.strip_prefix("refs/tags/") {
                // 移除 ^ 后缀（表示 peeled tag）
                let tag_name = tag_ref.strip_suffix("^{}").unwrap_or(tag_ref);
                if !tags.contains(&tag_name.to_string()) {
                    tags.push(tag_name.to_string());
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
                open_repo()
                    .ok()
                    .and_then(|r| {
                        r.find_reference(&format!("refs/tags/{}", tag_name))
                            .ok()
                            .and_then(|ref_| ref_.target())
                            .map(|oid| oid.to_string())
                    })
                    .unwrap_or_default()
            } else if exists_remote {
                // 从远程获取 commit hash（使用 git2）
                // 注意：这里使用 GitCommand 作为回退，因为远程连接可能失败
                // 但这可能效率较低，如果性能是问题，可以考虑保留 GitCommand
                (|| -> Option<String> {
                    let r = open_repo().ok()?;
                    let mut remote = r.find_remote("origin").ok()?;
                    let callbacks = GitAuth::get_remote_callbacks();
                    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None).ok()?;
                    let remote_refs = remote.list().ok()?;
                    // 收集名称和 OID 到 Vec 以避免生命周期问题
                    let refs_info: Vec<(String, git2::Oid)> =
                        remote_refs.iter().map(|r| (r.name().to_string(), r.oid())).collect();
                    refs_info
                        .iter()
                        .find(|(name, _)| name == &format!("refs/tags/{}", tag_name))
                        .map(|(_, oid)| oid.to_string())
                })()
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
    /// 使用 git2 库检查 tag 是否存在。
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
        let repo = crate::git::helpers::open_repo()?;

        // 检查本地 tag
        let tag_ref = format!("refs/tags/{}", tag_name);
        let exists_local = repo.find_reference(&tag_ref).is_ok();

        // 检查远程 tag
        // 注意：git2 无法直接检查远程 tag，需要先 fetch 或使用 ls-remote
        // 为了保持一致性，我们使用 list_remote_tags() 来检查
        // 但这可能效率较低，如果性能是问题，可以考虑保留 GitCommand
        let exists_remote = Self::list_remote_tags()
            .ok()
            .map(|remote_tags| remote_tags.contains(&tag_name.to_string()))
            .unwrap_or(false);

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

        let repo = crate::git::helpers::open_repo()?;

        // 获取 commit hash
        let commit_hash = if exists_local {
            let tag_ref = format!("refs/tags/{}", tag_name);
            let reference =
                repo.find_reference(&tag_ref).wrap_err("Failed to find tag reference")?;
            reference
                .target()
                .ok_or_else(|| color_eyre::eyre::eyre!("Tag reference has no target"))?
                .to_string()
        } else {
            // 从远程获取（使用 git2）
            let mut remote =
                repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;
            let callbacks = GitAuth::get_remote_callbacks();
            remote
                .connect_auth(git2::Direction::Fetch, Some(callbacks), None)
                .wrap_err("Failed to connect to remote")?;
            let remote_refs = remote.list().wrap_err("Failed to list remote references")?;
            // 收集名称和 OID 到 Vec 以避免生命周期问题
            let refs_info: Vec<(String, git2::Oid)> =
                remote_refs.iter().map(|r| (r.name().to_string(), r.oid())).collect();
            refs_info
                .iter()
                .find(|(name, _)| name == &format!("refs/tags/{}", tag_name))
                .map(|(_, oid)| oid.to_string())
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
    /// 使用 git2 库删除本地 tag。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果 tag 不存在或删除失败，返回相应的错误信息。
    pub fn delete_local(tag_name: &str) -> Result<()> {
        let repo = crate::git::helpers::open_repo()?;
        let tag_ref = format!("refs/tags/{}", tag_name);

        let mut reference = repo
            .find_reference(&tag_ref)
            .wrap_err_with(|| format!("Tag '{}' does not exist locally", tag_name))?;

        reference
            .delete()
            .wrap_err_with(|| format!("Failed to delete local tag: {}", tag_name))?;

        Ok(())
    }

    /// 删除远程 tag
    ///
    /// 使用 git2 库删除远程 tag，通过推送空的 refspec 来实现。
    /// 这相当于 `git push origin --delete <tag_name>`。
    ///
    /// # 参数
    ///
    /// * `tag_name` - 要删除的 tag 名称
    ///
    /// # 错误
    ///
    /// 如果删除失败，返回相应的错误信息。
    pub fn delete_remote(tag_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建空的 refspec 来删除远程 tag
        // 格式：:refs/tags/<tag_name> 表示删除远程 tag
        let refspec = format!(":refs/tags/{}", tag_name);

        // 推送空的 refspec 来删除远程 tag
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to delete remote tag: {}", tag_name))?;

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
    /// 使用 git2 库创建 lightweight tag。
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
        let repo = crate::git::helpers::open_repo()?;

        // 获取目标 commit
        let commit = if let Some(sha) = commit_sha {
            let oid =
                Oid::from_str(sha).wrap_err_with(|| format!("Invalid commit SHA: {}", sha))?;
            repo.find_commit(oid).wrap_err_with(|| format!("Commit '{}' not found", sha))?
        } else {
            // 使用当前 HEAD
            repo.head()?.peel_to_commit().wrap_err("Failed to get HEAD commit")?
        };

        // 创建 lightweight tag（指向 commit）
        repo.reference(
            &format!("refs/tags/{}", tag_name),
            commit.id(),
            true,
            &format!("Create tag: {}", tag_name),
        )
        .wrap_err_with(|| format!("Failed to create tag: {}", tag_name))?;

        Ok(())
    }

    /// 推送 tag 到远程
    ///
    /// 使用 git2 库推送 tag 到远程仓库。
    /// 支持 SSH 和 HTTPS 认证，适用于私有仓库。
    ///
    /// # 参数
    ///
    /// * `tag_name` - tag 名称
    ///
    /// # 错误
    ///
    /// 如果推送失败，返回相应的错误信息。
    pub fn push(tag_name: &str) -> Result<()> {
        let repo = open_repo()?;
        let mut remote = repo.find_remote("origin").wrap_err("Failed to find remote 'origin'")?;

        // 获取认证回调
        let callbacks = GitAuth::get_remote_callbacks();

        // 配置推送选项
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // 构建 refspec
        let refspec = format!("refs/tags/{}:refs/tags/{}", tag_name, tag_name);

        // 推送 tag
        remote
            .push(&[&refspec], Some(&mut push_options))
            .wrap_err_with(|| format!("Failed to push tag: {}", tag_name))?;

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
                open_repo()
                    .ok()
                    .and_then(|r| r.head().ok().and_then(|h| h.target()).map(|oid| oid.to_string()))
                    .unwrap_or_default()
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
        let repo = match open_repo() {
            Ok(r) => r,
            Err(_) => return false,
        };
        let commit_oid = match git2::Oid::from_str(commit_sha) {
            Ok(oid) => oid,
            Err(_) => return false,
        };
        let ancestor_oid = match git2::Oid::from_str(ancestor_sha) {
            Ok(oid) => oid,
            Err(_) => return false,
        };
        match repo.merge_base(commit_oid, ancestor_oid) {
            Ok(base) => base == ancestor_oid,
            Err(_) => false,
        }
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
