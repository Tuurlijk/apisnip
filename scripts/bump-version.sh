#!/bin/bash
#=========================================================================
# 📦 Cargo.toml Version Bumper
# A smart script to bump your Rust package version with style!
#=========================================================================
set -e

# =========== 🔍 PARSE ARGUMENTS ===========
HELP=false
DRY_RUN=false
VERSION_ARG=""

for arg in "$@"; do
  case $arg in
    -h|--help)
      HELP=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    -*)
      echo "Unknown option: $arg"
      exit 1
      ;;
    *)
      # First non-option argument is the version
      if [ -z "$VERSION_ARG" ]; then
        VERSION_ARG="$arg"
      fi
      shift
      ;;
  esac
done

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
# Show help message
show_help() {
  echo -e "${BOLD}📦 Cargo.toml Version Bumper${RESET}"
  echo
  echo "A tool to bump your Rust package version with automatic git tag support."
  echo
  echo -e "${BOLD}USAGE:${RESET}"
  echo "  $0 [OPTIONS] [VERSION]"
  echo
  echo -e "${BOLD}OPTIONS:${RESET}"
  echo "  -h, --help    Show this help message and exit"
  echo "  --dry-run     Show what would be done without making changes"
  echo
  echo -e "${BOLD}ARGUMENTS:${RESET}"
  echo "  VERSION       Optional version to set (e.g., 1.2.3)"
  echo "                If not provided, will suggest a version bump"
  echo
  echo -e "${BOLD}EXAMPLES:${RESET}"
  echo "  $0            Interactive mode with auto-suggestions"
  echo "  $0 1.2.3      Set version to 1.2.3"
  echo "  $0 --dry-run  Preview changes without applying them"
  echo
  echo -e "${BOLD}ENVIRONMENT:${RESET}"
  echo "  NO_COLOR      Set to disable colorized output"
  echo
}

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

# Execute a command safely
execute() {
    local cmd="$1"
    local msg="$2"
    local error_msg="${3:-Command failed: $cmd}"
    
    info "$msg"
    if ! $DRY_RUN; then
        if ! eval "$cmd"; then
            warning "$error_msg"
            return 1
        fi
    else
        echo -e "${YELLOW}[DRY RUN]${RESET} Would execute: $cmd"
    fi
    return 0
}

# Function to get the latest git tag
get_latest_tag() {
    git tag --sort=-v:refname | grep -E '^v?[0-9]+\.[0-9]+\.[0-9]+' | head -n 1
}

# Function to get current git branch
get_current_branch() {
    git rev-parse --abbrev-ref HEAD
}

