# Documentation

Welcome to the template-rust documentation!

## 📚 Available Documentation

### Getting Started
- **[Quick Start Guide](QUICK-START.md)** - Get up and running in minutes with your preferred method

### Setup & Configuration
- **[Setup Guide](../SETUP.md)** - Comprehensive setup instructions for all environments
  - Local development
  - Docker & Docker Compose
  - Nix & Nix Flakes
  - GitHub Codespaces & Devcontainer

### Development
- **[CI/CD Documentation](CI-CD.md)** - Complete guide to our GitHub Actions workflows
  - CI workflow (build, test, lint)
  - Security auditing
  - Release automation
  - Docker builds

### Usage
- **[Main README](../README.md)** - Project overview, features, and usage examples

## 🎯 Quick Links

### For Contributors
1. [QUICK-START.md](QUICK-START.md) - Choose your setup method
2. [../SETUP.md](../SETUP.md) - Detailed environment setup
3. [CI-CD.md](CI-CD.md) - Understand our automated workflows

### For Users
1. [../README.md](../README.md) - Learn how to use the application
2. [QUICK-START.md](QUICK-START.md) - Quick reference for common tasks

### For Maintainers
1. [CI-CD.md](CI-CD.md) - Managing workflows and releases
2. [../SETUP.md](../SETUP.md) - Supporting different development environments

## 🔧 Configuration Files Reference

### Docker
- `/Dockerfile` - Multi-stage Docker build
- `/docker-compose.yml` - Development and production services
- `/.dockerignore` - Build optimization

### Nix
- `/flake.nix` - Nix flakes configuration
- `/shell.nix` - Traditional Nix shell
- `/.envrc` - Direnv integration

### Development Environment
- `/.devcontainer/devcontainer.json` - VS Code devcontainer config
- `/Makefile` - Common development commands

### CI/CD
- `/.github/workflows/ci.yml` - Build, test, lint
- `/.github/workflows/security.yml` - Security audits
- `/.github/workflows/release.yml` - Automated releases
- `/.github/workflows/docker.yml` - Docker builds

## 📖 Documentation Standards

When contributing documentation:

1. **Be Clear**: Use simple language and examples
2. **Be Concise**: Respect the reader's time
3. **Be Complete**: Cover edge cases and troubleshooting
4. **Use Examples**: Show, don't just tell
5. **Keep Updated**: Update docs when code changes

## 🤝 Contributing to Docs

Found an issue or want to improve the documentation?

1. Check if there's an [existing issue](https://github.com/pnstack/template-rust/issues)
2. Open a new issue describing the problem
3. Submit a PR with improvements

Good documentation helps everyone! 🎉

## 📞 Questions?

If you can't find what you're looking for:

1. Check all documentation files listed above
2. Search [existing issues](https://github.com/pnstack/template-rust/issues)
3. Open a [new issue](https://github.com/pnstack/template-rust/issues/new) with your question

---

**Thank you for reading! 🦀**
