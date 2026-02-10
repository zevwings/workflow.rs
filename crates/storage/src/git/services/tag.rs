//! Tag 业务逻辑服务
//!
//! 提供 Tag 相关的业务逻辑实现。

use git2::{Direction, PushOptions};
use glob::Pattern;

use super::GitContext;
use domain::{GitError, TagCreateInfo, TagCreateScope, TagDeleteInfo, TagDeleteScope};

/// Tag 服务接口
pub trait TagService: Send + Sync {
    /// 检查本地 tag 是否存在
    fn tag_exists_local(&self, name: &str) -> bool;

    /// 获取远程 tag 列表
    fn get_remote_tags(&self) -> Result<Vec<String>, GitError>;

    /// 检查远程 tag 是否存在
    fn tag_exists_remote(&self, name: &str) -> Result<bool, GitError>;

    /// 删除本地 tag
    fn delete_local_tag(&self, name: &str) -> Result<(), GitError>;

    /// 删除远程 tag
    fn delete_remote_tag(&self, name: &str) -> Result<(), GitError>;

    /// 推送 tag 到远程
    fn push_tag(&self, name: &str, force: bool) -> Result<(), GitError>;

    /// 创建 tag
    fn create_tag(
        &self,
        name: &str,
        target: Option<&str>,
        message: Option<&str>,
        scope: TagCreateScope,
        force: bool,
    ) -> Result<TagCreateInfo, GitError>;

    /// 删除 tag
    fn delete_tag(
        &self,
        name: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<TagDeleteInfo, GitError>;

    /// 根据模式删除 tags
    fn delete_tags_by_pattern(
        &self,
        pattern: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<Vec<TagDeleteInfo>, GitError>;

    /// 获取 tag 列表
    fn list_tags(&self, include_remote: bool) -> Result<Vec<String>, GitError>;

    /// 检查 tag 是否存在
    fn has_tag(&self, name: &str) -> Result<(bool, bool), GitError>;

    /// 预览将要删除的 tags
    fn preview_delete(
        &self,
        name: Option<&str>,
        pattern: Option<&str>,
        scope: TagDeleteScope,
    ) -> Result<Vec<TagDeleteInfo>, GitError>;
}

/// Tag 服务实现
pub struct TagServiceImpl {
    ctx: GitContext,
}

impl TagServiceImpl {
    /// 创建新的 Tag 服务实例
    pub fn new(ctx: GitContext) -> Self {
        Self { ctx }
    }
}

impl TagService for TagServiceImpl {
    fn tag_exists_local(&self, name: &str) -> bool {
        let repo = self.ctx.repository();
        let tag_ref = format!("refs/tags/{}", name);
        let result = repo.find_reference(&tag_ref);
        result.is_ok()
    }

    fn get_remote_tags(&self) -> Result<Vec<String>, GitError> {
        let repo = self.ctx.repository();

        let mut remote = match repo.find_remote("origin") {
            Ok(r) => r,
            Err(_) => return Ok(Vec::new()),
        };

        // 连接到远程
        let callbacks = GitContext::create_callbacks();
        if remote.connect_auth(Direction::Fetch, Some(callbacks), None).is_err() {
            return Ok(Vec::new());
        }

        // 获取远程引用列表
        let list = match remote.list() {
            Ok(l) => l,
            Err(_) => return Ok(Vec::new()),
        };

        let tags: Vec<String> = list
            .iter()
            .filter_map(|head| {
                let name = head.name();
                if let Some(tag_name) = name.strip_prefix("refs/tags/") {
                    // 跳过 ^{} 后缀
                    if !tag_name.ends_with("^{}") {
                        return Some(tag_name.to_string());
                    }
                }
                None
            })
            .collect();

        remote.disconnect().ok();
        Ok(tags)
    }

    fn tag_exists_remote(&self, name: &str) -> Result<bool, GitError> {
        let remote_tags = self.get_remote_tags()?;
        Ok(remote_tags.contains(&name.to_string()))
    }

    fn delete_local_tag(&self, name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();
        let tag_ref = format!("refs/tags/{}", name);

        let mut reference = repo
            .find_reference(&tag_ref)
            .map_err(|_| GitError::OperationFailed(format!("Tag '{}' does not exist", name)))?;

        reference.delete().map_err(|e| GitError::OperationFailed(e.to_string()))?;

        Ok(())
    }

    fn delete_remote_tag(&self, name: &str) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut remote =
            repo.find_remote("origin").map_err(|e| GitError::RemoteError(e.to_string()))?;

        // 使用空引用删除远程 tag
        let refspec = format!(":refs/tags/{}", name);

        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| GitError::RemoteError(e.to_string()))?;

        Ok(())
    }

