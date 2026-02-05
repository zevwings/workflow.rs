//! Git 服务注册

use std::sync::Arc;

use domain::{GitRepoRepository, GitRepository};
use registry::{bind, Container, Scope};

use crate::git::services::{
    BlameService, BlameServiceImpl, BranchService, BranchServiceImpl, CommitService,
    CommitServiceImpl, DiffService, DiffServiceImpl, DiscoveredContext, GitContext,
    GitContextHolder, HookService, HookServiceImpl, MergeService, MergeServiceImpl, RemoteService,
    RemoteServiceImpl, StashService, StashServiceImpl, TagService, TagServiceImpl,
};
use crate::git::{GitRepositoryImpl, GitRepositoryServices};

/// 将 GitRepository 包装为 GitRepoRepository，供仅需 get_repo_info 的依赖使用
struct GitRepoRepositoryWrapper(Arc<dyn GitRepository>);

impl GitRepoRepository for GitRepoRepositoryWrapper {
    fn get_repo_info(&self) -> domain::RepoInfo {
        self.0.get_repo_info()
    }
}

/// 注册 Git 相关服务
pub fn register_git() -> registry::Result<()> {
    bind!(dyn GitContextHolder, |_: &Container| {
        let ctx = GitContext::discover().expect("must run in a git repo");
        Arc::new(DiscoveredContext(ctx))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn BlameService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before BlameService");
        Arc::new(BlameServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn BranchService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before BranchService");
        Arc::new(BranchServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn HookService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before HookService");
        Arc::new(HookServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn CommitService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before CommitService");
        let hook_service = c
            .get::<dyn HookService>()
            .expect("HookService must be registered before CommitService");
        Arc::new(CommitServiceImpl::new(holder.context(), hook_service))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn DiffService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before DiffService");
        Arc::new(DiffServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn MergeService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before MergeService");
        Arc::new(MergeServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn RemoteService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before RemoteService");
        let hook_service = c
            .get::<dyn HookService>()
            .expect("HookService must be registered before RemoteService");
        Arc::new(RemoteServiceImpl::new(holder.context(), hook_service))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn TagService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before TagService");
        Arc::new(TagServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn StashService, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before StashService");
        Arc::new(StashServiceImpl::new(holder.context()))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn GitRepository, |c: &Container| {
        let holder = c
            .get::<dyn GitContextHolder>()
            .expect("GitContextHolder must be registered before GitRepository");
        let ctx = holder.context();
        let blame = c
            .get::<dyn BlameService>()
            .expect("BlameService must be registered before GitRepository");
        let branch = c
            .get::<dyn BranchService>()
            .expect("BranchService must be registered before GitRepository");
        let commit = c
            .get::<dyn CommitService>()
            .expect("CommitService must be registered before GitRepository");
        let diff = c
            .get::<dyn DiffService>()
            .expect("DiffService must be registered before GitRepository");
        let merge = c
            .get::<dyn MergeService>()
            .expect("MergeService must be registered before GitRepository");
        let remote = c
            .get::<dyn RemoteService>()
            .expect("RemoteService must be registered before GitRepository");
        let tag = c
            .get::<dyn TagService>()
            .expect("TagService must be registered before GitRepository");
        let stash = c
            .get::<dyn StashService>()
            .expect("StashService must be registered before GitRepository");
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
        Arc::new(GitRepositoryImpl::new(ctx, services))
    })
    .in_scope(Scope::Singleton)?;

    bind!(dyn GitRepoRepository, |c: &Container| {
        let repo = c
            .get::<dyn GitRepository>()
            .expect("GitRepository must be registered before GitRepoRepository");
        Arc::new(GitRepoRepositoryWrapper(repo))
    })
    .in_scope(Scope::Singleton)?;

    Ok(())
}
