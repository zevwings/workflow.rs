//! 工作树状态相关实体

/// 文件状态信息
///
/// 包含文件的详细状态，支持 git2 提供的精确状态分类
#[derive(Debug, Clone)]
pub struct FileStatusInfo {
    /// 文件路径
    pub path: String,
    /// 文件状态类型
    pub status_type: FileStatusType,
    /// 原始路径（重命名时有值）
    pub old_path: Option<String>,
}

/// 文件状态类型
///
/// 区分暂存区和工作区的不同状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatusType {
    /// 新文件，未跟踪
    NewUntracked,
    /// 新文件，已暂存
    NewStaged,
    /// 已修改，未暂存
    ModifiedUnstaged,
    /// 已修改，已暂存
    ModifiedStaged,
    /// 已删除，未暂存
    DeletedUnstaged,
    /// 已删除，已暂存
    DeletedStaged,
    /// 已重命名
    Renamed,
    /// 类型变更
    TypeChanged,
    /// 冲突
    Conflicted,
}

impl FileStatusType {
    /// 是否已暂存
    pub fn is_staged(&self) -> bool {
        matches!(
            self,
            FileStatusType::NewStaged
                | FileStatusType::ModifiedStaged
                | FileStatusType::DeletedStaged
        )
    }

    /// 获取状态标签
    pub fn label(&self) -> &str {
        match self {
            FileStatusType::NewUntracked => "untracked",
            FileStatusType::NewStaged => "new file",
            FileStatusType::ModifiedUnstaged => "modified",
            FileStatusType::ModifiedStaged => "modified",
            FileStatusType::DeletedUnstaged => "deleted",
            FileStatusType::DeletedStaged => "deleted",
            FileStatusType::Renamed => "renamed",
            FileStatusType::TypeChanged => "typechange",
            FileStatusType::Conflicted => "conflicted",
        }
    }
}

/// 工作树状态
///
/// 包含暂存区、工作区和未跟踪文件的完整状态
#[derive(Debug, Clone)]
pub struct WorkingTreeStatus {
    /// 已暂存的文件
    pub staged: Vec<FileStatusInfo>,
    /// 未暂存的修改
    pub unstaged: Vec<FileStatusInfo>,
    /// 未跟踪的文件
    pub untracked: Vec<FileStatusInfo>,
    /// 冲突的文件
    pub conflicted: Vec<FileStatusInfo>,
}

impl WorkingTreeStatus {
    /// 是否工作区干净（无任何更改）
    pub fn is_clean(&self) -> bool {
        self.staged.is_empty()
            && self.unstaged.is_empty()
            && self.untracked.is_empty()
            && self.conflicted.is_empty()
    }

    /// 是否有暂存的更改
    pub fn has_staged(&self) -> bool {
        !self.staged.is_empty()
    }

    /// 是否有未暂存的更改
    pub fn has_unstaged(&self) -> bool {
        !self.unstaged.is_empty()
    }

    /// 是否有冲突
    pub fn has_conflicts(&self) -> bool {
        !self.conflicted.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_status_type_is_staged() {
        assert!(FileStatusType::NewStaged.is_staged());
        assert!(FileStatusType::ModifiedStaged.is_staged());
        assert!(FileStatusType::DeletedStaged.is_staged());
        assert!(!FileStatusType::NewUntracked.is_staged());
        assert!(!FileStatusType::ModifiedUnstaged.is_staged());
    }

    #[test]
    fn test_file_status_type_label() {
        assert_eq!(FileStatusType::NewUntracked.label(), "untracked");
        assert_eq!(FileStatusType::NewStaged.label(), "new file");
        assert_eq!(FileStatusType::ModifiedUnstaged.label(), "modified");
        assert_eq!(FileStatusType::ModifiedStaged.label(), "modified");
        assert_eq!(FileStatusType::DeletedUnstaged.label(), "deleted");
        assert_eq!(FileStatusType::DeletedStaged.label(), "deleted");
        assert_eq!(FileStatusType::Renamed.label(), "renamed");
        assert_eq!(FileStatusType::TypeChanged.label(), "typechange");
        assert_eq!(FileStatusType::Conflicted.label(), "conflicted");
    }

    #[test]
    fn test_working_tree_status_helpers() {
        let status = WorkingTreeStatus {
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            conflicted: vec![],
        };
        assert!(status.is_clean());
        assert!(!status.has_staged());
        assert!(!status.has_unstaged());
        assert!(!status.has_conflicts());

        let status = WorkingTreeStatus {
            staged: vec![FileStatusInfo {
                path: "file.txt".to_string(),
                status_type: FileStatusType::ModifiedStaged,
                old_path: None,
            }],
            unstaged: vec![],
            untracked: vec![],
            conflicted: vec![],
        };
        assert!(!status.is_clean());
        assert!(status.has_staged());
    }

    #[test]
    fn test_working_tree_status_with_unstaged_and_conflicted() {
        let status = WorkingTreeStatus {
            staged: vec![],
            unstaged: vec![FileStatusInfo {
                path: "main.rs".to_string(),
                status_type: FileStatusType::ModifiedUnstaged,
                old_path: None,
            }],
            untracked: vec![],
            conflicted: vec![FileStatusInfo {
                path: "conflict.txt".to_string(),
                status_type: FileStatusType::Conflicted,
                old_path: None,
            }],
        };

        assert!(!status.is_clean());
        assert!(!status.has_staged());
        assert!(status.has_unstaged());
        assert!(status.has_conflicts());
    }

    #[test]
    fn test_working_tree_status_with_untracked() {
        let status = WorkingTreeStatus {
            staged: vec![],
            unstaged: vec![],
            untracked: vec![FileStatusInfo {
                path: "new.txt".to_string(),
                status_type: FileStatusType::NewUntracked,
                old_path: None,
            }],
            conflicted: vec![],
        };

        assert!(!status.is_clean());
        assert!(!status.has_staged());
        assert!(!status.has_unstaged());
        assert!(!status.has_conflicts());
    }
}
