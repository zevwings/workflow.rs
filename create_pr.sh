#!/bin/bash

# 完整的 curl 命令用于创建 Pull Request
# 版本：1.5.2
# 分支：bump-version-1.5.2

curl -s -w "\n%{http_code}" -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer YOUR_GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/repos/zevwings/workflow.rs/pulls" \
  -d '{
    "title": "chore: bump version to 1.5.2",
    "head": "bump-version-1.5.2",
    "base": "master",
    "body": "Automated version bump to 1.5.2\n\nThis PR was created automatically by the release workflow.\n\n**Changes:**\n- Updated version in Cargo.toml to 1.5.2\n- Updated version in Cargo.lock to 1.5.2\n\n**Workflow Run:** https://github.com/zevwings/workflow.rs/actions/runs/20070951088"
  }'
