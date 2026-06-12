#!/usr/bin/env bash
set -euo pipefail

echo "🔗 Creating Universal Binary & XCFramework..."

# Expected inputs (should be downloaded/placed by CI):
# - dist/x86_64/libyorion_engine.a
# - dist/aarch64/libyorion_engine.a
# - dist/swift/yorion_engine.swift
# - dist/swift/yorion_engineFFI.modulemap (optional)
# - dist/swift/yorion_engineFFI.h (optional)

# Create output dir
mkdir -p engine/target/universal-apple-darwin/release

echo "Merging binaries..."
lipo -create \
    dist/x86_64/libyorion_engine.a \
    dist/aarch64/libyorion_engine.a \
    -output engine/target/universal-apple-darwin/release/libyorion_engine.a

# Create XCFramework structure
rm -rf dist/BsCalendarCore.xcframework
mkdir -p dist/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers

# Copy universal library
cp engine/target/universal-apple-darwin/release/libyorion_engine.a \
   dist/BsCalendarCore.xcframework/macos-arm64_x86_64/

# Copy module map if it exists
if [ -f "dist/swift/yorion_engineFFI.modulemap" ]; then
    cp dist/swift/yorion_engineFFI.modulemap \
       dist/BsCalendarCore.xcframework/macos-arm64_x86_64/module.modulemap
fi

# Copy C headers
if [ -f "dist/swift/yorion_engineFFI.h" ]; then
    cp dist/swift/yorion_engineFFI.h \
       dist/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers/
fi

# Create Info.plist
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
            <string>libyorion_engine.a</string>
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

echo "✅ XCFramework created at dist/BsCalendarCore.xcframework"
