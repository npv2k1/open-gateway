# Setup Guide

This guide will help you set up the development environment for template-rust using various methods.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Quick Start](#quick-start)
- [Setup Methods](#setup-methods)
  - [Local Development](#local-development)
  - [Docker](#docker)
  - [Docker Compose](#docker-compose)
  - [Nix](#nix)
  - [GitHub Codespaces / Devcontainer](#github-codespaces--devcontainer)
- [Building the Project](#building-the-project)
- [Running Tests](#running-tests)
- [Common Issues](#common-issues)

## Prerequisites

Choose one of the following setup methods based on your preference:

- **Local Development**: Rust 1.70+, SQLite3
- **Docker**: Docker 20.10+ and Docker Compose (optional)
- **Nix**: Nix package manager with flakes enabled
- **Codespaces**: GitHub account (no local setup required)

## Quick Start

The fastest way to get started depends on your environment:

```bash
# Local development
cargo run -- --help

# Docker
docker compose up

# Nix (with flakes)
nix develop
cargo run

# GitHub Codespaces
# Just open the repository in Codespaces - everything is preconfigured!
```

## Setup Methods

### Local Development

#### Installation

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Install SQLite** (if not already installed):
   
   - **Ubuntu/Debian**:
     ```bash
     sudo apt-get update
     sudo apt-get install sqlite3 libsqlite3-dev
     ```
   
   - **macOS**:
     ```bash
     brew install sqlite
     ```
   
   - **Windows**:
     Download from [SQLite Download Page](https://www.sqlite.org/download.html)

3. **Clone and build**:
   ```bash
   git clone https://github.com/pnstack/template-rust.git
   cd template-rust
   cargo build --release
   ```

4. **Run the application**:
   ```bash
   cargo run -- --help
   ```

#### Development Tools

Install additional development tools:

```bash
# Format checker
rustup component add rustfmt

# Linter
rustup component add clippy

# IDE support
rustup component add rust-analyzer
```

### Docker

#### Building and Running

1. **Build the Docker image**:
   ```bash
   docker build -t template-rust:latest .
   ```

2. **Run the container**:
   ```bash
   # Run with help
   docker run --rm template-rust:latest --help

   # Run TUI mode (interactive)
   docker run --rm -it -v $(pwd)/data:/app/data template-rust:latest tui

   # Run CLI commands
   docker run --rm -v $(pwd)/data:/app/data template-rust:latest list
   ```

3. **Persist data**:
   
   The container stores data in `/app/data`. Mount a volume to persist data:
   ```bash
   mkdir -p data
   docker run --rm -it -v $(pwd)/data:/app/data template-rust:latest tui
   ```

### Docker Compose

Docker Compose simplifies multi-service development and provides predefined configurations.

#### Running with Docker Compose

1. **Start the application**:
   ```bash
   docker compose up template-rust
   ```

2. **Start in development mode**:
   ```bash
   # Development mode with live code mounting
   docker compose up dev
   ```

3. **Build and run**:
   ```bash
   docker compose up --build
   ```

4. **Run in detached mode**:
   ```bash
   docker compose up -d
   ```

5. **Stop services**:
   ```bash
   docker compose down
   ```

#### Development Workflow

The `dev` service in docker-compose.yml provides:
- Live code mounting for rapid development
- Cargo cache for faster builds
- Interactive terminal access

```bash
# Enter development container
docker compose run --rm dev bash

# Inside container
cargo build
cargo test
cargo run -- tui
```

### Nix

Nix provides reproducible development environments across different systems.

#### Using Nix Flakes (Recommended)

1. **Enable flakes** (if not already enabled):
   ```bash
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

2. **Enter development environment**:
   ```bash
   nix develop
   ```

   This provides:
   - Rust toolchain with rust-analyzer
   - All build dependencies
   - Development tools

3. **Build the project**:
   ```bash
   nix build
   ```

4. **Run the application**:
   ```bash
   nix run
   ```

#### Using Traditional Nix Shell

If you prefer not to use flakes:

```bash
nix-shell
```

This provides the same development environment using `shell.nix`.

#### Direnv Integration (Optional)

For automatic environment activation:

1. **Install direnv**:
   ```bash
   # Via Nix
   nix-env -iA nixpkgs.direnv
   
   # Via package manager
   # Ubuntu/Debian: sudo apt install direnv
   # macOS: brew install direnv
   ```

2. **Configure direnv**:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   eval "$(direnv hook bash)"  # or zsh
   ```

3. **Create .envrc**:
   ```bash
   echo "use flake" > .envrc
   direnv allow
   ```

Now the environment activates automatically when you enter the directory!

### GitHub Codespaces / Devcontainer

The easiest way to get started with zero local setup.

#### Using GitHub Codespaces

1. **Open in Codespaces**:
   - Navigate to the repository on GitHub
   - Click "Code" → "Codespaces" → "Create codespace on main"
   - Wait for the environment to build (first time only)

2. **Start developing**:
   - All tools are pre-installed
   - VS Code with Rust extensions ready
   - Project automatically builds on creation

#### Using Local Devcontainer

If you have Docker and VS Code locally:

1. **Install prerequisites**:
   - Docker Desktop
   - VS Code
   - "Dev Containers" extension for VS Code

2. **Open in container**:
   - Open the repository in VS Code
   - Press `F1` → "Dev Containers: Reopen in Container"
   - Wait for container to build

3. **Start developing**:
   - Integrated terminal has all tools
   - Extensions auto-installed
   - Same environment as Codespaces

## Building the Project

### Development Build

```bash
cargo build
```

### Release Build (Optimized)

```bash
cargo build --release
```

The binary will be in `target/release/template-rust`.

### Checking Code Without Building

```bash
cargo check
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test

```bash
cargo test test_name
```

### With Output

```bash
cargo test -- --nocapture
```

### Integration Tests Only

```bash
cargo test --test integration_tests
```

## Code Quality

### Format Code

```bash
cargo fmt
```

### Check Formatting

```bash
cargo fmt --check
```

### Run Linter

```bash
cargo clippy
```

### Run Linter (Strict)

```bash
cargo clippy -- -D warnings
```

## Common Issues

### Issue: SQLite not found

**Solution**: Install SQLite development libraries

```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# macOS
brew install sqlite

# Nix
nix develop  # Automatically provides SQLite
```

### Issue: OpenSSL not found

**Solution**: Install OpenSSL development libraries

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"

# Nix
nix develop  # Automatically provides OpenSSL
```

### Issue: Cargo is slow on first build

**Solution**: This is normal - Cargo downloads and compiles all dependencies on first build. Subsequent builds are much faster due to caching.

For Docker users, the multi-stage build and layer caching help reduce rebuild times.

### Issue: Permission denied in Docker

**Solution**: The container runs as non-root user. Ensure mounted volumes have appropriate permissions:

```bash
mkdir -p data
chmod 777 data  # Or use appropriate user/group ownership
```

### Issue: Database locked

**Solution**: Close any other instances of the application accessing the same database file, or use a different database path:

```bash
./template-rust --database another.db tui
```

## Environment Variables

The application supports the following environment variables:

- `DATABASE_URL`: Path to the SQLite database file (default: `todo.db`)
- `RUST_BACKTRACE`: Set to `1` or `full` for detailed error traces
- `RUST_LOG`: Set log level (e.g., `debug`, `info`, `warn`, `error`)

Example:

```bash
export DATABASE_URL=":memory:"  # Use in-memory database
export RUST_BACKTRACE=1          # Enable backtraces
cargo run
```

## Next Steps

After setting up your environment:

1. Read the [README.md](README.md) for usage instructions
2. Explore the [examples/](examples/) directory
3. Check the [tests/](tests/) directory for test examples
4. Review the [CI/CD workflows](.github/workflows/) to understand the automation

## Getting Help

If you encounter issues:

1. Check this guide's [Common Issues](#common-issues) section
2. Search existing [GitHub Issues](https://github.com/pnstack/template-rust/issues)
3. Open a new issue with:
   - Your setup method (Local/Docker/Nix/Codespaces)
   - Operating system and version
   - Rust version (`rustc --version`)
   - Complete error message
   - Steps to reproduce

## Contributing

See [README.md](README.md#contributing) for contribution guidelines.
