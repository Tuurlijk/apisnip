#!/bin/bash
#=========================================================================
# 📦 Cargo.toml Version Bumper
# A smart script to bump your Rust package version with style!
#=========================================================================
set -e

# =========== 🎨 COLORS & STYLES ===========
# Check if terminal supports colors
if [ -t 1 ]; then
  # Check if NO_COLOR is set (respect color disabling)
  if [ -z "$NO_COLOR" ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    PURPLE='\033[0;35m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    RESET='\033[0m'
  fi
fi

# Default to empty strings if colors aren't set
RED="${RED:-}"
GREEN="${GREEN:-}"
YELLOW="${YELLOW:-}"
BLUE="${BLUE:-}"
PURPLE="${PURPLE:-}"
CYAN="${CYAN:-}"
BOLD="${BOLD:-}"
RESET="${RESET:-}"

# =========== 🛠️ HELPER FUNCTIONS ===========
# Show a fancy header
print_header() {
    echo -e "\n${BOLD}${CYAN}🚀 Cargo Version Bumper${RESET}"
    echo -e "${CYAN}══════════════════════${RESET}\n"
}

# Print success message
success() {
    echo -e "${GREEN}✅ $1${RESET}"
}

# Print warning message
warning() {
    echo -e "${YELLOW}⚠️  $1${RESET}"
}

# Print error message
error() {
    echo -e "${RED}❌ $1${RESET}"
    exit 1
}

# Print info message
info() {
    echo -e "${BLUE}ℹ️  $1${RESET}"
}

# Print step header
step() {
    echo -e "\n${PURPLE}▶ $1${RESET}"
}

# Function to get the latest git tag
get_latest_tag() {
    git tag --sort=-v:refname | grep -E '^v?[0-9]+\.[0-9]+\.[0-9]+' | head -n 1
}

# Function to increment version
increment_version() {
    local version=$1
    local major=$(echo $version | cut -d. -f1)
    local minor=$(echo $version | cut -d. -f2)
    local patch=$(echo $version | cut -d. -f3 | cut -d- -f1 | cut -d+ -f1)  # Handle pre-release or build metadata

    # Remove 'v' prefix if present
    if [[ "$major" == v* ]]; then
        major="${major#v}"
    fi

    case "$2" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "$major.$((minor + 1)).0"
            ;;
        patch|bugfix)
            echo "$major.$minor.$((patch + 1))"
            ;;
        *)
            echo "$version"
            ;;
    esac
}

# =========== 📋 MAIN SCRIPT ===========
print_header

# Check if git repository exists
if [ ! -d .git ]; then
    warning "Not a git repository! Version suggestions may not work correctly."
fi

step "Reading Current Version 📖"
CARGO_TOML="Cargo.toml"

# Check if Cargo.toml exists
if [ ! -f "$CARGO_TOML" ]; then
    error "Cargo.toml not found! Are you running this from the project root? 🤔"
fi

# Get the current version from Cargo.toml
CURRENT_VERSION=$(grep '^version =' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')
info "Current Cargo.toml version: ${BOLD}$CURRENT_VERSION${RESET}"

step "Checking Git Tags 🏷️"
# Get the latest git tag
LATEST_TAG=$(get_latest_tag)
if [ -z "$LATEST_TAG" ]; then
    warning "No version tags found. Using version from Cargo.toml instead."
    LATEST_VERSION=$CURRENT_VERSION
else
    info "Latest git tag: ${BOLD}$LATEST_TAG${RESET}"
    # Extract version from tag (remove 'v' prefix if present)
    LATEST_VERSION=${LATEST_TAG#v}
fi

step "Determining New Version 🔢"
# If no command line argument was provided, suggest a bugfix increment
if [ $# -eq 0 ]; then
    # Suggest incremented bugfix version
    SUGGESTED_VERSION=$(increment_version $LATEST_VERSION bugfix)
    echo -e "${BOLD}Recommended version bump:${RESET} ${GREEN}$LATEST_VERSION${RESET} → ${GREEN}$SUGGESTED_VERSION${RESET}"
    printf "Accept this suggestion? [Y/n] "
    read -r ACCEPT
    
    if [[ "$ACCEPT" =~ ^[Yy]$ ]] || [ -z "$ACCEPT" ]; then
        NEW_VERSION=$SUGGESTED_VERSION
        success "Using suggested version: $NEW_VERSION"
    else
        echo -e "\n${BOLD}Choose version bump type:${RESET}"
        echo -e "${CYAN}1)${RESET} Major version (${YELLOW}x${RESET}.0.0) - Breaking changes"
        echo -e "${CYAN}2)${RESET} Minor version (0.${YELLOW}x${RESET}.0) - New features"
        echo -e "${CYAN}3)${RESET} Manual input - You decide!"
        printf "Your choice [1-3]: "
        read -r CHOICE

        case $CHOICE in
            1)
                NEW_VERSION=$(increment_version $LATEST_VERSION major)
                success "Major version bump: $LATEST_VERSION → $NEW_VERSION 💥"
                ;;
            2)
                NEW_VERSION=$(increment_version $LATEST_VERSION minor)
                success "Minor version bump: $LATEST_VERSION → $NEW_VERSION ✨"
                ;;
            3)
                printf "Enter version manually (default $LATEST_VERSION): "
                read -r MANUAL_VERSION
                # Use latest version as default if no input provided
                if [ -z "$MANUAL_VERSION" ]; then
                    NEW_VERSION=$LATEST_VERSION
                else
                    NEW_VERSION=$MANUAL_VERSION
                fi
                success "Using custom version: $NEW_VERSION 🔧"
                ;;
            *)
                error "Invalid choice. Exiting."
                ;;
        esac
    fi
else
    # Use provided version from command line
    NEW_VERSION="$1"
    info "Using version from command line: $NEW_VERSION"
fi

# Remove 'v' prefix if present
if [[ "$NEW_VERSION" == v* ]]; then
    NEW_VERSION="${NEW_VERSION#v}"
    warning "Removed 'v' prefix. Using version: $NEW_VERSION"
fi

# Validate version format (semantic versioning)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    error "Version must follow semantic versioning (e.g., 1.0.0, 1.0.0-alpha, 1.0.0+build.1)"
fi

step "Updating Cargo.toml 📝"
echo -e "Changing version: ${RED}$CURRENT_VERSION${RESET} → ${GREEN}$NEW_VERSION${RESET}"

# Update the version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

success "Updated $CARGO_TOML to version $NEW_VERSION"

step "Next Steps 👣"
echo -e "${CYAN}1️⃣  Review changes:${RESET} git diff $CARGO_TOML"
echo -e "${CYAN}2️⃣  Commit changes:${RESET} git commit -am \"Bump version to $NEW_VERSION\""
echo -e "${CYAN}3️⃣  Create tag:${RESET} git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
echo -e "${CYAN}4️⃣  Push changes:${RESET} git push && git push --tags"

echo -e "\n${GREEN}${BOLD}🎉 All done! Happy releasing! 🚀${RESET}\n" 