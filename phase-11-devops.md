# Phase 11: DevOps Lead & Pipeline Architecture

## Agent Assignment: Agent 11 (DevOps Lead)

### Objective
Design and implement the DevOps persona, pipeline architecture, and deployment strategies matching the Omarchy ecosystem, based on conventions observed in the `omacom` organization.

### Key Requirements
1. **CI/CD Pipeline Design:**
   - Define GitHub Actions workflows for linting, testing, and building the web interface and backend daemon.
   - Implement automated release tagging and artifact generation for Arch Linux (`PKGBUILD`).
2. **Commit & PR Conventions:**
   - Enforce conventional commits and clear PR titles (e.g., matching the style of Omarchy PRs like "Add a Grok collector to the agents usage panel").
   - Set up automated branch management and PR templates.
3. **Packaging for Omarchy:**
   - Create the necessary Arch Linux packaging scripts (`PKGBUILD`, `.install` files) to package the application as an official or AUR package.
   - Ensure the installation respects Omarchy's paths and permissions (e.g., using `/usr/bin/` for binaries and `/etc/` or `~/.config/` for configs).
4. **Agent Operations Guide:**
   - Maintain the `AGENTS.md` to instruct AI agents and human contributors on the coding style, deployment scripts, and command prefixes required for the project.

### Deliverables
- GitHub Actions workflow files (`.github/workflows/`).
- `PKGBUILD` and packaging assets.
- PR/Issue templates (`.github/PULL_REQUEST_TEMPLATE.md`).
- A fully documented DevOps strategy aligning with Omarchy's deployment philosophy.
