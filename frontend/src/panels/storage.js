export class StoragePanel {
  constructor(app) {
    this.app = app;
  }

  onActivate() {
    this.bindEvents();
    this.refresh();
  }

  bindEvents() {
    document
      .getElementById("refresh-storage")
      ?.addEventListener("click", () => this.refresh());
  }

  refresh() {
    this.app.send({
      action: "execute",
      channel_id: `storage-${Date.now()}`,
      command: "omarchy-storage-list",
      args: [],
    });
  }

  handleStream(msg) {
    if (!msg.channel_id.startsWith("storage-")) return;

    if (msg.data.type === "stdout") {
      this.parseStorageInfo(msg.data.stdout);
    }
  }

  parseStorageInfo(output) {
    const lines = output.trim().split("\n");
    const tbody = document.querySelector("#storage-table tbody");
    if (!tbody) return;

    tbody.innerHTML = "";

    for (const line of lines) {
      if (!line.trim()) continue;

      const parts = line.split(/\s+/);
      if (parts.length >= 4) {
        const [device, size, type, mountpoint, ...rest] = parts;
        const usage = rest.join(" ") || "--";
        const smart = "OK";

        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td><code>${device}</code></td>
          <td>${size}</td>
          <td>${type}</td>
          <td><code>${mountpoint}</code></td>
          <td>${usage}</td>
          <td><span class="status-badge active">${smart}</span></td>
          <td>
            <button class="tui-btn tui-btn-secondary action-btn" data-action="mount" data-device="${device}" ${mountpoint !== "-" ? "disabled" : ""}>
              Mount
            </button>
            <button class="tui-btn tui-btn-secondary action-btn" data-action="unmount" data-device="${device}" ${mountpoint === "-" ? "disabled" : ""}>
              Unmount
            </button>
          </td>
        `;
        tbody.appendChild(tr);
      }
    }

    tbody.querySelectorAll('[data-action="mount"]').forEach((btn) => {
      btn.addEventListener("click", (e) =>
        this.mountDevice(e.target.dataset.device),
      );
    });
    tbody.querySelectorAll('[data-action="unmount"]').forEach((btn) => {
      btn.addEventListener("click", (e) =>
        this.unmountDevice(e.target.dataset.device),
      );
    });
  }

  mountDevice(device) {
    this.app.send({
      action: "execute",
      channel_id: `storage-mount-${Date.now()}`,
      command: "omarchy-storage-mount",
      args: [device],
    });
  }

  unmountDevice(device) {
    this.app.send({
      action: "execute",
      channel_id: `storage-unmount-${Date.now()}`,
      command: "omarchy-storage-unmount",
      args: [device],
    });
  }
}
