# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Workflow CLI (`workflow`) — a Rust CLI tool that automates development workflows (PR creation, Jira integration, Git branch management, LLM-powered commit summaries, etc.). The project documentation and commit messages are primarily in Chinese.

## Build & Development Commands

```bash
# Build
cargo build                    # debug build
cargo build --release          # release build
make dev                       # debug build (via Makefile)
make release                   # release build (via Makefile)

# Lint (run all three before committing)
cargo fmt --check              # check formatting
cargo clippy --all-targets --all-features -- -D warnings  # lint
cargo check --all-features     # type check
make lint                      # runs all three in sequence

# Auto-fix
cargo fmt                      # format code
make fix                       # format + clippy fix + cargo fix

# Tests
cargo test                     # all tests
cargo test -p <crate>          # test a single crate (e.g. -p di, -p services)
cargo test <test_name>         # run a single test by name
cargo test -- --include-ignored  # include ignored tests
make test                      # shorthand for cargo test

# Install to system
make install                   # builds release (--no-default-features) and installs
```

## Workspace Architecture

Cargo workspace with 9 crates. Dependency direction flows downward:

```
app  (CLI entry, commands, bootstrap, interactive)
 ├── domain     (entities, config models, repository/service traits — no implementations)
 ├── storage    (repository implementations: git, github, jira, config)
 ├── services   (application services: PR, branch, commit, summary, completion, alias)
 ├── client     (client traits & types: Http, LLM, GitHub, Jira, LanguageManager)
 ├── infra      (infrastructure implementations: HTTP client, LLM client, retry, bootstrap)
 ├── toolkit    (utilities: logging, paths, templates, shell, rollback)
 ├── prompt     (terminal UI: dialogs, forms, progress bars, styled output)
 └── di         (dependency injection container)
```

**Key rule**: `domain` defines traits only; `storage`/`infra` provide implementations. `app` depends on everything; lower crates never depend on `app`.

## Dependency Injection System

The `di` crate provides a global DI container using `DashMap` with `Singleton`/`Transient` scopes.

**Registration** happens in `crates/app/src/bootstrap/mod.rs` via `LazyLock`:
1. `context::register_context()` — config contexts (LLM, Jira, GitHub)
2. `infra::register_client()` — client layer (LLM client, HTTP)
3. `storage::register_storage()` — repository implementations
4. `services::register_services()` — application services
5. `app::register_app()` — app-layer services

**Usage**: Call `get_service::<dyn SomeTrait>()` from `bootstrap` module, or use convenience functions like `get_git_repository()`, `get_pull_request_service()`, etc.

**Binding macros** in the `di` crate:
- `bind!(dyn Trait, |c| { ... })` — bind trait object with factory
- `bind_instance!(|c| { ... })` — bind concrete type
- `registry!(|c| { ... })` — register multiple bindings

## Interactive Workflow System

The `interactive` module in `app` implements a stage-based workflow for `setup`/`check` commands:

- **`WorkflowStage`** trait — each platform (Jira, GitHub, LLM, Log) implements this
- **`WorkflowStageRegistry`** — returns stages in fixed order: Jira → GitHub → LLM → Log
- **`WorkflowExecutor`** — runs `setup` or `check` on a stage with a `WorkflowContext`
- **`WorkflowContext`** — carries config state and `WorkflowMode` (Setup/Check)
- Platform-specific implementations live in `crates/app/src/interactive/platforms/`

## Command Pattern

Commands live in `crates/app/src/commands/<name>/`. Each command module typically has:
- `mod.rs` — module re-exports
- `cli.rs` — clap argument definitions
- `command.rs` or action files — implementation using services from `bootstrap::get_*`

Commands get services via `bootstrap::get_service::<dyn Trait>()`, never by constructing implementations directly.

## Feature Flags

The `app` crate has a `develop` feature (default-on) for dev-only commands. Release builds use `--no-default-features` to exclude them and reduce binary size.

## Error Handling

- Use `anyhow::Result<T>` with `.context()` for adding error context
- `thiserror` for defining error types in library crates
- The `prompt` crate provides `is_user_cancelled()` to detect Ctrl+C in interactive flows

## Conventions

- **Naming**: `snake_case` for modules/functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants
- **Import order**: std → third-party → internal crates
- **Commits**: Conventional Commits format (`feat(scope): subject`, `fix(scope): subject`, etc.)
- **Branches**: `feature/*`, `fix/*`, `hotfix/*` from `master`
- **Tests**: Unit tests in `#[cfg(test)]` within source files; integration tests in `tests/`; AAA pattern (Arrange-Act-Assert); tests that touch global DI state use `#[serial]`
- **Doc comments**: Public APIs require `///` with params, returns, errors
