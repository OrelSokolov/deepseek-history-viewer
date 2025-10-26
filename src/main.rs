use anyhow::Result;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod generator;
mod server;
mod templates;

// Use from lib
use deepseek_viewer::{indexer, search};

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
    let output_dir = "dist";
    let index_path = "data/search_index";

    // Step 1: Generate HTML site
    let index_file = std::path::Path::new(output_dir).join("index.html");
    if !index_file.exists() {
        tracing::info!("📦 Generating HTML site...");
        generator::generate_site(conversations_path, output_dir).await?;
        tracing::info!("✅ HTML site generated in {}/", output_dir);
    } else {
        tracing::info!("✅ Using existing HTML site in {}/", output_dir);
    }

    // Step 2: Build search index
    if !std::path::Path::new(index_path).exists() {
        tracing::info!("📚 Building search index...");
        indexer::build_index(conversations_path, index_path).await?;
        tracing::info!("✅ Search index built");
    } else {
        tracing::info!("✅ Using existing search index");
    }

    // Step 3: Start server
    let search_engine = search::SearchEngine::new(index_path)?;
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    
    tracing::info!("🌐 Starting web server on http://{}", addr);
    tracing::info!("📁 Serving files from {}/", output_dir);
    tracing::info!("");
    tracing::info!("✨ Ready! Open http://localhost:8080 in your browser");
    tracing::info!("🔍 Search API: http://localhost:8080/api/search?q=<query>");
    tracing::info!("");
    tracing::info!("Press Ctrl+C to stop");
    
    server::serve(addr, search_engine, output_dir).await?;

    Ok(())
}
