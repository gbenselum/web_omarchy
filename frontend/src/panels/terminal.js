export class TerminalPanel {
  constructor(app) {
    this.app = app;
    this.ptyId = null;
    this.terminal = null;
  }

  onActivate() {
    if (!this.terminal) {
      this.initTerminal();
    }
  }

  async initTerminal() {
    const container = document.getElementById("terminal-container");
    if (!container) return;

    try {
      const { Terminal } =
        await import("https://cdn.jsdelivr.net/npm/@xterm/xterm@5.5.0/+esm");
      const { FitAddon } =
        await import("https://cdn.jsdelivr.net/npm/@xterm/addon-fit@0.10.0/+esm");
      const { WebLinksAddon } =
        await import("https://cdn.jsdelivr.net/npm/@xterm/addon-web-links@0.11.0/+esm");

      this.terminal = new Terminal({
        cursorBlink: true,
        fontFamily: "JetBrains Mono, Fira Code, monospace",
        fontSize: 14,
        lineHeight: 1.4,
        theme: {
          background: getComputedStyle(document.documentElement)
            .getPropertyValue("--bg-secondary")
            .trim(),
          foreground: getComputedStyle(document.documentElement)
            .getPropertyValue("--fg-primary")
            .trim(),
          cursor: getComputedStyle(document.documentElement)
            .getPropertyValue("--accent-primary")
            .trim(),
          selection: "rgba(137, 180, 250, 0.3)",
        },
      });

      const fitAddon = new FitAddon();
      this.terminal.loadAddon(fitAddon);
      this.terminal.loadAddon(new WebLinksAddon());

      container.innerHTML = "";
      this.terminal.open(container);
      fitAddon.fit();

      window.addEventListener("resize", () => fitAddon.fit());

      this.terminal.onData((data) => {
        if (this.app.ws && this.ptyId) {
          this.app.send({
            action: "pty.stdin",
            channel_id: this.ptyId,
            data: data,
          });
        }
      });

      this.ptyId = `pty-${Date.now()}`;
      this.app.send({
        action: "pty.spawn",
        channel_id: this.ptyId,
        cols: this.terminal.cols,
        rows: this.terminal.rows,
      });

      document
        .getElementById("terminal-clear")
        ?.addEventListener("click", () => this.terminal.clear());
      document
        .getElementById("terminal-reset")
        ?.addEventListener("click", () => this.terminal.reset());
    } catch (err) {
      console.error("Failed to initialize terminal:", err);
      container.innerHTML = `<div class="terminal-loader">Failed to load terminal: ${err.message}</div>`;
    }
  }

  handleStream(msg) {
    if (!this.ptyId || !msg.channel_id.startsWith("pty-")) return;

    if (msg.data.type === "stdout") {
      this.terminal?.write(msg.data.stdout);
    }
  }
}
