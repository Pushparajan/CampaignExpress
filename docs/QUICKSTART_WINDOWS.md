# Campaign Express — Windows PowerShell Quickstart

Get the full stack running on Windows with PowerShell in under 10 minutes.

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Git | 2.40+ | `winget install Git.Git` |
| Rust | 1.77+ | [rustup.rs](https://rustup.rs) + VS C++ Build Tools |
| Docker Desktop | 24+ | `winget install Docker.DockerDesktop` |
| Node.js | 18+ | `winget install OpenJS.NodeJS.LTS` |

> Docker Desktop requires **WSL 2**. If not enabled, run `wsl --install` from an elevated PowerShell and reboot.

---

## 1. Install Rust Toolchain

Download and run the installer from [rustup.rs](https://rustup.rs). When prompted, select **default installation**. You need the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) — select "Desktop development with C++".

After installation, open a **new** PowerShell window:

```powershell
rustup default stable
rustup component add clippy rustfmt

# Verify
rustc --version    # 1.77+
cargo --version
```

---

## 2. Clone and Build

```powershell
git clone https://github.com/Pushparajan/CampaignExpress.git
cd CampaignExpress

# Full workspace check (compiles all 26 crates)
cargo check --workspace

# Run clippy lints
cargo clippy --workspace -- -D warnings

# Run tests
cargo test --workspace
```

---

## 3. Start Infrastructure (Docker Compose)

Make sure Docker Desktop is running, then:

```powershell
docker compose -f deploy/docker/docker-compose.yml up -d
```

Wait for all services to become healthy:

```powershell
docker compose -f deploy/docker/docker-compose.yml ps
```

| Service | Ports | Purpose |
|---------|-------|---------|
| campaign-express | 8080, 9090, 9091 | REST, gRPC, metrics |
| nats | 4222, 8222 | Message broker |
| redis | 6379 | Cache |
| clickhouse | 8123, 9000 | Analytics DB |
| prometheus | 9092 | Metrics |
| grafana | 3000 | Dashboards |

---

## 4. Verify the Stack

```powershell
# Health check
Invoke-RestMethod http://localhost:8080/health

# Readiness check
Invoke-RestMethod http://localhost:8080/ready

# NATS monitoring
Invoke-RestMethod http://localhost:8222/varz

# Grafana (browser)
Start-Process http://localhost:3000
# Login: admin / campaign-express
```

---

## 5. Run Locally (Without Docker)

If you prefer running the Rust binary directly against Dockerized infra:

```powershell
# Start only infrastructure services
docker compose -f deploy/docker/docker-compose.yml up -d nats redis clickhouse

# Set environment variables
$env:CAMPAIGN_EXPRESS__NODE_ID = "local-win-01"
$env:CAMPAIGN_EXPRESS__AGENTS_PER_NODE = "4"
$env:CAMPAIGN_EXPRESS__NATS__URLS = "nats://localhost:4222"
$env:CAMPAIGN_EXPRESS__REDIS__URLS = "redis://localhost:6379"
$env:CAMPAIGN_EXPRESS__CLICKHOUSE__URL = "http://localhost:8123"
$env:CAMPAIGN_EXPRESS__NPU__DEVICE = "cpu"
$env:RUST_LOG = "campaign_express=debug"

# Build and run
cargo run --bin campaign-express -- --api-only
```

---

## 6. Hot-Reload Dev Mode (Optional)

```powershell
cargo install cargo-watch

# Auto-rebuild on file changes
cargo watch -x "run --bin campaign-express -- --api-only"
```

---

## 7. Management UI (Next.js)

```powershell
cd ui
npm install
npm run dev

# Opens at http://localhost:3000 (ensure Grafana is on a different port or stopped)
```

---

## 8. Tear Down

```powershell
# Stop all containers and remove volumes
docker compose -f deploy/docker/docker-compose.yml down -v

# Or stop without removing data
docker compose -f deploy/docker/docker-compose.yml down
```

---

## Common PowerShell Tips

### View container logs

```powershell
docker compose -f deploy/docker/docker-compose.yml logs -f campaign-express
```

### Test a bid request

```powershell
$body = @{
    request_id = "test-001"
    user_id    = "user-123"
    context    = @{ channel = "web"; locale = "en-US" }
} | ConvertTo-Json

Invoke-RestMethod -Uri http://localhost:8080/v1/bid `
    -Method Post `
    -ContentType "application/json" `
    -Body $body
```

### Set persistent environment variables

```powershell
# Current session only (shown above)
$env:RUST_LOG = "campaign_express=debug"

# Persist across sessions (user-level)
[Environment]::SetEnvironmentVariable("RUST_LOG", "campaign_express=debug", "User")
```

### Firewall issues

If Docker containers can't reach each other, ensure Docker Desktop's WSL integration is enabled:
**Settings > Resources > WSL Integration > Enable for your distro**

---

## Next Steps

- [Prerequisites (full details)](PREREQUISITES.md) — all supported platforms
- [Deployment Guide](DEPLOYMENT.md) — Kubernetes, Terraform, and production setup
