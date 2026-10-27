import { mustQuery } from "../dom";
import { CHIP_LABELS, TEXT, updatedSecondsAgo } from "../labels";
import { CONNECTIVITY_TONES, connectivity, type AppState } from "../state";
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
`;

mustQuery<HTMLElement>("#overview-connection-mount").innerHTML = TEMPLATE;

const els = {
  card: mustQuery<HTMLElement>("#overview-connection"),
  chip: mustQuery<HTMLParagraphElement>("#overview-chip"),
  refresh: mustQuery<HTMLButtonElement>("#overview-refresh"),
  model: mustQuery<HTMLParagraphElement>("#overview-model"),
  host: mustQuery<HTMLParagraphElement>("#overview-host"),
  updated: mustQuery<HTMLParagraphElement>("#overview-updated"),
  escl: mustQuery<HTMLElement>("#overview-escl"),
  esclRetry: mustQuery<HTMLButtonElement>("#overview-escl-retry"),
};

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

export function renderOverviewConnection(state: AppState): void {
  const chip = connectivity(state);
  els.card.dataset["state"] = chip;
  els.chip.textContent = CHIP_LABELS[chip];
  els.chip.dataset["tone"] = CONNECTIVITY_TONES[chip];

  els.model.textContent = state.caps === null ? TEXT.unreported : state.caps.makeAndModel;
  els.host.textContent = state.status === null ? TEXT.unreported : state.status.host;
  renderUpdated(state.status);

  els.escl.hidden = !state.esclFailed;
  els.esclRetry.disabled = state.capsLoading;
  els.refresh.disabled = state.refreshingStatus;
}

function startUpdatedTicker(getState: () => AppState): void {
  window.setInterval(() => {
    renderUpdated(getState().status);
  }, 10_000);
}

export function initOverviewConnection(
  getState: () => AppState,
  refresh: () => Promise<void>,
  onRefreshed: () => void,
): void {
  els.refresh.addEventListener("click", () => {
    void refresh().then(onRefreshed);
  });
  startUpdatedTicker(getState);
}
