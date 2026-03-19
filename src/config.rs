#[derive(Debug, Clone)]
pub struct AppConfig {
    pub seed: u64,
    pub profile: String,
    pub x_api_base_url: String,
    pub x_bearer_token: Option<String>,
    pub request_timeout_seconds: u64,
    pub webhook_secret: Option<String>,
}