# Function to check git status
check_git_status() {
    # Check if we have uncommitted changes
    if ! git diff-index --quiet HEAD --; then
        warning "You have uncommitted changes in your working directory."
        warning "Consider committing or stashing them before bumping version."
        printf "Continue anyway? [y/N] "
        read -r CONTINUE
        if [[ ! "$CONTINUE" =~ ^[Yy]$ ]]; then
            echo "Exiting without changes."
            exit 0
        fi
    fi
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

# Show help if requested
if $HELP; then
    show_help
    exit 0
fi

print_header

# Check if git repository exists
if [ ! -d .git ]; then
    warning "Not a git repository! Version suggestions may not work correctly."
else
    # If it's a git repo, check for uncommitted changes
    check_git_status
    
    # Show current branch
    CURRENT_BRANCH=$(get_current_branch)
    info "Current branch: ${BOLD}$CURRENT_BRANCH${RESET}"
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
if [ -z "$VERSION_ARG" ]; then
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
    NEW_VERSION="$VERSION_ARG"
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
if ! $DRY_RUN; then
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"
    success "Updated $CARGO_TOML to version $NEW_VERSION"
    
    # Also update Cargo.lock
    step "Updating Cargo.lock 🔄"
    if [ -f "Cargo.lock" ]; then
        # Extract package name from Cargo.toml
        PACKAGE_NAME=$(grep '^name =' "$CARGO_TOML" | head -1 | sed 's/name = "\(.*\)"/\1/')
        
        info "Running cargo update for package: $PACKAGE_NAME"
        cargo update --package "$PACKAGE_NAME"
        success "Updated Cargo.lock with new version"
    else
        warning "Cargo.lock not found. It will be generated the next time you run cargo build."
    fi
else
    echo -e "${YELLOW}[DRY RUN]${RESET} Would update $CARGO_TOML to version $NEW_VERSION"
    echo -e "${YELLOW}[DRY RUN]${RESET} Would update Cargo.lock"
fi

# Ask to execute the remaining steps
if ! $DRY_RUN; then
    step "Planned Actions 📋"
    echo -e "${CYAN}1️⃣  Commit changes:${RESET} git commit -am \"Bump version to $NEW_VERSION\""
    echo -e "${CYAN}2️⃣  Create tag:${RESET} git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
    echo -e "${CYAN}3️⃣  Push changes:${RESET} git push && git push --tags"
    echo
    
    printf "${YELLOW}Would you like to execute these steps automatically?${RESET} [y/N] "
    read -r AUTO_EXECUTE

    if [[ "$AUTO_EXECUTE" =~ ^[Yy]$ ]]; then
        step "Executing Steps 🚀"
        
        # Check if there are changes to commit
        info "Checking git status..."
        if ! git diff --quiet HEAD -- "$CARGO_TOML" "Cargo.lock"; then
            # There are changes to commit
            execute "git add $CARGO_TOML Cargo.lock" \
                    "Staging changes..." \
                    "Failed to stage changes."
                    
            if [ $? -eq 0 ]; then
                execute "git commit -m \"Bump version to $NEW_VERSION\"" \
                        "Committing changes..." \
                        "Failed to commit changes. Please check your git status."
            fi
        else
            warning "No changes detected in Cargo.toml or Cargo.lock. They might already be at version $NEW_VERSION."
            info "Proceeding with tag creation anyway..."
        fi
        
        # Always try to create the tag, even if commit failed (might be using existing commit)
        # Check if tag already exists
        if git rev-parse "v$NEW_VERSION" >/dev/null 2>&1; then
            warning "Tag v$NEW_VERSION already exists! Skipping tag creation."
        else 
            execute "git tag -a \"v$NEW_VERSION\" -m \"Release v$NEW_VERSION\"" \
                    "Creating tag v$NEW_VERSION..." \
                    "Failed to create tag. Perhaps it already exists?"
        fi
        
        # Only proceed with push if we have something to push (commit or tag)
        if [ $? -eq 0 ]; then
            # Confirm before pushing (this is a potentially risky operation)
            printf "${YELLOW}Ready to push changes and tags to remote. Continue?${RESET} [y/N] "
            read -r CONFIRM_PUSH
            
            if [[ "$CONFIRM_PUSH" =~ ^[Yy]$ ]]; then
                execute "git push && git push --tags" \
                        "Pushing changes and tags..." \
                        "Failed to push. Do you have upstream permissions?"
                
                if [ $? -eq 0 ]; then
                    success "All steps completed successfully! 🎉"
                fi
            else
                info "Skipping push operation. Changes are committed locally."
            fi
        fi
    else
        info "No actions taken. You can run these steps manually."
    fi
else
    step "Next Steps (Dry Run) 👣"
    echo -e "${YELLOW}[DRY RUN]${RESET} Would propose these next steps:"
    echo -e "${CYAN}1️⃣  Commit changes:${RESET} git commit -am \"Bump version to $NEW_VERSION\""
    echo -e "${CYAN}2️⃣  Create tag:${RESET} git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
    echo -e "${CYAN}3️⃣  Push changes:${RESET} git push && git push --tags"
fi

echo -e "\n${GREEN}${BOLD}🎉 All done! Happy releasing! 🚀${RESET}\n" 