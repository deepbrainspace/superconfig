#!/bin/bash

set -e

PROJECT_NAME="$1"
OVERRIDE_VERSION="$2"
NO_CONFIRM="$3"

if [ -z "$PROJECT_NAME" ]; then
    echo "❌ Usage: $0 <project-name> [version-override] [--yes]"
    echo "   Example: $0 superffi"
    echo "   Example: $0 superffi 0.2.0"
    echo "   Example: $0 superffi 0.2.0 --yes"
    exit 1
fi

# Get version from Cargo.toml or use override
if [ -n "$OVERRIDE_VERSION" ]; then
    VERSION="$OVERRIDE_VERSION"
    echo "📝 Using override version: $VERSION"
else
    VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r ".packages[] | select(.name == \"$PROJECT_NAME\") | .version")
    if [ -z "$VERSION" ]; then
        echo "❌ Could not find version for $PROJECT_NAME"
        exit 1
    fi
fi

echo "📦 Package: $PROJECT_NAME"
echo "🏷️  Version: $VERSION"

# Check if version already exists on crates.io
echo "🔍 Checking if version already exists on crates.io..."
if curl -s "https://crates.io/api/v1/crates/$PROJECT_NAME" | jq -e ".versions[] | select(.num == \"$VERSION\")" > /dev/null 2>&1; then
    echo "⚠️  WARNING: Version $VERSION is already published on crates.io"
    echo "   You can:"
    echo "   1. Use version override: .moon/scripts/publish.sh $PROJECT_NAME <new-version>"
    echo "   2. Update version in Cargo.toml and try again"
    exit 1
fi
echo "✅ Version $VERSION is available for publishing"
echo ""

# Confirmation prompt (skip if --yes flag provided)
if [ "$NO_CONFIRM" != "--yes" ]; then
    read -p "🤔 Do you want to release $PROJECT_NAME v$VERSION? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Release cancelled"
        exit 1
    fi
else
    echo "🚀 Auto-confirming release (--yes flag provided)"
fi

echo "🚀 Releasing $PROJECT_NAME v$VERSION"



cd "$(git rev-parse --show-toplevel)"
if ! git diff-index --quiet HEAD -- 2>/dev/null; then
    echo "❌ Git repository has uncommitted changes:"
    git status --short
    exit 1
fi

echo "📦 Running pre-release checks..."

# Run all checks
moon run "$PROJECT_NAME:build-release"
moon run "$PROJECT_NAME:test" 
moon run "$PROJECT_NAME:clippy"
moon run "$PROJECT_NAME:fmt-check"

echo "🧪 Running dry run..."
cd "crates/$PROJECT_NAME"
cargo publish --dry-run

echo "📝 Creating git tag..."
cd ../..
git tag "$PROJECT_NAME-v$VERSION"

echo "📡 Publishing to crates.io..."
cd "crates/$PROJECT_NAME"
cargo publish

echo "🌐 Pushing tag to origin..."
cd ../..
git push origin "$PROJECT_NAME-v$VERSION"

echo ""
echo "✅ Successfully released $PROJECT_NAME v$VERSION"
echo "🔗 Tag: $PROJECT_NAME-v$VERSION"
echo "📦 Published to: https://crates.io/crates/$PROJECT_NAME"