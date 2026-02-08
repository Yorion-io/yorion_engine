# Setup Guide

This guide walks you through setting up the WASM build and release system.

## Prerequisites

Install the required tools:

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# cocogitto (conventional commits)
cargo install --locked cocogitto

# git-cliff (changelog generation)
cargo install git-cliff

# wasm-pack (WASM builds)
cargo install wasm-pack

# act (local GitHub Actions testing - optional)
brew install act  # macOS
# or
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash  # Linux
```

## GitHub Repository Setup

### 1. Create Branches

```bash
cd /Users/gummy/BsCalendarCore/00_core

# Create dev branch
git checkout -b dev
git push -u origin dev

# Create main branch (if not exists)
git checkout -b main
git push -u origin main
```

### 2. Configure GitHub Token

**For Public Repositories:**
The `GITHUB_TOKEN` is automatically provided by GitHub Actions - no configuration needed!

**For Private Repositories:**
You'll need a Personal Access Token (PAT) with `repo` and `workflow` scopes:

1. Go to GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click **Generate new token (classic)**
3. Give it a name: "BS Calendar Release Token"
4. Select scopes:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
5. Click **Generate token** and copy it

Then add it to your repository:
1. Go to your repository on GitHub
2. Click **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `RELEASE_TOKEN_PAT`
5. Paste your token
6. Click **Add secret**

### 3. Enable GitHub Actions

The workflows already include the necessary permissions. Just verify GitHub Actions is enabled:

1. Go to **Settings** → **Actions** → **General**
2. Under **Actions permissions**, ensure:
   - ✅ "Allow all actions and reusable workflows" is selected
3. Under **Workflow permissions**:
   - ✅ "Read and write permissions" is selected
   - ✅ "Allow GitHub Actions to create and approve pull requests" is checked
4. Click **Save**

> **Note**: The workflows include `permissions: contents: write` which grants the automatic GITHUB_TOKEN the necessary access to create releases and tags.

## Initial Commit

Make your first conventional commit:

```bash
# Stage all files
git add .

# Make a conventional commit
git commit -m "feat: initial WASM release system setup

- Add WASM build scripts for web, bundler, and Node.js
- Configure GitHub Actions for dev and prod releases
- Set up semantic versioning with cocogitto
- Add comprehensive documentation"

# Push to dev branch
git push origin dev
```

This will trigger the dev build workflow!

## Testing Locally

### Build WASM Locally

```bash
# Build all WASM targets
./scripts/build-wasm.sh

# Verify outputs
ls -lh dist/wasm/
```

### Run Tests

```bash
cd engine
cargo test --all-features
```

### Test with act (Optional)

Before pushing to GitHub, test the workflows locally:

```bash
# Test dev workflow
act push -W .github/workflows/dev-build.yml

# Test with secrets (if needed)
act push -W .github/workflows/dev-build.yml --secret RELEASE_TOKEN_PAT=<your_token>
```

> **Note**: `act` runs workflows in Docker containers. The first run will download images and may take a while.

## Workflow Overview

### Dev Branch Workflow

When you push to `dev`:
1. ✅ Validates conventional commits
2. 🧪 Runs tests
3. 🔨 Builds WASM for web, bundler, and Node.js
4. 📦 Creates pre-release with `-dev` tag
5. 🚀 Uploads WASM artifacts to GitHub release

### Main Branch Workflow

When you merge to `main`:
1. ✅ Validates conventional commits
2. 🔢 Determines next semantic version
3. 🧪 Runs full test suite
4. 🔨 Builds WASM for all targets
5. 📝 Generates changelog with git-cliff
6. 🏷️ Creates git tag
7. 📦 Creates GitHub release
8. 🚀 Uploads WASM binaries

## Making Releases

### Development Release

```bash
# Make changes
git checkout dev
# ... make changes ...

# Commit with conventional format
git commit -m "feat: add new feature"

# Push to dev
git push origin dev

# GitHub Actions will create a pre-release automatically
```

### Production Release

```bash
# Merge dev to main
git checkout main
git merge dev

# Push to main
git push origin main

# GitHub Actions will:
# 1. Determine version (e.g., 0.1.0 → 0.2.0)
# 2. Create tag v0.2.0
# 3. Generate changelog
# 4. Create release with WASM binaries
```

## Semantic Versioning Rules

Based on conventional commits:

- `feat:` → **Minor** version bump (0.1.0 → 0.2.0)
- `fix:` → **Patch** version bump (0.1.0 → 0.1.1)
- `feat!:` or `BREAKING CHANGE:` → **Major** version bump (0.1.0 → 1.0.0)
- Other types (`docs:`, `chore:`, etc.) → **Patch** bump

## Troubleshooting

### Build Failures

If builds fail in GitHub Actions:

1. Check the Actions tab for error logs
2. Test locally first:
   ```bash
   ./scripts/build-wasm.sh
   ```

### WASM Build Issues

If WASM build fails:

1. Ensure `wasm-pack` is installed:
   ```bash
   cargo install wasm-pack
   ```
2. Verify WASM target is added:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

### Act Issues

If `act` fails:

1. Ensure Docker is running
2. Use the `-v` flag for verbose output:
   ```bash
   act -v push -W .github/workflows/dev-build.yml
   ```
3. Some features may not work in `act` (like creating releases)

## Next Steps

1. ✅ Install prerequisites
2. ✅ Create branches (dev, main)
3. ✅ Configure GitHub token (if private repo)
4. ✅ Enable GitHub Actions
5. ✅ Test locally
6. ✅ Make initial commit to dev
7. ✅ Verify dev build succeeds
8. ✅ Merge to main for first release
