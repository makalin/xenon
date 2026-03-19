use std::io::{self, BufRead, Write};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::exporter;
use crate::model::{EventKind, ExportFormat, MonitorRequest, WebhookSignatureResponse};
use crate::monitor::MonitorService;
use crate::{analytics, webhook};

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    id: serde_json::Value,
    result: serde_json::Value,
}

pub async fn run_stdio(service: MonitorService) -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line.context("failed to read stdin")?;
        if line.trim().is_empty() {
            continue;
        }

        let request: RpcRequest = serde_json::from_str(&line).context("invalid rpc request")?;
        info!(method = %request.method, "received MCP request");

        let result = match request.method.as_str() {
            "initialize" => json!({
                "name": "xenon",
                "version": env!("CARGO_PKG_VERSION"),
                "capabilities": {
                    "tools": true
                }
            }),
            "tools/list" => json!({
                "tools": [
                    {
                        "name": "monitor_profile",
                        "description": "Fetch recent tweet and reply events for a monitored X profile"
                    },
                    {
                        "name": "analyze_profile",
                        "description": "Summarize recent X profile events with counts and score totals"
                    },
                    {
                        "name": "export_profile",
                        "description": "Export recent X profile events in json, jsonl, csv, or markdown"
                    },
                    {
                        "name": "sign_webhook",
                        "description": "Generate an HMAC-SHA256 signature for webhook payloads"
                    }
                ]
            }),
            "tools/call" => {
                let params = request.params.unwrap_or_default();
                let tool_name = params
                    .get("name")
                    .and_then(|value| value.as_str())
                    .unwrap_or("monitor_profile");

                match run_tool(tool_name, &params, &service).await {
                    Ok(result) => result,
                    Err(error) => json!({ "error": error.to_string() }),
                }
            }
            _ => json!({
                "error": format!("unknown method: {}", request.method)
            }),
        };

        let response = RpcResponse {
            id: request.id,
            result,
        };

        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

async fn run_tool(
    tool_name: &str,
    params: &serde_json::Value,
    service: &MonitorService,
) -> Result<serde_json::Value> {
    match tool_name {
        "monitor_profile" => {
            let request = monitor_request_from_params(params);
            let events = service.generate_batch(&request).await?;
            Ok(json!({ "events": events }))
        }
        "analyze_profile" => {
            let request = monitor_request_from_params(params);
            let events = service.generate_batch(&request).await?;
            Ok(json!(analytics::summarize(&events)))
        }
        "export_profile" => {
            let request = monitor_request_from_params(params);
            let format = match params
                .get("format")
                .and_then(|value| value.as_str())
                .unwrap_or("json")
            {
                "json" => ExportFormat::Json,
                "jsonl" => ExportFormat::Jsonl,
                "csv" => ExportFormat::Csv,
                "markdown" => ExportFormat::Markdown,
                other => {
                    return Ok(json!({ "error": format!("unsupported export format: {other}") }))
                }
            };
            let events = service.generate_batch(&request).await?;
            Ok(json!(exporter::render(&events, format)?))
        }
        "sign_webhook" => {
            let secret = params
                .get("secret")
                .and_then(|value| value.as_str())
                .ok_or_else(|| anyhow!("missing secret"))?;
            let payload = params
                .get("payload")
                .and_then(|value| value.as_str())
                .unwrap_or_default();
            let signature = webhook::sign_payload(secret, payload)?;
            Ok(json!(WebhookSignatureResponse {
                algorithm: "hmac-sha256",
                signature,
            }))
        }
        _ => Ok(json!({ "error": format!("unknown tool: {tool_name}") })),
    }
}

fn monitor_request_from_params(params: &serde_json::Value) -> MonitorRequest {
    let handle = params
        .get("handle")
        .and_then(|value| value.as_str())
        .unwrap_or("@xenon");
    let limit = params
        .get("limit")
        .and_then(|value| value.as_u64())
        .unwrap_or(5) as usize;

    MonitorRequest {
        handle: handle.to_string(),
        kinds: vec![EventKind::Tweet, EventKind::Reply],
        limit,
    }
}
