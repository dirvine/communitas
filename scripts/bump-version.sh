#!/bin/bash
# Version Bump Script for Communitas
# Automatically bumps version across all project files

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
BUMP_TYPE="${1:-patch}"
DRY_RUN="${2}"

# Validate bump type
if [[ ! "$BUMP_TYPE" =~ ^(major|minor|patch|prerelease)$ ]]; then
    echo -e "${RED}❌ Error: Invalid bump type '$BUMP_TYPE'${NC}"
    echo ""
    echo "Usage: $0 <major|minor|patch|prerelease> [--dry-run]"
    echo ""
    echo "Examples:"
    echo "  $0 patch           # 0.1.0 → 0.1.1"
    echo "  $0 minor           # 0.1.0 → 0.2.0"
    echo "  $0 major           # 0.1.0 → 1.0.0"
    echo "  $0 prerelease      # 0.1.0 → 0.1.1-beta.1"
    echo "  $0 patch --dry-run # Preview changes without applying"
    exit 1
fi

echo -e "${BLUE}🔧 Communitas Version Bump Tool${NC}"
echo ""

# Read current version from workspace Cargo.toml
CURRENT_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo -e "${RED}❌ Error: Could not read current version from Cargo.toml${NC}"
    exit 1
fi

echo -e "📌 Current version: ${YELLOW}$CURRENT_VERSION${NC}"

# Calculate new version using semver logic
calculate_new_version() {
    local version=$1
    local bump=$2

    # Parse version (remove any prerelease/metadata)
    IFS='.-' read -r MAJOR MINOR PATCH PRERELEASE <<< "$version"

    case $bump in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        prerelease)
            if [ -z "$PRERELEASE" ]; then
                PATCH=$((PATCH + 1))
                echo "${MAJOR}.${MINOR}.${PATCH}-beta.1"
                return
            else
                # Increment prerelease number
                PRERELEASE_NUM=$(echo "$PRERELEASE" | grep -o '[0-9]*$')
                if [ -z "$PRERELEASE_NUM" ]; then
                    PRERELEASE_NUM=1
                else
                    PRERELEASE_NUM=$((PRERELEASE_NUM + 1))
                fi
                echo "${MAJOR}.${MINOR}.${PATCH}-beta.${PRERELEASE_NUM}"
                return
            fi
            ;;
    esac

    echo "${MAJOR}.${MINOR}.${PATCH}"
}

NEW_VERSION=$(calculate_new_version "$CURRENT_VERSION" "$BUMP_TYPE")

echo -e "🎯 New version:     ${GREEN}$NEW_VERSION${NC}"
echo ""

if [ "$DRY_RUN" = "--dry-run" ]; then
    echo -e "${YELLOW}🔍 DRY RUN MODE - No changes will be made${NC}"
    echo ""
fi

# Files to update
declare -a FILES_TO_UPDATE=(
    "Cargo.toml"
    "package.json"
    "communitas-desktop/Cargo.toml"
    "communitas-desktop/tauri.conf.json"
)

echo -e "${BLUE}📝 Files to update:${NC}"
for file in "${FILES_TO_UPDATE[@]}"; do
    if [ -f "$PROJECT_ROOT/$file" ]; then
        echo "  ✓ $file"
    else
        echo -e "  ${YELLOW}⚠ $file (not found, skipping)${NC}"
    fi
done
echo ""

# Confirm unless dry-run
if [ "$DRY_RUN" != "--dry-run" ]; then
    read -p "Continue with version bump? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}❌ Version bump cancelled${NC}"
        exit 0
    fi
fi

# Function to update version in a file
update_version_in_file() {
    local file=$1
    local pattern=$2
    local replacement=$3

    if [ ! -f "$file" ]; then
        echo -e "  ${YELLOW}⚠ Skipping $file (not found)${NC}"
        return
    fi

    if [ "$DRY_RUN" = "--dry-run" ]; then
        echo -e "  ${BLUE}Would update: $file${NC}"
        grep -n "$pattern" "$file" | head -1 || true
    else
        if grep -q "$pattern" "$file"; then
            sed -i.bak "$replacement" "$file"
            rm "${file}.bak"
            echo -e "  ${GREEN}✓ Updated: $file${NC}"
        else
            echo -e "  ${YELLOW}⚠ Pattern not found in $file${NC}"
        fi
    fi
}

# Update workspace Cargo.toml
echo -e "${BLUE}Updating Cargo.toml (workspace)...${NC}"
update_version_in_file \
    "$PROJECT_ROOT/Cargo.toml" \
    '^version = "' \
    "s/^version = \".*\"/version = \"$NEW_VERSION\"/"

# Update package.json
echo -e "${BLUE}Updating package.json...${NC}"
update_version_in_file \
    "$PROJECT_ROOT/package.json" \
    '"version":' \
    "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/"

# Update communitas-desktop/Cargo.toml (if it has its own version)
if grep -q '^version = ' "$PROJECT_ROOT/communitas-desktop/Cargo.toml" 2>/dev/null; then
    echo -e "${BLUE}Updating communitas-desktop/Cargo.toml...${NC}"
    update_version_in_file \
        "$PROJECT_ROOT/communitas-desktop/Cargo.toml" \
        '^version = "' \
        "s/^version = \".*\"/version = \"$NEW_VERSION\"/"
fi

# Update tauri.conf.json
echo -e "${BLUE}Updating tauri.conf.json...${NC}"
update_version_in_file \
    "$PROJECT_ROOT/communitas-desktop/tauri.conf.json" \
    '"version":' \
    "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/"

if [ "$DRY_RUN" = "--dry-run" ]; then
    echo ""
    echo -e "${YELLOW}✓ Dry run complete - no files were modified${NC}"
    exit 0
fi

echo ""
echo -e "${GREEN}✅ Version bumped successfully!${NC}"
echo ""
echo -e "${BLUE}📋 Next steps:${NC}"
echo "  1. Review changes: git diff"
echo "  2. Update CHANGELOG.md with release notes"
echo "  3. Commit changes: git add -A && git commit -m \"chore: bump version to $NEW_VERSION\""
echo "  4. Create tag: git tag v$NEW_VERSION"
echo "  5. Push: git push origin main --tags"
echo ""
echo -e "${BLUE}💡 Tip: Use ./scripts/create-release.sh to automate steps 2-5${NC}"
