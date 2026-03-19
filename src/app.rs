use std::time::Duration;

use anyhow::{anyhow, Result};
use tokio::time::sleep;

use crate::analytics;
use crate::api::{self, ApiState};
use crate::cli::{
    AnalyzeArgs, Cli, ConfigArgs, DashboardArgs, DrawArgs, EventKindArg, ExportArgs,
    ExportFormatArg, MonitorArgs, ServeArgs, WebhookArgs, WebhookCommands,
};
use crate::config::AppConfig;
use crate::draw::pick_winners;
use crate::exporter;
use crate::mcp;
use crate::model::{
    ConfigResponse, EventKind, ExportFormat, MonitorRequest, WebhookSignatureResponse,
    WebhookVerifyResponse,
};
use crate::monitor::MonitorService;
use crate::tui;
use crate::webhook;

pub struct App {
    config: AppConfig,
    monitor_service: MonitorService,
}

impl App {
    pub fn bootstrap(cli: &Cli) -> Result<Self> {
        let config = AppConfig {
            seed: cli.seed,
            profile: cli.profile.clone(),
            x_api_base_url: std::env::var("XENON_X_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.x.com/2".to_string()),
            x_bearer_token: std::env::var("X_BEARER_TOKEN")
                .ok()
                .or_else(|| std::env::var("XENON_X_BEARER_TOKEN").ok()),
            request_timeout_seconds: std::env::var("XENON_REQUEST_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(15),
            webhook_secret: std::env::var("XENON_WEBHOOK_SECRET").ok(),
        };

        Ok(Self {
            monitor_service: MonitorService::new(
                config.x_api_base_url.clone(),
                config.x_bearer_token.clone(),
                config.request_timeout_seconds,
            )?,
            config,
        })
    }

    pub async fn run_server(&self, args: ServeArgs) -> Result<()> {
        if args.mcp {
            return mcp::run_stdio(self.monitor_service.clone()).await;
        }

        api::run(
            ApiState {
                config: self.config.clone().into(),
                monitor_service: self.monitor_service.clone(),
            },
            args.host,
            args.port,
        )
        .await
    }

    pub async fn run_monitor(&self, args: MonitorArgs) -> Result<()> {
        let request = monitor_request(args.handle, args.events, args.limit);
        let batch = self.monitor_service.generate_batch(&request).await?;

        for (index, event) in batch.into_iter().enumerate() {
            if index > 0 {
                sleep(Duration::from_millis(args.interval_ms)).await;
            }
            if args.json {
                println!("{}", serde_json::to_string(&event)?);
            } else {
                println!(
                    "{} {:<7} {:<8} score={} {}",
                    event.timestamp.to_rfc3339(),
                    event.handle,
                    event.kind,
                    event.score,
                    event.message
                );
            }
        }

        Ok(())
    }

    pub async fn run_dashboard(&self, args: DashboardArgs) -> Result<()> {
        tui::run_dashboard(self.monitor_service.clone(), &args.handle, args.tick_ms).await
    }

    pub async fn run_draw(&self, args: DrawArgs) -> Result<()> {
        let result = pick_winners(&args.input, args.count, self.config.seed)?;
        println!("{}", serde_json::to_string_pretty(&result)?);
        Ok(())
    }

    pub async fn run_export(&self, args: ExportArgs) -> Result<()> {
        let request = monitor_request(args.handle, args.events, args.limit);
        let events = self.monitor_service.generate_batch(&request).await?;
        let export = exporter::render(&events, map_export_format(args.format))?;

        if let Some(path) = args.output {
            exporter::write_output(&path, &export.content)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "written": true,
                    "path": path,
                    "format": export.format,
                    "event_count": export.event_count
                }))?
            );
        } else {
            println!("{}", export.content);
        }

        Ok(())
    }

    pub async fn run_analyze(&self, args: AnalyzeArgs) -> Result<()> {
        let request = monitor_request(args.handle, args.events, args.limit);
        let events = self.monitor_service.generate_batch(&request).await?;
        let summary = analytics::summarize(&events);
        println!("{}", serde_json::to_string_pretty(&summary)?);
        Ok(())
    }

    pub fn run_config(&self, args: ConfigArgs) -> Result<()> {
        let response = ConfigResponse {
            profile: self.config.profile.clone(),
            x_api_base_url: self.config.x_api_base_url.clone(),
            x_api_configured: self.monitor_service.is_configured(),
            webhook_secret_configured: self.config.webhook_secret.is_some(),
            request_timeout_seconds: self.config.request_timeout_seconds,
        };

        if args.json {
            println!("{}", serde_json::to_string_pretty(&response)?);
        } else {
            println!("profile={}", response.profile);
            println!("x_api_base_url={}", response.x_api_base_url);
            println!("x_api_configured={}", response.x_api_configured);
            println!(
                "webhook_secret_configured={}",
                response.webhook_secret_configured
            );
            println!(
                "request_timeout_seconds={}",
                response.request_timeout_seconds
            );
        }

        Ok(())
    }

    pub fn run_webhook(&self, args: WebhookArgs) -> Result<()> {
        match args.command {
            WebhookCommands::Sign(sign) => {
                let secret = sign
                    .secret
                    .or_else(|| self.config.webhook_secret.clone())
                    .ok_or_else(|| {
                        anyhow!("missing webhook secret; set --secret or XENON_WEBHOOK_SECRET")
                    })?;
                let signature = webhook::sign_payload(&secret, &sign.payload)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&WebhookSignatureResponse {
                        algorithm: "hmac-sha256",
                        signature,
                    })?
                );
            }
            WebhookCommands::Verify(verify) => {
                let secret = verify
                    .secret
                    .or_else(|| self.config.webhook_secret.clone())
                    .ok_or_else(|| {
                        anyhow!("missing webhook secret; set --secret or XENON_WEBHOOK_SECRET")
                    })?;
                let valid = webhook::verify_payload(&secret, &verify.payload, &verify.signature)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&WebhookVerifyResponse { valid })?
                );
            }
        }

        Ok(())
    }
}

fn monitor_request(handle: String, events: Vec<EventKindArg>, limit: usize) -> MonitorRequest {
    MonitorRequest {
        handle,
        kinds: events.into_iter().map(EventKind::from).collect(),
        limit,
    }
}

fn map_export_format(format: ExportFormatArg) -> ExportFormat {
    match format {
        ExportFormatArg::Json => ExportFormat::Json,
        ExportFormatArg::Jsonl => ExportFormat::Jsonl,
        ExportFormatArg::Csv => ExportFormat::Csv,
        ExportFormatArg::Markdown => ExportFormat::Markdown,
    }
}
