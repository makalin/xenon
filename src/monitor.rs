use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::model::{Event, EventKind, MonitorRequest};

#[derive(Debug, Clone)]
pub struct MonitorService {
    client: Client,
    api_base_url: String,
    bearer_token: Option<String>,
}

impl MonitorService {
    pub fn new(
        api_base_url: String,
        bearer_token: Option<String>,
        request_timeout_seconds: u64,
    ) -> Result<Self> {
        let client = Client::builder()
            .user_agent(format!("xenon/{}", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(request_timeout_seconds))
            .build()
            .context("failed to build X API client")?;

        Ok(Self {
            client,
            api_base_url,
            bearer_token,
        })
    }

    pub fn is_configured(&self) -> bool {
        self.bearer_token.is_some()
    }

    pub async fn generate_batch(&self, request: &MonitorRequest) -> Result<Vec<Event>> {
        let token = self.bearer_token.as_deref().ok_or_else(|| {
            anyhow!("missing X bearer token; set X_BEARER_TOKEN or XENON_X_BEARER_TOKEN")
        })?;

        if request.kinds.is_empty() {
            bail!("at least one event kind is required");
        }

        let username = normalize_handle(&request.handle)?;
        let mut events = Vec::new();

        for (index, kind) in request.kinds.iter().enumerate() {
            let remaining_limit = request.limit.saturating_sub(events.len());
            if remaining_limit == 0 {
                break;
            }

            let remaining_kinds = request.kinds.len() - index;
            let fetch_limit = remaining_limit.div_ceil(remaining_kinds).clamp(1, 100);
            let query = build_query(&username, kind)?;
            let mut kind_events = self
                .fetch_recent_posts(token, &request.handle, kind.clone(), &query, fetch_limit)
                .await?;
            events.append(&mut kind_events);
        }

        events.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
        events.truncate(request.limit);
        Ok(events)
    }

    async fn fetch_recent_posts(
        &self,
        token: &str,
        handle: &str,
        kind: EventKind,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Event>> {
        let url = format!(
            "{}/tweets/search/recent",
            self.api_base_url.trim_end_matches('/')
        );
        let response = self
            .client
            .get(url)
            .bearer_auth(token)
            .query(&[
                ("query", query),
                ("max_results", &limit.to_string()),
                (
                    "tweet.fields",
                    "created_at,public_metrics,referenced_tweets",
                ),
            ])
            .send()
            .await
            .context("failed to call X recent search endpoint")?;

        let status = response.status();
        let body = response
            .text()
            .await
            .context("failed to read X API response body")?;

        if !status.is_success() {
            bail!("X API request failed with {status}: {body}");
        }

        let payload: RecentSearchResponse =
            serde_json::from_str(&body).context("failed to decode X API response")?;

        if let Some(errors) = payload.errors {
            if payload.data.as_ref().is_none_or(Vec::is_empty) {
                let detail = errors
                    .into_iter()
                    .map(|error| error.detail.unwrap_or(error.title))
                    .collect::<Vec<_>>()
                    .join("; ");
                bail!("X API returned errors: {detail}");
            }
        }

        Ok(payload
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|post| map_post_to_event(post, handle, kind.clone()))
            .collect())
    }
}

fn normalize_handle(handle: &str) -> Result<String> {
    let username = handle.trim().trim_start_matches('@');
    if username.is_empty() {
        bail!("handle cannot be empty");
    }

    if !username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        bail!("handle must contain only letters, numbers, or underscores");
    }

    Ok(username.to_string())
}

fn build_query(username: &str, kind: &EventKind) -> Result<String> {
    match kind {
        EventKind::Tweet => Ok(format!("from:{username} -is:reply -is:retweet")),
        EventKind::Reply => Ok(format!("from:{username} is:reply")),
        EventKind::Follow => {
            bail!("follow events are not available through the X v2 bearer-token endpoints used by Xenon")
        }
        EventKind::Trend => {
            bail!("trend events are not available through the X v2 bearer-token endpoints used by Xenon")
        }
    }
}

fn map_post_to_event(post: XPost, handle: &str, kind: EventKind) -> Event {
    let metrics = post.public_metrics.unwrap_or_default();
    let score =
        metrics.like_count + metrics.retweet_count + metrics.reply_count + metrics.quote_count;

    Event {
        id: post.id,
        handle: handle.to_string(),
        kind,
        message: compact_text(&post.text),
        score,
        timestamp: post.created_at.unwrap_or_else(Utc::now),
    }
}

fn compact_text(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut compact = compact.trim().to_string();
    if compact.len() > 180 {
        compact.truncate(177);
        compact.push_str("...");
    }
    compact
}

#[derive(Debug, Deserialize)]
struct RecentSearchResponse {
    data: Option<Vec<XPost>>,
    errors: Option<Vec<XApiError>>,
}

#[derive(Debug, Deserialize)]
struct XApiError {
    title: String,
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XPost {
    id: String,
    text: String,
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    public_metrics: Option<XPublicMetrics>,
}

#[derive(Debug, Default, Deserialize)]
struct XPublicMetrics {
    #[serde(default)]
    retweet_count: u32,
    #[serde(default)]
    reply_count: u32,
    #[serde(default)]
    like_count: u32,
    #[serde(default)]
    quote_count: u32,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn normalizes_handles() -> Result<()> {
        assert_eq!(normalize_handle("@XDevelopers")?, "XDevelopers");
        assert!(normalize_handle("@bad-handle").is_err());
        Ok(())
    }

    #[test]
    fn builds_queries_for_supported_kinds() -> Result<()> {
        assert_eq!(
            build_query("XDevelopers", &EventKind::Tweet)?,
            "from:XDevelopers -is:reply -is:retweet"
        );
        assert_eq!(
            build_query("XDevelopers", &EventKind::Reply)?,
            "from:XDevelopers is:reply"
        );
        assert!(build_query("XDevelopers", &EventKind::Trend).is_err());
        Ok(())
    }

    #[test]
    fn maps_recent_search_payload_to_event() -> Result<()> {
        let post: XPost = serde_json::from_value(json!({
            "id": "123",
            "text": "hello\nworld",
            "created_at": "2025-01-06T18:40:40Z",
            "public_metrics": {
                "retweet_count": 3,
                "reply_count": 4,
                "like_count": 5,
                "quote_count": 2
            }
        }))?;

        let event = map_post_to_event(post, "@xenon", EventKind::Tweet);

        assert_eq!(event.id, "123");
        assert_eq!(event.score, 14);
        assert_eq!(event.message, "hello world");
        Ok(())
    }

    #[test]
    fn service_configuration_state_tracks_token_presence() -> Result<()> {
        let configured =
            MonitorService::new("https://api.x.com/2".to_string(), Some("x".into()), 5)?;
        let unconfigured = MonitorService::new("https://api.x.com/2".to_string(), None, 5)?;

        assert!(configured.is_configured());
        assert!(!unconfigured.is_configured());
        Ok(())
    }
}
