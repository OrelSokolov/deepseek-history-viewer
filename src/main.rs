use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod generator;
mod server;
mod templates;

// Use from lib
use deepseek_app::{indexer, search};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepseek_viewer=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 DeepSeek Chat Viewer - Pure Rust Edition");

    let conversations_path = "conversations.json";
    // Use user-local data directory to avoid permission issues
    let base_data_dir: PathBuf = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("deepseek-viewer");
    let output_dir = base_data_dir.join("dist");
    let index_path = base_data_dir.join("search_index");

    // Determine source for conversations: real file or an empty fallback
    let conversations_source: String = if std::path::Path::new(conversations_path).exists() {
        conversations_path.to_string()
    } else {
        tracing::warn!(
            "⚠️  conversations.json not found. Using an empty conversations file so the app can start."
        );
        let empty_path = base_data_dir.join("empty_conversations.json");
        if !empty_path.exists() {
            std::fs::create_dir_all(&base_data_dir)?;
            std::fs::write(&empty_path, "[]")?;
        }
        empty_path.to_string_lossy().to_string()
    };

    // Step 1: Generate HTML site
    let index_file = output_dir.join("index.html");
    if !index_file.exists() {
        tracing::info!("📦 Generating HTML site in {}...", output_dir.display());
        std::fs::create_dir_all(&output_dir)?;
        generator::generate_site(&conversations_source, output_dir.to_str().unwrap()).await?;
        tracing::info!("✅ HTML site generated in {}/", output_dir.display());
    } else {
        tracing::info!("✅ Using existing HTML site in {}/", output_dir.display());
    }

    // Step 2: Build search index
    if !index_path.exists() {
        tracing::info!("📚 Building search index in {}...", index_path.display());
        std::fs::create_dir_all(&index_path)?;
        indexer::build_index(&conversations_source, index_path.to_str().unwrap()).await?;
        tracing::info!("✅ Search index built");
    } else {
        tracing::info!("✅ Using existing search index");
    }

    // Step 3: Start server
    let search_engine = search::SearchEngine::new(index_path.to_str().unwrap())?;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    
    tracing::info!("🌐 Starting web server on http://{}", addr);
    tracing::info!("📁 Serving files from {}/", output_dir.display());
    tracing::info!("");
    tracing::info!("✨ Ready! Open http://127.0.0.1:8080 in your browser");
    tracing::info!("🔍 Search API: http://127.0.0.1:8080/api/search?q=<query>");
    tracing::info!("");
    tracing::info!("Press Ctrl+C to stop");
    
    server::serve(addr, search_engine, output_dir.to_str().unwrap()).await?;

    Ok(())
}
