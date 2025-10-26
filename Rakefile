# frozen_string_literal: true

desc "Build the project in release mode"
task :build do
  puts "🔨 Building Rust project (release mode)..."
  sh "cargo build --release"
  puts "✅ Build complete!"
end

desc "Run the application (rebuilds if needed)"
task :run do
  puts "🚀 Running DeepSeek Chat Viewer..."
  sh "cargo run --release --bin deepseek-viewer"
end

desc "Run tests"
task :test do
  puts "🧪 Running tests..."
  sh "cargo test --release -- --nocapture"
  puts "✅ All tests passed!"
end

desc "Clean build artifacts"
task :clean do
  puts "🧹 Cleaning build artifacts..."
  sh "cargo clean"
  sh "rm -rf dist data"
  puts "✅ Clean complete!"
end

desc "Rebuild index (deletes old index)"
task :reindex do
  puts "🔄 Rebuilding search index..."
  sh "rm -rf data"
  sh "cargo run --release"
end

desc "Update CSS without full rebuild"
task :css do
  puts "🎨 Updating CSS..."
  sh "cp static/main.css dist/assets/css/main.css"
  puts "✅ CSS updated! Refresh browser (Ctrl+Shift+R)"
end

desc "Update JavaScript without full rebuild"
task :js do
  puts "⚡ Updating JavaScript..."
  sh "cp static/search.js dist/assets/js/search.js"
  puts "✅ JS updated! Refresh browser (Ctrl+Shift+R)"
end

desc "Update both CSS and JS"
task :assets => [:css, :js]

desc "Full rebuild (clean + build + run)"
task :rebuild => [:clean, :build, :run]

desc "Run tests and then start server"
task :test_and_run => [:test, :run]

task :default => :run

