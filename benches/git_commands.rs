//! Git 命令包装层性能基准测试
//!
//! 测试 Git 命令包装层的性能，包括命令执行、分支操作、提交操作等。

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use workflow::git::commands::{
    GitBranchCommand, GitCommand, GitCommitCommand, GitConfigCommand, GitRepoCommand,
};

fn bench_git_command_run(c: &mut Criterion) {
    // 需要在 Git 仓库中运行，使用临时仓库
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // 初始化 Git 仓库
    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    c.bench_function("git_command_run_rev_parse", |b| {
        b.iter(|| {
            black_box(
                GitCommand::run(&["rev-parse", "--git-dir"], Some(repo_path)).unwrap_or_default(),
            );
        });
    });

    c.bench_function("git_command_run_status", |b| {
        b.iter(|| {
            black_box(
                GitCommand::run(&["status", "--porcelain"], Some(repo_path)).unwrap_or_default(),
            );
        });
    });
}

fn bench_git_command_check(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    c.bench_function("git_command_check_branch_exists", |b| {
        b.iter(|| {
            black_box(GitCommand::check(
                &["show-ref", "--verify", "--quiet", "refs/heads/main"],
                Some(repo_path),
            ));
        });
    });
}

fn bench_git_branch_command(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // 配置 Git 用户（提交需要）
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    c.bench_function("git_branch_current_branch", |b| {
        b.iter(|| {
            black_box(GitBranchCommand::current_branch(Some(repo_path)).unwrap_or_default());
        });
    });

    c.bench_function("git_branch_exists_local", |b| {
        b.iter(|| {
            black_box(
                GitBranchCommand::branch_exists_local("main", Some(repo_path)).unwrap_or(false),
            );
        });
    });

    c.bench_function("git_branch_list_branches", |b| {
        b.iter(|| {
            black_box(GitBranchCommand::list_branches(Some(repo_path)).unwrap_or_default());
        });
    });
}

fn bench_git_commit_command(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // 创建初始提交
    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    c.bench_function("git_commit_status", |b| {
        b.iter(|| {
            black_box(GitCommitCommand::status(Some(repo_path)).unwrap_or_default());
        });
    });

    c.bench_function("git_commit_has_changes", |b| {
        b.iter(|| {
            black_box(GitCommitCommand::has_changes(Some(repo_path)).unwrap_or(false));
        });
    });

    c.bench_function("git_commit_get_head_sha", |b| {
        b.iter(|| {
            black_box(GitCommitCommand::get_head_sha(Some(repo_path)).unwrap_or_default());
        });
    });
}

fn bench_git_config_command(c: &mut Criterion) {
    c.bench_function("git_config_get_user_email", |b| {
        b.iter(|| {
            black_box(
                GitConfigCommand::get_user_email(true, None).unwrap_or(None).unwrap_or_default(),
            );
        });
    });

    c.bench_function("git_config_get_user_name", |b| {
        b.iter(|| {
            black_box(
                GitConfigCommand::get_user_name(true, None).unwrap_or(None).unwrap_or_default(),
            );
        });
    });
}

fn bench_git_repo_command(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    std::process::Command::new("git")
        .args(["init", "--initial-branch=main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    c.bench_function("git_repo_is_git_repo", |b| {
        b.iter(|| {
            black_box(GitRepoCommand::is_git_repo(Some(repo_path)));
        });
    });

    c.bench_function("git_repo_get_git_dir", |b| {
        b.iter(|| {
            black_box(GitRepoCommand::get_git_dir(Some(repo_path)).unwrap_or_default());
        });
    });
}

criterion_group!(
    benches,
    bench_git_command_run,
    bench_git_command_check,
    bench_git_branch_command,
    bench_git_commit_command,
    bench_git_config_command,
    bench_git_repo_command
);
criterion_main!(benches);
