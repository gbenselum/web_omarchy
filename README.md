# Omarchy Web Management Interface

A stripped-down, ultra-fast web interface for Omarchy Linux, inspired by Cockpit.

## Overview
This project aims to extract the core logic of Cockpit (a web interface based on plugins) and strip it down to the absolute bare minimum, focusing heavily on security and responsiveness. Instead of using PatternFly, this project is built entirely around Omarchy themes.

- **Login UI:** Uses the specific **Omarchy unlock theme**.
- **Main App UI:** Dynamically reads and applies the **user's configured theme** from the operating system.
- **Design Philosophy:** The UI is designed to feel like a **Web TUI** (Terminal User Interface in the browser) rather than a traditional, heavy web application. It aims for speed, efficiency, and a keyboard-friendly layout.
- **Tech Stack:** Tailored for Arch Linux/Omarchy. The backend uses Rust for maximum performance and security, interfacing seamlessly with existing distro tools.

## Current Status (as of 2026-09-18)

### ✅ IMPLEMENTED - Working Components

#### Backend (Rust) - `backend/`
- [x] WebSocket server on port 8080 with HTTP static file serving (Axum)
- [x] JSON-RPC protocol for frontend-backend communication
- [x] PAM authentication via polkit helper (`/usr/lib/web-omarchy/pam-auth`)
- [x] Command execution bridge for `omarchy-*` system commands
- [x] Session management with JWT-like tokens
- [x] systemd service with security hardening (ProtectSystem=strict, ReadWritePaths)
- [x] Polkit policy for PAM authentication
- [x] Architecture: single binary `web-omarchy-daemon`

#### Frontend (Vanilla JS + Vite) - `frontend/`
- [x] Web TUI with dynamic theming (Catppuccin Mocha/Latte, Tokyo Night, Dracula, Nord)
- [x] Login page with Omarchy unlock theme
- [x] Main app shell with tab navigation and sidebar plugin menu
- [x] Panels implemented:
  - [x] Dashboard - system info via `omarchy-system-info`
  - [x] Networking - interfaces, IPs, MACs, traffic via `omarchy-network-status`
  - [x] Storage - disks, partitions, usage, SMART via `omarchy-storage-list`
  - [x] Btrfs - snapshots via `omarchy-btrfs-list` (snapper/btrfs)
  - [x] Updates - available updates via `omarchy-pkg-check`, apply via `omarchy-pkg-update`
  - [x] Terminal - xterm.js with PTY bridge (partial - see below)
  - [x] Services - systemd units via `omarchy-service-control`, logs via `omarchy-service-logs`
- [x] Keyboard navigation (Tab, Vim-style, Escape, ? for help)
- [x] Auto-reconnecting WebSocket client
- [x] Modal dialogs, toast notifications
- [x] Responsive layout (collapsible sidebar on mobile)

#### omarchy-* Commands (Rust) - `omarchy-commands/`
All installed to `/usr/local/bin/`:
- [x] `omarchy-system-info` - hostname, kernel, uptime, packages, CPU, memory
- [x] `omarchy-network-status` - interfaces, IPs, MACs, traffic stats
- [x] `omarchy-storage-list` - disks, partitions, usage, SMART status
- [x] `omarchy-btrfs-list` - snapshots via snapper/btrfs subvolumes
- [x] `omarchy-pkg-check` - available updates (checkupdates/pacman)
- [x] `omarchy-pkg-update` - apply updates (pacman/yay/paru)
- [x] `omarchy-service-control` - systemd start/stop/restart/enable/disable
- [x] `omarchy-service-logs` - journalctl for services

#### Packaging & DevOps - `packaging/`
- [x] systemd service with hardening (NoNewPrivileges, ProtectSystem=strict, etc.)
- [x] ReadWritePaths=/var/lib/web-omarchy for session persistence
- [x] Polkit policy for PAM authentication (`org.omarchy.web.policy`)
- [x] Polkit auth helper script + C PAM auth binary
- [x] Arch PKGBUILD with sysusers/tmpfiles
- [x] GitHub Actions CI/CD (lint, test, build, release)
- [x] Install script (`install-complete.sh`)

#### Test Status
- [x] Backend compiles with `cargo check` and `cargo clippy`
- [x] Frontend builds with `npm run build` and passes `npm run lint`
- [x] All omarchy-* commands compile and run successfully
- [x] Service starts and accepts WebSocket connections
- [x] Login works with `TEST_AUTH_BYPASS=1` (any credentials)
- [x] Panels load data via WebSocket execute commands

---

### 🔄 NEEDED - For Production

#### Authentication
- [ ] Remove `Environment=TEST_AUTH_BYPASS=1` from service file
- [ ] Ensure polkit agent is running (gnome-polkit, polkit-kde-agent, or polkit-gnome)
- [ ] Test real PAM authentication flow end-to-end
- [ ] Handle pkexec timeout/graceful fallback

