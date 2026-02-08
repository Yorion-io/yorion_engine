# BS Calendar Core - Multi-Platform Release System

A comprehensive Bikram Sambat calendar engine with automated multi-platform releases.

## 🚀 Features

- **WASM Bindings**: Use in web browsers, bundlers, and Node.js
- **Flutter Bindings**: Native Dart integration for mobile apps
- **Native Libraries**: C-compatible FFI for any language (Python, Go, Ruby, etc.)
- **Automated Releases**: Semantic versioning from conventional commits
- **Cross-Platform**: macOS, Linux, Windows, iOS, Android, Web

## 📦 Installation

### From GitHub Releases

Download pre-built binaries from the [Releases](https://github.com/YOUR_USERNAME/bs_calendar_core/releases) page.

#### WASM
```bash
# Download and extract
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-wasm-0.1.0.tar.gz | tar xz
```

#### Native Libraries
```bash
# macOS
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-native-macos-universal-0.1.0.tar.gz | tar xz

# Linux
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-native-linux-x86_64-0.1.0.tar.gz | tar xz

# Windows
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-native-windows-x86_64-0.1.0.tar.gz | tar xz
```

### Private Repository Access

For private repositories, you'll need a GitHub token:

```bash
# Set your GitHub token
export GITHUB_TOKEN=ghp_your_token_here

# Download with authentication
curl -L -H "Authorization: token $GITHUB_TOKEN" \
  https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-wasm-0.1.0.tar.gz | tar xz
```

## 📚 Usage

See [CONSUMING.md](./CONSUMING.md) for detailed integration guides for each platform.

## 🔧 Development

### Prerequisites

- Rust 1.70+
- For WASM: `wasm-pack`
- For Flutter: `flutter_rust_bridge_codegen`
- For cross-compilation: `cross`

### Building Locally

```bash
# Build all targets
./scripts/release.sh

# Build specific targets
./scripts/build-wasm.sh
./scripts/build-flutter.sh
./scripts/build-native.sh
```

### Running Tests

```bash
cd engine
cargo test --all-features
```

## 🤝 Contributing

We use conventional commits for automatic semantic versioning:

```bash
# Features (minor version bump)
git commit -m "feat: add new calendar conversion function"

# Bug fixes (patch version bump)
git commit -m "fix: correct tithi calculation for edge case"

# Breaking changes (major version bump)
git commit -m "feat!: redesign API for better ergonomics"
# or
git commit -m "feat: redesign API

BREAKING CHANGE: The API has been completely redesigned"
```

### Commit Types

- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test additions/changes
- `chore`: Maintenance tasks
- `ci`: CI/CD changes

## 📋 Release Process

### Development Releases (dev branch)

1. Make changes following conventional commits
2. Push to `dev` branch
3. GitHub Actions automatically:
   - Validates commits
   - Runs tests
   - Builds all targets
   - Creates pre-release with `-dev` tag

### Production Releases (main branch)

1. Merge `dev` to `main`
2. GitHub Actions automatically:
   - Determines semantic version
   - Generates changelog
   - Builds all targets
   - Creates GitHub release
   - Tags with version

## 🏗️ Architecture

```
00_core/
├── engine/              # Core Rust library
│   ├── src/            # Source code
│   ├── tests/          # Tests
│   └── Cargo.toml      # Dependencies
├── scripts/            # Build scripts
│   ├── build-wasm.sh
│   ├── build-flutter.sh
│   ├── build-native.sh
│   └── release.sh
├── .github/workflows/  # CI/CD
│   ├── dev-build.yml
│   └── release.yml
├── .cog.toml          # Conventional commits config
├── cliff.toml         # Changelog config
└── Cross.toml         # Cross-compilation config
```

## 📄 License

MIT OR Apache-2.0

## 🙏 Acknowledgments

Built with:
- [cocogitto](https://github.com/cocogitto/cocogitto) - Conventional commits
- [git-cliff](https://github.com/orhun/git-cliff) - Changelog generation
- [cross](https://github.com/cross-rs/cross) - Cross-compilation
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - WASM builds
- [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge) - Flutter bindings
