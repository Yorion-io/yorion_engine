# Consuming BS Calendar Core

This guide shows how to integrate BS Calendar Core binaries into your projects.

## 📦 WASM Integration

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
<body>
    <h1>BS Calendar Demo</h1>
</body>
</html>
```

### Vite / Modern Bundlers

```javascript
// Install from local path or download from releases
import init, * as bsCalendar from './wasm/bundler/bs_calendar_core.js';

await init();
// Use the library
```

### Node.js

```javascript
const bsCalendar = require('./wasm/nodejs/bs_calendar_core.js');

// Use the library
```

### Automated Download in CI/CD

```yaml
# .github/workflows/build.yml
- name: Download WASM bindings
  run: |
    VERSION="0.1.0"
    curl -L -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
      https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v${VERSION}/bs_calendar_core-wasm-${VERSION}.tar.gz \
      | tar xz -C src/lib/
```

---

## 📱 Flutter Integration

### Download and Setup

```bash
# Create packages directory
mkdir -p packages

# Download Flutter bindings
VERSION="0.1.0"
curl -L -H "Authorization: token $GITHUB_TOKEN" \
  https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v${VERSION}/bs_calendar_core-flutter-${VERSION}.tar.gz \
  | tar xz -C packages/
```

### pubspec.yaml

```yaml
dependencies:
  flutter:
    sdk: flutter
  bs_calendar_core:
    path: ./packages/flutter
```

### Usage

```dart
import 'package:bs_calendar_core/bridge_generated.dart';

void main() async {
  // Use the library
  print('BS Calendar loaded!');
}
```

### CI/CD Integration

```yaml
# .github/workflows/flutter-build.yml
- name: Download Flutter bindings
  run: |
    VERSION="0.1.0"
    mkdir -p packages
    curl -L -H "Authorization: token ${{ secrets.GITHUB_TOKEN }}" \
      https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v${VERSION}/bs_calendar_core-flutter-${VERSION}.tar.gz \
      | tar xz -C packages/
    
- name: Get dependencies
  run: flutter pub get
```

---

## 🔧 Native FFI Integration

### C/C++

```c
#include "bs_calendar_core.h"

int main() {
    // Use the library
    return 0;
}
```

Compile:
```bash
# macOS
gcc main.c -L./native/macos-universal -lbs_calendar_core -o app

# Linux
gcc main.c -L./native/linux-x86_64 -lbs_calendar_core -o app

# Windows
gcc main.c -L./native/windows-x86_64 -lbs_calendar_core -o app.exe
```

### Python (ctypes)

```python
import ctypes
import platform

# Determine library path based on platform
if platform.system() == "Darwin":
    lib_path = "./native/macos-universal/libbs_calendar_core.dylib"
elif platform.system() == "Linux":
    lib_path = "./native/linux-x86_64/libbs_calendar_core.so"
elif platform.system() == "Windows":
    lib_path = "./native/windows-x86_64/bs_calendar_core.dll"

# Load library
lib = ctypes.CDLL(lib_path)

# Use the library
```

### Python Download Script

```python
# download_bindings.py
import os
import platform
import tarfile
import urllib.request

VERSION = "0.1.0"
GITHUB_TOKEN = os.environ.get("GITHUB_TOKEN")

# Determine platform
system = platform.system().lower()
machine = platform.machine().lower()

if system == "darwin":
    platform_name = "macos-universal"
elif system == "linux":
    platform_name = f"linux-{machine}"
elif system == "windows":
    platform_name = "windows-x86_64"

# Download URL
url = f"https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v{VERSION}/bs_calendar_core-native-{platform_name}-{VERSION}.tar.gz"

# Download with authentication
req = urllib.request.Request(url)
if GITHUB_TOKEN:
    req.add_header("Authorization", f"token {GITHUB_TOKEN}")

print(f"Downloading {platform_name} bindings...")
with urllib.request.urlopen(req) as response:
    with tarfile.open(fileobj=response, mode="r:gz") as tar:
        tar.extractall("./lib")

print("Download complete!")
```

### Go

```go
package main

/*
#cgo LDFLAGS: -L./native/macos-universal -lbs_calendar_core
#include "bs_calendar_core.h"
*/
import "C"

func main() {
    // Use the library
}
```

### Node.js (Native Addon)

```javascript
const ffi = require('ffi-napi');
const path = require('path');

const lib = ffi.Library(path.join(__dirname, 'native/macos-universal/libbs_calendar_core'), {
    // Define functions from header
});

// Use the library
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
  https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/bs_calendar_core-wasm-0.1.0.tar.gz \
  | tar xz
```

---

## 📌 Version Pinning

### Exact Version

```bash
VERSION="0.1.0"
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v${VERSION}/...
```

### Latest Release

```bash
# Get latest version
LATEST=$(curl -s https://api.github.com/repos/YOUR_USERNAME/bs_calendar_core/releases/latest | grep '"tag_name"' | cut -d'"' -f4)

# Download latest
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/${LATEST}/...
```

### Dev Builds

```bash
# Use pre-release versions
VERSION="0.1.0-dev.123"
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v${VERSION}/...
```

---

## 🛠️ Troubleshooting

### Authentication Errors

If you get 404 errors on a private repository:
1. Ensure `GITHUB_TOKEN` is set
2. Verify token has `repo` scope
3. Check token hasn't expired

### Platform Detection

For automated platform detection:

```bash
#!/bin/bash
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin) PLATFORM="macos-universal" ;;
    linux) PLATFORM="linux-${ARCH}" ;;
    mingw*|msys*) PLATFORM="windows-x86_64" ;;
esac

echo "Detected platform: $PLATFORM"
```

### Checksum Verification

```bash
# Download checksums
curl -L https://github.com/YOUR_USERNAME/bs_calendar_core/releases/download/v0.1.0/SHA256SUMS -o SHA256SUMS

# Verify
shasum -a 256 -c SHA256SUMS
```