#### Security
- [ ] Add SSL/TLS via reverse proxy (nginx/Caddy with Let's Encrypt)
- [ ] Configure firewall (ufw allow 8080, or better: 443 only via proxy)
- [ ] Add rate limiting on WebSocket connections
- [ ] Add CSRF protection for state-changing operations
- [ ] Audit all command execution paths for injection

#### Terminal (PTY)
- [ ] Full PTY bridge implementation using portable-pty
- [ ] Resize handling (SIGWINCH)
- [ ] Copy/paste support
- [ ] Session persistence across reconnects

#### Plugin System
- [ ] Dynamic plugin manifest loading from `/usr/share/omarchy/plugins/`
- [ ] Plugin enable/disable UI
- [ ] Sandboxed plugin execution (separate processes)

---

### ❌ MISSING - From Original Plan

#### Phase 5: Plugin Management System
- [ ] Manifest parser for plugin metadata
- [ ] Dynamic loading of plugin UI components
- [ ] Plugin marketplace/repository integration

#### Phase 8: Btrfs Snapshots Plugin (Advanced)
- [ ] Snapshot creation with custom names
- [ ] Snapshot deletion with confirmation
- [ ] Rollback interface (boot environment selection)
- [ ] Integration with grub-btrfs / snapper-gui

#### Phase 10: Terminal and Agents Plugin (Advanced)
- [ ] Full PTY implementation (currently partial)
- [ ] Multiple terminal sessions
- [ ] Terminal session sharing
- [ ] Agent/process management UI (beyond systemd services)

#### Phase 11: DevOps Lead & Pipeline Architecture
- [ ] Automated release tagging and artifact generation
- [ ] AUR package submission workflow
- [ ] Dependency update automation (Dependabot/Renovate)

#### Phase 12: Quality Assurance & Testing
- [ ] Bash-based test suite (`test/shell/`)
- [ ] Rust unit tests for backend modules
- [ ] Playwright E2E tests for frontend
- [ ] Visual regression testing against Omarchy themes
- [ ] Integration tests with mocked DBus/Polkit

#### Phase 13: Code Review & PR Standards
- [ ] PR template with "The problem", "The change", "Testing", "Context"
- [ ] Automated linting in CI (shellcheck, clippy, prettier)
- [ ] Security review checklist

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Browser                                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   Login      │  │   Main App   │  │   Terminal   │       │
│  │   (Omarchy   │  │   (Web TUI)  │  │   (xterm.js) │       │
│  │   Unlock)    │  │              │  │              │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
└─────────┼─────────────────┼─────────────────┼────────────────┘
          │                 │                 │
          │         ┌───────┴───────┐        │
          │         │  WebSocket    │        │
          │         │  (ws://)      │        │
          │         └───────┬───────┘        │
          │                 │                 │
          ▼                 ▼                 ▼
┌─────────────────────────────────────────────────────────────┐
│                    web-omarchy-daemon (Rust)                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  HTTP/WS    │  │   Auth      │  │   Command Bridge    │  │
│  │  Server     │  │  (PAM/Polkit)│  │   (omarchy-*)       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
          │                 │                 │
          ▼                 ▼                 ▼
   ┌────────────┐    ┌────────────┐    ┌────────────┐
   │ Static     │    │  Polkit    │    │ omarchy-*  │
   │ Files      │    │  Agent     │    │  commands  │
   └────────────┘    └────────────┘    └────────────┘
```

---

## Quick Start

```bash
# Install
sudo /home/gabriel/Projects/web_omarchy/install-complete.sh

# Access
http://localhost:8080/login.html
# Currently: any credentials work (TEST_AUTH_BYPASS=1)
```

## Project Structure

```
web_omarchy/
├── backend/                    # Rust daemon
│   ├── src/
│   │   ├── main.rs            # Axum HTTP/WS server
│   │   ├── protocol.rs        # JSON-RPC message types
│   │   ├── auth.rs            # PAM session management
│   │   ├── bridge.rs          # Command execution
│   │   └── router.rs          # Message routing
│   └── Cargo.toml
├── frontend/                   # Vanilla JS Web TUI
│   ├── src/
│   │   ├── main.js            # App init, WS client
│   │   ├── login.js           # Login form handler
│   │   ├── panels/            # Feature panels
│   │   └── styles/            # CSS (theming, TUI, components)
│   ├── package.json
│   └── vite.config.js
├── omarchy-commands/           # System command binaries
│   ├── src/*.rs               # 8 command implementations
│   └── Cargo.toml
├── packaging/                  # Arch Linux packaging
│   ├── PKGBUILD
│   ├── web-omarchy-daemon.service
│   ├── web-omarchy.install
│   ├── config.toml
│   └── polkit/
├── .github/workflows/          # CI/CD
│   ├── ci.yml
│   └── release.yml
├── install.sh                  # Quick install
├── install-complete.sh         # Full build + install
└── phase-*.md                  # Original phase specifications
```

---

## Development

```bash
# Backend
cd backend && cargo build --release
cargo check && cargo clippy

# Frontend
cd frontend && npm run build && npm run lint

# Commands
cd omarchy-commands && cargo build --release

# Run locally (without systemd)
cd backend && cargo run
# Then open http://localhost:8080
```

## License

GPL-3.0-or-later