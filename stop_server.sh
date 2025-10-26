#!/bin/bash

# Stop any existing deepseek-viewer processes
echo "🛑 Stopping any existing deepseek-viewer servers..."

# Find and kill processes
pkill -f "deepseek-viewer" && echo "✅ Server stopped"
sleep 1

# Double check
if lsof -i :8080 > /dev/null 2>&1; then
    echo "⚠️  Port 8080 still in use"
    lsof -i :8080
else
    echo "✅ Port 8080 is free"
fi

