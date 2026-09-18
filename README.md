# Omarchy Web Management Interface

A stripped-down, ultra-fast web interface for Omarchy Linux, inspired by Cockpit.

## Overview
This project aims to extract the core logic of Cockpit (a web interface based on plugins) and strip it down to the absolute bare minimum, focusing heavily on security and responsiveness. Instead of using PatternFly, this project is built entirely around Omarchy themes.

- **Login UI:** Uses the specific **Omarchy unlock theme**.
- **Main App UI:** Dynamically reads and applies the **user's configured theme** from the operating system.
- **Design Philosophy:** The UI is designed to feel like a **Web TUI** (Terminal User Interface in the browser) rather than a traditional, heavy web application. It aims for speed, efficiency, and a keyboard-friendly layout.
- **Tech Stack:** Tailored for Arch Linux/Omarchy. The backend should ideally use Rust or Go for maximum performance and security, interfacing seamlessly with existing distro tools.

## Development Phases
The project is divided into 10 distinct phases, designed to be executed by separate specialized agents.

See the specification documents for details:
- [Phase 1: Architecture Investigation](phase-01-architecture.md)
- [Phase 2: Core Minimal Backend](phase-02-core-backend.md)
- [Phase 3: Authentication and Login UI](phase-03-auth-login.md)
- [Phase 4: Frontend Base Platform & Theming](phase-04-frontend-platform.md)
- [Phase 5: Plugin Management System](phase-05-plugin-management.md)
- [Phase 6: Networking Plugin](phase-06-networking-plugin.md)
- [Phase 7: Storage Plugin](phase-07-storage-plugin.md)
- [Phase 8: Btrfs Snapshots Plugin](phase-08-btrfs-plugin.md)
- [Phase 9: Updates Plugin](phase-09-updates-plugin.md)
- [Phase 10: Terminal and Agents Plugin](phase-10-terminal-and-agents.md)
