use anyhow::Result;
use serde_json::json;
use std::fs;
use tempfile::TempDir;

// Import from the main crate
use deepseek_viewer::search::SearchEngine;
use deepseek_viewer::indexer;

#[tokio::test]
async fn test_ngram_substring_search() -> Result<()> {
    // Create temporary directories for test
    let temp_dir = TempDir::new()?;
    let index_path = temp_dir.path().join("test_index");
    let conversations_path = temp_dir.path().join("conversations.json");
    
    // Create test data with various conversations
    let test_data = json!([
        {
            "id": "1",
            "title": "О гравитации",
            "inserted_at": "2024-01-01T00:00:00Z",
            "mapping": {
                "root": {
                    "children": ["msg1"]
                },
                "msg1": {
                    "message": {
                        "fragments": [
                            {"type": "text", "content": "Что такое гравитация и как она работает?"}
                        ]
                    },
                    "children": []
                }
            }
        },
        {
            "id": "2",
            "title": "Правила поиска",
            "inserted_at": "2024-01-02T00:00:00Z",
            "mapping": {
                "root": {
                    "children": ["msg2"]
                },
                "msg2": {
                    "message": {
                        "fragments": [
                            {"type": "text", "content": "Правила работы с ngram поиском очень простые"}
                        ]
                    },
                    "children": []
                }
            }
        },
        {
            "id": "3",
            "title": "Формулы математики",
            "inserted_at": "2024-01-03T00:00:00Z",
            "mapping": {
                "root": {
                    "children": ["msg3"]
                },
                "msg3": {
                    "message": {
                        "fragments": [
                            {"type": "text", "content": "Математические формулы и их применение в платформах"}
                        ]
                    },
                    "children": []
                }
            }
        }
    ]);
    
    // Write test data to file
    fs::write(&conversations_path, test_data.to_string())?;
    
    // Build index
    indexer::build_index(
        conversations_path.to_str().unwrap(),
        index_path.to_str().unwrap()
    ).await?;
    
    // Create search engine
    let search = SearchEngine::new(index_path.to_str().unwrap())?;
    
    // Test 1: "гра" should find "гравитация"
    let results = search.search("гра", 10)?;
    assert!(!results.is_empty(), "Should find results for 'гра'");
    assert!(
        results.iter().any(|r| r.title.contains("гравитац") || r.snippet.contains("гравитац")),
        "Should find conversation about гравитация with query 'гра'"
    );
    
    // Test 2: "рав" should find both "гравитация" and "правила"
    let results = search.search("рав", 10)?;
    assert!(results.len() >= 2, "Should find at least 2 results for 'рав'");
    
    // Test 3: "форм" should find "формулы" and "платформ"
    let results = search.search("форм", 10)?;
    assert!(!results.is_empty(), "Should find results for 'форм'");
    assert!(
        results.iter().any(|r| r.title.contains("Формул") || r.snippet.contains("формул")),
        "Should find conversation about формулы with query 'форм'"
    );
    
    // Test 4: "пои" should find "поиск"
    let results = search.search("пои", 10)?;
    assert!(!results.is_empty(), "Should find results for 'пои'");
    assert!(
        results.iter().any(|r| r.snippet.contains("поиск")),
        "Should find text containing 'поиск' with query 'пои'"
    );
    
    // Test 5: Too short query (less than 3 chars) - should still work but might not find anything
    let results = search.search("гр", 10);
    // This might fail or return empty, which is expected for ngram(3,10)
    assert!(results.is_ok(), "Should handle short queries gracefully");
    
    // Test 6: Multi-word search
    let results = search.search("форм мат", 10)?;
    assert!(!results.is_empty(), "Should find results for multi-word query");
    
    Ok(())
}

#[tokio::test]
async fn test_search_returns_snippets() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let index_path = temp_dir.path().join("test_index");
    let conversations_path = temp_dir.path().join("conversations.json");
    
    let test_data = json!([
        {
            "id": "1",
            "title": "Длинный текст",
            "inserted_at": "2024-01-01T00:00:00Z",
            "mapping": {
                "root": {
                    "children": ["msg1"]
                },
                "msg1": {
                    "message": {
                        "fragments": [
                            {
                                "type": "text",
                                "content": "А".repeat(300) + "гравитация" + &"Б".repeat(300)
                            }
                        ]
                    },
                    "children": []
                }
            }
        }
    ]);
    
    fs::write(&conversations_path, test_data.to_string())?;
    indexer::build_index(
        conversations_path.to_str().unwrap(),
        index_path.to_str().unwrap()
    ).await?;
    
    let search = SearchEngine::new(index_path.to_str().unwrap())?;
    let results = search.search("грав", 10)?;
    
    assert!(!results.is_empty(), "Should find results");
    
    // Check that snippet is truncated and ends with "..."
    let snippet = &results[0].snippet;
    // Snippet should be ~200 chars (not bytes!) + "..." = 203 chars max
    let char_count = snippet.chars().count();
    assert!(char_count <= 210, "Snippet should be truncated to ~200 chars, got {}", char_count);
    if char_count > 200 {
        assert!(snippet.ends_with("..."), "Long snippet should end with '...'");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_utf8_safety() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let index_path = temp_dir.path().join("test_index");
    let conversations_path = temp_dir.path().join("conversations.json");
    
    // Test with various UTF-8 characters
    let test_data = json!([
        {
            "id": "1",
            "title": "UTF-8 тест 测试 🚀",
            "inserted_at": "2024-01-01T00:00:00Z",
            "mapping": {
                "root": {
                    "children": ["msg1"]
                },
                "msg1": {
                    "message": {
                        "fragments": [
                            {
                                "type": "text",
                                "content": "Привет мир! Hello world! 你好世界！ Эмодзи: 🦀⚡🔍"
                            }
                        ]
                    },
                    "children": []
                }
            }
        }
    ]);
    
    fs::write(&conversations_path, test_data.to_string())?;
    indexer::build_index(
        conversations_path.to_str().unwrap(),
        index_path.to_str().unwrap()
    ).await?;
    
    let search = SearchEngine::new(index_path.to_str().unwrap())?;
    
    // This should not panic with "byte index is not a char boundary"
    let results = search.search("при", 10)?;
    assert!(!results.is_empty(), "Should find UTF-8 text");
    
    // Snippet should be properly truncated without panicking
    for result in results {
        assert!(result.snippet.len() > 0, "Snippet should not be empty");
    }
    
    Ok(())
}

