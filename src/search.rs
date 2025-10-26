use anyhow::Result;
use serde::Serialize;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::tokenizer::{NgramTokenizer, LowerCaser, TextAnalyzer};
use tantivy::{Index, ReloadPolicy};

#[derive(Debug, Clone)]
pub struct SearchEngine {
    index: Arc<Index>,
    schema: Schema,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub conversation_id: String,
    pub title: String,
    pub date: String,
    pub score: f32,
    pub snippet: String,
}

impl SearchEngine {
    pub fn new(index_path: &str) -> Result<Self> {
        let index = Index::open_in_dir(index_path)?;
        let schema = index.schema();
        
        // Register the same ngram tokenizer for searching (min=2, max=10)
        let ngram_tokenizer = TextAnalyzer::builder(NgramTokenizer::new(2, 10, false).unwrap())
            .filter(LowerCaser)
            .build();
        index.tokenizers().register("ngram2", ngram_tokenizer);
        
        Ok(Self {
            index: Arc::new(index),
            schema,
        })
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        let searcher = reader.searcher();

        // Get fields
        let conversation_id = self.schema.get_field("conversation_id").unwrap();
        let title_field = self.schema.get_field("title").unwrap();
        let content_field = self.schema.get_field("content").unwrap();
        let date_field = self.schema.get_field("date").unwrap();

        // BLAZING FAST ngram search - работает с 2 символов!
        // Ngram tokenizer сам разобьёт "гр" на биграммы и найдёт "гравитация"
        let mut query_parser = QueryParser::for_index(&self.index, vec![title_field, content_field]);
        query_parser.set_field_boost(title_field, 2.0); // Boost title results
        
        let query = query_parser.parse_query(&query_str.to_lowercase())?;

        // Search
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        // Collect results
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc: tantivy::TantivyDocument = searcher.doc(doc_address)?;
            
            let conv_id = retrieved_doc
                .get_first(conversation_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let title = retrieved_doc
                .get_first(title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string();
            
            let date = retrieved_doc
                .get_first(date_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Create snippet (first 200 chars from content) - UTF-8 safe!
            let content_text = retrieved_doc
                .get_first(content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            let snippet = if content_text.chars().count() > 200 {
                let truncated: String = content_text.chars().take(200).collect();
                format!("{}...", truncated)
            } else {
                content_text.to_string()
            };

            results.push(SearchResult {
                conversation_id: conv_id,
                title,
                date,
                score,
                snippet,
            });
        }

        Ok(results)
    }
}

