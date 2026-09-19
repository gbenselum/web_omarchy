export class ThemeManager {
  static init() {
    const savedTheme =
      localStorage.getItem("web-omarchy-theme") || "catppuccin-mocha";
    this.applyTheme(savedTheme);
    this.loadSystemTheme();
  }

  static applyTheme(themeName) {
    document.documentElement.setAttribute("data-theme", themeName);
    localStorage.setItem("web-omarchy-theme", themeName);
  }

  static async loadSystemTheme() {
    try {
      const response = await fetch("/api/theme");
      if (response.ok) {
        const data = await response.json();
        if (data.theme) {
          this.applyTheme(data.theme);
        }
      }
    } catch (err) {
      console.debug("Could not load system theme:", err);
    }
  }

  static getCurrentTheme() {
    return localStorage.getItem("web-omarchy-theme") || "catppuccin-mocha";
  }

  static getAvailableThemes() {
    return [
      { id: "catppuccin-mocha", name: "Catppuccin Mocha", dark: true },
      { id: "catppuccin-latte", name: "Catppuccin Latte", dark: false },
      { id: "tokyo-night", name: "Tokyo Night", dark: true },
      { id: "dracula", name: "Dracula", dark: true },
      { id: "nord", name: "Nord", dark: true },
    ];
  }
}
