# Open Gateway

A simple and fast API gateway service with API key management and TUI monitoring.

## Features

- 🚀 **High Performance**: Built with Rust and Axum for optimal performance
- 🔧 **TOML Configuration**: Easy-to-read configuration files
- 🔑 **API Key Pool Management**: Support for multiple API key selection strategies
  - Round Robin
  - Random
  - Weighted
- 📊 **Prometheus Metrics**: Built-in metrics endpoint for monitoring
- ❤️ **Health Checks**: Liveness and readiness endpoints
- 🖥️ **TUI Monitor**: Terminal-based dashboard for monitoring
- 🛣️ **Flexible Routing**: Path-based routing with prefix stripping

## Installation

### From Source

```bash
git clone https://github.com/npv2k1/open-gateway.git
cd open-gateway
cargo build --release
```

### From Releases

Download the latest binary from the [Releases](https://github.com/npv2k1/open-gateway/releases) page.

## Quick Start

1. Generate a sample configuration file:

```bash
./open-gateway init -o config.toml
```

2. Edit the configuration file to match your needs.

3. Validate the configuration:

```bash
./open-gateway validate -c config.toml
```

4. Start the gateway:

```bash
./open-gateway start -c config.toml
```

5. Or start the TUI monitor:

```bash
./open-gateway monitor -c config.toml
```

## CLI Commands

```bash
# Show help
./open-gateway --help

# Start the gateway server
./open-gateway start -c config.toml

# Start the TUI monitor
./open-gateway monitor -c config.toml

# Validate configuration
./open-gateway validate -c config.toml

# Generate sample configuration
./open-gateway init -o config.toml
```

## Configuration

Open Gateway uses TOML configuration files. Here's an example:

```toml
# Server configuration
[server]
host = "0.0.0.0"
port = 8080
timeout = 30

# Metrics configuration
[metrics]
enabled = true
path = "/metrics"

# Health check configuration
[health]
enabled = true
path = "/health"

# Route configurations
[[routes]]
path = "/api/v1/*"
target = "http://localhost:3001"
strip_prefix = true
methods = ["GET", "POST", "PUT", "DELETE"]
api_key_pool = "default"
description = "API v1 routes"
enabled = true

# API Key Pools
[api_key_pools.default]
strategy = "round_robin"  # Options: round_robin, random, weight
header_name = "X-API-Key"
keys = [
    { key = "api-key-1", weight = 1, enabled = true },
    { key = "api-key-2", weight = 2, enabled = true },
]
```

### Configuration Options

#### Server

| Option | Description | Default |
|--------|-------------|---------|
| `host` | Host to bind to | `0.0.0.0` |
| `port` | Port to bind to | `8080` |
| `timeout` | Request timeout in seconds | `30` |

#### Routes

| Option | Description | Required |
|--------|-------------|----------|
| `path` | Path pattern (supports `*` wildcard) | Yes |
| `target` | Target URL to forward requests | Yes |
| `strip_prefix` | Strip matched prefix from path | No (default: false) |
| `methods` | HTTP methods to match (empty = all) | No |
| `api_key_pool` | API key pool name to use | No |
| `headers` | Additional headers to add | No |
| `description` | Route description | No |
| `enabled` | Whether route is enabled | No (default: true) |

#### API Key Pools

| Option | Description | Default |
|--------|-------------|---------|
| `strategy` | Selection strategy | `round_robin` |
| `header_name` | Header name for API key | `Authorization` |
| `keys` | List of API keys | Required |

##### API Key Configuration

| Option | Description | Default |
|--------|-------------|---------|
| `key` | The API key value | Required |
| `weight` | Weight for weighted selection | `1` |
| `enabled` | Whether key is enabled | `true` |

## Metrics

The gateway exposes Prometheus metrics at the `/metrics` endpoint (configurable):

- `gateway_requests_total`: Total number of requests (labels: method, path, status)
- `gateway_request_latency_seconds`: Request latency histogram (labels: method, path)
- `gateway_active_connections`: Number of active connections (labels: route)

## Health Checks

The gateway provides health check endpoints:

- `GET /health`: Returns health status in JSON format

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

## TUI Monitor

The TUI monitor provides a terminal-based dashboard with:

- **Overview Tab**: Metrics summary and health status
- **Routes Tab**: List of configured routes with details
- **Config Tab**: Current configuration overview
- **Help Tab**: Keyboard shortcuts and documentation

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` / `→` | Next tab |
| `Shift+Tab` / `←` | Previous tab |
| `1-4` | Jump to tab |
| `h` | Help tab |
| `↑` / `k` | Previous route (in Routes tab) |
| `↓` / `j` | Next route (in Routes tab) |
| `q` / `Esc` | Quit |

## Development

### Prerequisites

- Rust 1.70 or later

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Clippy (Linter)

```bash
cargo clippy -- -D warnings
```

### Formatting Code

```bash
cargo fmt
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
