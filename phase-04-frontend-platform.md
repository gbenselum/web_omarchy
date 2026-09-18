# Phase 4: Frontend Base Platform & Theming

## Agent Assignment: Agent 4 (Frontend Systems Engineer)

### Objective
Build the core frontend web shell (the "Web TUI") and implement the dynamic user theming system.

### Key Requirements
1. **Web TUI Architecture:** Design the frontend layout to mimic a Terminal User Interface. Focus on high-contrast, text-heavy data presentation, keyboard navigation (Vim-style bindings where appropriate), and minimal DOM overhead.
2. **Framework Selection:** Use Vanilla JS, Web Components, or an extremely lightweight reactive library (like SolidJS or Preact). Do *not* use heavy frameworks (React, Angular, Vue) or large UI libraries (PatternFly, Bootstrap).
3. **Dynamic Theming Engine:**
    - The backend must provide an endpoint to read the current user's OS theme configuration.
    - The frontend must dynamically load and apply the corresponding Omarchy theme CSS and assets to the web shell.
4. **WebSocket Client:** Implement a robust WebSocket client wrapper that handles auto-reconnection, state management, and dispatches messages to the appropriate UI components.

### Deliverables
- Base frontend HTML shell and routing logic.
- Dynamic theme loading mechanism reading from Omarchy OS configurations.
- Core WebSocket client service.
