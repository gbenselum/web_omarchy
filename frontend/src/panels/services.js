export class ServicesPanel {
  constructor(app) {
    this.app = app;
    this.units = [];
  }

  onActivate() {
    this.bindEvents();
    this.refresh();
  }

  bindEvents() {
    document
      .getElementById("service-filter")
      ?.addEventListener("input", (e) => this.filter(e.target.value));
  }

  refresh() {
    this.app.send({
      action: "systemd.list_units",
      channel_id: `services-${Date.now()}`,
    });
  }

  handleStream(msg) {
    if (!msg.channel_id.startsWith("services-")) return;
  }

  renderUnits(units) {
    this.units = units;
    this.renderFiltered("");
  }

  filter(query) {
    this.renderFiltered(query.toLowerCase());
  }

  renderFiltered(query) {
    const tbody = document.querySelector("#services-table tbody");
    if (!tbody) return;

    const filtered = this.units.filter(
      (u) =>
        u.name.toLowerCase().includes(query) ||
        u.description.toLowerCase().includes(query),
    );

    tbody.innerHTML = "";

    for (const unit of filtered) {
      const tr = document.createElement("tr");
      tr.innerHTML = `
        <td><code>${unit.name}</code></td>
        <td>${unit.description}</td>
        <td><span class="status-badge ${unit.active_state === "active" ? "active" : "inactive"}">${unit.active_state}</span></td>
        <td><span class="status-badge ${unit.load_state === "enabled" ? "enabled" : "disabled"}">${unit.load_state}</span></td>
        <td>
          <button class="tui-btn tui-btn-secondary action-btn" data-action="start" data-unit="${unit.name}" ${unit.active_state === "active" ? "disabled" : ""}>Start</button>
          <button class="tui-btn tui-btn-secondary action-btn" data-action="stop" data-unit="${unit.name}" ${unit.active_state !== "active" ? "disabled" : ""}>Stop</button>
          <button class="tui-btn tui-btn-secondary action-btn" data-action="restart" data-unit="${unit.name}">Restart</button>
          <button class="tui-btn tui-btn-secondary action-btn" data-action="logs" data-unit="${unit.name}">Logs</button>
        </td>
      `;
      tbody.appendChild(tr);
    }

    tbody.querySelectorAll("[data-action]").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        const action = e.target.dataset.action;
        const unit = e.target.dataset.unit;
        this.handleAction(action, unit);
      });
    });
  }

  handleAction(action, unit) {
    switch (action) {
      case "start":
      case "stop":
      case "restart":
        this.app.send({
          action: "systemd.control",
          channel_id: `svc-${action}-${Date.now()}`,
          unit: unit,
          action: action,
        });
        break;
      case "logs":
        this.showLogs(unit);
        break;
    }
  }

  showLogs(unit) {
    this.app.send({
      action: "systemd.logs",
      channel_id: `svc-logs-${Date.now()}`,
      unit: unit,
      lines: 100,
    });
  }
}
