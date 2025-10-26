use askama::Template;
use chrono::{DateTime, Utc};

#[derive(Template)]
#[template(path = "base.html")]
pub struct BaseTemplate<'a> {
    pub title: &'a str,
    pub content: String,
    pub conversations_html: String,
}

#[derive(Template)]
#[template(path = "conversation.html")]
pub struct ConversationTemplate<'a> {
    pub title: &'a str,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub message_count: usize,
    pub messages: &'a [Message],
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message_type: String,
    pub content_html: String,
    pub inserted_at: Option<DateTime<Utc>>,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub total_conversations: usize,
    pub conversations_by_month: Vec<MonthGroup>,
}

#[derive(Debug, Clone)]
pub struct MonthGroup {
    pub label: String,
    pub conversations: Vec<ConversationMeta>,
}

#[derive(Debug, Clone)]
pub struct ConversationMeta {
    pub id: String,
    pub title: String,
    pub url: String,
    pub inserted_at: Option<DateTime<Utc>>,
}

