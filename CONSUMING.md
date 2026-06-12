# Consuming BS Calendar Core

This guide shows how to integrate BS Calendar Core WASM bindings into your projects.

## 📦 WASM Integration

### Web (ES Modules)

For direct browser usage with ES modules:

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, * as bsCalendar from './wasm/web/yorion_engine.js';
        
        async function main() {
            await init();
            
            // Convert BS to Gregorian
            const engine = bsCalendar.CalendarEngine.new();
            const bsDate = bsCalendar.BsDate.new(2080, 1, 1);
            const gregorian = engine.bs_to_gregorian(bsDate);
            
            console.log('BS 2080/1/1 =', gregorian.toString());
        }
        
        main();
    </script>
</head>
<body>
    <h1>BS Calendar Demo</h1>
</body>
</html>
```

### Vite / Modern Bundlers

For use with Vite, Webpack, Rollup, or other bundlers:

```javascript
// Install from local path or download from releases
import init, * as bsCalendar from './wasm/bundler/yorion_engine.js';

async function setupCalendar() {
    await init();
    
    const engine = bsCalendar.CalendarEngine.new();
    
    // Convert Gregorian to BS
    const gregorian = new Date(2023, 3, 14); // April 14, 2023
    const bsDate = engine.gregorian_to_bs(
        gregorian.getFullYear(),
        gregorian.getMonth() + 1,
        gregorian.getDate()
    );
    
    console.log('Gregorian to BS:', bsDate);
}

setupCalendar();
```

### Node.js / NestJS

For server-side Node.js applications:

```javascript
const bsCalendar = require('./wasm/nodejs/yorion_engine.js');

// Use the library
const engine = bsCalendar.CalendarEngine.new();

// Get tithi for a date
const tithi = engine.get_tithi(2023, 4, 14);
console.log('Tithi:', tithi);
```

### TypeScript Support

All WASM targets include TypeScript definitions:

```typescript
import init, { CalendarEngine, BsDate } from './wasm/bundler/yorion_engine';

async function main() {
    await init();
    
    const engine = new CalendarEngine();
    const bsDate = BsDate.new(2080, 1, 1);
    const gregorian = engine.bs_to_gregorian(bsDate);
    
    console.log(gregorian);
}
```

### Automated Download in CI/CD

```yaml
# .github/workflows/build.yml
- name: Download WASM bindings
  run: |
    VERSION="0.1.0"
    curl -L -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
      https://github.com/Yorion-io/yorion_engine/releases/download/v${VERSION}/yorion_engine-wasm-${VERSION}.tar.gz \
      | tar xz -C src/lib/
```

---

## 🔐 Private Repository Authentication

### Environment Variable

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

### In CI/CD

```yaml
# GitHub Actions
env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

# GitLab CI
variables:
  GITHUB_TOKEN: $CI_GITHUB_TOKEN
```

### In Scripts

```bash
#!/bin/bash
if [ -z "$GITHUB_TOKEN" ]; then
    echo "Error: GITHUB_TOKEN not set"
    exit 1
fi

curl -L -H "Authorization: token $GITHUB_TOKEN" \
  https://github.com/Yorion-io/yorion_engine/releases/download/v0.1.0/yorion_engine-wasm-0.1.0.tar.gz \
  | tar xz
```

---

## 📌 Version Pinning

### Exact Version

```bash
VERSION="0.1.0"
curl -L https://github.com/Yorion-io/yorion_engine/releases/download/v${VERSION}/yorion_engine-wasm-${VERSION}.tar.gz | tar xz
```

### Latest Release

```bash
# Get latest version
LATEST=$(curl -s https://api.github.com/repos/Yorion-io/yorion_engine/releases/latest | grep '"tag_name"' | cut -d'"' -f4)

# Download latest
curl -L https://github.com/Yorion-io/yorion_engine/releases/download/${LATEST}/yorion_engine-wasm-${LATEST#v}.tar.gz | tar xz
```

### Dev Builds

```bash
# Use pre-release versions
VERSION="0.1.0-dev.123"
curl -L https://github.com/Yorion-io/yorion_engine/releases/download/v${VERSION}/yorion_engine-wasm-${VERSION}.tar.gz | tar xz
```

---

## 🛠️ Troubleshooting

### Authentication Errors

If you get 404 errors on a private repository:
1. Ensure `GITHUB_TOKEN` is set
2. Verify token has `repo` scope
3. Check token hasn't expired

### WASM Initialization

If WASM fails to load:
1. Ensure you're calling `await init()` before using the library
2. Check browser console for errors
3. Verify the WASM file path is correct
4. Ensure your server serves `.wasm` files with correct MIME type (`application/wasm`)

### Checksum Verification

```bash
# Download checksums
curl -L https://github.com/Yorion-io/yorion_engine/releases/download/v0.1.0/SHA256SUMS -o SHA256SUMS

# Verify
shasum -a 256 -c SHA256SUMS
```

---

## 📖 API Reference

The WASM bindings expose the following main types:

- `CalendarEngine` - Main engine for conversions and calculations
- `BsDate` - Bikram Sambat date representation
- `Tithi` - Lunar day enum
- `ZodiacSign` - Zodiac sign enum
- `Nakshatra` - Nakshatra enum

See the TypeScript definitions (`.d.ts` files) in each WASM target for complete API documentation.
