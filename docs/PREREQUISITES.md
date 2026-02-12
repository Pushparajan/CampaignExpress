# Prerequisites Setup Guide

Step-by-step installation guide for all tools required to build and run Campaign Express.

---

## Table of Contents

1. [Rust](#1-rust)
2. [Docker & Docker Compose](#2-docker--docker-compose)
3. [Node.js & npm](#3-nodejs--npm)
4. [Optional Tools](#4-optional-tools)
5. [Verify Installation](#5-verify-installation)

---

## 1. Rust

Campaign Express requires **Rust 1.77 or later**.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen prompts and select the default installation. Then load the environment:

```bash
source "$HOME/.cargo/env"
```

Set the stable toolchain and add required components:

```bash
rustup default stable
rustup component add clippy rustfmt
```

### Windows

Download and run the installer from https://rustup.rs. You will also need the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (select "Desktop development with C++").

After installation, open a new terminal and run:

```powershell
rustup default stable
rustup component add clippy rustfmt
```

### Updating Rust

```bash
rustup update stable
```

---

## 2. Docker & Docker Compose

Docker runs the infrastructure services (NATS, Redis, ClickHouse, Prometheus, Grafana) and can also build the application container.

### macOS

Install [Docker Desktop for Mac](https://docs.docker.com/desktop/install/mac-install/). Docker Compose is included.

```bash
brew install --cask docker
```

### Linux (Ubuntu/Debian)

```bash
# Remove old versions
sudo apt-get remove docker docker-engine docker.io containerd runc 2>/dev/null

# Install prerequisites
sudo apt-get update
sudo apt-get install -y ca-certificates curl gnupg

# Add Docker GPG key
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg

# Add Docker repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine + Compose plugin
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Allow running Docker without sudo
sudo usermod -aG docker $USER
newgrp docker
```

### Linux (Fedora/RHEL)

```bash
sudo dnf install -y dnf-plugins-core
sudo dnf config-manager --add-repo https://download.docker.com/linux/fedora/docker-ce.repo
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -aG docker $USER
```

### Windows

Install [Docker Desktop for Windows](https://docs.docker.com/desktop/install/windows-install/). Requires WSL 2 backend. Docker Compose is included.

---

## 3. Node.js & npm

The management UI is a Next.js 14 application requiring **Node.js 18 or later**.

### macOS

```bash
brew install node@18
```

Or use nvm for version management:

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18
```

### Linux (Ubuntu/Debian)

Using NodeSource:

```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

Or with nvm (recommended):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
source ~/.bashrc
nvm install 18
nvm use 18
```

### Linux (Fedora/RHEL)

```bash
curl -fsSL https://rpm.nodesource.com/setup_18.x | sudo bash -
sudo dnf install -y nodejs
```

### Windows

Download the LTS installer from https://nodejs.org. npm is included.

---

## 4. Optional Tools

These are not required but improve the development experience.

### cargo-watch — Auto-rebuild on file changes

```bash
cargo install cargo-watch
# Usage: cargo watch -x 'run -- --api-only'
```

### jq — JSON pretty-printing for API responses

```bash
# macOS
brew install jq

# Ubuntu/Debian
sudo apt-get install -y jq

# Fedora
sudo dnf install -y jq

# Usage: curl -s http://localhost:8080/health | jq
```

### httpie — User-friendly HTTP client

```bash
# macOS
brew install httpie

# Ubuntu/Debian
sudo apt-get install -y httpie

# Fedora
sudo dnf install -y httpie

# Usage: http GET localhost:8080/health
```

### kubectl — Kubernetes CLI (for K8s deployment)

```bash
# macOS
brew install kubectl

# Linux
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
rm kubectl
```

### kustomize — Kubernetes manifest management

```bash
# macOS
brew install kustomize

# Linux
curl -s "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash
sudo mv kustomize /usr/local/bin/
```

---

## 5. Verify Installation

Run these commands to confirm everything is installed correctly:

```bash
# Rust toolchain
rustc --version        # Expected: rustc 1.77.0 or later
cargo --version        # Expected: cargo 1.77.0 or later
rustup component list --installed | grep -E "clippy|rustfmt"

# Docker
docker --version              # Expected: 24.0 or later
docker compose version        # Expected: 2.20 or later

# Node.js
node --version         # Expected: v18.x or later
npm --version          # Expected: 9.x or later

# Quick build test
cd /path/to/CampaignExpress
cargo check --workspace
```

If all commands succeed, you are ready to go. See the [Local Deployment Guide](LOCAL_DEPLOYMENT.md) for running the full stack, or the [README](../README.md) for a quick start.
