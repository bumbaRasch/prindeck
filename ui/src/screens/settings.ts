import { openUrl } from "@tauri-apps/plugin-opener";
import { errorText, getConfig } from "../api";
import { mustQuery } from "../dom";
import { TEXT } from "../labels";
import { onScreenChange } from "../router";
import type { Config } from "../types";
import {
  initSettingsForm,
  loadSettingsForm,
  settingsFormMessage,
} from "./settings_form";

interface PanelLink {
  path: string;
  label: string;
}

const PUBLIC_LINKS: readonly PanelLink[] = [
  { path: "/general/status.html", label: TEXT.linkStatusPage },
  { path: "/general/information.html?kind=item", label: TEXT.linkMaintenanceInfo },
  { path: "/general/sleep.html", label: TEXT.linkSleepTimer },
  { path: "/general/powerdown.html", label: TEXT.linkAutoPowerOff },
  { path: "/general/replacetoner.html", label: TEXT.linkReplaceToner },
];

const AUTH_LINKS: readonly PanelLink[] = [
  { path: "/admin/date.html", label: TEXT.linkDateTime },
  { path: "/net/net/sntp.html?pageid=347", label: TEXT.linkSntp },
  { path: "/admin/firmwareupdate.html", label: TEXT.linkFirmware },
];

function linkButton(link: PanelLink): string {
  return `<button class="link-button" type="button" data-panel-path="${link.path}">${link.label}</button>`;
}

const TEMPLATE = `
  <section class="panel settings-links">
    <h2 class="settings-title">${TEXT.linksTitle}</h2>
    <p class="settings-group-title">${TEXT.linksPublicGroup}</p>
    <div class="link-list">${PUBLIC_LINKS.map(linkButton).join("")}</div>
    <p class="settings-group-title">${TEXT.linksAuthGroup}</p>
    <p class="hint">${TEXT.linksAuthHint}</p>
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
  settingsFormMessage(TEXT.settingsLoading, "busy");
  try {
    const config = await getConfig();
    panelHost = config.device.host;
    loadSettingsForm(config);
    setLinksEnabled(true);
  } catch (error) {
    panelHost = null;
    settingsFormMessage(`${TEXT.settingsLoadFailed}: ${errorText(error)}`, "error");
  }
}

async function openPanelLink(path: string): Promise<void> {
  const host = panelHost;
  if (host === null) {
    settingsFormMessage(TEXT.settingsLoadFailed, "error");
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
