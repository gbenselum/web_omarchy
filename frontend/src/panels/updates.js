export class UpdatesPanel {
  constructor(app) {
    this.app = app;
    this.updates = [];
  }

  onActivate() {
    this.bindEvents();
    this.checkUpdates();
  }

  bindEvents() {
    document
      .getElementById("check-updates")
      ?.addEventListener("click", () => this.checkUpdates());
    document
      .getElementById("apply-updates")
      ?.addEventListener("click", () => this.applyUpdates());
    document
      .getElementById("select-all-updates")
      ?.addEventListener("change", (e) => this.toggleAll(e.target.checked));
  }

  checkUpdates() {
    this.app.send({
      action: "execute",
      channel_id: `updates-check-${Date.now()}`,
      command: "omarchy-pkg-check",
      args: [],
    });
  }

  handleStream(msg) {
    if (!msg.channel_id.startsWith("updates-")) return;

    if (msg.data.type === "stdout") {
      this.parseUpdates(msg.data.stdout);
    }
  }

  parseUpdates(output) {
    const lines = output.trim().split("\n");
    const tbody = document.querySelector("#updates-table tbody");
    const summary = document.getElementById("update-summary");
    const applyBtn = document.getElementById("apply-updates");

    if (!tbody) return;

    this.updates = [];
    tbody.innerHTML = "";

    for (const line of lines) {
      if (!line.trim()) continue;

      const parts = line.split(/\s+/);
      if (parts.length >= 4) {
        const [name, current, newVer, size, repo] = parts;
        this.updates.push({
          name,
          current,
          newVer,
          size,
          repo,
          selected: true,
        });

        const tr = document.createElement("tr");
        tr.innerHTML = `
          <td><input type="checkbox" class="update-checkbox" data-name="${name}" checked></td>
          <td><code>${name}</code></td>
          <td>${current}</td>
          <td>${newVer}</td>
          <td>${size}</td>
          <td>${repo}</td>
        `;
        tbody.appendChild(tr);
      }
    }

    if (this.updates.length === 0) {
      summary.hidden = false;
      summary.querySelector(".summary-text").textContent =
        "No updates available";
      applyBtn.disabled = true;
    } else {
      summary.hidden = false;
      summary.querySelector(".summary-text").textContent =
        `${this.updates.length} updates available`;
      applyBtn.disabled = false;
    }

    tbody.querySelectorAll(".update-checkbox").forEach((cb) => {
      cb.addEventListener("change", (e) =>
        this.toggleUpdate(e.target.dataset.name, e.target.checked),
      );
    });
  }

  toggleUpdate(name, selected) {
    const update = this.updates.find((u) => u.name === name);
    if (update) update.selected = selected;

    const allChecked = this.updates.every((u) => u.selected);
    const selectAll = document.getElementById("select-all-updates");
    if (selectAll) selectAll.checked = allChecked;
  }

  toggleAll(selected) {
    this.updates.forEach((u) => (u.selected = selected));
    document.querySelectorAll(".update-checkbox").forEach((cb) => {
      cb.checked = selected;
    });
  }

  applyUpdates() {
    const selected = this.updates.filter((u) => u.selected).map((u) => u.name);
    if (selected.length === 0) return;

    this.app.send({
      action: "pty.spawn",
      channel_id: `updates-apply-${Date.now()}`,
      cols: 120,
      rows: 30,
    });

    setTimeout(() => {
      this.app.send({
        action: "pty.stdin",
        channel_id: `updates-apply-${Date.now()}`,
        data: `sudo pacman -S ${selected.join(" ")}\n`,
      });
    }, 500);
  }
}
