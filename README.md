# Xenon

Xenon is a Rust CLI and service skeleton for real-time X monitoring workflows. This repository now contains a working executable with:

- an HTTP API
- an MCP-compatible stdio mode
- a streaming monitor command
- a terminal dashboard
- a deterministic giveaway picker
- analytics and export tooling
- webhook signing and verification helpers
- runtime config inspection

The monitor pipeline now uses the live X API v2 recent-search endpoint. You must provide a bearer token before `serve`, `monitor`, or `dashboard` can fetch events.

## Environment

```bash
export X_BEARER_TOKEN=your_token_here
```

Optional overrides:

```bash
export XENON_X_BEARER_TOKEN=your_token_here
export XENON_X_API_BASE_URL=https://api.x.com/2
export XENON_REQUEST_TIMEOUT_SECONDS=15
export XENON_WEBHOOK_SECRET=supersecret
```

## Build

```bash
cargo build
```

## Run

Start the HTTP API:

```bash
cargo run -- serve
```

Start MCP stdio mode:

```bash
cargo run -- serve --mcp
```

Monitor a handle:

```bash
cargo run -- monitor @username --events tweets,replies --limit 5
```

Export recent events:

```bash
cargo run -- export @username --events tweets,replies --limit 20 --format markdown --output report.md
```

Analyze a profile:

```bash
cargo run -- analyze @username --events tweets,replies --limit 20
```

Launch the dashboard:

```bash
cargo run -- dashboard --handle @username
```

Pick giveaway winners from a newline-delimited file:

```bash
cargo run -- draw data/entrants.txt --count 2
```

Inspect runtime config:

```bash
cargo run -- config --json
```

Sign and verify webhook payloads:

```bash
cargo run -- webhook sign '{"event":"ping"}' --secret supersecret
cargo run -- webhook verify '{"event":"ping"}' <signature> --secret supersecret
```

## API

- `GET /health`
- `GET /api/v1/config`
- `POST /api/v1/monitors`
- `POST /api/v1/events`
- `POST /api/v1/analyze`
- `POST /api/v1/export`
- `POST /api/v1/webhook/sign`
- `POST /api/v1/webhook/verify`

Example body:

```json
{
  "handle": "@xenon",
  "kinds": ["tweet", "reply"],
  "limit": 5
}
```

## Notes

- `serve --mcp` accepts newline-delimited JSON-RPC requests over stdio.
- `tweet` maps to `from:<username> -is:reply -is:retweet`.
- `reply` maps to `from:<username> is:reply`.
- `follow` and `trend` are rejected because they are not exposed through the bearer-token X v2 endpoints used here.
- Export formats supported: `json`, `jsonl`, `csv`, `markdown`.
- Webhook utilities use `hmac-sha256`.
