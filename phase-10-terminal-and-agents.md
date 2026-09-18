# Phase 10: Terminal and Agents Plugin Development

## Agent Assignment: Agent 10 (Systems Integrator - Execution)

### Objective
Implement a secure web-based terminal and a plugin to manage background agents/services.

### Key Requirements
1. **PTY Bridge (Backend):** Implement a robust pseudo-terminal (PTY) bridge in the backend. It must spawn a shell (e.g., `bash` or `zsh` depending on user config) and bi-directionally stream stdin/stdout/stderr over WebSockets.
2. **Web Terminal (Frontend):** Integrate a lightweight terminal emulator (like `xterm.js`, heavily optimized, or a custom lighter alternative if feasible for Web TUI goals). It must support standard ANSI escape codes, resizing, and copy/paste.
3. **Agents Plugin:**
    - Interface with `systemd` (via DBus) to list, start, stop, and restart user and system services ("agents").
    - Provide a view to read the `journalctl` logs for a specific agent.
    - UI must be a clean, filterable list of services and their status.

### Deliverables
- Backend PTY spawner and WebSocket streaming logic.
- Frontend Terminal component.
- Systemd service management backend and frontend UI.
