# CI/CD Workflows

This directory contains GitHub Actions workflows for continuous integration and testing.

## Workflows

### CI (`ci.yml`)

Runs on every push to `v0.19.13-theatl` and `20251003/add-login-hook` branches, and on pull requests to `v0.19.13-theatl`.

**Jobs:**

1. **Formatting** (`formatting`)
   - Checks code formatting with `cargo fmt`
   - Ensures consistent code style across the codebase

2. **Linting** (`linting`)
   - Runs `cargo clippy` to catch common mistakes and anti-patterns
   - Treats warnings as errors (`-D warnings`)
   - Uses caching for faster builds

3. **Build** (`build`)
   - Checks compilation with `cargo check`
   - Builds release binary with `cargo build --release`
   - Validates that all code compiles successfully
   - Uses caching for faster builds

4. **Test** (`test`)
   - Runs full test suite with `cargo test`
   - Uses PostgreSQL 16 service container for database tests
   - Tests all workspace packages with all features enabled
   - Uses caching for faster builds

5. **Security Audit** (`security-audit`)
   - Runs `cargo audit` to check for known security vulnerabilities
   - Scans dependencies for security advisories

6. **Private API Validation** (`private-api-validation`)
   - Validates Private API modules compile correctly
   - Ensures documentation exists
   - Verifies all required files are present

## Status Badges

Add these to your README.md to show CI status:

```markdown
![CI](https://github.com/theatl-social/lemmy/workflows/CI/badge.svg?branch=v0.19.13-theatl)
```

## Local Development

To run the same checks locally:

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check compilation
cargo check --all-targets --all-features

# Build
cargo build --release

# Run tests (requires PostgreSQL)
export DATABASE_URL=postgres://lemmy:password@localhost:5432/lemmy
cargo test --workspace --all-features

# Security audit
cargo install cargo-audit
cargo audit
```

## Troubleshooting

### Formatting Failures
Run `cargo fmt --all` to automatically fix formatting issues.

### Clippy Warnings
Fix clippy warnings or add `#[allow(...)]` attributes if warnings are intentional.

### Test Failures
Ensure PostgreSQL is running and `DATABASE_URL` is set correctly.
The test database must be initialized with proper schema.

### Cache Issues
If builds are failing due to cache issues, clear the cache by going to:
Actions → Caches → Delete relevant caches
