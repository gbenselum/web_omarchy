export class PluginManager {
  static plugins = [];

  static render(plugins) {
    this.plugins = plugins;
    const menu = document.querySelector(".plugin-menu");
    if (!menu) return;

    const activePlugin = menu.querySelector("li.active")?.dataset.plugin;

    menu.innerHTML = plugins
      .map(
        (plugin) => `
      <li role="option" data-plugin="${plugin.id}" class="${plugin.id === activePlugin ? "active" : ""}">
        ▸ ${plugin.name}${!plugin.enabled ? " (disabled)" : ""}
      </li>
    `,
      )
      .join("");

    menu.querySelectorAll("li").forEach((item) => {
      item.addEventListener("click", (e) => {
        if (window.app) {
          window.app.switchTab(e.target.dataset.plugin);
        }
      });
    });
  }

  static getPlugin(id) {
    return this.plugins.find((p) => p.id === id);
  }

  static isEnabled(id) {
    const plugin = this.getPlugin(id);
    return plugin?.enabled ?? false;
  }
}
