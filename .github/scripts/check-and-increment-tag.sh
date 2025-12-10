#!/bin/bash
set -e

TAG="$1"
VERSION="$2"

if [ -z "$TAG" ] || [ -z "$VERSION" ]; then
  echo "❌ Error: Missing required arguments"
  echo "Usage: $0 <tag> <version>"
  exit 1
fi

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "⚠️  Tag $TAG already exists"
  echo "Current commit: $(git rev-parse HEAD)"
  echo "Tag commit: $(git rev-parse $TAG)"

  if [ "$(git rev-parse HEAD)" = "$(git rev-parse $TAG)" ]; then
    echo "exists=true" >> $GITHUB_OUTPUT
    echo "needs_increment=false" >> $GITHUB_OUTPUT
    echo "version=$VERSION" >> $GITHUB_OUTPUT
    echo "tag=$TAG" >> $GITHUB_OUTPUT
    echo "✅ Tag points to current commit, skipping tag creation"
  else
    echo "exists=true" >> $GITHUB_OUTPUT
    echo "needs_increment=true" >> $GITHUB_OUTPUT
    echo "⚠️  Tag points to different commit, auto-incrementing patch version"

    # 自动递增 patch 版本号 (0.0.1 -> 0.0.2)
    IFS='.' read -ra VERSION_PARTS <<< "$VERSION"
    MAJOR="${VERSION_PARTS[0]:-0}"
    MINOR="${VERSION_PARTS[1]:-0}"
    PATCH="${VERSION_PARTS[2]:-0}"

    # 递增 patch 版本
    PATCH=$((PATCH + 1))
    if [ $PATCH -gt 9 ]; then
      PATCH=0
      MINOR=$((MINOR + 1))
    fi

    if [ $MINOR -gt 9 ]; then
      MINOR=0
      MAJOR=$((MAJOR + 1))
    fi

    NEW_VERSION="$MAJOR.$MINOR.$PATCH"
    NEW_TAG="v$NEW_VERSION"

    echo "version=$NEW_VERSION" >> $GITHUB_OUTPUT
    echo "tag=$NEW_TAG" >> $GITHUB_OUTPUT
    echo "old_version=$VERSION" >> $GITHUB_OUTPUT
    echo "old_tag=$TAG" >> $GITHUB_OUTPUT

    echo "✅ Auto-incremented version: $VERSION -> $NEW_VERSION"
    echo "✅ New tag: $NEW_TAG"
  fi
else
  echo "exists=false" >> $GITHUB_OUTPUT
  echo "needs_increment=false" >> $GITHUB_OUTPUT
  echo "version=$VERSION" >> $GITHUB_OUTPUT
  echo "tag=$TAG" >> $GITHUB_OUTPUT
  echo "✅ Tag $TAG does not exist, will create it"
fi
