import { WebSocketClient } from "./ws-client.js";
import { ThemeManager } from "./theme.js";
import { TabManager } from "./tabs.js";
import { PluginManager } from "./plugins.js";
import { Dashboard } from "./panels/dashboard.js";
import { NetworkingPanel } from "./panels/networking.js";
import { StoragePanel } from "./panels/storage.js";
import { BtrfsPanel } from "./panels/btrfs.js";
import { UpdatesPanel } from "./panels/updates.js";
import { TerminalPanel } from "./panels/terminal.js";
import { ServicesPanel } from "./panels/services.js";
import { Modal } from "./modal.js";

class App {
  constructor() {
    this.ws = null;
    this.sessionId = null;
    this.user = null;
    this.panels = {
      dashboard: new Dashboard(this),
      networking: new NetworkingPanel(this),
      storage: new StoragePanel(this),
      btrfs: new BtrfsPanel(this),
      updates: new UpdatesPanel(this),
      terminal: new TerminalPanel(this),
      services: new ServicesPanel(this),
    };
    this.currentPanel = "dashboard";
    this.modal = new Modal();
  }

  async init() {
    ThemeManager.init();
    this.bindEvents();
    this.initPanels();
    await this.checkAuth();
  }

  initPanels() {
    for (const [name, panel] of Object.entries(this.panels)) {
      if (panel.init) panel.init();
    }
  }

  bindEvents() {
    document
      .getElementById("logout-btn")
      .addEventListener("click", () => this.logout());

    document.querySelectorAll(".tab-btn").forEach((btn) => {
      btn.addEventListener("click", (e) =>
        this.switchTab(e.target.dataset.tab),
      );
    });

    document.querySelectorAll(".plugin-menu li").forEach((item) => {
      item.addEventListener("click", (e) =>
        this.switchTab(e.target.dataset.plugin),
      );
    });

    window.addEventListener("keydown", (e) => this.handleGlobalKeys(e));
  }

  handleGlobalKeys(e) {
    if (e.key === "Tab" && !e.target.matches("input, textarea, select")) {
      e.preventDefault();
      this.focusNextTab(e.shiftKey);
    }
    if (e.key === "Escape") {
      this.modal.close();
    }
    if (e.key === "?" && !e.target.matches("input, textarea")) {
      e.preventDefault();
      this.showHelp();
    }
  }

  focusNextTab(backwards) {
    const tabs = Array.from(
      document.querySelectorAll(".tab-btn:not(:disabled)"),
    );
    const activeIndex = tabs.findIndex((t) => t.classList.contains("active"));
    const nextIndex = backwards
      ? (activeIndex - 1 + tabs.length) % tabs.length
      : (activeIndex + 1) % tabs.length;
    tabs[nextIndex].focus();
    tabs[nextIndex].click();
  }

  async checkAuth() {
    const sessionId = localStorage.getItem("web-omarchy-session");
    const user = localStorage.getItem("web-omarchy-user");

    if (sessionId && user) {
      this.sessionId = sessionId;
      this.user = user;
      await this.connect();
    } else {
      window.location.href = "/login.html";
    }
  }

  async connect() {
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const wsUrl = `${protocol}//${window.location.host}/ws`;

    this.ws = new WebSocketClient(wsUrl, {
      onOpen: () => this.onWsOpen(),
      onMessage: (msg) => this.onWsMessage(msg),
      onClose: () => this.onWsClose(),
      onError: (err) => this.onWsError(err),
    });

    this.ws.connect();
  }

  onWsOpen() {
    this.updateConnectionStatus(true);
    this.requestPluginList();
    this.loadDashboard();
  }

  onWsClose() {
    this.updateConnectionStatus(false);
    setTimeout(() => this.connect(), 5000);
  }

  onWsError(err) {
    console.error("WebSocket error:", err);
  }

  onWsMessage(msg) {
    switch (msg.type) {
      case "stream":
        this.handleStream(msg);
        break;
      case "plugin.list":
        this.handlePluginList(msg);
        break;
      case "systemd.units":
        this.handleSystemdUnits(msg);
        break;
      case "systemd.logs":
        this.handleSystemdLogs(msg);
        break;
      case "error":
        this.handleError(msg);
        break;
    }
  }

