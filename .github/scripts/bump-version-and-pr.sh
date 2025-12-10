#!/bin/bash
set -e

NEW_VERSION="$1"
GITHUB_TOKEN="$2"
REPOSITORY="$3"
RUN_ID="$4"

if [ -z "$NEW_VERSION" ] || [ -z "$GITHUB_TOKEN" ] || [ -z "$REPOSITORY" ] || [ -z "$RUN_ID" ]; then
  echo "❌ Error: Missing required arguments"
  echo "Usage: $0 <new_version> <github_token> <repository> <run_id>"
  exit 1
fi

# 配置 Git
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"

BRANCH_NAME="bump-version-${NEW_VERSION}"

# 创建新分支
echo "Step 1: Creating branch $BRANCH_NAME"
git checkout -b "$BRANCH_NAME"

# 更新 Cargo.toml
echo "Step 2: Updating Cargo.toml"
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
else
  sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# 更新 Cargo.lock
echo "Step 3: Updating Cargo.lock"
if grep -q 'name = "workflow"' Cargo.lock; then
  if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' '/^name = "workflow"$/,/^version = /{
      /^version = /s/version = "[^"]*"/version = "'"$NEW_VERSION"'"/
    }' Cargo.lock
  else
    sed -i '/^name = "workflow"$/,/^version = /{
      /^version = /s/version = "[^"]*"/version = "'"$NEW_VERSION"'"/
    }' Cargo.lock
  fi
fi

# 提交更改
echo "Step 4: Committing changes"
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to ${NEW_VERSION} [skip ci]"

# 推送分支
echo "Step 5: Pushing branch to origin"
git push origin "$BRANCH_NAME"

# 创建 Pull Request
echo "Step 6: Creating Pull Request"
PR_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer $GITHUB_TOKEN" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "https://api.github.com/repos/$REPOSITORY/pulls" \
  -d '{
    "title": "chore: bump version to '"$NEW_VERSION"'",
    "head": "'"$BRANCH_NAME"'",
    "base": "master",
    "body": "Automated version bump to '"$NEW_VERSION"'\n\nThis PR was created automatically by the release workflow.\n\n**Changes:**\n- Updated version in Cargo.toml to '"$NEW_VERSION"'\n- Updated version in Cargo.lock to '"$NEW_VERSION"'\n\n**Workflow Run:** https://github.com/'"$REPOSITORY"'/actions/runs/'"$RUN_ID"'"
  }')

HTTP_CODE=$(echo "$PR_RESPONSE" | tail -n1)
PR_BODY=$(echo "$PR_RESPONSE" | head -n-1)

if [ "$HTTP_CODE" != "201" ]; then
  if [ "$HTTP_CODE" = "422" ]; then
    echo "Checking for existing PR..."
    EXISTING_PR=$(curl -s \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "https://api.github.com/repos/$REPOSITORY/pulls?head=${REPOSITORY%%/*}:$BRANCH_NAME&base=master&state=open")

    PR_NUMBER=$(echo "$EXISTING_PR" | jq -r '.[0].number // empty' 2>/dev/null)
    if [ -n "$PR_NUMBER" ] && [ "$PR_NUMBER" != "null" ]; then
      PR_URL=$(echo "$EXISTING_PR" | jq -r '.[0].html_url // empty' 2>/dev/null)
      echo "✅ Found existing PR #$PR_NUMBER: $PR_URL"
    else
      exit 1
    fi
  else
    exit 1
  fi
else
  PR_NUMBER=$(echo "$PR_BODY" | jq -r '.number // empty' 2>/dev/null || echo "$PR_BODY" | grep -o '"number":[0-9]*' | head -1 | cut -d':' -f2)
  PR_URL=$(echo "$PR_BODY" | jq -r '.html_url // empty' 2>/dev/null || echo "$PR_BODY" | grep -o '"html_url":"[^"]*"' | head -1 | cut -d'"' -f4)
  echo "✅ PR #$PR_NUMBER created: $PR_URL"
fi

# 等待并合并 PR
echo "Step 7: Merging Pull Request"
MAX_WAIT=120
CHECK_INTERVAL=5
ELAPSED=0

while [ $ELAPSED -lt $MAX_WAIT ]; do
  PR_STATUS=$(curl -s \
    -H "Accept: application/vnd.github+json" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "https://api.github.com/repos/$REPOSITORY/pulls/$PR_NUMBER")

  MERGED=$(echo "$PR_STATUS" | grep -o '"merged":[^,}]*' | cut -d':' -f2 | tr -d ' "')
  MERGEABLE=$(echo "$PR_STATUS" | grep -o '"mergeable":[^,}]*' | cut -d':' -f2 | tr -d ' "')

  if [ "$MERGED" = "true" ]; then
    echo "✅ PR already merged"
    git checkout master 2>/dev/null || git checkout main 2>/dev/null || true
    git push origin --delete "$BRANCH_NAME" 2>/dev/null || true
    exit 0
  fi

  if [ "$MERGEABLE" = "true" ]; then
    MERGE_RESPONSE=$(curl -s -w "\n%{http_code}" -X PUT \
      -H "Accept: application/vnd.github+json" \
      -H "Authorization: Bearer $GITHUB_TOKEN" \
      -H "X-GitHub-Api-Version: 2022-11-28" \
      "https://api.github.com/repos/$REPOSITORY/pulls/$PR_NUMBER/merge" \
      -d '{
        "commit_title": "chore: bump version to '"$NEW_VERSION"' [skip ci]",
        "commit_message": "Automated version bump to '"$NEW_VERSION"'\n\nMerged via GitHub Actions workflow [skip ci]",
        "merge_method": "squash"
      }')

    MERGE_CODE=$(echo "$MERGE_RESPONSE" | tail -n1)
    if [ "$MERGE_CODE" = "200" ]; then
      echo "✅ PR merged successfully"
      git checkout master 2>/dev/null || git checkout main 2>/dev/null || true
      git push origin --delete "$BRANCH_NAME" 2>/dev/null || true
      exit 0
    fi
  fi

  sleep $CHECK_INTERVAL
  ELAPSED=$((ELAPSED + CHECK_INTERVAL))
done

echo "⏰ Timeout: PR did not become mergeable within ${MAX_WAIT}s"
exit 1
