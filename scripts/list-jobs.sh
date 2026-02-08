#!/bin/bash
# Quick test commands for GitHub workflows using act

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# List all jobs in dev-build
echo "📋 Dev Build Jobs:"
act -l --workflows .github/workflows/dev-build.yml

echo ""
echo "📋 Release Jobs:"
act -l --workflows .github/workflows/release.yml

echo ""
echo "✅ To run workflows, use: ./scripts/test-workflows.sh"
