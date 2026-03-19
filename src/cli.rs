use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "xenon", version, about = "Real-time X monitoring toolkit")]
pub struct Cli {
    #[arg(long, env = "XENON_SEED", default_value_t = 7)]
    pub seed: u64,
    #[arg(long, env = "XENON_PROFILE", default_value = "default")]
    pub profile: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serve(ServeArgs),
    Monitor(MonitorArgs),
    Dashboard(DashboardArgs),
    Draw(DrawArgs),
    Export(ExportArgs),
    Analyze(AnalyzeArgs),
    Config(ConfigArgs),
    Webhook(WebhookArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, default_value_t = 8080)]
    pub port: u16,
    #[arg(long, help = "Run the MCP server over stdio instead of the HTTP API")]
    pub mcp: bool,
}

#[derive(Debug, Clone, Args)]
pub struct MonitorArgs {
    pub handle: String,
    #[arg(long, value_delimiter = ',', default_value = "tweets")]
    pub events: Vec<EventKindArg>,
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
    #[arg(long, default_value_t = 350)]
    pub interval_ms: u64,
    #[arg(long, help = "Emit newline-delimited JSON instead of a text feed")]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct DashboardArgs {
    #[arg(long, default_value = "@xenon")]
    pub handle: String,
    #[arg(long, default_value_t = 250)]
    pub tick_ms: u64,
}

#[derive(Debug, Clone, Args)]
pub struct DrawArgs {
    pub input: String,
    #[arg(long, default_value_t = 1)]
    pub count: usize,
}

#[derive(Debug, Clone, Args)]
pub struct ExportArgs {
    pub handle: String,
    #[arg(long, value_delimiter = ',', default_value = "tweets,replies")]
    pub events: Vec<EventKindArg>,
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
    #[arg(long, value_enum, default_value_t = ExportFormatArg::Json)]
    pub format: ExportFormatArg,
    #[arg(long)]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct AnalyzeArgs {
    pub handle: String,
    #[arg(long, value_delimiter = ',', default_value = "tweets,replies")]
    pub events: Vec<EventKindArg>,
    #[arg(long, default_value_t = 10)]
    pub limit: usize,
}

#[derive(Debug, Clone, Args)]
pub struct ConfigArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Args)]
pub struct WebhookArgs {
    #[command(subcommand)]
    pub command: WebhookCommands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum WebhookCommands {
    Sign(WebhookSignArgs),
    Verify(WebhookVerifyArgs),
}

#[derive(Debug, Clone, Args)]
pub struct WebhookSignArgs {
    pub payload: String,
    #[arg(long)]
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct WebhookVerifyArgs {
    pub payload: String,
    pub signature: String,
    #[arg(long)]
    pub secret: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EventKindArg {
    Tweets,
    Replies,
    Follows,
    Trends,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormatArg {
    Json,
    Jsonl,
    Csv,
    Markdown,
}
