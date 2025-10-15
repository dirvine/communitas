#!/bin/bash
# Create Release Script for Communitas
# Automates the complete release process including changelog, commit, tag, and push

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR/.."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
VERSION="${1}"
RELEASE_TYPE="${2:-patch}"

if [ -z "$VERSION" ]; then
    # Read current version
    CURRENT_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

    echo -e "${BLUE}🚀 Communitas Release Tool${NC}"
    echo ""
    echo "Current version: $CURRENT_VERSION"
    echo ""
    echo "Usage: $0 <version> [release-type]"
    echo ""
    echo "Examples:"
    echo "  $0 0.2.0              # Create release v0.2.0"
    echo "  $0 0.2.0 minor        # Create minor release with auto-generated notes"
    echo "  $0 0.2.0-beta.1 pre   # Create prerelease"
    echo ""
    echo "Or use bump-version.sh first:"
    echo "  ./scripts/bump-version.sh patch"
    echo "  ./scripts/create-release.sh"
    exit 1
fi

echo -e "${BLUE}🚀 Creating Release v$VERSION${NC}"
echo ""

# Verify working directory is clean
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}❌ Error: Working directory has uncommitted changes${NC}"
    echo ""
    echo "Please commit or stash your changes before creating a release:"
    echo "  git status"
    exit 1
fi

# Verify we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${YELLOW}⚠️  Warning: Not on main branch (currently on: $CURRENT_BRANCH)${NC}"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 0
    fi
fi

# Update CHANGELOG.md
CHANGELOG_FILE="$PROJECT_ROOT/CHANGELOG.md"

echo -e "${BLUE}📝 Updating CHANGELOG.md...${NC}"

if [ ! -f "$CHANGELOG_FILE" ]; then
    echo -e "${YELLOW}⚠️  CHANGELOG.md not found, creating...${NC}"
    cat > "$CHANGELOG_FILE" <<EOF
# Changelog

All notable changes to Communitas will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

EOF
fi

# Get commit messages since last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")

if [ -z "$LAST_TAG" ]; then
    echo -e "${YELLOW}⚠️  No previous tags found, including all commits${NC}"
    COMMITS=$(git log --pretty=format:"- %s" --no-merges)
else
    echo "Last tag: $LAST_TAG"
    COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"- %s" --no-merges)
fi

# Categorize commits
FEATURES=$(echo "$COMMITS" | grep "^- feat" || true)
FIXES=$(echo "$COMMITS" | grep "^- fix" || true)
DOCS=$(echo "$COMMITS" | grep "^- docs" || true)
CHORES=$(echo "$COMMITS" | grep "^- chore\|^- refactor\|^- style" || true)

# Generate changelog entry
RELEASE_DATE=$(date +%Y-%m-%d)
CHANGELOG_ENTRY="## [$VERSION] - $RELEASE_DATE

"

if [ -n "$FEATURES" ]; then
    CHANGELOG_ENTRY+="### Added
$FEATURES

"
fi

if [ -n "$FIXES" ]; then
    CHANGELOG_ENTRY+="### Fixed
$FIXES

"
fi

if [ -n "$DOCS" ]; then
    CHANGELOG_ENTRY+="### Documentation
$DOCS

"
fi

if [ -n "$CHORES" ]; then
    CHANGELOG_ENTRY+="### Internal
$CHORES

"
fi

# Insert changelog entry after header
HEADER_END=$(grep -n "^## \[" "$CHANGELOG_FILE" | head -1 | cut -d: -f1)
if [ -z "$HEADER_END" ]; then
    # No previous releases, append after header
    echo "$CHANGELOG_ENTRY" >> "$CHANGELOG_FILE"
else
    # Insert before first release
    sed -i.bak "${HEADER_END}i\\
$CHANGELOG_ENTRY
" "$CHANGELOG_FILE"
    rm "${CHANGELOG_FILE}.bak"
fi

echo -e "${GREEN}✓ CHANGELOG.md updated${NC}"

# Show changelog entry
echo ""
echo -e "${BLUE}📋 Release Notes:${NC}"
echo "$CHANGELOG_ENTRY"

# Confirm release
echo ""
read -p "Continue with release creation? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}❌ Release cancelled${NC}"
    # Restore CHANGELOG.md
    git checkout "$CHANGELOG_FILE"
    exit 0
fi

# Commit changelog
echo -e "${BLUE}💾 Committing changelog...${NC}"
git add "$CHANGELOG_FILE"
git commit -m "chore: update CHANGELOG for v$VERSION"

# Create annotated tag
echo -e "${BLUE}🏷️  Creating tag v$VERSION...${NC}"
git tag -a "v$VERSION" -m "Release v$VERSION

$CHANGELOG_ENTRY"

echo -e "${GREEN}✓ Tag created${NC}"

# Push to remote
echo ""
echo -e "${BLUE}📤 Pushing to remote...${NC}"
read -p "Push to origin? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git push origin "$CURRENT_BRANCH"
    git push origin "v$VERSION"

    echo ""
    echo -e "${GREEN}🎉 Release v$VERSION created successfully!${NC}"
    echo ""
    echo -e "${BLUE}📋 Next steps:${NC}"
    echo "  1. GitHub Actions will automatically build and sign the release"
    echo "  2. Monitor: https://github.com/dirvine/communitas/actions"
    echo "  3. Review draft release: https://github.com/dirvine/communitas/releases"
    echo "  4. Add any additional release notes if needed"
    echo "  5. Publish the release"
    echo ""
    echo -e "${BLUE}🔗 Release URL:${NC}"
    echo "  https://github.com/dirvine/communitas/releases/tag/v$VERSION"
else
    echo ""
    echo -e "${YELLOW}⚠️  Release created locally but not pushed${NC}"
    echo ""
    echo "To push manually:"
    echo "  git push origin $CURRENT_BRANCH"
    echo "  git push origin v$VERSION"
    echo ""
    echo "To undo:"
    echo "  git tag -d v$VERSION"
    echo "  git reset --hard HEAD~1"
fi
