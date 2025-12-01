# Quick Start Guide

Choose your preferred development method and get started in minutes!

## 🚀 Fastest Methods

### GitHub Codespaces (Zero Setup!)
1. Click "Code" → "Codespaces" → "Create codespace on main"
2. Wait ~2 minutes for setup
3. Start coding!

**Best for:** Trying the project, contributing without local setup

---

### Docker Compose (Simple)
```bash
git clone https://github.com/pnstack/template-rust.git
cd template-rust
docker compose up
```

**Best for:** Quick testing, isolated environments

---

### Nix (Reproducible)
```bash
git clone https://github.com/pnstack/template-rust.git
cd template-rust
nix develop
cargo run
```

**Best for:** Guaranteed reproducible builds, NixOS users

---

### Local Development (Traditional)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/pnstack/template-rust.git
cd template-rust
cargo build --release
```

**Best for:** Full control, native performance

---

## 📋 Common Commands

### Using Make (Recommended)
```bash
make help          # Show all available commands
make build         # Build debug version
make test          # Run tests
make clippy        # Run linter
make fmt           # Format code
make all           # Format, lint, test, and build
```

### Using Cargo
```bash
cargo build                      # Debug build
cargo build --release            # Optimized build
cargo test                       # Run tests
cargo clippy -- -D warnings      # Lint
cargo fmt                        # Format
```

### Using Docker
```bash
docker build -t template-rust .                    # Build image
docker run --rm template-rust --help               # Show help
docker run --rm -it template-rust tui              # Interactive TUI
```

### Using Docker Compose
```bash
docker compose up                  # Start app
docker compose up dev              # Development mode
docker compose down                # Stop services
```

---

## 🎯 First Steps After Setup

1. **Run the application:**
   ```bash
   cargo run -- --help
   ```

2. **Try the TUI:**
   ```bash
   cargo run -- tui
   ```

3. **Add a todo:**
   ```bash
   cargo run -- add "My first task"
   ```

4. **List todos:**
   ```bash
   cargo run -- list
   ```

5. **Run tests:**
   ```bash
   cargo test
   ```

---

## 🔍 Need More Details?

- **Full setup instructions:** [SETUP.md](../SETUP.md)
- **Usage examples:** [README.md](../README.md)
- **CI/CD info:** [CI-CD.md](CI-CD.md)

---

## ❓ Troubleshooting

### "SQLite not found"
```bash
# Ubuntu/Debian
sudo apt-get install libsqlite3-dev

# macOS
brew install sqlite

# Nix/Codespaces - Already included!
```

### "OpenSSL not found"
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config

# macOS
brew install openssl

# Nix/Codespaces - Already included!
```

### "Slow first build"
This is normal! Cargo compiles all dependencies on first build.
Subsequent builds are much faster (seconds instead of minutes).

### Docker permission issues
```bash
# Create data directory with proper permissions
mkdir -p data
chmod 777 data
```

---

## 🎓 Learning Resources

### Rust Basics
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Project-Specific
- Explore [src/](../src/) for code organization
- Check [tests/](../tests/) for testing examples
- Read [examples/](../examples/) for usage patterns

### Development Tools
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)
- [Rustfmt Configuration](https://rust-lang.github.io/rustfmt/)

---

## 💡 Tips

1. **Use rust-analyzer**: Best IDE support for Rust
2. **Run clippy often**: Catches bugs early
3. **Format before commit**: `cargo fmt` or `make fmt`
4. **Test frequently**: `cargo test` or `make test`
5. **Use `--release` for production**: Much faster

---

## 🤝 Contributing

Ready to contribute?

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Make changes
4. Test: `make all`
5. Commit: `git commit -m "Add feature"`
6. Push: `git push origin feature/my-feature`
7. Open a Pull Request

---

## 📞 Getting Help

- 📖 Read the [full docs](../README.md)
- 🐛 [Open an issue](https://github.com/pnstack/template-rust/issues)
- 💬 Check [existing issues](https://github.com/pnstack/template-rust/issues?q=is%3Aissue)

---

**Happy coding! 🦀**
