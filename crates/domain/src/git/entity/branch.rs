//! 分支相关实体

/// 分支信息
///
/// 包含分支的完整元数据和显示信息
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// 原始分支名称（如 "origin/feature" 或 "feature"）
    pub name: String,
    /// 显示名称（如 "\[R\] feature" 或 "feature"）
    pub display_name: String,
    /// 是否为远程分支
    pub is_remote: bool,
    /// 是否为当前分支
    pub is_current: bool,
    /// 最新提交 SHA（短格式，可选）
    pub commit_sha: Option<String>,
    /// 最新提交消息（可选）
    pub commit_message: Option<String>,
    /// 上游分支（可选）
    pub upstream: Option<String>,
}

impl BranchInfo {
    /// 创建本地分支信息
    pub fn local(name: String) -> Self {
        Self {
            display_name: name.clone(),
            name,
            is_remote: false,
            is_current: false,
            commit_sha: None,
            commit_message: None,
            upstream: None,
        }
    }

    /// 创建远程分支信息
    pub fn remote(name: String, display_name: String) -> Self {
        Self {
            name,
            display_name,
            is_remote: true,
            is_current: false,
            commit_sha: None,
            commit_message: None,
            upstream: None,
        }
    }
}

/// 分支过滤器
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchFilter {
    /// 仅本地分支
    Local,
    /// 仅远程分支
    Remote,
    /// 所有分支
    All,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_info_local_defaults() {
        let info = BranchInfo::local("feature/test".to_string());
        assert_eq!(info.name, "feature/test");
        assert_eq!(info.display_name, "feature/test");
        assert!(!info.is_remote);
        assert!(!info.is_current);
        assert!(info.commit_sha.is_none());
        assert!(info.upstream.is_none());
    }

    #[test]
    fn test_branch_info_remote_defaults() {
        let info = BranchInfo::remote(
            "origin/feature/test".to_string(),
            "[R] feature/test".to_string(),
        );
        assert_eq!(info.name, "origin/feature/test");
        assert_eq!(info.display_name, "[R] feature/test");
        assert!(info.is_remote);
        assert!(!info.is_current);
        assert!(info.commit_message.is_none());
    }

    #[test]
    fn test_branch_info_local_with_all_fields() {
        let mut info = BranchInfo::local("main".to_string());
        info.is_current = true;
        info.commit_sha = Some("abc123".to_string());
        info.commit_message = Some("Initial commit".to_string());
        info.upstream = Some("origin/main".to_string());

        assert_eq!(info.name, "main");
        assert_eq!(info.display_name, "main");
        assert!(!info.is_remote);
        assert!(info.is_current);
        assert_eq!(info.commit_sha, Some("abc123".to_string()));
        assert_eq!(info.commit_message, Some("Initial commit".to_string()));
        assert_eq!(info.upstream, Some("origin/main".to_string()));
    }

    #[test]
    fn test_branch_info_remote_with_all_fields() {
        let mut info = BranchInfo::remote(
            "origin/feature/test".to_string(),
            "[R] feature/test".to_string(),
        );
        info.commit_sha = Some("def456".to_string());
        info.commit_message = Some("Add feature".to_string());
        info.upstream = Some("origin/main".to_string());

        assert_eq!(info.name, "origin/feature/test");
        assert_eq!(info.display_name, "[R] feature/test");
        assert!(info.is_remote);
        assert_eq!(info.commit_sha, Some("def456".to_string()));
        assert_eq!(info.commit_message, Some("Add feature".to_string()));
        assert_eq!(info.upstream, Some("origin/main".to_string()));
    }

    #[test]
    fn test_branch_info_clone() {
        let mut info = BranchInfo::local("test".to_string());
        info.is_current = true;
        info.commit_sha = Some("sha123".to_string());
        let cloned = info.clone();

        assert_eq!(info.name, cloned.name);
        assert_eq!(info.display_name, cloned.display_name);
        assert_eq!(info.is_remote, cloned.is_remote);
        assert_eq!(info.is_current, cloned.is_current);
        assert_eq!(info.commit_sha, cloned.commit_sha);
        assert_eq!(info.commit_message, cloned.commit_message);
        assert_eq!(info.upstream, cloned.upstream);
    }

    // ========================================================================
    // BranchFilter 测试
    // ========================================================================

    #[test]
    fn test_branch_filter_variants() {
        assert_eq!(BranchFilter::Local, BranchFilter::Local);
        assert_eq!(BranchFilter::Remote, BranchFilter::Remote);
        assert_eq!(BranchFilter::All, BranchFilter::All);
        assert_ne!(BranchFilter::Local, BranchFilter::Remote);
        assert_ne!(BranchFilter::Local, BranchFilter::All);
        assert_ne!(BranchFilter::Remote, BranchFilter::All);
    }

    #[test]
    fn test_branch_filter_clone() {
        let filter = BranchFilter::All;
        let cloned = filter;
        assert_eq!(filter, cloned);
    }
}
