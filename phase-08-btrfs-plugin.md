# Phase 8: Btrfs Snapshots Plugin Development

## Agent Assignment: Agent 8 (Systems Integrator - Filesystems)

### Objective
Create a specialized plugin for managing Btrfs snapshots, as this is a critical feature for modern Linux distributions.

### Key Requirements
1. **Tooling Integration:** Determine if Omarchy uses a specific wrapper (like `snapper` or `timeshift`) or relies on raw `btrfs-progs`. Interface with the appropriate tool.
2. **Snapshot Listing:** Retrieve and display a chronological list of snapshots, detailing size, type, and creation time.
3. **Actions:**
    - Create manual snapshots.
    - Delete snapshots.
    - (Optional but highly desired) Provide an interface to initiate a rollback (note: this requires careful handling of boot environments).

### Deliverables
- Backend Btrfs command wrapper/DBus integration.
- Frontend snapshot management table and action buttons.
