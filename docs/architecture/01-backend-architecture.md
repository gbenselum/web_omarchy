# Omarchy Web Management Interface - Backend Architecture

## 1. Overview
The Omarchy Web Management Interface is a stripped-down, ultra-fast web management interface specifically built for the Omarchy Linux distribution. It utilizes an "a la carte" plugin architecture and is inspired by the Cockpit project.

## 2. Cockpit Architecture Analysis
Cockpit relies on a `cockpit-bridge` program that acts as a translator between WebSocket JSON payloads from the web frontend and underlying system interfaces (primarily DBus and spawned background processes). The bridge itself runs with the privileges of the logged-in user, meaning privilege escalation is handled on a per-command basis.

For Omarchy, we will implement a minimalist version of this bridge pattern.

## 3. Language Selection: Rust
Following the `omacom` organization's preference and the requirement for a stripped-down, ultra-fast backend, **Rust** has been selected as the core language for the backend daemon.
- **Memory Safety:** Rust guarantees memory safety without needing a garbage collector, preventing common security vulnerabilities (e.g., buffer overflows) crucial for a system management tool.
- **Fast Startup:** Compiled Rust binaries start almost instantly with minimal footprint, perfectly aligning with the Web TUI requirement for speed and responsiveness.
- **Ecosystem:** Rust has excellent libraries for WebSocket handling (e.g., `tokio`, `tungstenite`), JSON parsing (`serde`), and potentially DBus interactions (`zbus`).

*Go was considered, but Rust is preferred due to tighter memory control and predictable performance without GC pauses, which aligns better with system-level tooling on Arch/Omarchy Linux.*

## 4. Core Interaction Pattern
1.  **Frontend Connection:** The frontend establishes a secure WebSocket connection to the main web daemon.
2.  **JSON Routing:** The frontend sends simplified JSON payloads.
3.  **Bridge Translation:** The backend daemon acts as the bridge. Instead of legacy Cockpit bloat, it uses a strict routing system to match JSON requests to specific Omarchy backend command scripts (prefixed with `omarchy-`) or native system calls.
4.  **Process Spawning & Execution:** The backend safely spawns the necessary processes, capturing standard output and error, and streams the results back via the WebSocket connection.
