import { errorText, openFirmwarePage, openPrinterPanel } from "../api";
import { mustQuery } from "../dom";
import { TEXT } from "../labels";
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
    <h2 class="overview-title">${TEXT.actionsTitle}</h2>
    <div class="actions">
      <button class="button button--primary" id="overview-scan" type="button">${TEXT.scanAction}</button>
      <button class="button button--secondary" id="overview-web-panel" type="button">${TEXT.webPanelAction}</button>
      <button class="button button--ghost" id="overview-firmware" type="button">${TEXT.firmwareAction}</button>
    </div>
    <p class="overview-hint">${TEXT.firmwareHint}</p>
  </article>
`;

mustQuery<HTMLElement>("#overview-actions-mount").innerHTML = TEMPLATE;

const els = {
  scan: mustQuery<HTMLButtonElement>("#overview-scan"),
  webPanel: mustQuery<HTMLButtonElement>("#overview-web-panel"),
  firmware: mustQuery<HTMLButtonElement>("#overview-firmware"),
};

export function renderOverviewScreen(state: AppState): void {
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
