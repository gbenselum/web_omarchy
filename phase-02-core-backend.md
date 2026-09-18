# Phase 2: Core Minimal Backend Development

## Agent Assignment: Agent 2 (Backend Core Developer)

### Objective
Develop the minimal, secure backend daemon based on the specifications from Phase 1.

### Key Requirements
1. **Daemon Implementation:** Build the core service (in Rust or Go). It must be a lightweight process meant to be managed by `systemd`.
2. **WebSocket Server:** Implement an asynchronous, high-performance WebSocket server to handle incoming connections from the web frontend.
3. **Message Router:** Implement the JSON message router that parses frontend requests and routes them to the appropriate OS-level handler (DBus, shell command, or plugin module).
4. **Security & Validation:** Ensure all inputs from the WebSocket are strictly validated. Implement rate limiting and connection boundaries.

### Deliverables
- Source code for the backend daemon.
- Systemd service unit files.
- Basic integration tests verifying WebSocket connectivity and message routing.
