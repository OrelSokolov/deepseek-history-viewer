#!/bin/bash
echo "🔄 Регенерируем HTML с единой версткой..."
rm -rf dist/conversations dist/index.html
cargo build --bin deepseek-viewer 2>&1 | grep -E "(Finished|error)" | tail -3
target/debug/deepseek-viewer &
SERVER_PID=$!
sleep 3
kill $SERVER_PID 2>/dev/null
echo "✅ Регенерация завершена!"
echo "Теперь запусти: cargo run --bin deepseek-viewer"
