# Quick Reference - Release System

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

### Test Workflows with act
```bash
# Simple test
act -W .github/workflows/test-local.yml

# Test dev workflow
act push -W .github/workflows/dev-build.yml
```

### Build Locally
```bash
# WASM
./scripts/build-wasm.sh

# Native (all platforms)
./scripts/build-native.sh

# Flutter
./scripts/build-flutter.sh

# Everything
./scripts/release.sh
```

## 📦 Release Artifacts

Each release includes:
- `bs_calendar_core-wasm-{version}.tar.gz`
- `bs_calendar_core-native-macos-universal-{version}.tar.gz`
- `bs_calendar_core-native-linux-x86_64-{version}.tar.gz`
- `bs_calendar_core-native-linux-aarch64-{version}.tar.gz`
- `bs_calendar_core-native-windows-x86_64-{version}.tar.gz`
- `SHA256SUMS` - Checksums

## 🔐 GitHub Token (Required)

Add your Personal Access Token as `RELEASE_TOKEN_PAT`:

1. Create PAT with `repo` + `workflow` scopes
2. Add as repository secret: `RELEASE_TOKEN_PAT`

## 🛠️ Prerequisites

```bash
# Install tools
cargo install --locked cocogitto
cargo install git-cliff
cargo install cross --git https://github.com/cross-rs/cross
cargo install wasm-pack
cargo install cbindgen
brew install act  # for local testing
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
cargo build --release
cargo test --all-features
```

### Workflow doesn't trigger
- Check branch name (dev or main)
- Verify GitHub Actions is enabled
- Check workflow permissions

## 📚 Documentation

- [README.md](./README.md) - Project overview
- [SETUP.md](./SETUP.md) - Detailed setup guide
- [CONSUMING.md](./CONSUMING.md) - Integration examples
- [walkthrough.md](./.gemini/antigravity/brain/.../walkthrough.md) - Implementation details
