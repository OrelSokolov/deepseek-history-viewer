#!/bin/bash

# Create simple placeholder icons
mkdir -p src-tauri/icons

# Create a simple SVG icon
cat > src-tauri/icons/icon.svg << 'EOF'
<svg xmlns="http://www.w3.org/2000/svg" width="512" height="512" viewBox="0 0 512 512">
  <rect width="512" height="512" fill="#2563eb"/>
  <text x="256" y="350" font-family="Arial" font-size="300" font-weight="bold" fill="white" text-anchor="middle">DS</text>
</svg>
EOF

echo "Icon created: src-tauri/icons/icon.svg"
echo "For production, replace these with proper icon files"

