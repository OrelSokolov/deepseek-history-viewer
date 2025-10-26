use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub conversations_file_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            conversations_file_path: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        let contents = std::fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        
        let config: AppConfig = serde_json::from_str(&contents)
            .context("Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        std::fs::write(&config_path, contents)
            .context("Failed to write config file")?;
        
        Ok(())
    }
    
    fn config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;
        
        Ok(config_dir.join("deepseek-viewer").join("config.json"))
    }
}

