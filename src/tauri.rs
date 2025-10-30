#![windows_subsystem = "windows"]

use anyhow::Result;
use tauri::{generate_handler, Emitter, Manager, State, Window};
use tracing_subscriber::prelude::*;
use std::sync::{Arc, Mutex};

mod config;
mod generator;
mod server;
mod templates;

use config::AppConfig;
use deepseek_app::indexer;
use deepseek_app::search::SearchEngine;
use std::path::PathBuf;

pub struct AppState {
    pub index_path: String,
    pub output_dir: String,
    pub config: Arc<Mutex<AppConfig>>,
}

// Tauri command to check if we have conversations
#[tauri::command]
async fn has_conversations(state: State<'_, AppState>) -> Result<bool, String> {
    let config = state.config.lock().unwrap();
    
    if let Some(path) = &config.conversations_file_path {
        Ok(std::path::Path::new(path).exists())
    } else {
        Ok(false)
    }
}

// Tauri command to get current file path
#[tauri::command]
async fn get_current_file_path(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let config = state.config.lock().unwrap();
    Ok(config.conversations_file_path.clone())
}

// Tauri command to process conversations file
#[tauri::command]
async fn process_conversations_file(
    file_path: String,
    window: Window,
    state: State<'_, AppState>,
) -> Result<(), String> {
    tracing::info!("📦 Processing conversations file: {}", file_path);
    
    // Emit progress event
    tracing::info!("Emitting progress: 0%");
    let emit_result = window.emit_to("main", "import-progress", serde_json::json!({
        "percent": 0,
        "message": "Reading file..."
    }));
    tracing::info!("Emit result: {:?}", emit_result);
    
    // Verify file exists
    tracing::info!("Checking if file exists: {}", file_path);
    if !std::path::Path::new(&file_path).exists() {
        tracing::error!("File not found: {}", file_path);
        return Err(format!("File not found: {}", file_path));
    }
    tracing::info!("File exists");
    
    // Verify file is valid JSON
    tracing::info!("Reading file content...");
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| {
            tracing::error!("Failed to read file: {}", e);
            format!("Failed to read file: {}", e)
        })?;
    
    tracing::info!("Parsing JSON...");
    let _: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| {
            tracing::error!("Invalid JSON format: {}", e);
            format!("Invalid JSON format: {}", e)
        })?;
    
    tracing::info!("Emitting progress: 20%");
    let _ = window.emit_to("main", "import-progress", serde_json::json!({
        "percent": 20,
        "message": "File validated successfully"
    }));
    
    // Save file path to config
    {
        let mut config = state.config.lock().unwrap();
        config.conversations_file_path = Some(file_path.clone());
        config.save().map_err(|e| format!("Failed to save config: {}", e))?;
    }
    
    let _ = window.emit_to("main", "import-progress", serde_json::json!({
        "percent": 30,
        "message": "Generating HTML site..."
    }));
    
    // Clean up old dist directory
    if std::path::Path::new(&state.output_dir).exists() {
        std::fs::remove_dir_all(&state.output_dir)
            .map_err(|e| format!("Failed to clean output directory: {}", e))?;
    }
    
    // Generate HTML site
    generator::generate_site(&file_path, &state.output_dir)
        .await
        .map_err(|e| format!("Failed to generate site: {}", e))?;
    
    let _ = window.emit_to("main", "import-progress", serde_json::json!({
        "percent": 70,
        "message": "Building search index..."
    }));
    
    // Clean up old index
    if std::path::Path::new(&state.index_path).exists() {
        std::fs::remove_dir_all(&state.index_path)
            .map_err(|e| format!("Failed to clean index directory: {}", e))?;
    }
    
    // Build search index
    indexer::build_index(&file_path, &state.index_path)
        .await
        .map_err(|e| format!("Failed to build index: {}", e))?;
    
    let _ = window.emit_to("main", "import-progress", serde_json::json!({
        "percent": 100,
        "message": "Processing complete!"
    }));
    
    tracing::info!("✅ Processing complete");
    
    Ok(())
}

// Tauri command for search
#[tauri::command]
async fn search(query: String, state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    tracing::info!("🔍 Searching for: {}", query);
    
    let search_engine = SearchEngine::new(&state.index_path)
        .map_err(|e| format!("Failed to create search engine: {}", e))?;
    
    let results = search_engine
        .search(&query, 10)
        .map_err(|e| format!("Search failed: {}", e))?;
    
    // Convert SearchResult to JSON
    let json_results: Vec<serde_json::Value> = results
        .into_iter()
        .map(|r| serde_json::json!({
            "conversation_id": r.conversation_id,
            "title": r.title,
            "date": r.date,
            "score": r.score,
            "snippet": r.snippet,
        }))
        .collect();
    
    Ok(json_results)
}

