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

namespace :build do
  namespace :windows do
    desc "Build Windows MSI bundle"
    task :msi do
      puts "🔨 Building Windows MSI bundle..."
      sh "cargo tauri build --bundles msi -- --bin deepseek-desktop"
      puts "✅ MSI built successfully!"
      puts "📦 Output: target/release/bundle/msi/*.msi"
    end

    desc "Build Windows NSIS bundle"
    task :nsis do
      puts "🔨 Building Windows NSIS bundle..."
      sh "cargo tauri build --bundles nsis -- --bin deepseek-desktop"
      puts "✅ NSIS built successfully!"
      puts "📦 Output: target/release/bundle/nsis/*.exe"
    end

    desc "Build all Windows bundles (MSI + NSIS)"
    task :all => [:msi, :nsis] do
      puts "✅ All Windows bundles built successfully!"
    end
  end

  namespace :mac do
    desc "Build macOS DMG bundle"
    task :dmg do
      puts "🔨 Building macOS DMG bundle..."
      sh "cargo tauri build --bundles dmg"
      puts "✅ DMG built successfully!"
      puts "📦 Output: target/release/bundle/dmg/*.dmg"
    end

    desc "Build all macOS bundles (DMG)"
    task :all => [:dmg] do
      puts "✅ All macOS bundles built successfully!"
    end
  end

  namespace :linux do
    desc "Build Linux DEB bundle"
    task :deb do
      puts "🔨 Building Linux DEB bundle..."
      sh "cargo tauri build --bundles deb"
      puts "✅ DEB built successfully!"
      puts "📦 Output: target/release/bundle/deb/*.deb"
    end

    desc "Build Linux RPM bundle"
    task :rpm do
      puts "🔨 Building Linux RPM bundle..."
      sh "cargo tauri build --bundles rpm"
      puts "✅ RPM built successfully!"
      puts "📦 Output: target/release/bundle/rpm/*.rpm"
    end

    desc "Build all Linux bundles (DEB + RPM)"
    task :all => [:deb, :rpm] do
      puts "✅ All Linux bundles built successfully!"
    end
  end
end

task :default => :run

