class LoginApp {
  constructor() {
    this.form = document.getElementById("login-form");
    this.usernameInput = document.getElementById("username");
    this.passwordInput = document.getElementById("password");
    this.submitBtn = document.getElementById("login-submit");
    this.messageEl = document.getElementById("form-message");
    this.passwordToggle = document.querySelector(".password-toggle");

    this.bindEvents();
  }

  bindEvents() {
    this.form.addEventListener("submit", (e) => this.handleSubmit(e));
    this.passwordToggle?.addEventListener("click", () => this.togglePassword());

    this.usernameInput.addEventListener("input", () =>
      this.clearError("username"),
    );
    this.passwordInput.addEventListener("input", () =>
      this.clearError("password"),
    );

    document.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && e.target === this.passwordInput) {
        this.handleSubmit(e);
      }
    });
  }

  togglePassword() {
    const type = this.passwordInput.type === "password" ? "text" : "password";
    this.passwordInput.type = type;
    this.passwordToggle.querySelector(".toggle-icon").textContent =
      type === "password" ? "▾" : "▴";
  }

  clearError(field) {
    const errorEl = document.getElementById(`${field}-error`);
    if (errorEl) errorEl.textContent = "";
    this.hideMessage();
  }

  showError(field, message) {
    const errorEl = document.getElementById(`${field}-error`);
    if (errorEl) errorEl.textContent = message;
  }

  showMessage(message, type = "error") {
    this.messageEl.textContent = message;
    this.messageEl.className = `form-message ${type}`;
    this.messageEl.hidden = false;
  }

  hideMessage() {
    this.messageEl.hidden = true;
  }

  setLoading(loading) {
    this.submitBtn.disabled = loading;
    this.submitBtn.querySelector(".btn-text").hidden = loading;
    this.submitBtn.querySelector(".btn-loader").hidden = !loading;
  }

  async handleSubmit(e) {
    e.preventDefault();
    this.hideMessage();

    const username = this.usernameInput.value.trim();
    const password = this.passwordInput.value;

    if (!username) {
      this.showError("username", "Username is required");
      this.usernameInput.focus();
      return;
    }

    if (!password) {
      this.showError("password", "Password is required");
      this.passwordInput.focus();
      return;
    }

    this.setLoading(true);

    try {
      const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
      const wsUrl = `${protocol}//${window.location.host}/ws`;

      const ws = new WebSocket(wsUrl);
      ws.binaryType = "arraybuffer";

      await new Promise((resolve, reject) => {
        const timeout = setTimeout(
          () => reject(new Error("Connection timeout")),
          10000,
        );

        ws.onopen = () => {
          console.log('WebSocket connected');
          clearTimeout(timeout);
          ws.send(
            JSON.stringify({
              action: "auth.login",
              username,
              password,
            }),
          );
        };

        ws.onmessage = (event) => {
          console.log('Received message:', event.data);
          try {
            const msg = JSON.parse(event.data);
            console.log('Parsed message:', msg);
            if (msg.type === "auth.response") {
              if (msg.success) {
                localStorage.setItem("web-omarchy-session", msg.session_id);
                localStorage.setItem("web-omarchy-user", msg.user);
                resolve();
              } else {
                reject(new Error(msg.error || "Authentication failed"));
              }
              ws.close();
            }
          } catch (err) {
            reject(err);
          }
        };

        ws.onerror = (err) => {
          console.error('WebSocket error:', err);
          reject(new Error("Connection failed"));
        };
        ws.onclose = (e) => {
          console.log('WebSocket closed:', e.code, e.reason);
          clearTimeout(timeout);
        };
      });

      this.showMessage("Authentication successful. Redirecting...", "success");
      setTimeout(() => {
        window.location.href = "/";
      }, 500);
    } catch (err) {
      console.error('Login error:', err);
      this.showMessage(err.message || "Login failed. Please try again.");
      this.setLoading(false);
    }
  }
}

document.addEventListener("DOMContentLoaded", () => {
  new LoginApp();
});
