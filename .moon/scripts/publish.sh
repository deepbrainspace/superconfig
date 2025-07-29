#!/bin/bash

set -e

PROJECT_NAME="$1"

if [ -z "$PROJECT_NAME" ]; then
    echo "❌ Usage: $0 <project-name>"
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(cargo metadata --format-version 1 --no-deps | jq -r ".packages[] | select(.name == \"$PROJECT_NAME\") | .version")

if [ -z "$VERSION" ]; then
    echo "❌ Could not find version for $PROJECT_NAME"
    exit 1
fi

echo "🚀 Releasing $PROJECT_NAME v$VERSION"

# Check git status
if ! git diff-index --quiet HEAD --; then
    echo "❌ Git repository has uncommitted changes. Please commit or stash them first."
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