#!/usr/bin/env bash
set -euo pipefail

echo "📦 Generating Swift bindings and XCFramework..."

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if uniffi-bindgen is installed
if ! command -v uniffi-bindgen &> /dev/null; then
    echo -e "${YELLOW}⚠️  uniffi-bindgen not found, installing...${NC}"
    cargo install uniffi-bindgen
fi

# Ensure universal binary exists
if [ ! -f "target/universal-apple-darwin/release/libbs_calendar_core.a" ]; then
    echo -e "${YELLOW}⚠️  Universal binary not found. Building it first...${NC}"
    ./scripts/build-macos.sh
fi

# Generate Swift bindings
echo -e "${BLUE}🔧 Generating Swift bindings from UDL...${NC}"
cd engine
uniffi-bindgen generate src/uniffi.udl --language swift --out-dir ../dist/swift
cd ..

echo -e "${GREEN}✅ Swift bindings generated at dist/swift/${NC}"

# Create XCFramework structure
echo -e "${BLUE}🏗️  Creating XCFramework...${NC}"
rm -rf dist/BsCalendarCore.xcframework
mkdir -p dist/BsCalendarCore.xcframework/macos-arm64_x86_64

# Copy universal library
cp target/universal-apple-darwin/release/libbs_calendar_core.a \
   dist/BsCalendarCore.xcframework/macos-arm64_x86_64/

# Copy Swift bindings
cp dist/swift/bs_calendar_core.swift \
   dist/BsCalendarCore.xcframework/

# Copy module map if it exists
if [ -f "dist/swift/bs_calendar_coreFFI.modulemap" ]; then
    cp dist/swift/bs_calendar_coreFFI.modulemap \
       dist/BsCalendarCore.xcframework/macos-arm64_x86_64/module.modulemap
fi

# Copy C headers
if [ -f "dist/swift/bs_calendar_coreFFI.h" ]; then
    mkdir -p dist/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers
    cp dist/swift/bs_calendar_coreFFI.h \
       dist/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers/
fi

# Create Info.plist for XCFramework
cat > dist/BsCalendarCore.xcframework/Info.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>AvailableLibraries</key>
    <array>
        <dict>
            <key>LibraryIdentifier</key>
            <string>macos-arm64_x86_64</string>
            <key>LibraryPath</key>
            <string>libbs_calendar_core.a</string>
            <key>SupportedArchitectures</key>
            <array>
                <string>arm64</string>
                <string>x86_64</string>
            </array>
            <key>SupportedPlatform</key>
            <string>macos</string>
        </dict>
    </array>
    <key>CFBundlePackageType</key>
    <string>XFWK</string>
    <key>XCFrameworkFormatVersion</key>
    <string>1.0</string>
</dict>
</plist>
EOF

echo -e "${GREEN}✅ XCFramework created at dist/BsCalendarCore.xcframework${NC}"
echo "📊 Contents:"
ls -lh dist/BsCalendarCore.xcframework/
echo ""
echo "📄 Swift bindings:"
ls -lh dist/swift/
