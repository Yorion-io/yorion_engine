#!/bin/bash
set -e

# Configuration
REPO_OWNER="${1:-CalNep}"
REPO_NAME="${2:-engine}"
DEST_DIR="${3:-Frameworks}" # Directory where artifacts will be saved (relative to script execution)
VERSION_FILE="$DEST_DIR/bs_calendar_version.txt"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed. Run 'brew install jq' first."
    exit 1
fi

# Check for GITHUB_TOKEN
if [ -z "$GITHUB_TOKEN" ]; then
    echo "Error: GITHUB_TOKEN environment variable is not set."
    echo "Please set GITHUB_TOKEN to a Personal Access Token with 'repo' scope."
    exit 1
fi

echo -e "${BLUE}🔍 Checking for latest release from $REPO_OWNER/$REPO_NAME...${NC}"

# Fetch latest release info strictly from GitHub (with Auth)
LATEST_RELEASE_JSON=$(curl -s -H "Authorization: token $GITHUB_TOKEN" "https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest")
LATEST_TAG=$(echo "$LATEST_RELEASE_JSON" | jq -r .tag_name)

if [ "$LATEST_TAG" == "null" ] || [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not find latest release. Check that the token has access and the repo exists."
    # Optional: Debug output
    # echo "$LATEST_RELEASE_JSON"
    exit 1
fi

# Strip 'v' prefix if present for version comparison
VERSION_NUM=${LATEST_TAG#v}

echo "Latest version: $VERSION_NUM (Tag: $LATEST_TAG)"

# Check local version
if [ -f "$VERSION_FILE" ]; then
    CURRENT_VERSION=$(cat "$VERSION_FILE")
    echo "Current local version: $CURRENT_VERSION"
    
    if [ "$CURRENT_VERSION" == "$VERSION_NUM" ]; then
        echo -e "${GREEN}✅ Already up to date!${NC}"
        exit 0
    fi
else
    echo "No local version found."
fi

echo -e "${BLUE}⬇️  Downloading version $VERSION_NUM...${NC}"

# Find the asset ID matching our filename pattern
ASSET_NAME="BsCalendarCore-macos-${VERSION_NUM}.zip"
ASSET_ID=$(echo "$LATEST_RELEASE_JSON" | jq -r ".assets[] | select(.name == \"$ASSET_NAME\") | .id")

if [ -z "$ASSET_ID" ] || [ "$ASSET_ID" == "null" ]; then
    echo "Error: Asset '$ASSET_NAME' not found in release $LATEST_TAG"
    exit 1
fi

# Create destination directory
mkdir -p "$DEST_DIR"

# Download with curl using API access for private assets
TEMP_ZIP="$DEST_DIR/temp_update.zip"
DOWNLOAD_URL="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/assets/$ASSET_ID"

HTTP_CODE=$(curl -L -w "%{http_code}" -o "$TEMP_ZIP" \
    -H "Authorization: token $GITHUB_TOKEN" \
    -H "Accept: application/octet-stream" \
    "$DOWNLOAD_URL")

if [ "$HTTP_CODE" != "200" ]; then
    echo "Error: Failed to download asset (HTTP $HTTP_CODE)"
    rm -f "$TEMP_ZIP"
    exit 1
fi

echo -e "${BLUE}📦 Extracting updates...${NC}"

# Unzip and overwrite
unzip -o "$TEMP_ZIP" -d "$DEST_DIR"
rm "$TEMP_ZIP"

# Update version file
echo "$VERSION_NUM" > "$VERSION_FILE"

echo -e "${GREEN}✅ Update complete! Frameworks updated to version $VERSION_NUM${NC}"
