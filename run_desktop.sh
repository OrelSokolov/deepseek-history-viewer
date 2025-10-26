#!/bin/bash

# Quick start script for DeepSeek Desktop Viewer

echo "🚀 Starting DeepSeek Desktop Viewer..."

# Check if conversations.json exists
if [ ! -f "conversations.json" ]; then
    echo "❌ Error: conversations.json not found"
    echo "Please export your conversations first"
    exit 1
fi

# Step 1: Start the web server in background
echo "🌐 Starting web server on http://localhost:8080..."
cargo run --bin deepseek-viewer &
SERVER_PID=$!

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 3

# Step 2: Run Tauri app
echo "🖥️  Starting Tauri desktop app..."
cd src-tauri && cargo run

# Cleanup: stop server when app exits
echo "🛑 Stopping web server..."
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "✅ Done"

