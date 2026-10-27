import { errorText, openPrinterPanel } from "../api";
import { mustQuery } from "../dom";
import { CHIP_LABELS, TEXT, updatedSecondsAgo } from "../labels";
import { setScreen } from "../router";
import {
  CONNECTIVITY_TONES,
  connectivity,
  type AppState,
} from "../state";
import { renderShell } from "../shell";
import type { DeviceStatus } from "../types";

const TEMPLATE = `
  <article class="panel overview-card" id="overview-connection" data-state="checking">
    <div class="overview-head">
      <p class="status" id="overview-chip" data-tone="idle"></p>
      <button class="button button--ghost" id="overview-refresh" type="button">${TEXT.refresh}</button>
    </div>
    <p class="overview-model" id="overview-model"></p>
    <p class="overview-host" id="overview-host"></p>
    <p class="overview-updated" id="overview-updated" hidden></p>
    <div class="overview-escl" id="overview-escl" hidden>
      <p class="overview-escl-text">${TEXT.esclUnavailable}</p>
      <button class="button button--ghost" id="overview-escl-retry" type="button">${TEXT.retry}</button>
    </div>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title">${TEXT.consumablesTitle}</h2>
    <div class="metrics">
      <div class="metric">
        <div class="metric-head">
          <span class="metric-name">${TEXT.tonerLabel}</span>
          <span class="metric-value" id="overview-toner">${TEXT.unreported}</span>
        </div>
        <div class="bar" id="overview-toner-bar" hidden><div class="bar-fill" id="overview-toner-fill"></div></div>
      </div>
      <div class="metric">
        <div class="metric-head">
          <span class="metric-name">${TEXT.drumLabel}</span>
          <span class="metric-value" id="overview-drum">${TEXT.unreported}</span>
        </div>
        <div class="bar" id="overview-drum-bar" hidden><div class="bar-fill" id="overview-drum-fill"></div></div>
      </div>
    </div>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title">${TEXT.countersTitle}</h2>
    <div class="counter-rows">
      <div class="counter-row"><span class="counter-name">${TEXT.pagesLabel}</span><span class="counter-value" id="overview-pages">${TEXT.unreported}</span></div>
      <div class="counter-row"><span class="counter-name">${TEXT.duplexLabel}</span><span class="counter-value" id="overview-duplex">${TEXT.unreported}</span></div>
      <div class="counter-row"><span class="counter-name">${TEXT.drumLeftLabel}</span><span class="counter-value" id="overview-drum-left">${TEXT.unreported}</span></div>
    </div>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title">${TEXT.actionsTitle}</h2>
    <div class="actions">
      <button class="button button--primary" id="overview-scan" type="button">${TEXT.scanAction}</button>
      <button class="button button--secondary" id="overview-web-panel" type="button">${TEXT.webPanelAction}</button>
    </div>
  </article>
`;

mustQuery<HTMLElement>('[data-screen="overview"]').innerHTML = TEMPLATE;

const els = {
  card: mustQuery<HTMLElement>("#overview-connection"),
  chip: mustQuery<HTMLParagraphElement>("#overview-chip"),
  refresh: mustQuery<HTMLButtonElement>("#overview-refresh"),
  model: mustQuery<HTMLParagraphElement>("#overview-model"),
  host: mustQuery<HTMLParagraphElement>("#overview-host"),
  updated: mustQuery<HTMLParagraphElement>("#overview-updated"),
  escl: mustQuery<HTMLElement>("#overview-escl"),
  esclRetry: mustQuery<HTMLButtonElement>("#overview-escl-retry"),
  toner: mustQuery<HTMLElement>("#overview-toner"),
  tonerFill: mustQuery<HTMLElement>("#overview-toner-fill"),
  tonerBar: mustQuery<HTMLElement>("#overview-toner-bar"),
  drum: mustQuery<HTMLElement>("#overview-drum"),
  drumFill: mustQuery<HTMLElement>("#overview-drum-fill"),
  drumBar: mustQuery<HTMLElement>("#overview-drum-bar"),
  pages: mustQuery<HTMLElement>("#overview-pages"),
  duplex: mustQuery<HTMLElement>("#overview-duplex"),
  drumLeft: mustQuery<HTMLElement>("#overview-drum-left"),
  scan: mustQuery<HTMLButtonElement>("#overview-scan"),
  webPanel: mustQuery<HTMLButtonElement>("#overview-web-panel"),
};

function percentText(value: number | null): string {
  return value === null ? TEXT.unreported : `${value} %`;
}

function countText(value: number | null): string {
  return value === null ? TEXT.unreported : String(value);
}

function renderBar(bar: HTMLElement, fill: HTMLElement, value: number | null): void {
  bar.hidden = value === null;
  if (value !== null) {
    fill.style.width = `${Math.min(100, Math.max(0, value))}%`;
  }
}

function updatedText(status: DeviceStatus): string | null {
  if (status.polledAt === null) {
    return null;
  }
  const iso = status.polledAt.replace(/(\.\d{3})\d+/, "$1");
  const polledMs = Date.parse(iso);
  if (Number.isNaN(polledMs)) {
    return null;
  }
  const seconds = Math.max(0, Math.floor((Date.now() - polledMs) / 1000));
  return updatedSecondsAgo(seconds);
}

function renderUpdated(status: DeviceStatus | null): void {
  const text = status === null ? null : updatedText(status);
  els.updated.hidden = text === null;
  if (text !== null && els.updated.textContent !== text) {
    els.updated.textContent = text;
  }
}

export function renderOverviewScreen(state: AppState): void {
  const chip = connectivity(state);
  els.card.dataset["state"] = chip;
  els.chip.textContent = CHIP_LABELS[chip];
  els.chip.dataset["tone"] = CONNECTIVITY_TONES[chip];

  els.model.textContent = state.caps === null ? TEXT.unreported : state.caps.makeAndModel;
  els.host.textContent = state.status === null ? TEXT.unreported : state.status.host;
  renderUpdated(state.status);

  const status = state.status;
  const toner = status === null ? null : status.tonerPercent;
  const drum = status === null ? null : status.drumPercent;
  els.toner.textContent = percentText(toner);
  renderBar(els.tonerBar, els.tonerFill, toner);
  els.drum.textContent = percentText(drum);
  renderBar(els.drumBar, els.drumFill, drum);

  els.pages.textContent = countText(status === null ? null : status.pagesTotal);
  els.duplex.textContent = countText(status === null ? null : status.duplexPages);
  els.drumLeft.textContent = countText(status === null ? null : status.drumPagesLeft);

  els.escl.hidden = !state.esclFailed;
  els.esclRetry.disabled = state.capsLoading;
  els.refresh.disabled = state.refreshingStatus;
}

async function handleWebPanel(state: AppState): Promise<void> {
  try {
    await openPrinterPanel();
  } catch (error) {
    state.errorText = errorText(error);
    renderShell(state);
    renderOverviewScreen(state);
  }
}

function startUpdatedTicker(getState: () => AppState): void {
  window.setInterval(() => {
    renderUpdated(getState().status);
  }, 10_000);
}

export function initOverviewScreen(
  getState: () => AppState,
  refresh: () => Promise<void>,
): void {
  els.refresh.addEventListener("click", () => {
    void refresh();
  });
  els.scan.addEventListener("click", () => {
    setScreen("scan");
  });
  els.webPanel.addEventListener("click", () => {
    void handleWebPanel(getState());
  });
  startUpdatedTicker(getState);
}
