# Contributing to Campaign Express

Thank you for your interest in contributing to Campaign Express. This document provides guidelines and procedures for contributing to the project.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Standards](#code-standards)
5. [Commit Guidelines](#commit-guidelines)
6. [Pull Request Process](#pull-request-process)
7. [Architecture Guidelines](#architecture-guidelines)
8. [Testing Requirements](#testing-requirements)
9. [Documentation](#documentation)
10. [Getting Help](#getting-help)

---

## Code of Conduct

All contributors are expected to maintain a professional, inclusive, and respectful environment. Harassment, discrimination, and disruptive behavior are not tolerated.

---

## Getting Started

### Prerequisites

- **Rust 1.77+** — `rustup install stable`
- **Docker & Docker Compose** — for local infrastructure
- **Node.js 18+** — for the management UI

See [docs/PREREQUISITES.md](docs/PREREQUISITES.md) for detailed platform-specific installation instructions.

### Clone & Build

```bash
git clone <repository-url>
cd CampaignExpress
cargo build --workspace
cargo test --workspace
```

### Local Development Stack

```bash
docker compose -f deploy/docker/docker-compose.yml up -d
cargo run --release -- --api-only --node-id dev-01
```

See [docs/LOCAL_DEPLOYMENT.md](docs/LOCAL_DEPLOYMENT.md) for the full local setup guide.

---

## Development Workflow

### Branch Naming

Use descriptive branch names with a category prefix:

| Prefix | Purpose |
|--------|---------|
| `feature/` | New features |
| `fix/` | Bug fixes |
| `refactor/` | Code refactoring |
| `docs/` | Documentation changes |
| `perf/` | Performance improvements |
| `test/` | Test additions or fixes |
| `infra/` | Infrastructure changes |

Example: `feature/add-whatsapp-channel`, `fix/cache-invalidation-race`

### Workflow Steps

1. Create a feature branch from `main`
2. Make focused, incremental changes
3. Run the full quality gate locally (see below)
4. Open a pull request against `main`
5. Address review feedback
6. Squash-merge once approved

---

## Code Standards

### Rust

All Rust code must pass the following quality gate before merging:

```bash
cargo fmt --all --check            # Formatting
cargo clippy --workspace -- -D warnings  # Linting (zero warnings)
cargo test --workspace             # All tests pass
cargo check --workspace            # Type checking
```

#### Style Guidelines

- **Formatting**: Always use `rustfmt` defaults. Do not add `#[rustfmt::skip]` without strong justification.
- **Clippy**: Treat all warnings as errors (`-D warnings`). If a lint is genuinely wrong, use a targeted `#[allow(clippy::...)]` with a comment explaining why.
- **Error Handling**: Use `thiserror` for library errors, `anyhow` sparingly in binary code. Avoid `.unwrap()` in production paths.
- **Async**: Use `tokio` for async runtime. Prefer `tokio::spawn` over `std::thread::spawn` unless CPU-bound work requires it.
- **Naming**: Follow Rust naming conventions — `snake_case` for functions/variables, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- **Dependencies**: Minimize new dependencies. Discuss additions in the PR description.

#### Clippy Notes

- Use `#[allow(clippy::unnecessary_map_or)]` not `manual_map_or`
- Prefer `is_none_or()` / `is_some_and()` idioms
- Use `#[derive(Default)]` + `#[default]` on the first variant for enum defaults

### TypeScript / React (Management UI)

```bash
cd ui
npm run lint
npm run build
```

---

## Commit Guidelines

### Format

```
<type>(<scope>): <short summary>

<optional body explaining the "why">
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `perf` | Performance improvement |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `docs` | Documentation only |
| `test` | Adding or correcting tests |
| `ci` | CI/CD pipeline changes |
| `chore` | Build process or auxiliary tool changes |
| `infra` | Infrastructure (k8s, terraform, helm, docker) |

### Scopes

Use the crate name as scope: `core`, `npu-engine`, `agents`, `cache`, `analytics`, `api-server`, `loyalty`, `dsp`, `channels`, `management`, `journey`, `dco`, `cdp`, `platform`, `billing`, `ops`, `personalization`, `segmentation`, `reporting`, `integrations`, `intelligent-delivery`, `rl-engine`, `mobile-sdk`, `plugin-marketplace`, `sdk-docs`, `wasm-edge`, `ui`.

### Examples

```
feat(loyalty): add Reserve tier upgrade notifications
fix(cache): resolve race condition in L1 invalidation
perf(npu-engine): batch inference requests with Nagle buffer
docs(api-server): document gRPC streaming endpoint
infra(k8s): add network policy for ClickHouse access
```

---

## Pull Request Process

### Before Opening a PR

1. Rebase on latest `main`
2. Run the full quality gate locally
3. Ensure no secrets, credentials, or `.env` files are included
4. Update documentation if behavior changes

### PR Description Template

```markdown
## Summary
Brief description of what this PR does and why.

## Changes
- Bullet list of key changes

## Testing
- How was this tested?
- Which test commands were run?

## Checklist
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Documentation updated (if applicable)
- [ ] No new dependencies added (or justified in description)
```

### Review Process

- All PRs require at least one approving review
- Performance-critical changes (inference, caching, bidding) require two reviewers
- Infrastructure changes require SRE team review
- Security-related changes require security review

---

## Architecture Guidelines

### Adding a New Crate

1. Create the crate under `crates/`
2. Add it to the workspace `Cargo.toml`
3. Follow the existing pattern: `lib.rs` with public API, internal modules private
4. Add the crate to the API server router in `crates/api-server/src/server.rs` if it exposes endpoints
5. Update `README.md` workspace tree and `docs/ARCHITECTURE.md`

### Key Design Principles

- **Hardware Agnostic**: Use the `CoLaNetProvider` trait for inference. Never hard-code hardware-specific logic outside of `npu-engine/backends/`.
- **Non-Blocking**: Never block the Tokio runtime. Use `tokio::task::spawn_blocking` for CPU-heavy work.
- **Two-Tier Cache**: DashMap L1 (lock-free) -> Redis L2. Check L1 first, populate from L2 on miss.
- **Analytics Pipeline**: Use the mpsc channel -> batched ClickHouse insert pattern. Never write to ClickHouse synchronously in request paths.
- **No CDN Downloads at Build Time**: Do not add dependencies that require downloading binaries from external CDNs (e.g., ONNX Runtime). Use pure-Rust alternatives or provide extension points.
- **No Protoc Dependency**: gRPC types are manually defined with `prost::Message`. Do not add `tonic-build` or require the `protoc` binary.

---

## Testing Requirements

### Test Levels

| Level | Command | Required |
|-------|---------|----------|
| Unit tests | `cargo test --workspace` | All PRs |
| Clippy | `cargo clippy --workspace -- -D warnings` | All PRs |
| Format | `cargo fmt --all --check` | All PRs |
| Integration tests | `cargo test --workspace -- --ignored` | Feature PRs |

### Writing Tests

- Place unit tests in a `#[cfg(test)] mod tests` block within each module
- Use descriptive test names: `test_loyalty_earn_stars_gold_tier_multiplier`
- Test error paths, not just happy paths
- Use `tokio::test` for async tests

See [docs/TEST_STRATEGY.md](docs/TEST_STRATEGY.md) for the comprehensive test strategy.

---

## Documentation

- Update `docs/ARCHITECTURE.md` when adding new modules or changing data flows
- Update `docs/API_REFERENCE.md` when adding or modifying API endpoints
- Update `README.md` workspace tree when adding new crates
- Add inline doc comments (`///`) to public APIs
- Role-specific guides (`RUST_ENGINEER_GUIDE.md`, `ML_ENGINEER_GUIDE.md`, `SRE_GUIDE.md`) should be updated when workflows change

---

## Getting Help

- **Architecture questions**: Review [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/REQUEST_FLOW.md](docs/REQUEST_FLOW.md)
- **Setup issues**: See [docs/PREREQUISITES.md](docs/PREREQUISITES.md) and [docs/LOCAL_DEPLOYMENT.md](docs/LOCAL_DEPLOYMENT.md)
- **Role-specific guidance**:
  - Rust engineers: [docs/RUST_ENGINEER_GUIDE.md](docs/RUST_ENGINEER_GUIDE.md)
  - ML engineers: [docs/ML_ENGINEER_GUIDE.md](docs/ML_ENGINEER_GUIDE.md)
  - SRE/DevOps: [docs/SRE_GUIDE.md](docs/SRE_GUIDE.md)
- **Slack**: `#campaign-express-dev`
- **Email**: campaign-express-team@company.com
