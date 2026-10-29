import { mustQuery } from "../dom";
import { formatRelativeSeconds, t } from "../i18n/runtime";
import { CONNECTIVITY_TONES, connectivity, type AppState } from "../state";
import type { Connectivity, DeviceStatus } from "../types";

const CHIP_LABEL_KEYS: Record<
  Connectivity,
  "chip.checking" | "chip.online" | "chip.offline" | "chip.scanning"
> = {
  checking: "chip.checking",
  online: "chip.online",
  offline: "chip.offline",
  scanning: "chip.scanning",
};

const TEMPLATE = `
  <article class="panel overview-card" id="overview-connection" data-state="checking">
    <div class="overview-head">
      <p class="status" id="overview-chip" data-tone="idle"></p>
      <button class="button button--ghost" id="overview-refresh" type="button"></button>
    </div>
    <p class="overview-model" id="overview-model"></p>
    <p class="overview-host" id="overview-host"></p>
    <p class="overview-updated" id="overview-updated" hidden></p>
    <div class="overview-escl" id="overview-escl" hidden>
      <p class="overview-escl-text" id="overview-escl-text"></p>
      <button class="button button--ghost" id="overview-escl-retry" type="button"></button>
    </div>
  </article>
`;

mustQuery<HTMLElement>("#overview-connection-mount").innerHTML = TEMPLATE;

const els = {
  card: mustQuery<HTMLElement>("#overview-connection"),
  chip: mustQuery<HTMLParagraphElement>("#overview-chip"),
  refresh: mustQuery<HTMLButtonElement>("#overview-refresh"),
  esclText: mustQuery<HTMLParagraphElement>("#overview-escl-text"),
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
  return t("shell.updatedAgo", { time: formatRelativeSeconds(seconds) });
}

function renderUpdated(status: DeviceStatus | null): void {
  const text = status === null ? null : updatedText(status);
  els.updated.hidden = text === null;
  if (text !== null && els.updated.textContent !== text) {
    els.updated.textContent = text;
  }
}

export function renderOverviewConnection(state: AppState): void {
  els.refresh.textContent = t("overview.refresh");
  els.esclText.textContent = t("scan.esclUnavailable");
  els.esclRetry.textContent = t("overview.retry");
  const chip = connectivity(state);
  els.card.dataset["state"] = chip;
  els.chip.textContent = t(CHIP_LABEL_KEYS[chip]);
  els.chip.dataset["tone"] = CONNECTIVITY_TONES[chip];

  els.model.textContent = state.caps === null ? t("overview.unreported") : state.caps.makeAndModel;
  els.host.textContent = state.status === null ? t("overview.unreported") : state.status.host;
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
