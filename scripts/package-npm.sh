#!/usr/bin/env bash
set -euo pipefail

# Assembles the npm package from the output of scripts/build-wasm.sh.
#
#   ./scripts/build-wasm.sh          # writes dist/wasm/{web,bundler,nodejs}
#   ./scripts/package-npm.sh         # writes dist/npm/, ready for `npm publish`
#
# One package serves every JS environment through conditional exports:
#   import { gregorian_to_bs } from '@yorion/engine'       → Node (CJS build) or bundler build
#   import init, { ... } from '@yorion/engine/web'         → plain browser ES modules, no bundler
#
# The version is read from engine/Cargo.toml so npm, crates.io and PyPI always
# ship the same number. Override with NPM_PACKAGE_NAME to publish under another name.

cd "$(dirname "$0")/.."

PKG_NAME="${NPM_PACKAGE_NAME:-@yorion/engine}"
SRC="dist/wasm"
OUT="dist/npm"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' engine/Cargo.toml | head -1)"

for target in web bundler nodejs; do
  if [[ ! -f "$SRC/$target/yorion_engine.js" ]]; then
    echo "❌ $SRC/$target is missing. Run scripts/build-wasm.sh first." >&2
    exit 1
  fi
done

rm -rf "$OUT"
mkdir -p "$OUT"
cp -R "$SRC/web" "$SRC/bundler" "$SRC/nodejs" "$OUT/"
cp npm/README.md "$OUT/README.md"
cp LICENSE-MIT LICENSE-Apache-2.0 "$OUT/"

# The nodejs target is CommonJS. Mark that directory as such so it still loads
# if a consumer's tooling treats the package as ESM.
echo '{ "type": "commonjs" }' > "$OUT/nodejs/package.json"

PKG_NAME="$PKG_NAME" VERSION="$VERSION" node - "$OUT/package.json" <<'NODE'
const fs = require('fs');
const out = process.argv[2];
const pkg = {
  name: process.env.PKG_NAME,
  version: process.env.VERSION,
  description:
    'Nepali (Bikram Sambat) calendar engine: BS/AD conversion, panchanga (tithi, nakshatra, yoga, karana) and festival recurrence. Compiled from Rust to WebAssembly.',
  license: 'MIT OR Apache-2.0',
  author: 'Umesh Shrestha <umesh@yorion.io>',
  homepage: 'https://github.com/Yorion-io/yorion_engine',
  repository: { type: 'git', url: 'git+https://github.com/Yorion-io/yorion_engine.git' },
  bugs: { url: 'https://github.com/Yorion-io/yorion_engine/issues' },
  keywords: [
    'bikram-sambat', 'nepali-calendar', 'nepali-date', 'bs-to-ad', 'panchanga',
    'panchang', 'tithi', 'nakshatra', 'nepal', 'calendar', 'wasm',
  ],
  main: './nodejs/yorion_engine.js',
  module: './bundler/yorion_engine.js',
  types: './bundler/yorion_engine.d.ts',
  exports: {
    '.': {
      types: './bundler/yorion_engine.d.ts',
      node: './nodejs/yorion_engine.js',
      default: './bundler/yorion_engine.js',
    },
    './web': {
      types: './web/yorion_engine.d.ts',
      default: './web/yorion_engine.js',
    },
    './web/yorion_engine_bg.wasm': './web/yorion_engine_bg.wasm',
    './package.json': './package.json',
  },
  // The bundler build instantiates the wasm module on import.
  sideEffects: ['./bundler/yorion_engine.js', './bundler/snippets/*'],
  files: ['web/', 'bundler/', 'nodejs/', 'README.md', 'LICENSE-MIT', 'LICENSE-Apache-2.0'],
  publishConfig: { access: 'public' },
};
fs.writeFileSync(out, JSON.stringify(pkg, null, 2) + '\n');
NODE

echo "✅ npm package for $PKG_NAME@$VERSION assembled in $OUT"
echo "   Inspect with: (cd $OUT && npm pack --dry-run)"
