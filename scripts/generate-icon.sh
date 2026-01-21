#!/usr/bin/env bash
# Generate macOS app icon from SVG
# Requires: macOS with sips, iconutil, and either rsvg-convert or qlmanage

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ASSETS_DIR="$PROJECT_ROOT/communitas-dioxus/assets"
SVG_FILE="$ASSETS_DIR/icon.svg"
ICONSET_DIR="$ASSETS_DIR/icon.iconset"

# Clean up any existing iconset
rm -rf "$ICONSET_DIR"
mkdir -p "$ICONSET_DIR"

echo "Generating macOS app icon from SVG..."

# Function to convert SVG to PNG at a specific size
convert_svg_to_png() {
    local size=$1
    local output=$2

    # Try rsvg-convert first (most reliable)
    if command -v rsvg-convert &> /dev/null; then
        rsvg-convert -w "$size" -h "$size" "$SVG_FILE" -o "$output"
        return 0
    fi

    # Try qlmanage (macOS built-in, but less precise)
    if command -v qlmanage &> /dev/null; then
        # qlmanage creates a thumbnail, we'll resize it
        local temp_dir=$(mktemp -d)
        qlmanage -t -s "$size" -o "$temp_dir" "$SVG_FILE" 2>/dev/null || true
        local thumb_file=$(find "$temp_dir" -name "*.png" 2>/dev/null | head -1)
        if [[ -n "$thumb_file" && -f "$thumb_file" ]]; then
            cp "$thumb_file" "$output"
            sips -z "$size" "$size" "$output" >/dev/null 2>&1
            rm -rf "$temp_dir"
            return 0
        fi
        rm -rf "$temp_dir"
    fi

    # Fallback: try convert from ImageMagick
    if command -v convert &> /dev/null; then
        convert -background none -resize "${size}x${size}" "$SVG_FILE" "$output"
        return 0
    fi

    echo "ERROR: No SVG converter found. Install one of:"
    echo "  - librsvg (brew install librsvg)"
    echo "  - ImageMagick (brew install imagemagick)"
    exit 1
}

# Generate all required icon sizes
# macOS iconset requires specific sizes with @2x variants
SIZES=(16 32 64 128 256 512)

for size in "${SIZES[@]}"; do
    echo "  Generating ${size}x${size}..."
    convert_svg_to_png "$size" "$ICONSET_DIR/icon_${size}x${size}.png"

    # Generate @2x variant (double the size)
    size_2x=$((size * 2))
    echo "  Generating ${size}x${size}@2x (${size_2x}x${size_2x})..."
    convert_svg_to_png "$size_2x" "$ICONSET_DIR/icon_${size}x${size}@2x.png"
done

# Generate 512@2x (1024) explicitly
echo "  Generating 512x512@2x (1024x1024)..."
convert_svg_to_png 1024 "$ICONSET_DIR/icon_512x512@2x.png"

# Copy the 1024 version as icon.png for other uses
cp "$ICONSET_DIR/icon_512x512@2x.png" "$ASSETS_DIR/icon.png"
echo "  Created icon.png (1024x1024)"

# Convert iconset to icns
echo "Creating icon.icns..."
iconutil -c icns "$ICONSET_DIR" -o "$ASSETS_DIR/icon.icns"

# Clean up iconset directory
rm -rf "$ICONSET_DIR"

echo ""
echo "Icon generation complete!"
echo "  - $ASSETS_DIR/icon.png (1024x1024)"
echo "  - $ASSETS_DIR/icon.icns (all sizes)"
