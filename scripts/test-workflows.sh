#!/bin/bash
# Script to test GitHub workflows locally using act

# Ensure we're in the project root
cd "$(dirname "$0")/.."

set -e

echo "🧪 Testing GitHub Workflows Locally with Act"
echo "=============================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if act is installed
if ! command -v act &> /dev/null; then
    print_error "act is not installed. Please install it first:"
    echo "  brew install act"
    exit 1
fi

print_success "act is installed"

# Check architecture for Apple Silicon
ARCH_OPTS=""
if [[ "$(uname -m)" == "arm64" ]]; then
    print_info "Detected Apple Silicon Mac, adding --container-architecture linux/amd64"
    ARCH_OPTS="--container-architecture linux/amd64"
fi

# Artifact Server setup (Required for actions/upload-artifact@v4 and cache in act)
ARTIFACT_PATH="/tmp/artifacts"
mkdir -p "$ARTIFACT_PATH"
# We exclude --container-architecture from native host runs as it can cause 'invalid reference format'
COMMON_OPTS="--artifact-server-path $ARTIFACT_PATH --secret-file .secrets"

# Platform mapping options
# 1. Containerized - maps macos to Ubuntu (Safer/Logic check)
CONTAINER_PLAT_OPTS="-P macos-latest=catthehacker/ubuntu:act-22.04"
# 2. Host Native - runs macos jobs directly on your Mac host
HOST_PLAT_OPTS="-P macos-latest=-"

# Important note about macOS builds
print_warning "Note: You are on a Mac, so you can run macOS jobs directly on your host."
print_warning "Options 1 & 2 run in Docker (logic check only)."
print_warning "Option 3 runs NATIVE jobs directly on your host (Mac builds will work)."
echo ""

# Menu for user selection
echo "Select which workflow to test:"
echo "1) Dev Build (dev-build.yml) - Linux Containers"
echo "2) Release (release.yml) - Linux Containers"
echo "3) Dev Build (dev-build.yml) - NATIVE HOST (Runs macos-latest jobs on your Mac!)"
echo "4) Build WASM only (Linux Container)"
echo "5) List all jobs"
echo "6) Dry run (Native Host Routing)"
echo ""
read -p "Enter your choice (1-6): " choice

case $choice in
    1)
        print_info "Testing Dev Build (Linux Containers)..."
        act push $COMMON_OPTS $ARCH_OPTS $CONTAINER_PLAT_OPTS \
            --workflows .github/workflows/dev-build.yml \
            --env GITHUB_REF=refs/heads/dev \
            --verbose
        ;;
    2)
        print_info "Testing Release (Linux Containers)..."
        act push $COMMON_OPTS $ARCH_OPTS $CONTAINER_PLAT_OPTS \
            --workflows .github/workflows/release.yml \
            --env GITHUB_REF=refs/heads/main \
            --verbose
        ;;
    3)
        print_info "🚀 Running Dev Build jobs (macos-latest) NATIVELY on your host..."
        print_info "Linux jobs (WASM/Bindings) will still run in Docker if they use ubuntu-latest."
        # Note: we omit $ARCH_OPTS here because native host routing doesn't like them
        act push $COMMON_OPTS $HOST_PLAT_OPTS \
            --workflows .github/workflows/dev-build.yml \
            --env GITHUB_REF=refs/heads/dev \
            --verbose
        ;;
    4)
        print_info "Running Build WASM job..."
        act push $COMMON_OPTS $ARCH_OPTS $CONTAINER_PLAT_OPTS \
            --workflows .github/workflows/dev-build.yml \
            --job build-wasm \
            --env GITHUB_REF=refs/heads/dev \
            --verbose
        ;;
    5)
        print_info "Jobs in dev-build.yml:"
        act -l --workflows .github/workflows/dev-build.yml
        echo ""
        print_info "Jobs in release.yml:"
        act -l --workflows .github/workflows/release.yml
        ;;
    6)
        print_info "Dry run (Native Host Routing)..."
        act push $COMMON_OPTS $HOST_PLAT_OPTS \
            --workflows .github/workflows/dev-build.yml \
            --dryrun
        ;;
    *)
        print_error "Invalid choice"
        exit 1
        ;;
esac

echo ""
print_success "Done!"
