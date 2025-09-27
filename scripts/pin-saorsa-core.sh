#!/bin/bash
# Script to pin saorsa-core to a specific commit across all Cargo.toml files

set -e

COMMIT_SHA="fa49064"
OLD_PATTERN='saorsa-core = { git = "https://github.com/dirvine/saorsa-core-foundation.git", branch = "main" }'
NEW_PATTERN="saorsa-core = { git = \"https://github.com/dirvine/saorsa-core-foundation.git\", rev = \"$COMMIT_SHA\" }"

echo "Pinning saorsa-core to commit $COMMIT_SHA..."

# Find all Cargo.toml files and update them
find . -name "Cargo.toml" -not -path "./target/*" -exec sed -i '' "s|$OLD_PATTERN|$NEW_PATTERN|g" {} \;

echo "Updated saorsa-core dependency in all Cargo.toml files"

# Show which files were affected
echo "Files with saorsa-core dependency:"
grep -r "saorsa-core.*git" . --include="*.toml" --exclude-dir=target | grep -v ".sh:"
