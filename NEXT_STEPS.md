# Next Steps - Release System Setup

## ✅ Completed
- ✅ Release system fully configured
- ✅ Workflows use `RELEASE_TOKEN_PAT` with fallback
- ✅ Permissions added to workflows
- ✅ Documentation updated

## 🔧 Required Actions

### 1. Add Personal Access Token (Required for Private Repo)

Since this is a **private repository**, you need to add a PAT:

1. **Create PAT**:
   - Go to GitHub → Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Name: "BS Calendar Release Token"
   - Scopes: ✅ `repo` + ✅ `workflow`
   - Generate and copy the token

2. **Add to Repository**:
   - Go to repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `RELEASE_TOKEN_PAT`
   - Click "Add secret"

### 2. Enable GitHub Actions Permissions

1. Go to repository Settings → Actions → General
2. Under "Workflow permissions":
   - ✅ Select "Read and write permissions"
   - ✅ Check "Allow GitHub Actions to create and approve pull requests"
3. Click "Save"

### 3. Fix Compilation Errors (Before Testing)

The code has compilation errors that need to be fixed:
```bash
cd engine
cargo build --release
# Fix the errors shown
```

Main issue: `BsCalendarError::InvalidDate` variant is missing.

### 4. Test the Workflow

Once compilation errors are fixed:

```bash
# Commit the fixes
git add .
git commit -m "fix: resolve compilation errors"
git push origin dev

# This will trigger the dev-build workflow
# Check: https://github.com/CalNep/engine/actions
```

### 5. Create First Production Release

After dev build succeeds:

```bash
git checkout main
git merge dev
git push origin main

# This creates production release with semantic version
```

## 📚 Documentation

- **[SETUP.md](./SETUP.md)** - Detailed setup guide
- **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** - Common operations
- **[CONSUMING.md](./CONSUMING.md)** - Integration examples
- **[README.md](./README.md)** - Project overview

## ❓ Why PAT is Needed

For private repos, the default `GITHUB_TOKEN`:
- ✅ Can create releases and tags
- ✅ Can push commits
- ❌ Cannot trigger workflows on those pushes

Using `RELEASE_TOKEN_PAT` ensures:
- Commits pushed by the workflow can trigger other workflows
- Full control over repository operations
- Better suited for automated release processes
