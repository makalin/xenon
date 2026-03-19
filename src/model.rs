use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::cli::EventKindArg;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Tweet,
    Reply,
    Follow,
    Trend,
}

impl Display for EventKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tweet => write!(f, "tweet"),
            Self::Reply => write!(f, "reply"),
            Self::Follow => write!(f, "follow"),
            Self::Trend => write!(f, "trend"),
        }
    }
}

impl From<EventKindArg> for EventKind {
    fn from(value: EventKindArg) -> Self {
        match value {
            EventKindArg::Tweets => Self::Tweet,
            EventKindArg::Replies => Self::Reply,
            EventKindArg::Follows => Self::Follow,
            EventKindArg::Trends => Self::Trend,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub handle: String,
    pub kind: EventKind,
    pub message: String,
    pub score: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorRequest {
    pub handle: String,
    pub kinds: Vec<EventKind>,
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub profile: String,
    pub x_api_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Json,
    Jsonl,
    Csv,
    Markdown,
}

impl Display for ExportFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "json"),
            Self::Jsonl => write!(f, "jsonl"),
            Self::Csv => write!(f, "csv"),
            Self::Markdown => write!(f, "markdown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub handle: String,
    pub kinds: Vec<EventKind>,
    pub limit: usize,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResponse {
    pub format: ExportFormat,
    pub content: String,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_events: usize,
    pub total_score: u32,
    pub average_score: f64,
    pub highest_score: u32,
    pub by_kind: Vec<KindStat>,
    pub top_events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindStat {
    pub kind: EventKind,
    pub count: usize,
    pub total_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub profile: String,
    pub x_api_base_url: String,
    pub x_api_configured: bool,
    pub webhook_secret_configured: bool,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSignatureResponse {
    pub algorithm: &'static str,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookVerifyResponse {
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawResult {
    pub winners: Vec<String>,
    pub total_candidates: usize,
}
