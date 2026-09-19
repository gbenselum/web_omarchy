export class BtrfsPanel {
  constructor(app) {
    this.app = app;
  }

  onActivate() {
    this.bindEvents();
    this.refresh();
  }

  bindEvents() {
    document
      .getElementById("create-snapshot")
      ?.addEventListener("click", () => this.createSnapshot());
    document
      .getElementById("refresh-btrfs")
      ?.addEventListener("click", () => this.refresh());
  }

  refresh() {
    this.app.send({
      action: "execute",
      channel_id: `btrfs-${Date.now()}`,
      command: "omarchy-btrfs-list",
      args: [],
    });
  }

  handleStream(msg) {
    if (!msg.channel_id.startsWith("btrfs-")) return;

    if (msg.data.type === "stdout") {
      this.parseBtrfsInfo(msg.data.stdout);
    }
  }

  parseBtrfsInfo(output) {
    const lines = output.trim().split("\n");
    const tbody = document.querySelector("#btrfs-table tbody");
    if (!tbody) return;

    tbody.innerHTML = "";

    for (const line of lines) {
      if (!line.trim()) continue;

      const parts = line.split(/\s+/);
      if (parts.length >= 4) {
        const [id, name, created, size, type, ...rest] = parts;

        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td><code>${id}</code></td>
          <td>${name}</td>
          <td>${created}</td>
          <td>${size}</td>
          <td>${type}</td>
          <td>
            <button class="tui-btn tui-btn-danger action-btn" data-action="delete" data-id="${id}">
              Delete
            </button>
          </td>
        `;
        tbody.appendChild(tr);
      }
    }

    tbody.querySelectorAll('[data-action="delete"]').forEach((btn) => {
      btn.addEventListener("click", (e) =>
        this.deleteSnapshot(e.target.dataset.id),
      );
    });
  }

  createSnapshot() {
    const name = prompt("Enter snapshot name:");
    if (!name) return;

    this.app.send({
      action: "execute",
      channel_id: `btrfs-create-${Date.now()}`,
      command: "omarchy-btrfs-create",
      args: [name],
    });
  }

  deleteSnapshot(id) {
    if (!confirm(`Delete snapshot ${id}?`)) return;

    this.app.send({
      action: "execute",
      channel_id: `btrfs-delete-${Date.now()}`,
      command: "omarchy-btrfs-delete",
      args: [id],
    });
  }
}
