# Phase 9: Updates Plugin Development

## Agent Assignment: Agent 9 (Systems Integrator - Packages)

### Objective
Develop a plugin for managing system updates natively via Arch Linux tools.

### Key Requirements
1. **Pacman Integration:** Interface with `pacman` (and potentially an AUR helper like `yay` or `paru` if standard in Omarchy). Do *not* use PackageKit if native pacman output can be parsed cleanly (for performance).
2. **Update Checking:** Implement a non-blocking background check for available updates (`checkupdates` script is a good, safe reference).
3. **Web TUI Interface:** Display a list of available updates (Package Name, Current Version, New Version, Size).
4. **Update Execution:** Provide a mechanism to execute the update process. Since this requires a TTY to handle pacman's interactive prompts, this module *must* integrate heavily with the Terminal bridge (Phase 10) to stream output and accept user input during the update.

### Deliverables
- Backend update checker and pacman wrapper.
- Frontend updates list.
- Integration logic to hand off the update execution to the web terminal.
