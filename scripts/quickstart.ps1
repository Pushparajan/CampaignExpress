#Requires -Version 5.1
<#
.SYNOPSIS
    Campaign Express — Windows PowerShell Quickstart Script
.DESCRIPTION
    Interactive menu-driven script that automates all 9 quickstart steps:
      1) Check / install prerequisites
      2) Clone and build the workspace
      3) Start infrastructure (Docker Compose)
      4) Verify the stack
      5) Run locally (without Docker app container)
      6) Hot-reload dev mode
      7) Management UI (Next.js)
      8) Tear down
      9) Utilities (logs, test bid, persistent env vars)
.NOTES
    Run from the CampaignExpress repo root, or the script will clone it for you.
    Requires an elevated (admin) shell for winget installs and WSL setup.
#>

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$script:ComposeFile = "deploy/docker/docker-compose.yml"
$script:RepoUrl     = "https://github.com/Pushparajan/CampaignExpress.git"

# ─── Helpers ────────────────────────────────────────────────────────────────

function Write-Step {
    param([string]$Number, [string]$Title)
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Step $Number — $Title" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Ok   { param([string]$Msg) Write-Host "  [OK] $Msg" -ForegroundColor Green }
function Write-Warn { param([string]$Msg) Write-Host "  [WARN] $Msg" -ForegroundColor Yellow }
function Write-Err  { param([string]$Msg) Write-Host "  [FAIL] $Msg" -ForegroundColor Red }
function Write-Info { param([string]$Msg) Write-Host "  $Msg" -ForegroundColor Gray }

function Test-Command {
    param([string]$Name)
    $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Confirm-Continue {
    param([string]$Prompt = "Continue?")
    $choice = Read-Host "$Prompt [Y/n]"
    if ($choice -and $choice -notmatch '^[Yy]') {
        Write-Info "Skipped."
        return $false
    }
    return $true
}

function Assert-RepoRoot {
    if (-not (Test-Path "Cargo.toml") -or -not (Test-Path $script:ComposeFile)) {
        Write-Err "Not in CampaignExpress repo root. Run Step 2 first or cd into the repo."
        return $false
    }
    return $true
}

# ─── Step 1: Prerequisites ─────────────────────────────────────────────────

function Invoke-Step1 {
    Write-Step "1" "Check / Install Prerequisites"

    $missing = @()

    # --- Git ---
    if (Test-Command "git") {
        $gitVer = (git --version) -replace '[^0-9.]', ''
        Write-Ok "Git $gitVer"
    } else {
        Write-Warn "Git not found"
        $missing += "Git.Git"
    }

    # --- Rust ---
    if (Test-Command "rustc") {
        $rustVer = (rustc --version).Split(" ")[1]
        Write-Ok "Rust $rustVer"
    } else {
        Write-Warn "Rust not found — install from https://rustup.rs + VS C++ Build Tools"
        Write-Info "  After installing, reopen PowerShell and re-run this step."
    }

    # --- Cargo ---
    if (Test-Command "cargo") {
        $cargoVer = (cargo --version).Split(" ")[1]
        Write-Ok "Cargo $cargoVer"
    } else {
        Write-Warn "Cargo not found (installs with Rust)"
    }

    # --- Docker ---
    if (Test-Command "docker") {
        $dockerVer = (docker --version) -replace 'Docker version ([0-9.]+).*', '$1'
        Write-Ok "Docker $dockerVer"
    } else {
        Write-Warn "Docker not found"
        $missing += "Docker.DockerDesktop"
    }

    # --- Docker Compose ---
    try {
        $composeVer = (docker compose version 2>$null) -replace '.*v([0-9.]+).*', '$1'
        Write-Ok "Docker Compose $composeVer"
    } catch {
        Write-Warn "Docker Compose not available (included with Docker Desktop)"
    }

    # --- Node.js ---
    if (Test-Command "node") {
        $nodeVer = (node --version)
        Write-Ok "Node.js $nodeVer"
    } else {
        Write-Warn "Node.js not found"
        $missing += "OpenJS.NodeJS.LTS"
    }

    # --- npm ---
    if (Test-Command "npm") {
        $npmVer = (npm --version)
        Write-Ok "npm $npmVer"
    } else {
        Write-Warn "npm not found (installs with Node.js)"
    }

    # --- WSL 2 check ---
    try {
        $wslStatus = wsl --status 2>$null
        if ($wslStatus) { Write-Ok "WSL 2 available" }
        else            { Write-Warn "WSL 2 may not be configured — run 'wsl --install' if Docker fails" }
    } catch {
        Write-Warn "WSL not detected — Docker Desktop requires WSL 2"
        Write-Info "  Run 'wsl --install' from an elevated PowerShell, then reboot."
    }

    # --- Offer winget installs ---
    if ($missing.Count -gt 0) {
        Write-Host ""
        Write-Info "Missing packages: $($missing -join ', ')"
        if (Test-Command "winget") {
            if (Confirm-Continue "Install missing packages via winget?") {
                foreach ($pkg in $missing) {
                    Write-Info "Installing $pkg ..."
                    winget install --id $pkg --accept-source-agreements --accept-package-agreements
                }
                Write-Ok "Install commands complete. You may need to restart PowerShell."
            }
        } else {
            Write-Warn "winget not available. Install the missing tools manually."
        }
    }

    # --- Rust components ---
    if (Test-Command "rustup") {
        Write-Host ""
        Write-Info "Ensuring Rust stable toolchain + clippy + rustfmt ..."
        rustup default stable
        rustup component add clippy rustfmt
        Write-Ok "Rust toolchain configured"
    }
}

# ─── Step 2: Clone and Build ───────────────────────────────────────────────

function Invoke-Step2 {
    Write-Step "2" "Clone and Build"

    # Clone if not already in repo
    if (-not (Test-Path "Cargo.toml")) {
        if (Test-Path "CampaignExpress/Cargo.toml") {
            Write-Info "Found CampaignExpress subdirectory, entering it ..."
            Set-Location "CampaignExpress"
        } else {
            Write-Info "Cloning repository ..."
            git clone $script:RepoUrl
            Set-Location "CampaignExpress"
        }
    }

    Write-Ok "Working directory: $(Get-Location)"

    Write-Info "Running cargo check --workspace ..."
    cargo check --workspace
    Write-Ok "Workspace check passed"

    Write-Info "Running cargo clippy --workspace -- -D warnings ..."
    cargo clippy --workspace -- -D warnings
    Write-Ok "Clippy passed"

    Write-Info "Running cargo test --workspace ..."
    cargo test --workspace
    Write-Ok "All tests passed"
}

# ─── Step 3: Start Infrastructure ──────────────────────────────────────────

function Invoke-Step3 {
    Write-Step "3" "Start Infrastructure (Docker Compose)"

    if (-not (Assert-RepoRoot)) { return }

    Write-Info "Starting all services ..."
    docker compose -f $script:ComposeFile up -d

    Write-Info "Waiting for services to become healthy ..."
    $maxAttempts = 30
    for ($i = 1; $i -le $maxAttempts; $i++) {
        $ps = docker compose -f $script:ComposeFile ps --format json 2>$null
        if ($ps) {
            $services = $ps | ConvertFrom-Json
            $total   = ($services | Measure-Object).Count
            $healthy = ($services | Where-Object {
                $_.Health -eq "healthy" -or $_.State -eq "running"
            } | Measure-Object).Count

            Write-Info "  Attempt $i/$maxAttempts — $healthy/$total services ready"
            if ($healthy -ge $total -and $total -gt 0) {
                Write-Ok "All services are up"
                docker compose -f $script:ComposeFile ps
                return
            }
        }
        Start-Sleep -Seconds 5
    }

    Write-Warn "Some services may not be healthy yet. Check with:"
    Write-Info "  docker compose -f $script:ComposeFile ps"
}

# ─── Step 4: Verify the Stack ──────────────────────────────────────────────

function Invoke-Step4 {
    Write-Step "4" "Verify the Stack"

    $endpoints = @(
        @{ Name = "Health";    Url = "http://localhost:8080/health" },
        @{ Name = "Ready";     Url = "http://localhost:8080/ready" },
        @{ Name = "NATS varz"; Url = "http://localhost:8222/varz" }
    )

    foreach ($ep in $endpoints) {
        try {
            $resp = Invoke-RestMethod -Uri $ep.Url -TimeoutSec 5
            Write-Ok "$($ep.Name): OK"
            if ($resp) {
                $preview = ($resp | ConvertTo-Json -Depth 1 -Compress)
                if ($preview.Length -gt 120) { $preview = $preview.Substring(0, 120) + "..." }
                Write-Info "  $preview"
            }
        } catch {
            Write-Err "$($ep.Name): $($_.Exception.Message)"
        }
    }

    Write-Host ""
    if (Confirm-Continue "Open Grafana in browser? (admin / campaign-express)") {
        Start-Process "http://localhost:3000"
        Write-Ok "Grafana opened — login: admin / campaign-express"
    }
}

# ─── Step 5: Run Locally ───────────────────────────────────────────────────

function Invoke-Step5 {
    Write-Step "5" "Run Locally (Without Docker App Container)"

    if (-not (Assert-RepoRoot)) { return }

    Write-Info "Starting infrastructure-only services (nats, redis, clickhouse) ..."
    docker compose -f $script:ComposeFile up -d nats redis clickhouse

    Write-Info "Setting environment variables ..."
    $env:CAMPAIGN_EXPRESS__NODE_ID          = "local-win-01"
    $env:CAMPAIGN_EXPRESS__AGENTS_PER_NODE  = "4"
    $env:CAMPAIGN_EXPRESS__NATS__URLS       = "nats://localhost:4222"
    $env:CAMPAIGN_EXPRESS__REDIS__URLS      = "redis://localhost:6379"
    $env:CAMPAIGN_EXPRESS__CLICKHOUSE__URL  = "http://localhost:8123"
    $env:CAMPAIGN_EXPRESS__NPU__DEVICE      = "cpu"
    $env:RUST_LOG                           = "campaign_express=debug"
    Write-Ok "Environment configured"

    Write-Info "Building and running campaign-express (Ctrl+C to stop) ..."
    cargo run --bin campaign-express -- --api-only
}

# ─── Step 6: Hot-Reload Dev Mode ───────────────────────────────────────────

function Invoke-Step6 {
    Write-Step "6" "Hot-Reload Dev Mode"

    if (-not (Assert-RepoRoot)) { return }

    if (-not (Test-Command "cargo-watch")) {
        Write-Info "Installing cargo-watch ..."
        cargo install cargo-watch
        Write-Ok "cargo-watch installed"
    } else {
        Write-Ok "cargo-watch already installed"
    }

    # Ensure infra is running
    Write-Info "Ensuring infrastructure services are running ..."
    docker compose -f $script:ComposeFile up -d nats redis clickhouse

    # Set env vars if not already set
    if (-not $env:CAMPAIGN_EXPRESS__NODE_ID) {
        $env:CAMPAIGN_EXPRESS__NODE_ID          = "local-win-01"
        $env:CAMPAIGN_EXPRESS__AGENTS_PER_NODE  = "4"
        $env:CAMPAIGN_EXPRESS__NATS__URLS       = "nats://localhost:4222"
        $env:CAMPAIGN_EXPRESS__REDIS__URLS      = "redis://localhost:6379"
        $env:CAMPAIGN_EXPRESS__CLICKHOUSE__URL  = "http://localhost:8123"
        $env:CAMPAIGN_EXPRESS__NPU__DEVICE      = "cpu"
        $env:RUST_LOG                           = "campaign_express=debug"
        Write-Ok "Environment configured"
    }

    Write-Info "Starting cargo-watch (auto-rebuilds on file changes, Ctrl+C to stop) ..."
    cargo watch -x "run --bin campaign-express -- --api-only"
}

# ─── Step 7: Management UI ─────────────────────────────────────────────────

function Invoke-Step7 {
    Write-Step "7" "Management UI (Next.js)"

    if (-not (Assert-RepoRoot)) { return }

    $uiDir = Join-Path (Get-Location) "ui"
    if (-not (Test-Path $uiDir)) {
        Write-Err "ui/ directory not found at $uiDir"
        return
    }

    Push-Location $uiDir
    try {
        Write-Info "Installing npm dependencies ..."
        npm install
        Write-Ok "Dependencies installed"

        Write-Warn "If Grafana is running on port 3000, stop it first or the UI will fail to bind."
        Write-Info "Starting Next.js dev server (Ctrl+C to stop) ..."
        npm run dev
    } finally {
        Pop-Location
    }
}

# ─── Step 8: Tear Down ─────────────────────────────────────────────────────

function Invoke-Step8 {
    Write-Step "8" "Tear Down"

    if (-not (Assert-RepoRoot)) { return }

    Write-Host "  1) Stop containers (keep data volumes)" -ForegroundColor White
    Write-Host "  2) Stop containers AND remove volumes" -ForegroundColor White
    Write-Host "  3) Cancel" -ForegroundColor White
    $choice = Read-Host "  Choose [1/2/3]"

    switch ($choice) {
        "1" {
            docker compose -f $script:ComposeFile down
            Write-Ok "Containers stopped. Data volumes preserved."
        }
        "2" {
            docker compose -f $script:ComposeFile down -v
            Write-Ok "Containers stopped and volumes removed."
        }
        default {
            Write-Info "Cancelled."
        }
    }

    # Clear session env vars
    if (Confirm-Continue "Clear Campaign Express environment variables from this session?") {
        Remove-Item Env:CAMPAIGN_EXPRESS__NODE_ID         -ErrorAction SilentlyContinue
        Remove-Item Env:CAMPAIGN_EXPRESS__AGENTS_PER_NODE -ErrorAction SilentlyContinue
        Remove-Item Env:CAMPAIGN_EXPRESS__NATS__URLS      -ErrorAction SilentlyContinue
        Remove-Item Env:CAMPAIGN_EXPRESS__REDIS__URLS     -ErrorAction SilentlyContinue
        Remove-Item Env:CAMPAIGN_EXPRESS__CLICKHOUSE__URL -ErrorAction SilentlyContinue
        Remove-Item Env:CAMPAIGN_EXPRESS__NPU__DEVICE     -ErrorAction SilentlyContinue
        Remove-Item Env:RUST_LOG                          -ErrorAction SilentlyContinue
        Write-Ok "Environment variables cleared."
    }
}

# ─── Step 9: Utilities ─────────────────────────────────────────────────────

function Invoke-Step9 {
    Write-Step "9" "Utilities (Logs, Test Bid, Env Vars)"

    Write-Host "  a) View container logs (follow mode)" -ForegroundColor White
    Write-Host "  b) Send a test bid request" -ForegroundColor White
    Write-Host "  c) Set persistent environment variables (user-level)" -ForegroundColor White
    Write-Host "  d) Show all container statuses" -ForegroundColor White
    Write-Host "  e) Back to main menu" -ForegroundColor White
    $choice = Read-Host "  Choose [a/b/c/d/e]"

    switch ($choice) {
        "a" {
            if (-not (Assert-RepoRoot)) { return }
            Write-Info "Tailing logs (Ctrl+C to stop) ..."
            docker compose -f $script:ComposeFile logs -f campaign-express
        }
        "b" {
            Write-Info "Sending test bid to http://localhost:8080/v1/bid ..."
            $body = @{
                request_id = "test-$(Get-Random -Maximum 99999)"
                user_id    = "user-123"
                context    = @{ channel = "web"; locale = "en-US" }
            } | ConvertTo-Json

            try {
                $resp = Invoke-RestMethod -Uri "http://localhost:8080/v1/bid" `
                    -Method Post `
                    -ContentType "application/json" `
                    -Body $body `
                    -TimeoutSec 10
                Write-Ok "Response:"
                $resp | ConvertTo-Json -Depth 5 | Write-Host
            } catch {
                Write-Err "Bid request failed: $($_.Exception.Message)"
            }
        }
        "c" {
            Write-Info "Setting persistent (user-level) environment variables ..."
            $vars = @{
                "CAMPAIGN_EXPRESS__NODE_ID"         = "local-win-01"
                "CAMPAIGN_EXPRESS__AGENTS_PER_NODE" = "4"
                "CAMPAIGN_EXPRESS__NATS__URLS"      = "nats://localhost:4222"
                "CAMPAIGN_EXPRESS__REDIS__URLS"     = "redis://localhost:6379"
                "CAMPAIGN_EXPRESS__CLICKHOUSE__URL" = "http://localhost:8123"
                "CAMPAIGN_EXPRESS__NPU__DEVICE"     = "cpu"
                "RUST_LOG"                          = "campaign_express=debug"
            }
            foreach ($kv in $vars.GetEnumerator()) {
                [Environment]::SetEnvironmentVariable($kv.Key, $kv.Value, "User")
                Write-Ok "$($kv.Key) = $($kv.Value)"
            }
            Write-Info "Restart PowerShell for changes to take effect in new sessions."
        }
        "d" {
            if (-not (Assert-RepoRoot)) { return }
            docker compose -f $script:ComposeFile ps
        }
        default {
            return
        }
    }
}

# ─── Main Menu ──────────────────────────────────────────────────────────────

function Show-Menu {
    Write-Host ""
    Write-Host "╔══════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║     Campaign Express — Windows Quickstart        ║" -ForegroundColor Cyan
    Write-Host "╠══════════════════════════════════════════════════╣" -ForegroundColor Cyan
    Write-Host "║  1) Check / install prerequisites                ║" -ForegroundColor White
    Write-Host "║  2) Clone and build workspace                    ║" -ForegroundColor White
    Write-Host "║  3) Start infrastructure (Docker Compose)        ║" -ForegroundColor White
    Write-Host "║  4) Verify the stack                             ║" -ForegroundColor White
    Write-Host "║  5) Run locally (without Docker app container)   ║" -ForegroundColor White
    Write-Host "║  6) Hot-reload dev mode (cargo-watch)            ║" -ForegroundColor White
    Write-Host "║  7) Management UI (Next.js)                      ║" -ForegroundColor White
    Write-Host "║  8) Tear down                                    ║" -ForegroundColor White
    Write-Host "║  9) Utilities (logs, test bid, env vars)         ║" -ForegroundColor White
    Write-Host "║  A) Run all steps (1-4) sequentially             ║" -ForegroundColor Yellow
    Write-Host "║  Q) Quit                                         ║" -ForegroundColor Gray
    Write-Host "╚══════════════════════════════════════════════════╝" -ForegroundColor Cyan
}

function Invoke-AllSteps {
    Write-Host ""
    Write-Host "Running Steps 1-4 sequentially ..." -ForegroundColor Yellow
    Write-Host ""

    Invoke-Step1
    if (-not (Confirm-Continue "Proceed to Clone and Build?")) { return }

    Invoke-Step2
    if (-not (Confirm-Continue "Proceed to Start Infrastructure?")) { return }

    Invoke-Step3
    Write-Info "Waiting 10 seconds for services to initialize ..."
    Start-Sleep -Seconds 10

    Invoke-Step4

    Write-Host ""
    Write-Ok "Steps 1-4 complete. The full stack is running."
    Write-Info "Use Steps 5-9 from the menu for local dev, hot-reload, UI, and utilities."
}

# ─── Entry Point ────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "Campaign Express Quickstart" -ForegroundColor Cyan
Write-Host "Working directory: $(Get-Location)" -ForegroundColor Gray
Write-Host ""

while ($true) {
    Show-Menu
    $selection = Read-Host "Select an option"

    switch ($selection) {
        "1" { Invoke-Step1 }
        "2" { Invoke-Step2 }
        "3" { Invoke-Step3 }
        "4" { Invoke-Step4 }
        "5" { Invoke-Step5 }
        "6" { Invoke-Step6 }
        "7" { Invoke-Step7 }
        "8" { Invoke-Step8 }
        "9" { Invoke-Step9 }
        { $_ -in "A","a" } { Invoke-AllSteps }
        { $_ -in "Q","q" } {
            Write-Host ""
            Write-Info "Goodbye!"
            exit 0
        }
        default {
            Write-Warn "Invalid selection. Enter 1-9, A, or Q."
        }
    }
}
