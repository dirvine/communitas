#!/bin/bash
# Build script for Communitas Swift XCFramework
# This creates a universal XCFramework for macOS and iOS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
FRAMEWORKS_DIR="$SCRIPT_DIR/Frameworks"
XCFRAMEWORK_DIR="$FRAMEWORKS_DIR/CommunitasBindings.xcframework"

echo "=== Building Communitas Swift XCFramework ==="

# Set deployment targets
export MACOSX_DEPLOYMENT_TARGET=14.0
export IPHONEOS_DEPLOYMENT_TARGET=17.0

cd "$PROJECT_ROOT"

echo ""
echo "Step 1: Building Rust library for all targets..."

echo "  - macOS arm64..."
cargo build --release -p communitas-bindings --target aarch64-apple-darwin

echo "  - iOS device (arm64)..."
cargo build --release -p communitas-bindings --target aarch64-apple-ios

echo "  - iOS simulator (arm64)..."
cargo build --release -p communitas-bindings --target aarch64-apple-ios-sim

echo ""
echo "Step 2: Generating Swift bindings..."
cargo run --release -p communitas-bindings --bin uniffi-bindgen --features="uniffi/cli" -- \
    generate \
    --library target/aarch64-apple-darwin/release/libcommunitas_bindings.dylib \
    --language swift \
    --out-dir "$SCRIPT_DIR/CommunitasKit/Sources/CommunitasKit"

echo ""
echo "Step 3: Updating XCFramework libraries..."

# Update libraries in XCFramework
cp target/aarch64-apple-darwin/release/libcommunitas_bindings.a \
   "$XCFRAMEWORK_DIR/macos-arm64/"

cp target/aarch64-apple-ios/release/libcommunitas_bindings.a \
   "$XCFRAMEWORK_DIR/ios-arm64/"

cp target/aarch64-apple-ios-sim/release/libcommunitas_bindings.a \
   "$XCFRAMEWORK_DIR/ios-arm64-simulator/"

# Update headers (using the generated C header)
cp "$SCRIPT_DIR/CommunitasKit/Sources/CommunitasKit/communitas_bindingsFFI.h" \
   "$XCFRAMEWORK_DIR/macos-arm64/Headers/" 2>/dev/null || true
cp "$SCRIPT_DIR/CommunitasKit/Sources/CommunitasKit/communitas_bindingsFFI.h" \
   "$XCFRAMEWORK_DIR/ios-arm64/Headers/" 2>/dev/null || true
cp "$SCRIPT_DIR/CommunitasKit/Sources/CommunitasKit/communitas_bindingsFFI.h" \
   "$XCFRAMEWORK_DIR/ios-arm64-simulator/Headers/" 2>/dev/null || true

echo ""
echo "Step 4: Building Swift app..."
cd "$SCRIPT_DIR/CommunitasApp"
swift build

echo ""
echo "=== Build Complete ==="
echo "XCFramework: $XCFRAMEWORK_DIR"
echo "Swift App: $SCRIPT_DIR/CommunitasApp/.build/arm64-apple-macosx/debug/CommunitasApp"
