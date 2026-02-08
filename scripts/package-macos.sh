#!/usr/bin/env bash
set -euo pipefail

echo "🔗 Creating Universal Binary & XCFramework..."

# Expected inputs (should be downloaded/placed by CI):
# - dist/x86_64/libbs_calendar_core.a
# - dist/aarch64/libbs_calendar_core.a
# - dist/swift/bs_calendar_core.swift
# - dist/swift/bs_calendar_coreFFI.modulemap (optional)
# - dist/swift/bs_calendar_coreFFI.h (optional)

# Create output dir
mkdir -p engine/target/universal-apple-darwin/release

echo "Merging binaries..."
lipo -create \
    dist/x86_64/libbs_calendar_core.a \
    dist/aarch64/libbs_calendar_core.a \
    -output engine/target/universal-apple-darwin/release/libbs_calendar_core.a

# Create XCFramework structure
rm -rf dist/BsCalendarCore.xcframework
mkdir -p dist/BsCalendarCore.xcframework/macos-arm64_x86_64/Headers

# Copy universal library
cp engine/target/universal-apple-darwin/release/libbs_calendar_core.a \
   dist/BsCalendarCore.xcframework/macos-arm64_x86_64/

# Copy module map if it exists
if [ -f "dist/swift/bs_calendar_coreFFI.modulemap" ]; then
    cp dist/swift/bs_calendar_coreFFI.modulemap \
       dist/BsCalendarCore.xcframework/macos-arm64_x86_64/module.modulemap
fi

# Copy C headers
if [ -f "dist/swift/bs_calendar_coreFFI.h" ]; then
    cp dist/swift/bs_calendar_coreFFI.h \
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

echo "✅ XCFramework created at dist/BsCalendarCore.xcframework"
