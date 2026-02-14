//! Git 服务注册

use std::sync::Arc;

use di::{bind, Container, InjectionError, Scope};
use domain::{GitRepoRepository, GitRepository, RepoInfo};

use crate::git::{
    services::{
        BlameService, BlameServiceImpl, BranchService, BranchServiceImpl, CommitService,
        CommitServiceImpl, DiffService, DiffServiceImpl, DiscoveredContext, GitContext,
        GitContextHolder, HookService, HookServiceImpl, MergeService, MergeServiceImpl,
        RemoteService, RemoteServiceImpl, StashService, StashServiceImpl, TagService,
        TagServiceImpl,
    },
    GitRepositoryImpl, GitRepositoryServices,
};

/// 将 GitRepository 包装为 GitRepoRepository，供仅需 get_repo_info 的依赖使用
struct GitRepoRepositoryWrapper(Arc<dyn GitRepository>);

impl GitRepoRepository for GitRepoRepositoryWrapper {
    fn get_repo_info(&self) -> RepoInfo {
        self.0.get_repo_info()
    }
}

/// 注册 Git 相关服务
///
/// # 注册顺序和依赖关系
///
/// 服务注册顺序很重要，必须先注册基础服务，再注册依赖它们的服务：
/// 1. **GitContextHolder** (无依赖) - 必须最先注册
/// 2. **HookService** (依赖 GitContextHolder)
/// 3. **其他服务** (依赖 GitContextHolder, 部分依赖 HookService)
///
/// Factory 闭包中的 `.expect()` 表示程序员错误（注册顺序错误），
/// 而非运行时错误。如果触发 panic，说明注册顺序不正确。
pub(super) fn register_git() -> Result<(), InjectionError> {
    // 第一步：注册 GitContextHolder (基础服务，无依赖)
    // 注意：GitContext::discover() 要求程序在 Git 仓库中运行
    bind!(dyn GitContextHolder, |_: &Container| {
        let ctx = GitContext::discover().map_err(|_| {
            di::InjectionError::ValidationError(
                "Current directory is not a Git repository.\n\n\
                Please run this command in the project directory managed by the workflow, e.g.:\n  cd /path/to/your/project\n  workflow check"
                    .to_string(),
            )
        })?;
        Ok(Arc::new(DiscoveredContext(ctx)))
    })
    .in_scope(Scope::Singleton)?;

    // 第二步：注册依赖 GitContextHolder 的服务
    // 注意：以下 .expect() 表示依赖未注册（程序员错误），而非运行时错误
    bind!(dyn BlameService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(BlameServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn BranchService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(BranchServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn HookService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(HookServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn CommitService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        let hook_service = c.get::<dyn HookService>()?;
        Ok(Arc::new(CommitServiceImpl::new(
            holder.context(),
            hook_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn DiffService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(DiffServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn MergeService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(MergeServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn RemoteService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        let hook_service = c.get::<dyn HookService>()?;
        Ok(Arc::new(RemoteServiceImpl::new(
            holder.context(),
            hook_service,
        )))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn TagService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(TagServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn StashService, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        Ok(Arc::new(StashServiceImpl::new(holder.context())))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn GitRepository, |c: &Container| {
        let holder = c.get::<dyn GitContextHolder>()?;
        let ctx = holder.context();
        let blame = c.get::<dyn BlameService>()?;
        let branch = c.get::<dyn BranchService>()?;
        let commit = c.get::<dyn CommitService>()?;
        let diff = c.get::<dyn DiffService>()?;
        let merge = c.get::<dyn MergeService>()?;
        let remote = c.get::<dyn RemoteService>()?;
        let tag = c.get::<dyn TagService>()?;
        let stash = c.get::<dyn StashService>()?;
        let services = GitRepositoryServices {
            blame,
            branch,
            commit,
            diff,
            merge,
            remote,
            tag,
            stash,
        };
        Ok(Arc::new(GitRepositoryImpl::new(ctx, services)))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn GitRepoRepository, |c: &Container| {
        let repo = c.get::<dyn GitRepository>()?;
        Ok(Arc::new(GitRepoRepositoryWrapper(repo)))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
