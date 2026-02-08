# Quick Reference - WASM Release System

## 🚀 Making a Release

### Development Release (Pre-release)
```bash
# On dev branch
git add .
git commit -m "feat: add new feature"
git push origin dev
# → Creates v0.1.0-dev.{build_number}
```

### Production Release
```bash
# Merge dev to main
git checkout main
git merge dev
git push origin main
# → Creates v0.2.0 (version auto-determined)
```

## 📝 Conventional Commit Format

```
<type>(<scope>): <subject>
```

### Common Types
- `feat:` → Minor version bump (0.1.0 → 0.2.0)
- `fix:` → Patch version bump (0.1.0 → 0.1.1)
- `feat!:` → Major version bump (0.1.0 → 1.0.0)
- `docs:`, `chore:`, `refactor:` → Patch bump

### Examples
```bash
git commit -m "feat: add lunar calendar support"
git commit -m "fix: correct tithi calculation"
git commit -m "feat!: redesign API

BREAKING CHANGE: Complete API redesign"
```

## 🔧 Local Testing

### Build Locally
```bash
# Build WASM for all targets
./scripts/build-wasm.sh

# Run tests
cd engine
cargo test --all-features
```

### Test Workflows with act
```bash
# Test dev workflow
act push -W .github/workflows/dev-build.yml
```

## 📦 Release Artifacts

Each release includes:
- `bs_calendar_core-wasm-{version}.tar.gz` - WASM bindings for web, bundler, and Node.js
- `SHA256SUMS` - Checksums for verification

## 🔐 GitHub Token

**Public Repositories:** No token needed - `GITHUB_TOKEN` is automatic!

**Private Repositories:** Add Personal Access Token as `RELEASE_TOKEN_PAT`:
1. Create PAT with `repo` + `workflow` scopes
2. Add as repository secret: `RELEASE_TOKEN_PAT`

## 🛠️ Prerequisites

```bash
# Install tools
rustup target add wasm32-unknown-unknown
cargo install --locked cocogitto
cargo install git-cliff
cargo install wasm-pack
brew install act  # for local testing (optional)
```

## 📊 Workflow Status

Check GitHub Actions tab to see:
- ✅ Build status
- 📦 Release creation
- 🔍 Test results

## 🐛 Troubleshooting

### Build fails
```bash
# Test locally first
cd engine
cargo build --release --features wasm --no-default-features
cargo test --all-features
```

### WASM build fails
```bash
# Ensure wasm-pack is installed
cargo install wasm-pack

# Verify WASM target
rustup target add wasm32-unknown-unknown
```

### Workflow doesn't trigger
- Check branch name (dev or main)
- Verify GitHub Actions is enabled
- Check workflow permissions

## 📚 Documentation

- [README.md](./README.md) - Project overview
- [SETUP.md](./SETUP.md) - Detailed setup guide
- [CONSUMING.md](./CONSUMING.md) - WASM integration examples
