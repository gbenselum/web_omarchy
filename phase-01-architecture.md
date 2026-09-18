# Phase 1: Architecture Investigation & Core Extraction

## Agent Assignment: Agent 1 (Architecture Lead)

### Objective
Analyze the existing Cockpit architecture, particularly the `cockpit-bridge` and WebSocket communication layer, and design a stripped-down, ultra-fast backend architecture suitable for Omarchy Linux.

### Key Requirements
1. **Analyze Cockpit C/Bridge Protocol:** Understand how Cockpit translates WebSocket JSON payloads into DBus calls and spawned processes. Determine the absolute minimum required to maintain this interaction pattern.
2. **Language Selection:** Propose a backend language (preferably Rust, or Go) that aligns with Omarchy/Arch Linux tooling, prioritizes memory safety, and guarantees rapid startup times.
3. **Privilege Model Design:** Document how the daemon will run (e.g., as a dedicated unprivileged user) and how it will securely escalate privileges to perform OS-level tasks (e.g., via polkit, directly executing sudo, or a custom secure bridge).
4. **WebSocket/JSON Spec:** Define the simplified JSON schema for frontend-to-backend communication. Strip away all legacy or unused Cockpit protocol bloat.

### Deliverables
- Architecture Design Document outlining the new minimal bridge.
- Specifications for the WebSocket messaging protocol.
- Security and privilege escalation model diagram.
