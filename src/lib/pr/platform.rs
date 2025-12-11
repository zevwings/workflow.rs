use crate::branch::BranchType;
use crate::git::{GitRepo, RepoType};
// use crate::pr::codeup::Codeup;  // Codeup support has been removed
use crate::pr::github::GitHub;
use crate::pr::PullRequestRow;
use anyhow::Result;

/// PR 变更类型结构体
///
/// 包含变更类型的完整信息，包括名称、描述和示例
#[derive(Debug, Clone)]
pub struct ChangeType {
    /// 变更类型名称（用于显示和匹配）
    pub name: &'static str,
    /// 详细描述
    pub description: &'static str,
    /// 使用示例
    pub example: &'static str,
}

/// PR 变更类型定义（扩展版）
///
/// 每个类型包含名称、描述和示例信息
pub const CHANGE_TYPES: &[ChangeType] = &[
    ChangeType {
        name: "Bug fix (non-breaking change which fixes an issue)",
        description: "Fix errors or issues in code without changing existing functionality interfaces or behavior",
        example: "Fix null pointer exception in login validation logic",
    },
    ChangeType {
        name: "New feature (non-breaking change which adds functionality)",
        description: "Add new features or capabilities without affecting existing functionality",
        example: "Add user avatar upload functionality",
    },
    ChangeType {
        name: "Refactoring (non-breaking change which does not change functionality)",
        description: "Restructure code to improve quality without changing functional behavior",
        example: "Extract duplicate code into common functions and optimize code structure",
    },
    ChangeType {
        name: "Hotfix (urgent fix for production issues)",
        description: "Urgent fix for critical production issues that require immediate deployment",
        example: "Fix critical security vulnerability in authentication system",
    },
    ChangeType {
        name: "Chore (maintenance tasks and non-functional changes)",
        description: "Maintenance tasks, dependency updates, configuration changes, or other non-functional improvements",
        example: "Update dependencies, improve build configuration, or update documentation",
    },
];

/// PR 变更类型定义（向后兼容的字符串数组）
///
/// 用于生成 PR body 中的变更类型复选框和交互式选择
/// 这个常量保持向后兼容，内部使用 CHANGE_TYPES 数组
pub const TYPES_OF_CHANGES: &[&str] = &[
    CHANGE_TYPES[0].name,
    CHANGE_TYPES[1].name,
    CHANGE_TYPES[2].name,
    CHANGE_TYPES[3].name,
    CHANGE_TYPES[4].name,
];

/// 将分支类型映射到 PR 变更类型索引
///
/// # Arguments
/// * `branch_type` - 分支类型
///
/// # Returns
/// 返回对应的 PR 变更类型索引（在 TYPES_OF_CHANGES 中的位置）
/// 如果无法映射，返回 None
pub fn map_branch_type_to_change_type_index(
    branch_type: crate::branch::BranchType,
) -> Option<usize> {
    match branch_type {
        BranchType::Feature => Some(1),     // "New feature"
        BranchType::Bugfix => Some(0),      // "Bug fix"
        BranchType::Refactoring => Some(2), // "Refactoring"
        BranchType::Hotfix => Some(3),      // "Hotfix"
        BranchType::Chore => Some(4),       // "Chore"
    }
}

/// 将分支类型映射到 PR 变更类型布尔向量
///
/// # Arguments
/// * `branch_type` - 分支类型
///
/// # Returns
/// 返回布尔向量，表示每个 PR 变更类型是否被选中
pub fn map_branch_type_to_change_types(branch_type: crate::branch::BranchType) -> Vec<bool> {
    let mut result = vec![false; TYPES_OF_CHANGES.len()];

    if let Some(index) = map_branch_type_to_change_type_index(branch_type) {
        result[index] = true;
    }

    result
}

/// 根据索引获取变更类型信息
///
/// # Arguments
/// * `index` - 变更类型索引
///
/// # Returns
/// 返回对应的 ChangeType，如果索引无效则返回 None
pub fn get_change_type_by_index(index: usize) -> Option<&'static ChangeType> {
    CHANGE_TYPES.get(index)
}

/// 根据名称查找变更类型信息
///
/// # Arguments
/// * `name` - 变更类型名称
///
/// # Returns
/// 返回对应的 ChangeType，如果未找到则返回 None
pub fn get_change_type_by_name(name: &str) -> Option<&'static ChangeType> {
    CHANGE_TYPES.iter().find(|ct| ct.name == name)
}

/// 获取所有变更类型的完整信息
///
/// # Returns
/// 返回所有变更类型的切片
pub fn get_all_change_types() -> &'static [ChangeType] {
    CHANGE_TYPES
}

