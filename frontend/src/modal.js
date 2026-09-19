export class Modal {
  constructor() {
    this.overlay = document.getElementById("modal-overlay");
    this.title = document.getElementById("modal-title");
    this.body = document.getElementById("modal-body");
    this.confirmBtn = this.overlay?.querySelector(".modal-confirm");
    this.cancelBtn = this.overlay?.querySelector(".modal-cancel");
    this.closeBtn = this.overlay?.querySelector(".modal-close");
    this.onConfirmCallback = null;
    this.onCancelCallback = null;

    this.bindEvents();
  }

  bindEvents() {
    if (!this.overlay) return;

    this.confirmBtn?.addEventListener("click", () => this.confirm());
    this.cancelBtn?.addEventListener("click", () => this.cancel());
    this.closeBtn?.addEventListener("click", () => this.cancel());

    this.overlay.addEventListener("click", (e) => {
      if (e.target === this.overlay) this.cancel();
    });

    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && !this.overlay.hasAttribute("hidden")) {
        this.cancel();
      }
    });
  }

  open(options = {}) {
    if (!this.overlay) return;

    this.title.textContent = options.title || "Confirm";
    this.body.innerHTML = options.body || "";
    this.confirmBtn.textContent = options.confirmText || "Confirm";
    this.cancelBtn.textContent = options.cancelText || "Cancel";

    this.confirmBtn.className = `tui-btn ${options.confirmClass || "tui-btn-primary"} modal-confirm`;
    this.cancelBtn.className = `tui-btn tui-btn-secondary modal-cancel`;

    this.onConfirmCallback = options.onConfirm || null;
    this.onCancelCallback = options.onCancel || null;

    this.overlay.hidden = false;
    this.confirmBtn.focus();
  }

  confirm() {
    this.close();
    if (this.onConfirmCallback) this.onConfirmCallback();
  }

  cancel() {
    this.close();
    if (this.onCancelCallback) this.onCancelCallback();
  }

  close() {
    if (this.overlay) {
      this.overlay.hidden = true;
    }
    this.onConfirmCallback = null;
    this.onCancelCallback = null;
  }
}
