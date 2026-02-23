//! 值对象测试数据构建器
//!
//! 提供领域值对象的 Builder 或工厂方法，便于在测试中创建 PrContent、PullRequestStatus 等。

use crate::{FileStatusInfo, FileStatusType, PrContent, PullRequestStatus, WorkingTreeStatus};

/// 值对象测试数据工厂
///
/// 提供值对象的构建器或预定义实例，用于测试中组合到实体或服务调用中。
pub struct TestValueObjectFactory;

impl TestValueObjectFactory {
    /// 创建 PR 状态构建器
    pub fn pull_request_status() -> PullRequestStatusBuilder {
        PullRequestStatusBuilder::default()
    }

    /// 创建 PR 内容构建器
    pub fn pr_content() -> PrContentBuilder {
        PrContentBuilder::default()
    }

    /// 创建文件状态信息构建器
    pub fn file_status_info() -> FileStatusInfoBuilder {
        FileStatusInfoBuilder::default()
    }

    /// 创建工作树状态构建器
    pub fn working_tree_status() -> WorkingTreeStatusBuilder {
        WorkingTreeStatusBuilder::default()
    }

    /// 返回默认的「open」PR 状态
    pub fn open_pr_status() -> PullRequestStatus {
        PullRequestStatus {
            state: "open".to_string(),
            merged: false,
            merged_at: None,
        }
    }

    /// 返回默认的「merged」PR 状态
    pub fn merged_pr_status() -> PullRequestStatus {
        PullRequestStatus {
            state: "closed".to_string(),
            merged: true,
            merged_at: Some("2024-01-01T12:00:00Z".to_string()),
        }
    }

    /// 返回干净的工作树状态（无任何变更）
    pub fn clean_working_tree() -> WorkingTreeStatus {
        WorkingTreeStatus {
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            conflicted: vec![],
        }
    }
}

// =============================================================================
// PullRequestStatus 构建器
// =============================================================================

/// PR 状态构建器
#[derive(Default)]
pub struct PullRequestStatusBuilder {
    state: Option<String>,
    merged: bool,
    merged_at: Option<String>,
}

impl PullRequestStatusBuilder {
    /// 设置为 open 状态
    pub fn open(mut self) -> Self {
        self.state = Some("open".to_string());
        self.merged = false;
        self.merged_at = None;
        self
    }

    /// 设置为 closed 状态
    pub fn closed(mut self) -> Self {
        self.state = Some("closed".to_string());
        self
    }

    /// 标记为已合并
    pub fn merged(mut self, merged_at: Option<impl Into<String>>) -> Self {
        self.merged = true;
        self.state = Some("closed".to_string());
        self.merged_at = merged_at.map(|s| s.into());
        self
    }

    /// 设置状态字符串
    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// 构建 PullRequestStatus
    pub fn build(self) -> PullRequestStatus {
        PullRequestStatus {
            state: self.state.unwrap_or_else(|| "open".to_string()),
            merged: self.merged,
            merged_at: self.merged_at,
        }
    }
}

// =============================================================================
// PrContent 构建器
// =============================================================================

/// PR 内容构建器
#[derive(Default)]
pub struct PrContentBuilder {
    title: Option<String>,
    description: Option<String>,
}

impl PrContentBuilder {
    /// 设置标题
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 构建 PrContent
    pub fn build(self) -> PrContent {
        PrContent {
            title: self.title.unwrap_or_else(|| "Test PR Title".to_string()),
            description: self.description.unwrap_or_else(|| "Test PR description.".to_string()),
        }
    }
}

// =============================================================================
// FileStatusInfo 构建器
// =============================================================================

/// 文件状态信息构建器
#[derive(Default)]
pub struct FileStatusInfoBuilder {
    path: Option<String>,
    status_type: Option<FileStatusType>,
    old_path: Option<String>,
}

impl FileStatusInfoBuilder {
    /// 设置文件路径
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// 设置状态类型
    pub fn with_status_type(mut self, status_type: FileStatusType) -> Self {
        self.status_type = Some(status_type);
        self
    }

    /// 设置原路径（重命名时）
    pub fn with_old_path(mut self, old_path: impl Into<String>) -> Self {
        self.old_path = Some(old_path.into());
        self
    }

    /// 构建 FileStatusInfo
    pub fn build(self) -> FileStatusInfo {
        FileStatusInfo {
            path: self.path.unwrap_or_else(|| "file.txt".to_string()),
            status_type: self.status_type.unwrap_or(FileStatusType::NewUntracked),
            old_path: self.old_path,
        }
    }
}

// =============================================================================
// WorkingTreeStatus 构建器
// =============================================================================

/// 工作树状态构建器
#[derive(Default)]
pub struct WorkingTreeStatusBuilder {
    staged: Vec<FileStatusInfo>,
    unstaged: Vec<FileStatusInfo>,
    untracked: Vec<FileStatusInfo>,
    conflicted: Vec<FileStatusInfo>,
}

impl WorkingTreeStatusBuilder {
    /// 添加已暂存文件
    pub fn add_staged(mut self, info: FileStatusInfo) -> Self {
        self.staged.push(info);
        self
    }

    /// 添加未暂存修改
    pub fn add_unstaged(mut self, info: FileStatusInfo) -> Self {
        self.unstaged.push(info);
        self
    }

    /// 添加未跟踪文件
    pub fn add_untracked(mut self, info: FileStatusInfo) -> Self {
        self.untracked.push(info);
        self
    }

    /// 添加冲突文件
    pub fn add_conflicted(mut self, info: FileStatusInfo) -> Self {
        self.conflicted.push(info);
        self
    }

    /// 构建 WorkingTreeStatus
    pub fn build(self) -> WorkingTreeStatus {
        WorkingTreeStatus {
            staged: self.staged,
            unstaged: self.unstaged,
            untracked: self.untracked,
            conflicted: self.conflicted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pull_request_status_builder_open() {
        let status = TestValueObjectFactory::pull_request_status().open().build();
        assert_eq!(status.state, "open");
        assert!(!status.merged);
        assert!(status.merged_at.is_none());
    }

    #[test]
    fn test_pull_request_status_builder_merged() {
        let status = TestValueObjectFactory::pull_request_status()
            .merged(Some("2024-01-01T00:00:00Z"))
            .build();
        assert!(status.merged);
        assert_eq!(status.merged_at.as_deref(), Some("2024-01-01T00:00:00Z"));
    }

    #[test]
    fn test_pr_content_builder_default() {
        let content = TestValueObjectFactory::pr_content().build();
        assert_eq!(content.title, "Test PR Title");
        assert!(content.description.starts_with("Test PR description"));
    }

    #[test]
    fn test_file_status_info_builder_default() {
        let info = TestValueObjectFactory::file_status_info().build();
        assert_eq!(info.path, "file.txt");
        assert_eq!(info.status_type, FileStatusType::NewUntracked);
    }

    #[test]
    fn test_file_status_info_builder_with_status() {
        let info = TestValueObjectFactory::file_status_info()
            .with_path("src/main.rs")
            .with_status_type(FileStatusType::ModifiedStaged)
            .build();
        assert_eq!(info.path, "src/main.rs");
        assert_eq!(info.status_type, FileStatusType::ModifiedStaged);
    }

    #[test]
    fn test_clean_working_tree() {
        let status = TestValueObjectFactory::clean_working_tree();
        assert!(status.is_clean());
    }
}
