# CI/CD Documentation

This document describes the Continuous Integration and Continuous Deployment (CI/CD) workflows configured for this project.

## Overview

The project uses GitHub Actions for automated testing, building, and releasing. All workflows are located in `.github/workflows/`.

## Workflows

### 1. CI Workflow (`ci.yml`)

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch

**Jobs:**

#### Test
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Install Rust stable
  3. Cache dependencies (Cargo registry, git, and target directory)
  4. Run `cargo test --verbose`

#### Clippy (Linter)
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Install Rust stable with clippy component
  3. Cache dependencies
  4. Run `cargo clippy -- -D warnings` (fails on any warnings)

#### Rustfmt (Code Formatting)
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Install Rust stable with rustfmt component
  3. Run `cargo fmt --all -- --check` (fails if code is not formatted)

#### Build
- Runs on: Matrix of OS (Ubuntu, Windows, macOS)
- Steps:
  1. Checkout code
  2. Install Rust stable
  3. Cache dependencies
  4. Run `cargo build --verbose --release`

**Purpose:** Ensures code quality, passes tests, follows formatting standards, and builds successfully on all target platforms.

### 2. Security Workflow (`security.yml`)

**Triggers:**
- Weekly schedule (Sunday at 00:00 UTC)
- Push to `main` branch
- Pull requests to `main` branch

**Jobs:**

#### Security Audit
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Install Rust stable
  3. Install `cargo-audit`
  4. Cache dependencies
  5. Run `cargo audit` to check for security vulnerabilities in dependencies

**Purpose:** Regularly scans dependencies for known security vulnerabilities and alerts on any issues.

### 3. Release Workflow (`release.yml`)

**Triggers:**
- Push of tags matching `v*.*.*` (e.g., `v0.1.0`, `v1.2.3`)

**Jobs:**

#### Build and Release
- Runs on: Matrix of OS and targets
- Targets:
  - Linux: `x86_64-unknown-linux-gnu`
  - Windows: `x86_64-pc-windows-msvc`
  - macOS: `x86_64-apple-darwin`
- Steps:
  1. Checkout code
  2. Install Rust stable with target
  3. Cache dependencies (per target)
  4. Build release binary: `cargo build --release --target <target>`
  5. Upload artifact with platform-specific name

#### Create Release
- Depends on: build-and-release
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Download all artifacts
  3. Create GitHub release with:
     - All platform binaries
     - Auto-generated release notes
     - Tag information

**Purpose:** Automatically builds and publishes release binaries for all supported platforms when a version tag is pushed.

**Usage:**
```bash
# Create a release
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0

# The workflow will automatically:
# 1. Build binaries for Linux, macOS, and Windows
# 2. Create a GitHub release
# 3. Upload binaries as release assets
```

### 4. Docker Workflow (`docker.yml`)

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch

**Jobs:**

#### Docker Build Test
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Set up Docker Buildx
  3. Build Docker image with cache
  4. Test Docker image (run help command)

**Purpose:** Ensures Docker image builds successfully and can run basic commands.

#### Docker Compose Validation
- Runs on: Ubuntu Latest
- Steps:
  1. Checkout code
  2. Validate `docker-compose.yml` syntax

**Purpose:** Ensures docker-compose configuration is valid.

## Caching Strategy

All workflows use GitHub Actions caching to speed up builds:

```yaml
path: |
  ~/.cargo/registry
  ~/.cargo/git
  target
key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

This caches:
- Cargo package registry
- Git repositories of dependencies
- Build artifacts

The cache is invalidated when `Cargo.lock` changes (i.e., when dependencies are updated).

## Branch Protection

To ensure code quality, consider enabling branch protection rules for `main`:

1. **Require pull request reviews**: At least 1 approval
2. **Require status checks**: All CI jobs must pass
3. **Require branches to be up to date**: Prevent merge conflicts
4. **Require linear history**: Keep history clean

## Adding New Workflows

### Example: Add a Coverage Workflow

Create `.github/workflows/coverage.yml`:

```yaml
name: Coverage

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
      
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml
```

## Best Practices

### 1. Keep Workflows Fast
- Use caching extensively
- Run jobs in parallel when possible
- Use matrix builds for multiple platforms

### 2. Fail Fast
- Use `-- -D warnings` for clippy to catch issues early
- Run format checks before heavy builds
- Use `--check` flags for validation-only steps

### 3. Security
- Never commit secrets to workflows
- Use GitHub Secrets for sensitive data
- Regularly update dependencies with security patches
- Use `cargo audit` to catch vulnerabilities

### 4. Versioning
- Use semantic versioning (SemVer) for releases
- Document breaking changes in release notes
- Test thoroughly before tagging releases

### 5. Documentation
- Update release notes automatically
- Keep workflow documentation current
- Document any manual release steps

## Troubleshooting

### Build Failures

**Problem:** Build fails on specific platform

**Solution:**
1. Check the workflow logs for the specific error
2. Test locally on that platform if possible
3. Check for platform-specific dependencies
4. Review recent changes that might affect that platform

### Cache Issues

**Problem:** Builds are slower than expected

**Solution:**
1. Check if cache is being hit (look for "Cache restored" in logs)
2. Verify cache key includes `Cargo.lock` hash
3. Clear cache if corrupted (manually delete in GitHub UI)

### Release Failures

**Problem:** Release workflow fails to create release

**Solution:**
1. Verify tag follows `v*.*.*` format
2. Check `GITHUB_TOKEN` permissions
3. Ensure all build jobs completed successfully
4. Verify artifact names match expected patterns

### Security Audit Failures

**Problem:** `cargo audit` finds vulnerabilities

**Solution:**
1. Review the security advisory
2. Update affected dependencies: `cargo update`
3. If no fix available, consider:
   - Finding alternative dependency
   - Waiting for upstream fix
   - Documenting known issue

## Monitoring

### Workflow Status

Monitor workflow runs:
- GitHub repository → Actions tab
- View logs for failed runs
- Check timing to optimize performance

### Notifications

Enable notifications for:
- Failed workflow runs on main branch
- Security vulnerabilities
- Failed releases

Configure in: GitHub Settings → Notifications

## Local Testing

Before pushing, test locally:

```bash
# Run all checks
make all

# Or individually
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release

# Test Docker
docker build -t template-rust:test .
docker run --rm template-rust:test --help

# Validate docker-compose
docker compose config
```

## Future Improvements

Consider adding:

1. **Code Coverage**: Track test coverage with codecov or coveralls
2. **Benchmarking**: Automated performance regression testing
3. **Deploy to Registry**: Publish Docker images to GitHub Container Registry
4. **Nightly Builds**: Test against Rust nightly to catch future issues
5. **Dependency Updates**: Automated PRs for dependency updates (Dependabot)
6. **Documentation Deployment**: Auto-deploy docs to GitHub Pages

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI/CD Guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [Actions for Rust](https://github.com/actions-rs)
- [Semantic Versioning](https://semver.org/)

## Questions or Issues

If you have questions about the CI/CD setup:
1. Check this documentation
2. Review workflow files in `.github/workflows/`
3. Check workflow run logs for details
4. Open an issue for discussion
