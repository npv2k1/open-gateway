# Template Rust

A Rust project template featuring a todo application with SQLite database and terminal user interface (TUI).

## Features

- 📝 Todo management with SQLite persistence
- 🖥️ Interactive Terminal User Interface (TUI)
- 🔧 Command Line Interface (CLI)
- 🧪 Comprehensive test suite
- 🚀 CI/CD with GitHub Actions
- 📦 Cross-platform releases
- 🔒 Security auditing
- 🐳 Docker and Docker Compose support
- ❄️ Nix flakes for reproducible environments
- 📦 Devcontainer configuration for GitHub Codespaces

## Installation

> **💡 Quick Start**: See [SETUP.md](SETUP.md) for detailed setup instructions using Docker, Nix, Codespaces, or local development.

### From Source

```bash
git clone https://github.com/pnstack/template-rust.git
cd template-rust
cargo build --release
```

### From Releases

Download the latest binary from the [Releases](https://github.com/pnstack/template-rust/releases) page.

### With Docker

```bash
# Build the image
docker build -t template-rust:latest .

# Run with interactive TUI
docker run --rm -it -v $(pwd)/data:/app/data template-rust:latest tui

# Or use Docker Compose
docker compose up
```

### With Nix

```bash
# Enter development environment
nix develop

# Or run directly
nix run
```

### With GitHub Codespaces

Click the "Code" button on GitHub and select "Create codespace on main" - everything is pre-configured!

## Usage

### Command Line Interface

```bash
# Show help
./template-rust --help

# Add a new todo
./template-rust add "Buy groceries" --description "Milk, eggs, bread"

# List all todos
./template-rust list

# List only completed todos
./template-rust list --completed

# List only pending todos
./template-rust list --pending

# Complete a todo (use the ID from list command)
./template-rust complete <todo-id>

# Delete a todo
./template-rust delete <todo-id>

# Start interactive TUI (default mode)
./template-rust tui
```

### Terminal User Interface (TUI)

Start the interactive mode:

```bash
./template-rust tui
```

#### TUI Commands:
- `h` - Show help
- `n` - Add new todo
- `d` - Delete selected todo
- `c` - Toggle todo completion status
- `a` - Show all todos
- `p` - Show pending todos only
- `f` - Show completed todos only
- `↑↓` - Navigate todos
- `q` - Quit application

## Project Structure

```
template-rust/
├── .github/workflows/    # CI/CD workflows
├── src/
│   ├── database/         # Database layer
│   ├── models/           # Data models
│   ├── tui/              # Terminal UI
│   ├── lib.rs            # Library root
│   └── main.rs           # CLI application
├── tests/                # Integration tests
├── docs/                 # Documentation
└── examples/             # Usage examples
```

## Development

> **📚 Full Setup Guide**: See [SETUP.md](SETUP.md) for comprehensive development environment setup instructions.

### Prerequisites

Choose your preferred development method:

- **Local**: Rust 1.70 or later, SQLite3
- **Docker**: Docker 20.10+ and Docker Compose
- **Nix**: Nix package manager with flakes enabled
- **Codespaces**: Just a GitHub account!

### Building

```bash
# Local
cargo build

# Docker
docker compose up --build

# Nix
nix build
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

### Development Environments

The project provides multiple development environment options:

- **Docker Compose**: `docker compose up dev` - Containerized development with live code mounting
- **Nix Flakes**: `nix develop` - Reproducible environment with all dependencies
- **Devcontainer**: Open in VS Code or GitHub Codespaces - Fully configured IDE
- **Traditional**: Local Rust installation with cargo

## Database

The application uses SQLite for persistence. By default, it creates a `todo.db` file in the current directory. You can specify a different database path:

```bash
./template-rust --database /path/to/your/todos.db list
```

For testing with in-memory database:

```bash
./template-rust --database ":memory:" add "Test todo"
```

## CI/CD

The project includes comprehensive GitHub Actions workflows:

- **CI** (`ci.yml`): Build, test, lint, and format checks on multiple platforms (Linux, macOS, Windows)
- **Security** (`security.yml`): Weekly security audits with `cargo audit`
- **Release** (`release.yml`): Automated binary releases for Linux, macOS, and Windows on version tags
- **Docker** (`docker.yml`): Docker image build testing and docker-compose validation

All workflows run automatically on push and pull requests to ensure code quality and security.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
