# Omarchy Web Management Interface - WebSocket Protocol Specification

## 1. Overview
The Omarchy WebSocket protocol is a brand-new, minimalist JSON schema for communication between the Web TUI frontend and the Rust backend bridge. It deliberately drops backward compatibility with Cockpit to eliminate legacy bloat and prioritize speed and clarity.

## 2. Core Principles
- **JSON-RPC Inspired:** The structure is loosely inspired by JSON-RPC but tailored for asynchronous stream processing and system command execution.
- **Strict Typing:** All payloads must conform to the defined schema.
- **Asynchronous Channels:** Each command execution opens a logical "channel" to allow for streaming output (e.g., standard output from a long-running bash script) before closing.

## 3. Schema Definitions

### 3.1 Client -> Server (Request)

**`CommandRequest`**
Sent by the frontend to initiate an action.

```json
{
  "channel_id": "string (UUID)",
  "action": "string (e.g., 'execute', 'read_file', 'dbus_call')",
  "payload": {
    // Action-specific payload
    "command": "omarchy-pkg-update",
    "args": ["--check"]
  }
}
```

### 3.2 Server -> Client (Response/Stream)

**`StreamResponse`**
Sent by the backend while a command is running, or to signal completion.

```json
{
  "channel_id": "string (UUID matching request)",
  "type": "string ('data', 'error', 'control')",
  "data": {
    // Type-specific data
    // If type == 'data': standard output fragment
    "stdout": "string"

    // If type == 'error': standard error fragment or execution failure
    // "stderr": "string",
    // "message": "string"

    // If type == 'control': signals like completion
    // "status": "completed",
    // "exit_code": 0
  }
}
```

## 4. Example Interaction flow

1. **Frontend Request:**
```json
{
  "channel_id": "req-1234",
  "action": "execute",
  "payload": {
    "command": "omarchy-network-status",
    "args": []
  }
}
```

2. **Backend Stream (Data):**
```json
{
  "channel_id": "req-1234",
  "type": "data",
  "data": {
    "stdout": "eth0: up\n"
  }
}
```

3. **Backend Stream (Control - Completion):**
```json
{
  "channel_id": "req-1234",
  "type": "control",
  "data": {
    "status": "completed",
    "exit_code": 0
  }
}
```
