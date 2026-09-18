# Omarchy Web Management Interface - Agent Guidelines

This repository relies on automated AI agents (subagents) to execute development phases. To maintain consistency with the broader Omarchy ecosystem (`omacom` organization), all agents must adhere to the following rules, skills, and persona behaviors.

## Subagent Personas (The 13 Agents)
1. **Agent 1 (Architecture Lead):** Focuses on secure design, backend architecture, and WebSocket protocol definitions.
2. **Agent 2 (Backend Core Developer):** Writes the core daemon (Rust/Go), focusing on raw performance, strict JSON routing, and systemd integration.
3. **Agent 3 (Security & Identity Developer):** Handles PAM authentication, session state, and security hardening.
4. **Agent 4 (Frontend Systems Engineer):** Builds the Web TUI framework. Prioritizes lightweight JS/CSS, dynamic Omarchy theme loading, and minimal DOM manipulation.
5. **Agent 5 (Plugin Architecture Lead):** Designs the manifest parser and dynamic loading logic for the a la carte plugin system.
6. **Agent 6 (Systems Integrator - Networking):** Interfaces with NetworkManager/systemd-networkd.
7. **Agent 7 (Systems Integrator - Storage):** Interfaces with `lsblk`, `udisks2`, and `smartctl`.
8. **Agent 8 (Systems Integrator - Filesystems):** Interfaces with Btrfs snapshot tooling.
9. **Agent 9 (Systems Integrator - Packages):** Interfaces securely with `pacman` and AUR helpers.
10. **Agent 10 (Systems Integrator - Execution):** Implements PTY bridges and systemd service management.
11. **Agent 11 (DevOps Lead):** Manages CI/CD pipelines, Arch Linux packaging (`PKGBUILD`), and enforces repository commit standards.
12. **Agent 12 (QA Lead):** Constructs Bash-based test suites mapping to Omarchy's `test/` structure and orchestrates E2E frontend acceptance tests.
13. **Agent 13 (Code Review Lead):** Enforces strict PR templates ("The problem", "The change", "Testing") and validates code against Omarchy's security and performance baselines.

## Code Style & Conventions

### Shell Scripts & Backend
- **Indentation:** Two spaces, no tabs.
- **Bash:** Use bash 5 conditionals (`[[ ]]` and `(( ))`). Never use `#!/usr/bin/env bash`; strictly use `#!/bin/bash`.
- **Naming:** Follow Omarchy standard command naming prefixes (e.g., `omarchy-setup-`, `omarchy-pkg-`, `omarchy-capture-`). If creating backend utility scripts, prefix them appropriately.
- **Paths:** Avoid hardcoded paths when variables like `$OMARCHY_PATH` or `$HOME/.config/omarchy` are applicable.

### Frontend (Web TUI)
- **Frameworks:** Do not use React, Vue, Angular, or PatternFly. Use Vanilla JS, Web Components, or minimal reactive libraries (Preact/Solid) to ensure rapid startup.
- **Theming:** All CSS must dynamically reference the current user's Omarchy OS theme variables (e.g., pulling from `~/.config/omarchy/themes/`). No hardcoded colors.
- **Accessibility/Keyboard:** Ensure Vim-style keybindings or clear tab-navigation where applicable, enforcing the "Web TUI" feel.

## Commit & PR Strategy (Agent 11 Enforced)
- **Commits:** Write clear, descriptive commit messages. Example: "Add a Grok collector to the agents usage panel". Avoid vague messages.
- **Scoping:** Commits should be atomic, focusing on a single feature or fix at a time.
- **Verification:** Before calling submit or proposing a PR, verify the work locally. For frontend changes, capture screenshots to verify theme compliance.

## Integration & Hardware Rules
- **Native Tools:** Rely on existing Arch Linux and Omarchy tooling first. Only install new dependencies if native options (like DBus, `systemctl`, `pacman`) cannot fulfill the requirement.
- **Privilege Escalation:** Never run the entire backend daemon as `root`. Drop privileges immediately, run as a dedicated user, and use `polkit` or explicit `sudo` wrappers for specific commands requiring elevation.

## Code Review Checklist (Agent 13)
All AI agents and human contributors must verify the following before proposing a PR:
- [ ] **PR Structure:** Does the PR use the standard template ("The problem", "The change", "Testing", "Context")?
- [ ] **Security:** Are there any hardcoded paths? Is the web process running as `root` (rejected!)? Is `sudo` used correctly/minimalistically?
- [ ] **Performance:** Are heavy web frameworks (React, Vue, Angular, PatternFly) avoided?
- [ ] **Style:** Are Bash 5 conditionals used (`[[ ]]`, `(( ))`)? Is the indentation 2 spaces? Is the shebang strictly `#!/bin/bash`? Are Omarchy naming conventions followed?
- [ ] **Testing:** Were automated tests written and verified?
