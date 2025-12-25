# Overview

> **⚠️ Synchronization Note**: This file must be kept in sync with its Chinese version at `docs/cursorrules/overview.md`. When modifying this file, you must immediately update the corresponding Chinese version.

---

## Project Overview

This is a CLI tool written in Rust for automating development workflows, providing PR management, Jira integration, log processing, and other features.

### Architecture

The project adopts a three-layer architecture:
- **CLI Entry Layer** (`bin/`, `main.rs`): Command-line argument parsing and command dispatching
- **Command Wrapper Layer** (`commands/`): CLI command wrappers, handling user interactions
- **Core Business Logic Layer** (`lib/`): All business logic implementation

For detailed module architecture information, refer to `docs/architecture/architecture.md`.

## Code and Document Modification Rules

### Code Modification Rules

**Prohibited from modifying existing code files without explicit user consent or instruction.** If code modification is needed, user consent or explicit instruction must be obtained first.

### Document Modification Rules

**Prohibited from modifying existing document files without explicit user consent or instruction.** If document modification is needed, user consent or explicit instruction must be obtained first.

## Notes

1. **Cross-platform Support**: Project supports macOS, Linux, Windows, pay attention to platform-specific code. For platform-specific code organization and testing requirements, reference `docs/guidelines/development/module-organization.md` (Platform-specific Code Organization section)
2. **Clipboard Functionality**: Linux ARM64 and musl static linking versions do not support clipboard functionality
3. **Configuration Files**: Configuration files stored in `~/.workflow/config/workflow.toml` (macOS/Linux) or `%APPDATA%\workflow\config\workflow.toml` (Windows)
4. **Error Handling**: All errors should provide clear error messages and context
5. **Logging**: Use `tracing` for logging, supports different log levels
6. **GitHub Configuration**: When setting up the project for the first time, need to configure GitHub Secrets, Variables, and branch protection rules, reference `docs/guidelines/github-setup.md`

## Document Index Rules

- **Prohibited Indexing**: Documents in `analysis/` and `report/` directories **must not** be indexed in `docs/README.md` or project root `README.md`
- These directories contain **temporary analysis documents**, not reference documents, and do not need to be displayed in document index
- **Requirement Document Index Restriction**: Requirement documents in `docs/requirements/` directory **only** need to be indexed in `docs/requirements/README.md`, **not** in `docs/README.md` or project root `README.md`
- **Reference Documents**: Other documents in `docs/` directory (architecture documents, guidelines documents, migration documents) are reference documents and should be indexed in `docs/README.md`
- **Document Index Update**: When creating new documents, update corresponding document index (as applicable):
  - TODO Documents: Only index in `docs/requirements/README.md`
  - Other Reference Documents: Index in `docs/README.md`

## Document Deletion Rules

- **Temporary Documents (can be deleted directly)**:
  - All documents in `analysis/` directory are temporary technical analysis and can be deleted at any time
  - All documents in `report/` directory are temporary analysis reports and can be deleted at any time
  - These documents are used for analysis and recording during development and do not need long-term retention
- **Reference Documents (deletion requires caution)**:
  - Documents in `docs/` directory are **reference documents and architecture documents**, deletion requires great care
  - Including: architecture documents (`docs/architecture/`), guidelines documents (`docs/guidelines/`), migration documents (`docs/migration/`), TODO documents (`docs/requirements/`)
  - These documents are the project's knowledge base and reference materials, must confirm no longer needed before deletion
  - **Special Note**: TODO documents, although reference documents, are only indexed in `docs/requirements/README.md`, not in the main document index

---

**Last Updated**: 2025-12-25

