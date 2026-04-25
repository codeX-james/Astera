# Windows/WSL2 Developer Setup Guide

This guide covers the steps required to set up the Astera development environment on Windows using Windows Subsystem for Linux (WSL2).

## 1. WSL2 Installation and Ubuntu Setup

1. Open PowerShell as Administrator and run:
   ```powershell
   wsl --install
   ```
2. Restart your computer if prompted.
3. Once installed, set up your Ubuntu user and password.
4. Update your Ubuntu packages:
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```

## 2. Docker Desktop with WSL2 Backend

1. Install [Docker Desktop for Windows](https://docs.docker.com/desktop/install/windows-install/).
2. In Docker Desktop Settings, go to **General** and ensure **Use the WSL 2 based engine** is checked.
3. Go to **Resources > WSL Integration** and enable integration for your Ubuntu distro.
4. Verify Docker is running inside WSL:
   ```bash
   docker ps
   ```

## 3. Rust and Cargo Installation

Inside your WSL terminal, run:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
```

## 4. Stellar CLI Installation

Install the Stellar CLI inside WSL:
```bash
cargo install --locked stellar-cli --features opt
```
Ensure `~/.cargo/bin` is in your PATH.

## 5. Node.js 20+ via NVM

Install NVM (Node Version Manager) inside WSL:
```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
source ~/.bashrc
nvm install 20
nvm use 20
```

## 6. Git Line Ending Configuration

To avoid issues with Windows CRLF line endings, configure Git inside WSL to use LF:
```bash
git config --global core.autocrlf false
```
If you have already cloned the repo, you might need to re-clone it or run:
```bash
git rm --cached -r .
git reset --hard
```

## 7. Running the Docker Compose Stack

From the root of the project:
```bash
docker-compose up -d
```
Note: If you encounter permission issues with mounted volumes, ensure the project is cloned into the WSL filesystem (e.g., `~/source/Astera`) rather than the Windows filesystem (`/mnt/c/...`).

## 8. Known Issues and Solutions

### File Permissions
**Issue:** Cargo builds fail or Docker volumes have incorrect permissions.
**Solution:** Move your project to the native WSL filesystem (e.g., `~/projects/Astera`) instead of `/mnt/c/`.

### Port Forwarding
**Issue:** `localhost` from Windows browser doesn't connect to services in WSL.
**Solution:** Usually, WSL2 forwards ports to `localhost` automatically. If not, find the WSL IP using `hostname -I` and use that.

### Stellar CLI Path
**Issue:** `stellar` command not found.
**Solution:** Add `export PATH="$HOME/.cargo/bin:$PATH"` to your `~/.bashrc` or `~/.zshrc`.