  handleStream(msg) {
    const panel = this.panels[this.currentPanel];
    if (panel && panel.handleStream) {
      panel.handleStream(msg);
    }
  }

  handlePluginList(msg) {
    PluginManager.render(msg.plugins);
  }

  handleSystemdUnits(msg) {
    const panel = this.panels.services;
    if (panel && panel.renderUnits) {
      panel.renderUnits(msg.units);
    }
  }

  handleSystemdLogs(msg) {
    const panel = this.panels.services;
    if (panel && panel.showLogs) {
      panel.showLogs(msg.logs);
    }
  }

  handleError(msg) {
    this.showToast(msg.message, "error");
  }

  updateConnectionStatus(connected) {
    const dot = document.querySelector(".status-dot");
    const text = document.querySelector(".status-text");
    if (connected) {
      dot.classList.remove("disconnected");
      dot.classList.add("connected");
      text.textContent = "Connected";
    } else {
      dot.classList.remove("connected");
      dot.classList.add("disconnected");
      text.textContent = "Disconnected";
    }
  }

  send(msg) {
    if (this.ws && this.ws.isConnected()) {
      this.ws.send(msg);
    }
  }

  requestPluginList() {
    this.send({
      action: "plugin.list",
      channel_id: `plugin-list-${Date.now()}`,
    });
  }

  loadDashboard() {
    this.send({
      action: "execute",
      channel_id: `dashboard-${Date.now()}`,
      command: "omarchy-system-info",
      args: [],
    });
  }

  switchTab(tabName) {
    document.querySelectorAll(".tab-btn").forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.tab === tabName);
      btn.setAttribute("aria-selected", btn.dataset.tab === tabName);
    });

    document.querySelectorAll(".plugin-menu li").forEach((item) => {
      item.classList.toggle("active", item.dataset.plugin === tabName);
    });

    document.querySelectorAll(".tui-panel").forEach((panel) => {
      panel.classList.toggle("active", panel.id === `panel-${tabName}`);
    });

    this.currentPanel = tabName;

    if (this.panels[tabName] && this.panels[tabName].onActivate) {
      this.panels[tabName].onActivate();
    }
  }

  showToast(message, type = "info") {
    const toast = document.createElement("div");
    toast.className = `toast toast-${type}`;
    toast.textContent = message;
    toast.style.cssText = `
      position: fixed; bottom: 50px; right: 20px; z-index: 1000;
      padding: 12px 20px; border-radius: var(--radius-md);
      background: var(--bg-secondary); border: 1px solid var(--border-primary);
      color: var(--fg-primary); animation: slideIn 0.3s ease;
    `;
    document.body.appendChild(toast);
    setTimeout(() => {
      toast.style.animation = "slideOut 0.3s ease";
      setTimeout(() => toast.remove(), 300);
    }, 3000);
  }

  showHelp() {
    this.modal.open({
      title: "Keyboard Shortcuts",
      body: `
        <div class="help-content">
          <h4>Global</h4>
          <dl>
            <dt><kbd>Tab</kbd></dt><dd>Next tab</dd>
            <dt><kbd>Shift+Tab</kbd></dt><dd>Previous tab</dd>
            <dt><kbd>Esc</kbd></dt><dd>Close modal</dd>
            <dt><kbd>?</kbd></dt><dd>Show this help</dd>
          </dl>
          <h4>Terminal</h4>
          <dl>
            <dt><kbd>Ctrl+C</kbd></dt><dd>Interrupt</dd>
            <dt><kbd>Ctrl+D</kbd></dt><dd>Exit shell</dd>
            <dt><kbd>Ctrl+L</kbd></dt><dd>Clear screen</dd>
          </dl>
        </div>
      `,
      confirmText: "Close",
      onConfirm: () => this.modal.close(),
    });
  }

  logout() {
    localStorage.removeItem("web-omarchy-session");
    localStorage.removeItem("web-omarchy-user");
    if (this.ws) {
      this.ws.close();
    }
    window.location.href = "/login.html";
  }
}

document.addEventListener("DOMContentLoaded", () => {
  const app = new App();
  app.init();
  window.app = app;
});

export { App };
