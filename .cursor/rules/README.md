# Cursor AI Rules

> This directory contains Cursor AI rules files for the Workflow CLI project. These rules guide AI assistants (like Cursor) in understanding project structure, development standards, and best practices.

---

## 📋 Overview

This directory contains rule files that are automatically loaded by Cursor IDE. Each rule file focuses on a specific aspect of the project.

**Important**: All rule files in this directory must be kept in sync with their Chinese versions in `docs/cursorrules/`.

## 📂 Rule Files

| File | Description | Type |
|------|-------------|------|
| `document.mdc` | Rules for document generation, classification, and storage | `always` |
| `overview.mdc` | Project overview, architecture, modification rules, document index/deletion rules, and general notes | `always` |
| `development.mdc` | Development standards and guidelines | `always` |
| `sync.mdc` | Rules for keeping English and Chinese versions synchronized | `always` |

## 🔄 Synchronization

**Critical**: Each file in `.cursor/rules/` has a corresponding file in `docs/cursorrules/` with the same name. These files must always be kept in sync.

### Synchronization Rules

- When modifying any file in `.cursor/rules/`, immediately update the corresponding file in `docs/cursorrules/`
- When modifying any file in `docs/cursorrules/`, immediately update the corresponding file in `.cursor/rules/`
- Keep chapter structure, content, and timestamps consistent between versions

For detailed synchronization rules, see `sync.mdc`.

## 📖 Usage

Cursor IDE automatically loads all `.mdc` files in this directory. These files use the MDC (Markdown Component) format with YAML front matter metadata to define rule types and behavior.

**Rule Types**:
- `always`: Always included in model context (default for core rules)
- `auto-attach`: Automatically attached when files matching glob patterns are referenced
- `agent-request`: Available for AI to include when needed (requires description)
- `manual`: Only included when explicitly referenced with `@ruleName`

**Note**: This project uses `.mdc` format (recommended by Cursor) instead of the legacy `.cursorrules` file format.

## 📚 Related Documents

- [Document Templates](../../docs/templates/cursorrules/README.md) - Templates for creating new rule files
- [Document Writing Guidelines](../../docs/guidelines/document.md) - General document writing standards
- [Development Guidelines](../../docs/guidelines/development/README.md) - Complete development guidelines

---

**Last Updated**: 2025-12-25

