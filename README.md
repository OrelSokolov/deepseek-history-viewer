# DeepSeek Chat Viewer

Native Rust desktop application for viewing DeepSeek chat conversations with fast search, syntax highlighting, and beautiful UI.

## Features

- 🚀 **Pure Rust**: No JavaScript runtime needed
- ⚡ **Fast**: Native performance with compile-time optimizations
- 🔍 **Full-Text Search**: Powered by Tantivy search engine
- 💻 **Syntax Highlighting**: Code blocks with beautiful highlighting
- 📝 **Markdown Support**: Rich text rendering
- 🎨 **Modern UI**: Clean and responsive design

## Build from Source

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- System dependencies for webview (varies by platform)

### Building the Tauri Desktop App

```bash
# Quick start (recommended)
./run_desktop.sh

# Or manually:
# Terminal 1: Start web server
cargo run --bin deepseek-viewer

# Terminal 2: Run Tauri app
cd src-tauri
cargo run
```

### Building the Web Server Version

```bash
# Build in release mode
cargo build --release --bin deepseek-viewer

# Run the server
./target/release/deepseek-viewer
# Opens at http://localhost:8080
```

## Usage

1. **Prepare your data**: Export your DeepSeek conversations as `conversations.json` in the project root
2. **Generate site**: The app will automatically generate the HTML site on first run
3. **View conversations**: Navigate through your conversations with a beautiful UI
4. **Search**: Use the search bar to find specific conversations or messages

## Project Structure

```
├── src/                    # Rust source code
│   ├── generator.rs        # HTML generator
│   ├── server.rs           # Web server
│   ├── indexer.rs          # Search indexer
│   └── search.rs           # Search engine
├── src-tauri/              # Tauri desktop app
│   ├── Cargo.toml          # Tauri dependencies
│   └── src/                # Tauri app source
├── templates/              # HTML templates
├── static/                 # Static assets
└── dist/                   # Generated HTML output

```

## Technology Stack

- **Backend**: Rust with Axum web framework
- **Desktop**: Tauri 2.0
- **Search**: Tantivy full-text search engine
- **Highlighting**: Syntect syntax highlighter
- **Templating**: Askama compile-time templates
- **Frontend**: Pure HTML/CSS with vanilla JavaScript

## License

MIT