/// PR 状态信息
#[derive(Debug, Clone)]
pub struct PullRequestStatus {
    /// PR 状态（如 "open", "closed", "merged"）
    pub state: String,
    /// 是否已合并
    pub merged: bool,
    /// 合并时间（如果已合并）
    pub merged_at: Option<String>,
}

/// PR 平台接口 trait
/// 定义所有 PR 平台（GitHub、Codeup 等）必须实现的共同方法
pub trait PlatformProvider {
    /// 创建 Pull Request
    ///
    /// # Arguments
    /// * `title` - PR 标题
    /// * `body` - PR 描述
    /// * `source_branch` - 源分支名
    /// * `target_branch` - 目标分支名（可选，默认由各平台决定）
    ///
    /// # Returns
    /// PR URL 字符串
    fn create_pull_request(
        &self,
        title: &str,
        body: &str,
        source_branch: &str,
        target_branch: Option<&str>,
    ) -> Result<String>;

    /// 合并 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    /// * `delete_branch` - 是否删除源分支
    fn merge_pull_request(&self, pull_request_id: &str, delete_branch: bool) -> Result<()>;

    /// 获取 PR 信息
    ///
    /// # Arguments
    /// * `pull_request_id_or_branch` - PR ID 或分支名（某些平台支持通过分支名查找）
    ///
    /// # Returns
    /// PR 信息的格式化字符串
    fn get_pull_request_info(&self, pull_request_id_or_branch: &str) -> Result<String>;

    /// 获取 PR URL
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR URL 字符串
    fn get_pull_request_url(&self, pull_request_id: &str) -> Result<String>;

    /// 获取 PR 标题
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 标题字符串
    fn get_pull_request_title(&self, pull_request_id: &str) -> Result<String>;

    /// 获取 PR body 内容
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR body 字符串（如果存在），否则返回 None
    fn get_pull_request_body(&self, pull_request_id: &str) -> Result<Option<String>>;

    /// 获取当前分支的 PR ID
    ///
    /// # Returns
    /// PR ID（如果存在），否则返回 None
    fn get_current_branch_pull_request(&self) -> Result<Option<String>>;

    /// 列出 PR（可选方法，某些平台可能不支持）
    ///
    /// # Arguments
    /// * `state` - PR 状态筛选（如 "open", "closed"）
    /// * `limit` - 返回数量限制
    ///
    /// # Returns
    /// PR 列表的表格行数据
    fn get_pull_requests(
        &self,
        _state: Option<&str>,
        _limit: Option<u32>,
    ) -> Result<Vec<PullRequestRow>> {
        // 默认实现：返回不支持的错误
        anyhow::bail!("get_pull_requests is not supported by this platform")
    }

    /// 获取 PR 状态
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 状态信息，包含是否已合并等信息
    fn get_pull_request_status(&self, pull_request_id: &str) -> Result<PullRequestStatus>;

    /// 关闭 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    fn close_pull_request(&self, pull_request_id: &str) -> Result<()>;

    /// 获取 PR 的 diff 内容
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    ///
    /// # Returns
    /// PR 的 diff 内容（字符串格式）
    fn get_pull_request_diff(&self, _pull_request_id: &str) -> Result<String> {
        // 默认实现：返回不支持的错误
        anyhow::bail!("get_pull_request_diff is not supported by this platform")
    }

    /// 添加评论到 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    /// * `comment` - 评论内容
    fn add_comment(&self, pull_request_id: &str, comment: &str) -> Result<()>;

    /// 批准 Pull Request
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    fn approve_pull_request(&self, pull_request_id: &str) -> Result<()>;

    /// 更新 PR 的 base 分支
    ///
    /// # Arguments
    /// * `pull_request_id` - PR ID
    /// * `new_base` - 新的 base 分支名称
    ///
    /// # Errors
    /// 如果更新失败，返回相应的错误信息。
    fn update_pr_base(&self, pull_request_id: &str, new_base: &str) -> Result<()>;
}

/// 创建平台提供者实例
///
/// 根据当前仓库类型自动检测并创建对应的平台提供者。
///
/// # 返回
///
/// 返回 `Box<dyn PlatformProvider>` trait 对象，可以用于调用平台无关的 PR 操作。
///
/// # 错误
///
/// 如果仓库类型未知或不支持，返回错误。
///
/// # 示例
///
/// ```rust,no_run
/// use workflow::pr::platform::create_provider;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let provider = create_provider()?;
/// let pr_url = provider.create_pull_request(
///     "Title",
///     "Body",
///     "feature-branch",
///     None,
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    match GitRepo::detect_repo_type()? {
        RepoType::GitHub => Ok(Box::new(GitHub)),
        RepoType::Codeup => {
            anyhow::bail!("Codeup support has been removed. Only GitHub is currently supported.")
        }
        RepoType::Unknown => {
            anyhow::bail!("Unsupported repository type. Only GitHub is currently supported.")
        }
    }
}
