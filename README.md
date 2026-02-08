# BS Calendar Core - WASM Library

A comprehensive Bikram Sambat calendar engine for web applications, built with Rust and compiled to WebAssembly.

## 🚀 Features

- **BS ↔ Gregorian Conversion**: Accurate date conversion for years 2000-2090
- **Tithi Calculations**: Lunar day calculations with astronomical precision
- **Recurrence Rules**: Support for BS, AD, and Tithi-based recurring events
- **Zodiac & Nakshatra**: Sun/Moon zodiac signs and Nakshatra calculations
- **TypeScript Support**: Full TypeScript definitions included
- **Zero Dependencies**: Pure WASM with no external runtime dependencies

## 📦 Installation

### From GitHub Releases

Download pre-built WASM binaries from the [Releases](https://github.com/CalNep/engine/releases) page.

```bash
# Download and extract
curl -L https://github.com/CalNep/engine/releases/download/v0.1.0/bs_calendar_core-wasm-0.1.0.tar.gz | tar xz
```

### Private Repository Access

For private repositories, use a GitHub token:

```bash
export GITHUB_TOKEN=ghp_your_token_here

curl -L -H "Authorization: token $GITHUB_TOKEN" \
  https://github.com/CalNep/engine/releases/download/v0.1.0/bs_calendar_core-wasm-0.1.0.tar.gz | tar xz
```

## 📚 Usage

The library provides three WASM targets for different environments:

### Web (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, * as bsCalendar from './wasm/web/bs_calendar_core.js';
        
        async function main() {
            await init();
            // Use the library
            console.log('BS Calendar loaded!');
        }
        
        main();
    </script>
</head>
</html>
```

### Bundler (Vite, Webpack, Rollup)

```javascript
import init, * as bsCalendar from './wasm/bundler/bs_calendar_core.js';

await init();
// Use the library
```

### Node.js / NestJS

```javascript
const bsCalendar = require('./wasm/nodejs/bs_calendar_core.js');

// Use the library
```

See [CONSUMING.md](./CONSUMING.md) for detailed integration guides.

## 🔧 Development

### Prerequisites

- Rust 1.70+
- `wasm-pack` for building WASM

### Building Locally

```bash
# Build WASM for all targets
./scripts/build-wasm.sh
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
   - Builds WASM targets
   - Creates pre-release with `-dev` tag

### Production Releases (main branch)

1. Merge `dev` to `main`
2. GitHub Actions automatically:
   - Determines semantic version
   - Generates changelog
   - Builds WASM targets
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
│   └── build-wasm.sh   # WASM build script
├── .github/workflows/  # CI/CD
│   ├── dev-build.yml
│   └── release.yml
├── .cog.toml          # Conventional commits config
└── cliff.toml         # Changelog config
```

## 📄 License

MIT OR Apache-2.0

## 🙏 Acknowledgments

Built with:
- [cocogitto](https://github.com/cocogitto/cocogitto) - Conventional commits
- [git-cliff](https://github.com/orhun/git-cliff) - Changelog generation
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - WASM builds
