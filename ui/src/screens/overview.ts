import { errorText, openFirmwarePage, openPrinterPanel } from "../api";
import { mustQuery } from "../dom";
import { t } from "../i18n/runtime";
import { setScreen } from "../router";
import type { AppState } from "../state";
import { renderShell } from "../shell";
import { initOverviewChart, reloadOverviewChart } from "./overview_chart";
import {
  initOverviewConnection,
  renderOverviewConnection,
} from "./overview_connection";
import {
  initOverviewSupplies,
  reloadSuppliesConfig,
  renderOverviewSupplies,
} from "./overview_supplies";

const TEMPLATE = `
  <article class="panel overview-card">
    <h2 class="overview-title" id="overview-actions-title"></h2>
    <div class="actions">
      <button class="button button--primary" id="overview-scan" type="button"></button>
      <button class="button button--secondary" id="overview-web-panel" type="button"></button>
      <button class="button button--ghost" id="overview-firmware" type="button"></button>
    </div>
    <p class="overview-hint" id="overview-firmware-hint"></p>
  </article>
`;

mustQuery<HTMLElement>("#overview-actions-mount").innerHTML = TEMPLATE;

const els = {
  title: mustQuery<HTMLHeadingElement>("#overview-actions-title"),
  scan: mustQuery<HTMLButtonElement>("#overview-scan"),
  webPanel: mustQuery<HTMLButtonElement>("#overview-web-panel"),
  firmware: mustQuery<HTMLButtonElement>("#overview-firmware"),
  hint: mustQuery<HTMLParagraphElement>("#overview-firmware-hint"),
};

function renderActions(): void {
  els.title.textContent = t("overview.actionsTitle");
  els.scan.textContent = t("overview.scanAction");
  els.webPanel.textContent = t("overview.webPanelAction");
  els.firmware.textContent = t("overview.firmwareAction");
  els.hint.textContent = t("overview.firmwareHint");
}

export function renderOverviewScreen(state: AppState): void {
  renderActions();
  renderOverviewConnection(state);
  renderOverviewSupplies(state);
}

async function openDevicePage(
  action: () => Promise<void>,
  state: AppState,
): Promise<void> {
  try {
    await action();
  } catch (error) {
    state.errorText = errorText(error);
    renderShell(state);
    renderOverviewScreen(state);
  }
}

function reloadOverviewData(): void {
  void reloadOverviewChart();
  void reloadSuppliesConfig();
}

export function initOverviewScreen(
  getState: () => AppState,
  refresh: () => Promise<void>,
): void {
  renderActions();
  initOverviewConnection(getState, refresh, reloadOverviewData);
  initOverviewSupplies(getState);
  initOverviewChart();

  els.scan.addEventListener("click", () => {
    setScreen("scan");
  });
  els.webPanel.addEventListener("click", () => {
    void openDevicePage(openPrinterPanel, getState());
  });
  els.firmware.addEventListener("click", () => {
    void openDevicePage(openFirmwarePage, getState());
  });
}
