export class Dashboard {
  constructor(app) {
    this.app = app;
  }

  onActivate() {
    this.app.loadDashboard();
  }

  handleStream(msg) {
    const channelId = msg.channel_id;
    if (!channelId.startsWith("dashboard-")) return;

    if (msg.data.type === "stdout") {
      this.parseSystemInfo(msg.data.stdout);
    }
  }

  parseSystemInfo(output) {
    const lines = output.trim().split("\n");
    const info = {};

    for (const line of lines) {
      const [key, ...valueParts] = line.split(":");
      if (key && valueParts.length > 0) {
        info[key.trim().toLowerCase().replace(/\s+/g, "-")] = valueParts
          .join(":")
          .trim();
      }
    }

    document.getElementById("sys-hostname").textContent = info.hostname || "--";
    document.getElementById("sys-kernel").textContent = info.kernel || "--";
    document.getElementById("sys-uptime").textContent = info.uptime || "--";
    document.getElementById("sys-packages").textContent = info.packages || "--";

    const logEl = document.getElementById("activity-log");
    if (logEl) {
      logEl.textContent = output;
    }
  }
}
