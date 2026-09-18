# Omarchy Web Management Interface - Security Model

## 1. Overview
This document outlines the privilege separation and escalation model for the Omarchy Web Management Interface, heavily inspired by Cockpit to prevent security regressions.

## 2. Core Privilege Rules
- **No Root Daemon:** The web process/daemon must **never** run as the `root` user.
- **Dedicated User:** The daemon runs as a dedicated, unprivileged system user (e.g., `omarchy-web`).
- **User Sessions:** When a user logs in via the UI (using PAM authentication), a dedicated bridge session/process is spawned running as that authenticated user.

## 3. Privilege Escalation Strategy
Just like Cockpit, privilege escalation happens only when strictly necessary, rather than elevating the entire daemon.

- **Polkit / Sudo wrappers:** For tasks that require elevated privileges (e.g., package management, disk manipulation, systemd control), the bridge process will utilize `polkit` (preferred) or explicit `sudo` execution wrappers.
- The web interface will prompt the user for password verification when an action requires elevated privileges via `polkit`.

## 4. Architecture Diagram
```mermaid
sequenceDiagram
    participant Frontend as Web TUI (Frontend)
    participant Auth as Auth Daemon (PAM)
    participant Daemon as Main Web Daemon (Unprivileged)
    participant Bridge as User Session Bridge (Logged-in User)
    participant System as System (Polkit/Sudo/DBus)

    Frontend->>Auth: Login Request (User/Pass)
    Auth-->>Frontend: Token/Session Cookie
    Frontend->>Daemon: WebSocket Connection (w/ Token)
    Daemon->>Bridge: Spawn Bridge Process (as User)
    Bridge-->>Daemon: Bridge Ready
    Daemon-->>Frontend: WebSocket Established

    Frontend->>Daemon: Send JSON Command Request
    Daemon->>Bridge: Forward Request

    alt Needs Privileges
        Bridge->>System: Request Action via Polkit/Sudo
        System-->>Bridge: Prompt for Auth (if needed)
        Bridge-->>Frontend: Polkit Auth Request
        Frontend->>Bridge: Polkit Auth Response
        Bridge->>System: Execute Action
    else Unprivileged
        Bridge->>System: Execute Action (as User)
    end

    System-->>Bridge: Action Result
    Bridge-->>Daemon: Forward Result
    Daemon-->>Frontend: JSON Response over WebSocket
```
