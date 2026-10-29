import { openUrl } from "@tauri-apps/plugin-opener";
import { getConfig } from "../api";
import { mustQuery } from "../dom";
import { t } from "../i18n/runtime";
import type { MessageKey } from "../locales/en";
import { onScreenChange } from "../router";
import { errorText } from "../scan_errors";
import type { Config } from "../types";
import {
  initSettingsForm,
  loadSettingsForm,
  settingsFormMessage,
} from "./settings_form";

interface PanelLink {
  path: string;
  labelKey: MessageKey;
}

const PUBLIC_LINKS: readonly PanelLink[] = [
  { path: "/general/status.html", labelKey: "settings.linkStatusPage" },
  { path: "/general/information.html?kind=item", labelKey: "settings.linkMaintenanceInfo" },
  { path: "/general/sleep.html", labelKey: "settings.linkSleepTimer" },
  { path: "/general/powerdown.html", labelKey: "settings.linkAutoPowerOff" },
  { path: "/general/replacetoner.html", labelKey: "settings.linkReplaceToner" },
];

const AUTH_LINKS: readonly PanelLink[] = [
  { path: "/admin/date.html", labelKey: "settings.linkDateTime" },
  { path: "/net/net/sntp.html?pageid=347", labelKey: "settings.linkSntp" },
  { path: "/admin/firmwareupdate.html", labelKey: "settings.linkFirmware" },
];

function linkButton(link: PanelLink): string {
  return `<button class="link-button" type="button" data-panel-path="${link.path}" data-i18n="${link.labelKey}"></button>`;
}

const TEMPLATE = `
  <section class="panel settings-links">
    <h2 class="settings-title" data-i18n="settings.linksTitle"></h2>
    <p class="settings-group-title" data-i18n="settings.linksPublicGroup"></p>
    <div class="link-list">${PUBLIC_LINKS.map(linkButton).join("")}</div>
    <p class="settings-group-title" data-i18n="settings.linksAuthGroup"></p>
    <p class="hint" data-i18n="settings.linksAuthHint"></p>
    <div class="link-list">${AUTH_LINKS.map(linkButton).join("")}</div>
  </section>
`;

mustQuery<HTMLElement>("#settings-links-mount").innerHTML = TEMPLATE;

const linkButtons = Array.from(
  document.querySelectorAll<HTMLButtonElement>("[data-panel-path]"),
);

let panelHost: string | null = null;

function setLinksEnabled(enabled: boolean): void {
  for (const button of linkButtons) {
    button.disabled = !enabled;
  }
}

async function reloadConfig(): Promise<void> {
  setLinksEnabled(false);
  settingsFormMessage(t("settings.loading"), "busy");
  try {
    const config = await getConfig();
    panelHost = config.device.host;
    loadSettingsForm(config);
    setLinksEnabled(true);
  } catch (error) {
    panelHost = null;
    settingsFormMessage(`${t("settings.loadFailed")}: ${errorText(error)}`, "error");
  }
}

async function openPanelLink(path: string): Promise<void> {
  const host = panelHost;
  if (host === null) {
    settingsFormMessage(t("settings.loadFailed"), "error");
    return;
  }
  try {
    await openUrl(`http://${host}${path}`);
  } catch (error) {
    settingsFormMessage(errorText(error), "error");
  }
}

export function initSettingsScreen(): void {
  initSettingsForm((applied: Config) => {
    panelHost = applied.device.host;
  });

  for (const button of linkButtons) {
    button.addEventListener("click", () => {
      void openPanelLink(button.dataset["panel-path"] ?? "");
    });
  }

  onScreenChange((screen) => {
    if (screen === "settings") {
      void reloadConfig();
    }
  });

  setLinksEnabled(false);
}
