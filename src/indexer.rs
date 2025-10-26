use anyhow::Result;
use serde::Deserialize;
use tantivy::schema::*;
use tantivy::tokenizer::{NgramTokenizer, LowerCaser, TextAnalyzer};
use tantivy::{doc, Index, IndexWriter};

#[derive(Debug, Deserialize)]
struct Conversation {
    id: String,
    title: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
    mapping: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct Message {
    message: MessageData,
}

#[derive(Debug, Deserialize)]
struct MessageData {
    fragments: Vec<Fragment>,
}

#[derive(Debug, Deserialize)]
struct Fragment {
    #[serde(rename = "type")]
    fragment_type: String,
    content: String,
}

pub async fn build_index(conversations_path: &str, index_path: &str) -> Result<()> {
    tracing::info!("Reading conversations from {}", conversations_path);
    
    let data = tokio::fs::read_to_string(conversations_path).await?;
    let conversations: Vec<Conversation> = serde_json::from_str(&data)?;
    
    tracing::info!("Found {} conversations", conversations.len());

    // Create schema with ngram tokenizer for BLAZING FAST substring search (min=2 chars!)
    let mut schema_builder = Schema::builder();
    let conversation_id = schema_builder.add_text_field("conversation_id", STRING | STORED);
    
    // Ngram tokenizer for substring matching: "гр" -> "гравитация"
    let ngram_text_options = tantivy::schema::TextOptions::default()
        .set_indexing_options(
            tantivy::schema::TextFieldIndexing::default()
                .set_tokenizer("ngram2")
                .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions)
        )
        .set_stored();
    
    let title = schema_builder.add_text_field("title", ngram_text_options.clone());
    let content = schema_builder.add_text_field("content", ngram_text_options.clone());
    let date = schema_builder.add_text_field("date", STRING | STORED);
    let schema = schema_builder.build();

    // Create index
    std::fs::create_dir_all(index_path)?;
    let index = Index::create_in_dir(index_path, schema.clone())?;
    
    // Register ngram tokenizer for substring search (min=2, max=10, prefix_only=false)
    let ngram_tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 10, false).unwrap())
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("ngram2", ngram_tokenizer);
    
    let mut index_writer: IndexWriter = index.writer(50_000_000)?;

    // Index conversations
    for (idx, conv) in conversations.iter().enumerate() {
        if idx % 100 == 0 {
            tracing::info!("Indexed {}/{} conversations", idx, conversations.len());
        }

        let conv_title = conv.title.clone().unwrap_or_else(|| format!("Conversation {}", idx + 1));
        let mut full_content = String::new();

        // Extract messages from mapping
        if let Some(mapping) = conv.mapping.as_object() {
            if let Some(root) = mapping.get("root") {
                if let Some(children) = root.get("children").and_then(|c| c.as_array()) {
                    extract_messages(mapping, children, &mut full_content);
                }
            }
        }

        // Add document
        index_writer.add_document(doc!(
            conversation_id => conv.id.clone(),
            title => conv_title,
            content => full_content,
            date => conv.inserted_at.clone().unwrap_or_default(),
        ))?;
    }

    index_writer.commit()?;
    tracing::info!("✅ Successfully indexed {} conversations", conversations.len());

    Ok(())
}

fn extract_messages(
    mapping: &serde_json::Map<String, serde_json::Value>,
    children: &[serde_json::Value],
    content: &mut String,
) {
    for child_id in children {
        if let Some(child_id_str) = child_id.as_str() {
            if let Some(child) = mapping.get(child_id_str) {
                if let Some(message) = child.get("message") {
                    if let Some(fragments) = message.get("fragments").and_then(|f| f.as_array()) {
                        for fragment in fragments {
                            if let Some(text) = fragment.get("content").and_then(|c| c.as_str()) {
                                content.push_str(text);
                                content.push(' ');
                            }
                        }
                    }
                }
                
                if let Some(grandchildren) = child.get("children").and_then(|c| c.as_array()) {
                    extract_messages(mapping, grandchildren, content);
                }
            }
        }
    }
}

