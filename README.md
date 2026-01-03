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
- 🌐 **Multiple Servers**: Run multiple gateway servers on different ports, each with its own routes
- 🔒 **HTTP/HTTPS Support**: Proxy to both HTTP and HTTPS backend targets
- 🔄 **Hot Reload**: Automatically reload configuration on file changes

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

5. Or start the gateway with hot reload (auto-reload on config changes):

```bash
./open-gateway start -c config.toml --watch
```

6. Or start the TUI monitor:

```bash
./open-gateway monitor -c config.toml
```

## CLI Commands

```bash
# Show help
./open-gateway --help

# Start the gateway server
./open-gateway start -c config.toml

# Start the gateway server with hot reload
./open-gateway start -c config.toml --watch

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
# Option 1: Single server (backward compatible)
[server]
host = "0.0.0.0"
port = 8080
timeout = 30

# Option 2: Multiple servers (each with optional route references)
[[servers]]
name = "api-server"
host = "0.0.0.0"
port = 8080
timeout = 30
routes = ["api-v1"]  # Reference routes by name

[[servers]]
name = "admin-server"
host = "0.0.0.0"
port = 9090
# No routes = uses all routes

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
name = "api-v1"  # Optional name for server references
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

#### Server (Single)

| Option | Description | Default |
|--------|-------------|---------|
| `host` | Host to bind to | `0.0.0.0` |
| `port` | Port to bind to | `8080` |
| `timeout` | Request timeout in seconds | `30` |

#### Servers (Multiple)

Use `[[servers]]` to configure multiple servers. Each server can have its own set of routes.

| Option | Description | Default |
|--------|-------------|---------|
| `name` | Server name (for display) | `host:port` |
| `host` | Host to bind to | `0.0.0.0` |
| `port` | Port to bind to | `8080` |
| `timeout` | Request timeout in seconds | `30` |
| `routes` | List of route names/paths to use | All routes |

**Note:** If `routes` is not specified or empty, the server will use all enabled routes.

#### Routes

| Option | Description | Required |
|--------|-------------|----------|
| `name` | Route name (for server references) | No |
| `path` | Path pattern (supports `*` wildcard) | Yes |
| `target` | Target URL (HTTP or HTTPS) | Yes |
| `strip_prefix` | Strip matched prefix from path | No (default: false) |
| `methods` | HTTP methods to match (empty = all) | No |
| `api_key_pool` | API key pool name to use | No |
| `headers` | Additional headers to add | No |
| `description` | Route description | No |
| `enabled` | Whether route is enabled | No (default: true) |

**Note:** The `target` field supports both `http://` and `https://` URLs for proxying to HTTP or HTTPS backends.

#### API Key Pools

| Option | Description | Default |
|--------|-------------|---------|
| `strategy` | Selection strategy | `round_robin` |
| `header_name` | Header name for API key (used when injecting as header) | `Authorization` |
| `query_param_name` | Query parameter name for API key (used when injecting as query param) | None |
| `keys` | List of API keys | Required |

**Note:** If `query_param_name` is set, the API key will be injected into the target URL's query parameters instead of as a header.

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
- `gateway_api_key_usage_total`: Total number of requests per API key (labels: api_key (hashed), route)

**Note**: API keys are hashed before being used as metric labels to protect sensitive credentials while maintaining observability.

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
