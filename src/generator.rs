use anyhow::Result;
use askama::Template;
use chrono::{DateTime, Utc};
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use super::templates::*;

#[derive(Debug, Deserialize)]
struct Conversation {
    id: String,
    title: Option<String>,
    inserted_at: Option<String>,
    updated_at: Option<String>,
    mapping: serde_json::Value,
}

pub async fn generate_site(conversations_path: &str, output_dir: &str) -> Result<()> {
    tracing::info!("📚 Reading conversations from {}", conversations_path);
    
    let data = tokio::fs::read_to_string(conversations_path).await?;
    let conversations: Vec<Conversation> = serde_json::from_str(&data)?;
    
    tracing::info!("Found {} conversations", conversations.len());

    // Create output directories
    let output_path = Path::new(output_dir);
    fs::create_dir_all(output_path.join("conversations"))?;
    fs::create_dir_all(output_path.join("assets/css"))?;
    fs::create_dir_all(output_path.join("assets/js"))?;

    // Initialize syntax highlighting
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.light"];

    // Generate sidebar HTML once (shared across all pages)
    let sidebar_html = generate_sidebar_html(&conversations);
    
    // Generate conversation pages in PARALLEL! 🚀
    let counter = Arc::new(Mutex::new(0usize));
    let total = conversations.len();
    
    let all_conversations: Vec<ConversationMeta> = conversations
        .par_iter()
        .filter_map(|conv| {
            // Progress counter
            {
                let mut count = counter.lock().unwrap();
                *count += 1;
                if *count % 100 == 0 {
                    tracing::info!("Generated {}/{} pages", *count, total);
                }
            }

            let conv_id = &conv.id;
            let title = conv.title.as_deref().unwrap_or("Untitled");
            let inserted_at = parse_datetime(&conv.inserted_at);
            let updated_at = parse_datetime(&conv.updated_at);

            // Extract and render messages
            let messages = match extract_and_render_messages(&conv.mapping, &ps, theme) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!("Failed to process conversation {}: {}", conv_id, e);
                    return None;
                }
            };
            
            // Generate conversation page
            let conversation_html = match (ConversationTemplate {
                title,
                inserted_at,
                updated_at,
                message_count: messages.len(),
                messages: &messages,
            }).render() {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("Failed to render conversation {}: {}", conv_id, e);
                    return None;
                }
            };

            let page_html = match (BaseTemplate {
                title,
                content: conversation_html,
                conversations_html: sidebar_html.clone(),
            }).render() {
                Ok(h) => h,
                Err(e) => {
                    tracing::warn!("Failed to render page {}: {}", conv_id, e);
                    return None;
                }
            };

            // Write to file
            let conv_dir = output_path.join("conversations").join(conv_id);
            if let Err(e) = fs::create_dir_all(&conv_dir) {
                tracing::warn!("Failed to create dir for {}: {}", conv_id, e);
                return None;
            }
            if let Err(e) = fs::write(conv_dir.join("index.html"), page_html) {
                tracing::warn!("Failed to write file for {}: {}", conv_id, e);
                return None;
            }

            // Return metadata
            Some(ConversationMeta {
                id: conv_id.clone(),
                title: title.to_string(),
                url: format!("/conversations/{}/", conv_id),
                inserted_at,
            })
        })
        .collect();

    // Generate index page
    let conversations_by_month = group_by_month(&all_conversations);
    let index_content = IndexTemplate {
        total_conversations: conversations.len(),
        conversations_by_month: conversations_by_month.clone(),
    }.render()?;

    let conversations_html = generate_sidebar_html(&conversations);
    let index_page = BaseTemplate {
        title: "Главная",
        content: index_content,
        conversations_html,
    }.render()?;

    fs::write(output_path.join("index.html"), index_page)?;

    // Copy CSS (simplified version from Jekyll)
    copy_static_assets(output_path)?;

    tracing::info!("✅ Generated {} conversation pages", conversations.len());

    Ok(())
}

