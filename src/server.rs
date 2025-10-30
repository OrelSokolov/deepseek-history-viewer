use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::ServeDir,
};

use deepseek_app::search::{SearchEngine, SearchResult};

#[derive(Clone)]
struct AppState {
    search_engine: Arc<SearchEngine>,
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    20
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    results: Vec<SearchResult>,
    total: usize,
    time_ms: u128,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct ConversationMeta {
    id: String,
    title: String,
    url: String,
}

pub async fn serve(addr: SocketAddr, search_engine: SearchEngine, output_dir: &str) -> anyhow::Result<()> {
    let state = AppState {
        search_engine: Arc::new(search_engine),
    };

    // Build router
    let app = Router::new()
        // API routes
        .route("/api/health", get(health_handler))
        .route("/api/search", get(search_handler))
        .route("/api/conversations", get(conversations_handler))
        // Import pages
        .route("/import", get(import_page_handler))
        .route("/import/process", get(processing_page_handler))
        // Serve static files from generated dist directory
        .nest_service(
            "/",
            ServeDir::new(output_dir)
                .append_index_html_on_directories(true),
        )
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Run server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("🚀 Server listening on http://{}", addr);
    tracing::info!("📁 Serving static files from {}/", output_dir);
    tracing::info!("🔍 Search API available at http://{}/api/search?q=<query>", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn conversations_handler() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    use std::fs;
    
    let conversations_dir = "dist/conversations";
    let mut conversations = Vec::new();
    
    if let Ok(entries) = fs::read_dir(conversations_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                
                if path.is_dir() {
                    let index_path = path.join("index.html");
                    if index_path.exists() {
                        if let Ok(html) = fs::read_to_string(&index_path) {
                            let title = extract_title_from_html(&html);
                            let conversation_id = path.file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            
                            conversations.push(serde_json::json!({
                                "id": conversation_id,
                                "title": title,
                                "url": format!("/conversations/{}/", conversation_id)
                            }));
                        }
                    }
                }
            }
        }
    }
    
    Ok(Json(conversations))
}

fn extract_title_from_html(html: &str) -> String {
    // Try to find <title> tag
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start..].find("</title>") {
            let title = &html[start + 7..start + end];
            return title.trim().to_string();
        }
    }
    
    // Try to find h1
    if let Some(start) = html.find("<h1") {
        if let Some(end) = html[start..].find("</h1>") {
            if let Some(text_start) = html[start..].find('>') {
                let title = &html[start + text_start + 1..start + end];
                return title.trim().to_string();
            }
        }
    }
    
    "Untitled conversation".to_string()
}

async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, StatusCode> {
    let start = std::time::Instant::now();

    let results = state
        .search_engine
        .search(&params.q, params.limit)
        .map_err(|e| {
            tracing::error!("Search error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let total = results.len();
    let time_ms = start.elapsed().as_millis();

    tracing::info!(
        "Search query='{}' returned {} results in {}ms",
        params.q,
        total,
        time_ms
    );

    Ok(Json(SearchResponse {
        query: params.q.clone(),
        results,
        total,
        time_ms,
    }))
}

async fn import_page_handler() -> impl IntoResponse {
    let html = include_str!("../templates/import.html");
    axum::response::Html(html)
}

async fn processing_page_handler() -> impl IntoResponse {
    let html = include_str!("../templates/processing.html");
    axum::response::Html(html)
}

