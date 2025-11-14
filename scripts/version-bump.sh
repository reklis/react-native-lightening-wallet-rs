#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}ℹ${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Get current version from package.json
get_current_version() {
    node -p "require('./package.json').version"
}

# Bump version using npm
bump_version() {
    local bump_type=$1
    npm version $bump_type --no-git-tag-version
}

# Update Cargo.toml version
update_cargo_version() {
    local new_version=$1
    
    # Update main Cargo.toml
    if [ -f "Cargo.toml" ]; then
        sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
        rm Cargo.toml.bak
    fi
    
    # Update rust/Cargo.toml
    if [ -f "rust/Cargo.toml" ]; then
        sed -i.bak "s/^version = \".*\"/version = \"$new_version\"/" rust/Cargo.toml
        rm rust/Cargo.toml.bak
    fi
}

# Main script
main() {
    # Check if we're in the project root
    if [ ! -f "package.json" ]; then
        print_error "package.json not found. Please run this script from the project root."
        exit 1
    fi

    # Check for uncommitted changes
    if [ -n "$(git status --porcelain)" ]; then
        print_warning "You have uncommitted changes. Please commit or stash them first."
        git status --short
        exit 1
    fi

    # Get bump type from argument
    BUMP_TYPE=${1:-patch}
    
    # Validate bump type
    if [[ ! "$BUMP_TYPE" =~ ^(patch|minor|major|[0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
        print_error "Invalid version bump type: $BUMP_TYPE"
        echo "Usage: $0 [patch|minor|major|X.Y.Z]"
        exit 1
    fi

    # Get current version
    CURRENT_VERSION=$(get_current_version)
    print_info "Current version: $CURRENT_VERSION"

    # Bump version
    if [[ "$BUMP_TYPE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        # Specific version provided
        NEW_VERSION=$BUMP_TYPE
        npm version $NEW_VERSION --no-git-tag-version
    else
        # Use npm to bump version
        NEW_VERSION=$(bump_version $BUMP_TYPE)
    fi

    print_info "New version: $NEW_VERSION"

    # Update Cargo.toml files
    print_info "Updating Cargo.toml files..."
    update_cargo_version $NEW_VERSION

    # Show changes
    print_info "Changes to be committed:"
    git diff package.json rust/Cargo.toml

    # Confirm
    read -p "$(echo -e ${YELLOW}Do you want to commit and tag version $NEW_VERSION? [y/N]:${NC} )" -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "Version bump cancelled. Reverting changes..."
        git checkout package.json package-lock.json rust/Cargo.toml 2>/dev/null || true
        exit 1
    fi

    # Commit changes
    print_info "Committing version bump..."
    git add package.json package-lock.json rust/Cargo.toml
    git commit -m "chore: bump version to $NEW_VERSION"

    # Create tag
    print_info "Creating tag v$NEW_VERSION..."
    git tag -a "v$NEW_VERSION" -m "Release v$NEW_VERSION"

    # Push changes and tag
    print_info "Pushing to remote..."
    git push
    git push --tags

    print_success "Version bumped to $NEW_VERSION and pushed!"
    print_info "GitHub Actions will now build and publish this version."
    echo ""
    print_info "Monitor the build at: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
}

main "$@"