fn extract_and_render_messages(
    mapping: &serde_json::Value,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Result<Vec<Message>> {
    let mut messages = Vec::new();
    
    if let Some(mapping_obj) = mapping.as_object() {
        if let Some(root) = mapping_obj.get("root") {
            if let Some(children) = root.get("children").and_then(|c| c.as_array()) {
                extract_messages_recursive(mapping_obj, children, &mut messages, ps, theme)?;
            }
        }
    }

    Ok(messages)
}

fn extract_messages_recursive(
    mapping: &serde_json::Map<String, serde_json::Value>,
    children: &[serde_json::Value],
    messages: &mut Vec<Message>,
    ps: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
) -> Result<()> {
    for child_id in children {
        if let Some(child_id_str) = child_id.as_str() {
            if let Some(child) = mapping.get(child_id_str) {
                if let Some(message) = child.get("message") {
                    if let Some(fragments) = message.get("fragments").and_then(|f| f.as_array()) {
                        for fragment in fragments {
                            let msg_type = fragment.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("UNKNOWN");
                            
                            let content = fragment.get("content")
                                .and_then(|c| c.as_str())
                                .unwrap_or("");

                            let content_html = if msg_type == "REQUEST" {
                                // Simple HTML escape for requests
                                html_escape::encode_text(content).replace('\n', "<br>")
                            } else {
                                // Render markdown for responses
                                render_markdown(content, ps, theme)?
                            };

                            let inserted_at = message.get("inserted_at")
                                .and_then(|d| d.as_str())
                                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                                .map(|dt| dt.with_timezone(&Utc));

                            messages.push(Message {
                                message_type: msg_type.to_string(),
                                content_html,
                                inserted_at,
                            });
                        }
                    }
                }
                
                if let Some(grandchildren) = child.get("children").and_then(|c| c.as_array()) {
                    extract_messages_recursive(mapping, grandchildren, messages, ps, theme)?;
                }
            }
        }
    }

    Ok(())
}

fn render_markdown(content: &str, ps: &SyntaxSet, theme: &syntect::highlighting::Theme) -> Result<String> {
    // Конвертируем LaTeX триггеры в KaTeX формат
    let content = convert_latex_delimiters(content);
    
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&content, options);
    
    let mut html_output = String::new();
    let mut in_code_block = false;
    let mut code_buffer = String::new();
    let mut code_lang = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                code_lang = lang.to_string();
                code_buffer.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_code_block {
                    // Highlight code with syntect (inline styles)
                    let syntax = ps.find_syntax_by_token(&code_lang)
                        .unwrap_or_else(|| ps.find_syntax_plain_text());
                    
                    let highlighted = syntect::html::highlighted_html_for_string(
                        &code_buffer,
                        ps,
                        syntax,
                        theme,
                    )?;
                    
                    // Escape code for data attribute
                    let escaped_code = html_escape::encode_double_quoted_attribute(&code_buffer);
                    
                    // Wrap in div with highlight class and toolbar
                    html_output.push_str(r#"<div class="code-block-wrapper">"#);
                    html_output.push_str(r#"<div class="code-toolbar">"#);
                    html_output.push_str(&format!(r#"<span class="code-lang">{}</span>"#, code_lang));
                    html_output.push_str(r#"<div class="code-actions">"#);
                    html_output.push_str(r#"<button class="code-btn copy-btn" title="Copy code"><svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M4 4V2.5C4 1.67157 4.67157 1 5.5 1H13.5C14.3284 1 15 1.67157 15 2.5V10.5C15 11.3284 14.3284 12 13.5 12H12V13.5C12 14.3284 11.3284 15 10.5 15H2.5C1.67157 15 1 14.3284 1 13.5V5.5C1 4.67157 1.67157 4 2.5 4H4Z" stroke="currentColor" stroke-width="1.5"/></svg>Copy</button>"#);
                    html_output.push_str(r#"<button class="code-btn download-btn" title="Download code"><svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 1V11M8 11L11 8M8 11L5 8M2 11V13.5C2 14.3284 2.67157 15 3.5 15H12.5C13.3284 15 14 14.3284 14 13.5V11" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>Download</button>"#);
                    html_output.push_str(r#"</div></div>"#);
                    html_output.push_str(&format!(r#"<div class="highlight" data-code="{}" data-lang="{}">"#, escaped_code, code_lang));
                    html_output.push_str(r#"<div class="syntax">"#);
                    html_output.push_str(&highlighted);
                    html_output.push_str("</div></div></div>");
                    
                    in_code_block = false;
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_buffer.push_str(&text);
                } else {
                    html_output.push_str(&html_escape::encode_text(&text));
                }
            }
            other => {
                if !in_code_block {
                    let mut temp = String::new();
                    html::push_html(&mut temp, std::iter::once(other));
                    html_output.push_str(&temp);
                }
            }
        }
    }

    Ok(html_output)
}

fn convert_latex_delimiters(content: &str) -> String {
    let mut result = content.to_string();
    
    // Конвертируем блочные формулы: \[ ... \] → $$...$$
    // Используем регулярное выражение для замены
    result = result.replace("\\[", "\n\n$$");
    result = result.replace("\\]", "$$\n\n");
    
    // Конвертируем inline формулы: \( ... \) → $...$
    result = result.replace("\\(", "$");
    result = result.replace("\\)", "$");
    
    result
}

fn parse_datetime(date_str: &Option<String>) -> Option<DateTime<Utc>> {
    date_str.as_ref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.to_utc())
}

fn generate_sidebar_html(conversations: &[Conversation]) -> String {
    let mut html = String::from(r#"<h3>Всего чатов: "#);
    html.push_str(&conversations.len().to_string());
    html.push_str("</h3>");

    // Group by month for better organization
    let mut conversations_by_month: HashMap<String, Vec<&Conversation>> = HashMap::new();
    
    for conv in conversations {
        if let Some(date_str) = &conv.inserted_at {
            if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                // Convert to UTC for consistent grouping
                let utc_dt = dt.to_utc();
                let month_key = utc_dt.format("%Y-%m").to_string();
                conversations_by_month.entry(month_key).or_default().push(conv);
            }
        }
    }

    // Sort months descending
    let mut months: Vec<_> = conversations_by_month.keys().collect();
    months.sort_by(|a, b| b.cmp(a));

    // Russian month names
    let month_names = [
        "", "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
        "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь"
    ];

    for month_key in months.iter().take(12) { // Limit to 12 months
        if let Some(convs) = conversations_by_month.get(*month_key) {
            let parts: Vec<&str> = month_key.split('-').collect();
            let year = parts[0];
            let month_num: usize = parts[1].parse().unwrap_or(0);
            let month_label = if month_num > 0 && month_num < 13 {
                format!("{} {}", month_names[month_num], year)
            } else {
                month_key.to_string()
            };

            html.push_str(r#"<div class="month-group">"#);
            html.push_str(&format!(r#"<div class="month-header">{}</div>"#, month_label));
            html.push_str(r#"<ul class="month-conversations">"#);

            for conv in convs.iter().take(50) { // Limit per month
                let title = conv.title.as_deref().unwrap_or("Untitled");
                html.push_str(&format!(
                    r#"<li class="conversation-item"><a href="/conversations/{}/" class="conversation-link"><div class="conversation-title">{}</div></a></li>"#,
                    conv.id,
                    html_escape::encode_text(title)
                ));
            }

            html.push_str("</ul></div>");
        }
    }

    html
}

fn group_by_month(conversations: &[ConversationMeta]) -> Vec<MonthGroup> {
    let mut grouped: HashMap<String, Vec<ConversationMeta>> = HashMap::new();

    for conv in conversations {
        if let Some(date) = conv.inserted_at {
            let month_key = date.format("%Y-%m").to_string();
            grouped.entry(month_key).or_default().push(conv.clone());
        }
    }

    let mut groups: Vec<MonthGroup> = grouped
        .into_iter()
        .map(|(key, convs)| {
            let label = if let Some(first) = convs.first() {
                if let Some(date) = first.inserted_at {
                    date.format("%B %Y").to_string()
                } else {
                    key.clone()
                }
            } else {
                key
            };

            MonthGroup {
                label,
                conversations: convs,
            }
        })
        .collect();

    groups.sort_by(|a, b| b.label.cmp(&a.label));
    groups
}

fn copy_static_assets(output_path: &Path) -> Result<()> {
    tracing::info!("📦 Copying static assets...");
    
    // Copy CSS from static folder if exists, otherwise from Jekyll
    let css_source = if Path::new("static/main.css").exists() {
        fs::read_to_string("static/main.css")?
    } else if Path::new("deepseek-chat-viewer/assets/css/main.scss").exists() {
        // Read SCSS (we'll use it as-is, browsers can handle basic CSS)
        let scss = fs::read_to_string("deepseek-chat-viewer/assets/css/main.scss")?;
        // Remove Jekyll front matter
        scss.lines().skip(3).collect::<Vec<_>>().join("\n")
    } else {
        // Minimal fallback CSS
        include_str!("../static/main.css").to_string()
    };
    
    fs::write(output_path.join("assets/css/main.css"), css_source)?;
    tracing::info!("✅ CSS copied");

    // Generate syntax highlighting CSS from syntect
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.light"];
    let mut css = syntect::html::css_for_theme_with_class_style(theme, syntect::html::ClassStyle::Spaced)?;
    
    // Add wrapper styles for code blocks
    css.push_str("\n\n/* Code block wrapper styles */\n");
    css.push_str(".highlight {\n");
    css.push_str("    padding: 1em;\n");
    css.push_str("    border-radius: 4px;\n");
    css.push_str("    border: 1px solid #e1e4e8;\n");
    css.push_str("    overflow-x: auto;\n");
    css.push_str("}\n\n");
    css.push_str(".highlight pre.syntax {\n");
    css.push_str("    margin: 0;\n");
    css.push_str("    padding: 0;\n");
    css.push_str("}\n");
    
    fs::write(output_path.join("assets/css/syntax.css"), css)?;
    tracing::info!("✅ Syntax highlighting CSS generated");

    // Copy search JS
    let js_source = if Path::new("static/search.js").exists() {
        fs::read_to_string("static/search.js")?
    } else {
        include_str!("../static/search.js").to_string()
    };
    
    fs::write(output_path.join("assets/js/search.js"), js_source)?;
    
    // Copy code-actions JS
    let code_actions_source = if Path::new("static/code-actions.js").exists() {
        fs::read_to_string("static/code-actions.js")?
    } else {
        include_str!("../static/code-actions.js").to_string()
    };
    
    fs::write(output_path.join("assets/js/code-actions.js"), code_actions_source)?;
    tracing::info!("✅ JavaScript copied");

    Ok(())
}

