use crate::git::{GitRepo, RepoType};
use crate::pr::codeup::Codeup;
use crate::pr::github::GitHub;
use crate::pr::provider::PlatformProvider;
use anyhow::Result;

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
/// use crate::pr::factory::create_provider;
///
/// let provider = create_provider()?;
/// let pr_url = provider.create_pull_request(
///     "Title",
///     "Body",
///     "feature-branch",
///     None,
/// )?;
/// ```
pub fn create_provider() -> Result<Box<dyn PlatformProvider>> {
    match GitRepo::detect_repo_type()? {
        RepoType::GitHub => Ok(Box::new(GitHub)),
        RepoType::Codeup => Ok(Box::new(Codeup)),
        RepoType::Unknown => {
            anyhow::bail!("Unsupported repository type. Only GitHub and Codeup are supported.")
        }
    }
}
