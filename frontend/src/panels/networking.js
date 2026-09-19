export class NetworkingPanel {
  constructor(app) {
    this.app = app;
    this.interfaces = [];
  }

  onActivate() {
    this.bindEvents();
    this.refresh();
  }

  bindEvents() {
    document
      .getElementById("refresh-network")
      ?.addEventListener("click", () => this.refresh());
  }

  refresh() {
    this.app.send({
      action: "execute",
      channel_id: `network-${Date.now()}`,
      command: "omarchy-network-status",
      args: [],
    });
  }

  handleStream(msg) {
    if (!msg.channel_id.startsWith("network-")) return;

    if (msg.data.type === "stdout") {
      this.parseNetworkInfo(msg.data.stdout);
    }
  }

  parseNetworkInfo(output) {
    const lines = output.trim().split("\n");
    const tbody = document.querySelector("#network-table tbody");
    if (!tbody) return;

    tbody.innerHTML = "";

    for (const line of lines) {
      if (!line.trim()) continue;

      const parts = line.split(/\s+/);
      if (parts.length >= 4) {
        const [name, status, ip, mac, ...rest] = parts;
        const rxTx = rest.join(" ") || "--";

        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td><code>${name}</code></td>
          <td><span class="status-badge ${status === "up" ? "active" : "inactive"}">${status}</span></td>
          <td><code>${ip}</code></td>
          <td><code>${mac}</code></td>
          <td><code>${rxTx}</code></td>
          <td>
            <button class="tui-btn tui-btn-secondary action-btn" data-action="toggle" data-iface="${name}">
              ${status === "up" ? "Down" : "Up"}
            </button>
          </td>
        `;
        tbody.appendChild(tr);
      }
    }

    tbody.querySelectorAll('[data-action="toggle"]').forEach((btn) => {
      btn.addEventListener("click", (e) =>
        this.toggleInterface(e.target.dataset.iface),
      );
    });
  }

  toggleInterface(iface) {
    const action =
      this.interfaces.find((i) => i.name === iface)?.status === "up"
        ? "down"
        : "up";
    this.app.send({
      action: "execute",
      channel_id: `network-toggle-${Date.now()}`,
      command: "omarchy-network-toggle",
      args: [iface, action],
    });
  }
}
