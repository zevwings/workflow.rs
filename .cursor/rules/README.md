# Cursor AI Rules

> This directory contains Cursor AI rules files for the Workflow CLI project. These rules guide AI assistants (like Cursor) in understanding project structure, development standards, and best practices.

---

## 📋 Overview

This directory contains rule files that are automatically loaded by Cursor IDE. Each rule file focuses on a specific aspect of the project.

**Important**: All rule files in this directory must be kept in sync with their Chinese versions in `docs/cursorrules/`.

## 📂 Rule Files

| File | Description |
|------|-------------|
| `document.md` | Rules for document generation, classification, and storage |
| `overview.md` | Project overview, architecture, modification rules, document index/deletion rules, and general notes |
| `development.md` | Development standards and guidelines |
| `sync.md` | Rules for keeping English and Chinese versions synchronized |

## 🔄 Synchronization

**Critical**: Each file in `.cursor/rules/` has a corresponding file in `docs/cursorrules/` with the same name. These files must always be kept in sync.

### Synchronization Rules

- When modifying any file in `.cursor/rules/`, immediately update the corresponding file in `docs/cursorrules/`
- When modifying any file in `docs/cursorrules/`, immediately update the corresponding file in `.cursor/rules/`
- Keep chapter structure, content, and timestamps consistent between versions

For detailed synchronization rules, see `sync.md`.

## 📖 Usage

Cursor IDE automatically loads all `.md` files in this directory. No additional configuration is needed.

## 📚 Related Documents

- [Document Templates](../../docs/templates/cursorrules/README.md) - Templates for creating new rule files
- [Document Writing Guidelines](../../docs/guidelines/document.md) - General document writing standards
- [Development Guidelines](../../docs/guidelines/development/README.md) - Complete development guidelines

---

**Last Updated**: 2025-12-25

