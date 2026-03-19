use std::fs;

use anyhow::{Context, Result};

use crate::model::{Event, ExportFormat, ExportResponse};

pub fn render(events: &[Event], format: ExportFormat) -> Result<ExportResponse> {
    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(events)?,
        ExportFormat::Jsonl => render_jsonl(events)?,
        ExportFormat::Csv => render_csv(events),
        ExportFormat::Markdown => render_markdown(events),
    };

    Ok(ExportResponse {
        format,
        content,
        event_count: events.len(),
    })
}

pub fn write_output(path: &str, content: &str) -> Result<()> {
    fs::write(path, content).with_context(|| format!("failed to write export file {path}"))
}

fn render_jsonl(events: &[Event]) -> Result<String> {
    let mut lines = Vec::with_capacity(events.len());
    for event in events {
        lines.push(serde_json::to_string(event)?);
    }
    Ok(lines.join("\n"))
}

fn render_csv(events: &[Event]) -> String {
    let mut rows = vec!["id,handle,kind,score,timestamp,message".to_string()];
    for event in events {
        rows.push(format!(
            "{},{},{},{},{},{}",
            csv_escape(&event.id),
            csv_escape(&event.handle),
            csv_escape(&event.kind.to_string()),
            event.score,
            csv_escape(&event.timestamp.to_rfc3339()),
            csv_escape(&event.message)
        ));
    }
    rows.join("\n")
}

fn render_markdown(events: &[Event]) -> String {
    let mut rows = vec![
        "| id | handle | kind | score | timestamp | message |".to_string(),
        "| --- | --- | --- | ---: | --- | --- |".to_string(),
    ];
    for event in events {
        rows.push(format!(
            "| {} | {} | {} | {} | {} | {} |",
            markdown_escape(&event.id),
            markdown_escape(&event.handle),
            markdown_escape(&event.kind.to_string()),
            event.score,
            markdown_escape(&event.timestamp.to_rfc3339()),
            markdown_escape(&event.message)
        ));
    }
    rows.join("\n")
}

fn csv_escape(value: &str) -> String {
    let escaped = value.replace('"', "\"\"");
    format!("\"{escaped}\"")
}

fn markdown_escape(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::Utc;

    use crate::model::{Event, EventKind, ExportFormat};

    use super::render;

    #[test]
    fn renders_csv() -> Result<()> {
        let events = vec![Event {
            id: "1".to_string(),
            handle: "@x".to_string(),
            kind: EventKind::Tweet,
            message: "hello".to_string(),
            score: 9,
            timestamp: Utc::now(),
        }];

        let export = render(&events, ExportFormat::Csv)?;

        assert!(export
            .content
            .contains("id,handle,kind,score,timestamp,message"));
        assert!(export.content.contains("\"@x\""));
        Ok(())
    }
}
