This guide shows how to integrate YorionEngine WASM bindings into your projects.

## <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/></svg> WASM Integration

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
    <\/script>
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

## <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg> Private Repository Authentication

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

## <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="17" x2="12" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24V17z"/></svg> Version Pinning

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

## <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg> Troubleshooting

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

## <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></svg> API Reference

The WASM bindings expose the following main types:

- `BsDate` - Bikram Sambat date representation
- `Tithi` - Lunar day enum
- `ZodiacSign` - Zodiac sign enum
- `Nakshatra` - Nakshatra enum
- `Yoga` - Yoga enum (one of the five panchanga angas)
- `Karana` - Karana enum (one of the five panchanga angas)

…and free functions including `gregorian_to_bs`, `bs_to_gregorian`, `get_tithi`, `get_daily_astro_info_with_location` (returns tithi, sun/moon sign, nakshatra, yoga, karana, sunrise, sunset), `get_month_calendar_with_location`, `get_month_events`, and the localized name helpers `get_tithi_name` / `get_zodiac_name` / `get_nakshatra_name` / `get_yoga_name` / `get_karana_name`.

> The WASM surface is intentionally narrower than the native Rust `CalendarEngine`: lower-level helpers such as `get_tithi_end`, `get_daily_panchanga`, `checked_bs_date`, and the raw `get_yoga`/`get_karana` enum getters are Rust-only. The data they produce (yoga, karana) is still available through `get_daily_astro_info_with_location` and `get_month_calendar_with_location`.

See the TypeScript definitions (`.d.ts` files) in each WASM target for complete API documentation.

---

## Binary size

Current distribution sizes (post `wasm-opt`):

| Target | `.wasm` | JS glue |
|---|---|---|
| web | ~3.3 MB | ~50 KB |
| bundler | ~3.3 MB | ~46 KB |
| nodejs | ~3.3 MB | ~48 KB |

Compresses to ~1 MB over HTTP/2 with gzip. Size comes from:

| Source | Contribution |
|---|---|
| 126 years of calendar data | ~800 KB |
| Astronomical code (`suncalc`, `astro`) | ~1.2 MB |
| Recurrence engine (`rrule` + BS extensions) | ~600 KB |
| Rust std (panic, fmt, allocator) | ~700 KB |

**To reduce size:**

1. `wasm-opt -Oz` — build script calls this automatically if `binaryen` is installed (~10–15% saving).
2. Feature-flag astronomical calculations — not yet implemented; estimated ~800 KB saving. PRs welcome.
3. Narrow the calendar data range from BS 1975–2100 — not yet implemented; would require a build-time env var.