#!/bin/bash
set -e

DEB_FILE=$(find target/release/bundle/deb -name "*.deb" -type f -name "DeepSeek*" | head -1)
if [ -z "$DEB_FILE" ]; then
    echo "No deb file found"
    exit 1
fi

echo "Fixing $DEB_FILE"

# Extract the deb
PROJECT_DIR=$(pwd)
mkdir -p /tmp/fix-deb
cd /tmp/fix-deb
rm -rf *
ar x "$PROJECT_DIR/$DEB_FILE"
mkdir -p control data

# Extract control - check for both .xz and .gz
if [ -f control.tar.xz ]; then
    tar -xJf control.tar.xz -C control
    tar -xJf data.tar.xz -C data
elif [ -f control.tar.gz ]; then
    tar -xzf control.tar.gz -C control
    tar -xzf data.tar.gz -C data
else
    ls -la
    exit 1
fi

# Fix the desktop files - replace all references to deepseek-viewer with deepseek-desktop
find data/usr/share/applications/ -name "*.desktop" -exec sed -i 's/Exec=deepseek-viewer/Exec=deepseek-desktop/g' {} \;
find data/usr/share/applications/ -name "*.desktop" -exec sed -i 's/StartupWMClass=deepseek-viewer/StartupWMClass=deepseek-desktop/g' {} \;
# Keep Icon as deepseek-viewer since that's what Tauri creates
find data/usr/share/applications/ -name "*.desktop" -exec sed -i 's/Icon=deepseek-desktop/Icon=deepseek-viewer/g' {} \;

# Remove duplicate desktop file created by Tauri, keep only deepseek-desktop.desktop
# Find all desktop files and keep the one with deepseek-desktop
DESKTOP_COUNT=$(find data/usr/share/applications/ -name "*.desktop" | wc -l)
if [ "$DESKTOP_COUNT" -gt 1 ]; then
    # Keep deepseek-desktop.desktop, remove others
    find data/usr/share/applications/ -name "*.desktop" ! -name "deepseek-desktop.desktop" -delete
fi

# Create new package
cd data
tar -czf ../data.tar.gz usr/
cd ../control
tar -czf ../control.tar.gz ./

cd ..
# Try both formats
if [ -f control.tar.gz ]; then
    ar rcs "$PROJECT_DIR/$DEB_FILE" debian-binary control.tar.gz data.tar.gz
else
    ar rcs "$PROJECT_DIR/$DEB_FILE" debian-binary control.tar.xz data.tar.xz
fi

echo "Fixed!"
ls -lh "$PROJECT_DIR/$DEB_FILE"

