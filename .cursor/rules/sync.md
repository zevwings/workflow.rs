# File Synchronization Rules

> **⚠️ Synchronization Note**: This file must be kept in sync with its Chinese version at `docs/cursorrules/sync.md`. When modifying this file, you must immediately update the corresponding Chinese version.

---

## File Synchronization Rules

**Important**: All rule files in `.cursor/rules/` and `docs/cursorrules/` must always be kept in sync. Each English file has a corresponding Chinese file with the same name.

### Synchronization Trigger

**Rule**: Whenever any file in `.cursor/rules/` or `docs/cursorrules/` is modified, the corresponding file in the other directory must be immediately updated to maintain consistency.

### Synchronization Direction

- **Modifying `.cursor/rules/{filename}.md` (English version)** → Must immediately synchronize and update `docs/cursorrules/{filename}.md` (Chinese version)
- **Modifying `docs/cursorrules/{filename}.md` (Chinese version)** → Must immediately synchronize and update `.cursor/rules/{filename}.md` (English version)

### Verification Methods

- **Manual Checklist**:
  1. Check that the chapter structure is consistent (heading levels, chapter order)
  2. Check that key rules exist in both files (e.g., code generation rules, document management rules)
  3. Check that the "Last Updated" timestamps are consistent
  4. Check that the directory structure is consistent
- **Automated Validation Script** (recommended):
  - Check that chapter headings are consistent (ignoring language differences)
  - Check that line counts are similar (considering Chinese/English length differences)
  - Check that key markers (e.g., `##`, `###`) have consistent counts
  - Check that "Last Updated" timestamps are consistent
- **Git Hooks Validation** (recommended):
  - In pre-commit hook: If any file in `.cursor/rules/` or `docs/cursorrules/` is modified, check if the corresponding file is also modified in the same commit
  - If only one file is modified, prompt to synchronize and update the other file
- **CI/CD Validation** (recommended):
  - Check synchronization status of corresponding files in CI pipeline
  - If out of sync is detected, fail CI build and prompt to synchronize

---

**Last Updated**: 2025-12-25