    fn push_tag(&self, name: &str, force: bool) -> Result<(), GitError> {
        let repo = self.ctx.repository();

        let mut remote =
            repo.find_remote("origin").map_err(|e| GitError::RemoteError(e.to_string()))?;

        let refspec = if force {
            format!("+refs/tags/{}:refs/tags/{}", name, name)
        } else {
            format!("refs/tags/{}:refs/tags/{}", name, name)
        };

        let callbacks = GitContext::create_callbacks();
        let mut opts = PushOptions::new();
        opts.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut opts))
            .map_err(|e| GitError::RemoteError(e.to_string()))?;

        Ok(())
    }

    fn create_tag(
        &self,
        name: &str,
        target: Option<&str>,
        message: Option<&str>,
        scope: TagCreateScope,
        force: bool,
    ) -> Result<TagCreateInfo, GitError> {
        let repo = self.ctx.repository();

        // 检查 tag 是否已存在
        if self.tag_exists_local(name) && !force {
            return Err(GitError::OperationFailed(format!(
                "Tag '{}' already exists, use --force to overwrite",
                name
            )));
        }

        // 如果 force 且 tag 存在，先删除
        if force && self.tag_exists_local(name) {
            self.delete_local_tag(name)?;
        }

        // 解析目标
        let target_oid = if let Some(target_ref) = target {
            let obj = repo
                .revparse_single(target_ref)
                .map_err(|e| GitError::InvalidReference(e.to_string()))?;
            obj.id()
        } else {
            let head = repo.head().map_err(|e| GitError::OperationFailed(e.to_string()))?;
            head.peel_to_commit()
                .map_err(|e| GitError::OperationFailed(e.to_string()))?
                .id()
        };

        // 获取目标对象（提取公共逻辑）
        let obj = repo
            .find_object(target_oid, None)
            .map_err(|e| GitError::OperationFailed(e.to_string()))?;

        // 创建 tag（annotated 或 lightweight）
        if let Some(msg) = message {
            let tagger = self.ctx.get_signature()?;
            repo.tag(name, &obj, &tagger, msg, force)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        } else {
            repo.tag_lightweight(name, &obj, force)
                .map_err(|e| GitError::OperationFailed(e.to_string()))?;
        }

        // 推送到远程
        let created_remote = scope == TagCreateScope::Both && self.push_tag(name, force).is_ok();

        Ok(TagCreateInfo {
            name: name.to_string(),
            created_local: true,
            created_remote,
        })
    }

    fn delete_tag(
        &self,
        name: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<TagDeleteInfo, GitError> {
        let exists_local = self.tag_exists_local(name);
        let exists_remote = self.tag_exists_remote(name)?;

        // 非强制模式下，检查 tag 是否存在
        if !force {
            match scope {
                TagDeleteScope::Local if !exists_local => {
                    return Err(GitError::OperationFailed(format!(
                        "Local tag '{}' does not exist",
                        name
                    )));
                }
                TagDeleteScope::Remote if !exists_remote => {
                    return Err(GitError::OperationFailed(format!(
                        "Remote tag '{}' does not exist",
                        name
                    )));
                }
                TagDeleteScope::Both if !exists_local && !exists_remote => {
                    return Err(GitError::OperationFailed(format!(
                        "Tag '{}' does not exist",
                        name
                    )));
                }
                _ => {}
            }
        }

        match scope {
            TagDeleteScope::Local => {
                if exists_local {
                    self.delete_local_tag(name)?;
                }
            }
            TagDeleteScope::Remote => {
                if exists_remote {
                    self.delete_remote_tag(name)?;
                }
            }
            TagDeleteScope::Both => {
                if exists_local {
                    self.delete_local_tag(name)?;
                }
                if exists_remote {
                    self.delete_remote_tag(name)?;
                }
            }
        }

        Ok(TagDeleteInfo {
            name: name.to_string(),
            exists_local,
            exists_remote,
        })
    }

    fn delete_tags_by_pattern(
        &self,
        pattern: &str,
        scope: TagDeleteScope,
        force: bool,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        let glob_pattern =
            Pattern::new(pattern).map_err(|e| GitError::OperationFailed(e.to_string()))?;

        let local_tags = self.list_tags(false)?;
        let remote_tags = self.get_remote_tags()?;

        // 合并所有 tag
        let mut all_tags: Vec<String> = local_tags.clone();
        for tag in &remote_tags {
            if !all_tags.contains(tag) {
                all_tags.push(tag.clone());
            }
        }

        // 过滤匹配的 tag
        let matched: Vec<String> =
            all_tags.iter().filter(|tag| glob_pattern.matches(tag)).cloned().collect();

        let mut results = Vec::new();
        for tag_name in matched {
            let result = self.delete_tag(&tag_name, scope, force)?;
            results.push(result);
        }

        Ok(results)
    }

    fn list_tags(&self, include_remote: bool) -> Result<Vec<String>, GitError> {
        let repo = self.ctx.repository();
        let mut tags = Vec::new();

        // 获取本地 tags
        let tag_names =
            repo.tag_names(None).map_err(|e| GitError::OperationFailed(e.to_string()))?;

        for i in 0..tag_names.len() {
            if let Some(name) = tag_names.get(i) {
                tags.push(name.to_string());
            }
        }

        // 如果需要，添加远程 tags
        if include_remote {
            let remote_tags = self.get_remote_tags()?;
            for tag in remote_tags {
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }

        tags.sort();
        Ok(tags)
    }

    fn has_tag(&self, name: &str) -> Result<(bool, bool), GitError> {
        let local = self.tag_exists_local(name);
        let remote = self.tag_exists_remote(name)?;
        Ok((local, remote))
    }

    fn preview_delete(
        &self,
        name: Option<&str>,
        pattern: Option<&str>,
        scope: TagDeleteScope,
    ) -> Result<Vec<TagDeleteInfo>, GitError> {
        let local_tags = self.list_tags(false)?;
        let remote_tags = self.get_remote_tags()?;

        let matched_tags: Vec<String> = if let Some(tag_name) = name {
            vec![tag_name.to_string()]
        } else if let Some(pat) = pattern {
            let glob_pattern =
                Pattern::new(pat).map_err(|e| GitError::OperationFailed(e.to_string()))?;

            let mut all_tags: Vec<String> = local_tags.clone();
            for tag in &remote_tags {
                if !all_tags.contains(tag) {
                    all_tags.push(tag.clone());
                }
            }

            all_tags.iter().filter(|tag| glob_pattern.matches(tag)).cloned().collect()
        } else {
            return Err(GitError::OperationFailed(
                "Must provide name or pattern".into(),
            ));
        };

        let results: Vec<TagDeleteInfo> = matched_tags
            .iter()
            .map(|tag_name| {
                let exists_local = local_tags.contains(tag_name);
                let exists_remote = remote_tags.contains(tag_name);

                let (show_local, show_remote) = match scope {
                    TagDeleteScope::Local => (exists_local, false),
                    TagDeleteScope::Remote => (false, exists_remote),
                    TagDeleteScope::Both => (exists_local, exists_remote),
                };

                TagDeleteInfo {
                    name: tag_name.clone(),
                    exists_local: show_local,
                    exists_remote: show_remote,
                }
            })
            .filter(|info| info.exists_local || info.exists_remote)
            .collect();

        Ok(results)
    }
}
