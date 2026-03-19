mod analytics;
mod api;
mod app;
mod cli;
mod config;
mod draw;
mod exporter;
mod mcp;
mod model;
mod monitor;
mod tui;
mod webhook;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::app::App;
use crate::cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    let app = App::bootstrap(&cli)?;

    match cli.command {
        Commands::Serve(args) => app.run_server(args).await,
        Commands::Monitor(args) => app.run_monitor(args).await,
        Commands::Dashboard(args) => app.run_dashboard(args).await,
        Commands::Draw(args) => app.run_draw(args).await,
        Commands::Export(args) => app.run_export(args).await,
        Commands::Analyze(args) => app.run_analyze(args).await,
        Commands::Config(args) => app.run_config(args),
        Commands::Webhook(args) => app.run_webhook(args),
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,xenon=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