// Tauri command to get conversation list
#[tauri::command]
async fn get_conversations(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let config = state.config.lock().unwrap();
    
    let path = config.conversations_file_path.as_ref()
        .ok_or_else(|| "No conversations file configured".to_string())?;
    
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read conversations: {}", e))?;
    
    let conversations: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse conversations: {}", e))?;
    
    Ok(conversations)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deepseek_viewer=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🚀 DeepSeek Chat Viewer - Desktop Edition");

    // Load config
    let config = AppConfig::load().unwrap_or_default();
    let config = Arc::new(Mutex::new(config));
    
    // Use user-local data directory to avoid permission issues
    let base_data_dir: PathBuf = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("deepseek-viewer");
    let output_dir = base_data_dir.join("dist");
    let index_path = base_data_dir.join("search_index");

    // Check if we have a configured file and it exists
    let has_valid_config = {
        let cfg = config.lock().unwrap();
        cfg.conversations_file_path.as_ref()
            .map(|p| std::path::Path::new(p).exists())
            .unwrap_or(false)
    };

    if has_valid_config {
        let conversations_path = {
            let cfg = config.lock().unwrap();
            cfg.conversations_file_path.clone().unwrap()
        };
        
        // Generate site if needed
        if !output_dir.exists() {
            tracing::info!("📦 Generating HTML site in {}...", output_dir.display());
            std::fs::create_dir_all(&output_dir)?;
            generator::generate_site(&conversations_path, output_dir.to_str().unwrap()).await?;
            tracing::info!("✅ HTML site generated");
        } else {
            tracing::info!("✅ Using existing HTML site in {}", output_dir.display());
        }

        // Build search index if needed
        if !index_path.exists() {
            tracing::info!("📚 Building search index in {}...", index_path.display());
            std::fs::create_dir_all(&index_path)?;
            indexer::build_index(&conversations_path, index_path.to_str().unwrap()).await?;
            tracing::info!("✅ Search index built");
        } else {
            tracing::info!("✅ Using existing search index");
        }
    } else {
        tracing::info!("⚠️  No conversations file configured - will show import page");
        
        // Create empty dist directory with just the import pages
        if !output_dir.exists() {
            std::fs::create_dir_all(&output_dir)?;
        }
        
        // Create empty index directory
        if !index_path.exists() {
            std::fs::create_dir_all(&index_path)?;
        }
    }

    // Always start embedded web server
    let server_output_dir = output_dir.to_string_lossy().to_string();
    let server_index_path = index_path.to_string_lossy().to_string();
    
    tokio::spawn(async move {
        tracing::info!("🌐 Starting embedded web server on http://127.0.0.1:8080");
        
        // Try to create search engine, create empty one if it fails
        let search_engine = match SearchEngine::new(&server_index_path) {
            Ok(engine) => {
                tracing::info!("✅ Search engine loaded");
                engine
            },
            Err(e) => {
                tracing::warn!("⚠️  No search index available, creating empty one: {}", e);
                // Create an empty index
                let empty_json = "[]";
                let temp_file = std::env::temp_dir().join("empty_conversations.json");
                if let Err(e) = std::fs::write(&temp_file, empty_json) {
                    tracing::error!("Failed to create temp file: {}", e);
                }
                
                // Try to build empty index
                if let Err(e) = indexer::build_index(temp_file.to_str().unwrap(), &server_index_path).await {
                    tracing::error!("Failed to create empty index: {}", e);
                }
                
                // Try again
                match SearchEngine::new(&server_index_path) {
                    Ok(engine) => engine,
                    Err(e) => {
                        tracing::error!("❌ Failed to create search engine: {}", e);
                        return;
                    }
                }
            }
        };
        
        // Start server
        let addr = "127.0.0.1:8080".parse().unwrap();
        if let Err(e) = server::serve(addr, search_engine, &server_output_dir).await {
            tracing::error!("❌ Server error: {}", e);
        }
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let app_state = AppState {
        index_path: index_path.to_string_lossy().to_string(),
        output_dir: output_dir.to_string_lossy().to_string(),
        config: config.clone(),
    };

    tracing::info!("✨ Opening application window...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(generate_handler![
            has_conversations,
            get_current_file_path,
            process_conversations_file,
            search,
            get_conversations
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            
            // Check if we should show import page
            let state = app.state::<AppState>();
            let config = state.config.lock().unwrap();
            
            if config.conversations_file_path.is_none() {
                // Navigate to import page
                let _ = window.eval("window.location.href = '/import'");
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
