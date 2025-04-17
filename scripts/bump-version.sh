#!/bin/bash
set -e

# Check if a version is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <new-version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

NEW_VERSION="$1"

# Remove 'v' prefix if present
if [[ "$NEW_VERSION" == v* ]]; then
    NEW_VERSION="${NEW_VERSION#v}"
    echo "Removing 'v' prefix. Using version: $NEW_VERSION"
fi

# Validate version format (semantic versioning)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?(\+[a-zA-Z0-9.]+)?$ ]]; then
    echo "Error: Version must follow semantic versioning (e.g., 1.0.0, 1.0.0-alpha, 1.0.0+build.1)"
    exit 1
fi

# Path to Cargo.toml
CARGO_TOML="Cargo.toml"

# Check if Cargo.toml exists
if [ ! -f "$CARGO_TOML" ]; then
    echo "Error: $CARGO_TOML not found. Are you running this from the project root?"
    exit 1
fi

# Get the current version
CURRENT_VERSION=$(grep '^version =' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/')

echo "Current version: $CURRENT_VERSION"
echo "New version: $NEW_VERSION"

# Update the version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$CARGO_TOML"

echo "✅ Updated $CARGO_TOML to version $NEW_VERSION"

# Remind about git commands
echo ""
echo "Next steps:"
echo "1. Review changes: git diff $CARGO_TOML"
echo "2. Commit changes: git commit -am \"Bump version to $NEW_VERSION\""
echo "3. Create tag: git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
echo "4. Push changes: git push && git push --tags" 