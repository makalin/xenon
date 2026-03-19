# ⚡ Xenon

**Xenon** is an ultra-high-performance, open-source engine for real-time X (Twitter) data extraction, monitoring, and AI agent integration. Built for developers who need raw speed without the overhead of proprietary platforms.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#)
[![Rust](https://img.shields.io/badge/language-Rust-orange)](#)

## 🚀 Why Xenon?

Current solutions are either closed-source, expensive, or slow. **Xenon** provides a direct, low-latency bridge to the X global stream, optimized for:

- **AI Agents:** Full [MCP (Model Context Protocol)](https://modelcontextprotocol.io) support for Cursor, Claude, and custom LLM workflows.
- **Real-time Monitoring:** HMAC-signed webhooks for instant notifications on tweets, follows, or trends.
- **Bulk Extraction:** High-throughput scraping of profiles, followers, and engagement metrics.
- **Privacy First:** No middleman. You own your data and your API keys.

## 🛠 Features

- **TUI Dashboard:** A retro-inspired terminal interface for managing monitors and viewing live streams.
- **Scalable Architecture:** Built in Rust for maximum concurrency and minimal memory footprint.
- **Unified API:** REST endpoints that simplify complex X v2 API interactions.
- **Advanced Draw Engine:** Automated, verifiable giveaway and contest picker.
- **Zero-Dependency Core:** No React/Next.js bloat. Just pure, compiled performance.

## 📦 Installation

```bash
# Clone the repository
git clone [https://github.com/makalin/xenon.git](https://github.com/makalin/xenon.git)

# Build the project
cd xenon
cargo build --release
```

## 🖥 Usage

### Starting the MCP Server
Integrate Xenon directly into your AI development environment:
```bash
./xenon serve --mcp
```

### Monitoring a Profile
```bash
./xenon monitor @username --events tweets,replies
```

## 🗺 Roadmap

- [ ] Support for Space & Community metadata.
- [ ] Export to CSV/JSONL/Markdown.
- [ ] Distributed proxy rotation for bulk extraction.
- [ ] Plugin system for custom data processors.

## 🤝 Contributing

This is an open-source project by **Mehmet T. AKALIN**. We welcome contributions from the community! Feel free to check the [Issues](https://github.com/makalin/xenon/issues) page.

---

**Digital Vision** | [Website](https://dv.com.tr) | [GitHub](https://github.com/makalin)
