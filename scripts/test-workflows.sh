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

# Important note about macOS builds
print_warning "Note: act doesn't fully support macOS runners natively."
print_warning "macOS jobs will run in Linux containers and may fail at build steps."
print_warning "This is mainly useful for testing workflow logic, not actual builds."
echo ""

# Menu for user selection
echo "Select which workflow to test:"
echo "1) Dev Build (dev-build.yml)"
echo "2) Release (release.yml)"
echo "3) List all jobs in dev-build.yml"
echo "4) List all jobs in release.yml"
echo "5) Run specific job from dev-build.yml"
echo "6) Run specific job from release.yml"
echo "7) Dry run (show what would run)"
echo ""
read -p "Enter your choice (1-7): " choice

case $choice in
    1)
        print_info "Testing Dev Build workflow..."
        print_info "This will simulate a push to the 'dev' branch"
        act push \
            --workflows .github/workflows/dev-build.yml \
            --secret-file .secrets \
            --env GITHUB_REF=refs/heads/dev \
            --verbose
        ;;
    2)
        print_info "Testing Release workflow..."
        print_info "This will simulate a push to the 'main' branch"
        act push \
            --workflows .github/workflows/release.yml \
            --secret-file .secrets \
            --env GITHUB_REF=refs/heads/main \
            --verbose
        ;;
    3)
        print_info "Listing all jobs in dev-build.yml..."
        act -l --workflows .github/workflows/dev-build.yml
        ;;
    4)
        print_info "Listing all jobs in release.yml..."
        act -l --workflows .github/workflows/release.yml
        ;;
    5)
        print_info "Available jobs in dev-build.yml:"
        act -l --workflows .github/workflows/dev-build.yml
        echo ""
        read -p "Enter job name to run: " job_name
        print_info "Running job: $job_name"
        act push \
            --workflows .github/workflows/dev-build.yml \
            --secret-file .secrets \
            --job "$job_name" \
            --env GITHUB_REF=refs/heads/dev \
            --verbose
        ;;
    6)
        print_info "Available jobs in release.yml:"
        act -l --workflows .github/workflows/release.yml
        echo ""
        read -p "Enter job name to run: " job_name
        print_info "Running job: $job_name"
        act push \
            --workflows .github/workflows/release.yml \
            --secret-file .secrets \
            --job "$job_name" \
            --env GITHUB_REF=refs/heads/main \
            --verbose
        ;;
    7)
        print_info "Dry run - showing what would run for dev-build.yml..."
        act push \
            --workflows .github/workflows/dev-build.yml \
            --secret-file .secrets \
            --env GITHUB_REF=refs/heads/dev \
            --dryrun
        echo ""
        print_info "Dry run - showing what would run for release.yml..."
        act push \
            --workflows .github/workflows/release.yml \
            --secret-file .secrets \
            --env GITHUB_REF=refs/heads/main \
            --dryrun
        ;;
    *)
        print_error "Invalid choice"
        exit 1
        ;;
esac

echo ""
print_success "Done!"
