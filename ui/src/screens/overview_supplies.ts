import { getConfig } from "../api";
import { mustQuery } from "../dom";
import { t, tp } from "../i18n/runtime";
import { setScreen } from "../router";
import type { AppState } from "../state";
import type { DeviceStatus } from "../types";

const TEMPLATE = `
  <article class="panel overview-card" id="overview-state">
    <h2 class="overview-title" id="overview-state-title"></h2>
    <ul class="state-rows">
      <li class="state-row" id="overview-state-jam" data-tone="idle" hidden></li>
      <li class="state-row" id="overview-state-drum" data-tone="idle" hidden></li>
    </ul>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title" id="overview-consumables-title"></h2>
    <div class="metrics">
      <div class="metric">
        <div class="metric-head">
          <span class="metric-name" id="overview-toner-label"></span>
          <span class="metric-value" id="overview-toner"></span>
        </div>
        <p class="metric-note" id="overview-toner-estimate" hidden></p>
        <div class="bar" id="overview-toner-bar" hidden><div class="bar-fill" id="overview-toner-fill"></div></div>
        <button class="metric-hint" id="overview-toner-hint" type="button" hidden></button>
      </div>
      <div class="metric">
        <div class="metric-head">
          <span class="metric-name" id="overview-drum-label"></span>
          <span class="metric-value" id="overview-drum"></span>
        </div>
        <div class="bar" id="overview-drum-bar" hidden><div class="bar-fill" id="overview-drum-fill"></div></div>
      </div>
    </div>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title" id="overview-counters-title"></h2>
    <div class="counter-rows">
      <div class="counter-row"><span class="counter-name" id="overview-pages-label"></span><span class="counter-value" id="overview-pages"></span></div>
      <div class="counter-row"><span class="counter-name" id="overview-duplex-label"></span><span class="counter-value" id="overview-duplex"></span></div>
      <div class="counter-row"><span class="counter-name" id="overview-drum-left-label"></span><span class="counter-value" id="overview-drum-left"></span></div>
    </div>
  </article>
`;

mustQuery<HTMLElement>("#overview-supplies-mount").innerHTML = TEMPLATE;

const els = {
  card: mustQuery<HTMLElement>("#overview-state"),
  stateTitle: mustQuery<HTMLHeadingElement>("#overview-state-title"),
  consumablesTitle: mustQuery<HTMLHeadingElement>("#overview-consumables-title"),
  countersTitle: mustQuery<HTMLHeadingElement>("#overview-counters-title"),
  tonerLabel: mustQuery<HTMLSpanElement>("#overview-toner-label"),
  drumLabel: mustQuery<HTMLSpanElement>("#overview-drum-label"),
  pagesLabel: mustQuery<HTMLSpanElement>("#overview-pages-label"),
  duplexLabel: mustQuery<HTMLSpanElement>("#overview-duplex-label"),
  drumLeftLabel: mustQuery<HTMLSpanElement>("#overview-drum-left-label"),
  stateJam: mustQuery<HTMLLIElement>("#overview-state-jam"),
  stateDrum: mustQuery<HTMLLIElement>("#overview-state-drum"),
  toner: mustQuery<HTMLElement>("#overview-toner"),
  tonerFill: mustQuery<HTMLElement>("#overview-toner-fill"),
  tonerBar: mustQuery<HTMLElement>("#overview-toner-bar"),
  tonerEstimate: mustQuery<HTMLParagraphElement>("#overview-toner-estimate"),
  tonerHint: mustQuery<HTMLButtonElement>("#overview-toner-hint"),
  drum: mustQuery<HTMLElement>("#overview-drum"),
  drumFill: mustQuery<HTMLElement>("#overview-drum-fill"),
  drumBar: mustQuery<HTMLElement>("#overview-drum-bar"),
  pages: mustQuery<HTMLElement>("#overview-pages"),
  duplex: mustQuery<HTMLElement>("#overview-duplex"),
  drumLeft: mustQuery<HTMLElement>("#overview-drum-left"),
};

let yieldPages: number | null = null;
let getState: (() => AppState) | null = null;

function percentText(value: number | null): string {
  return value === null ? t("overview.unreported") : `${value} %`;
}

function countText(value: number | null): string {
  return value === null ? t("overview.unreported") : String(value);
}

function renderBar(bar: HTMLElement, fill: HTMLElement, value: number | null): void {
  bar.hidden = value === null;
  if (value !== null) {
    fill.style.width = `${Math.min(100, Math.max(0, value))}%`;
  }
}

function renderJam(count: number | null): void {
  els.stateJam.hidden = count === null;
  if (count === null) {
    return;
  }
  const text = count === 0 ? t("overview.jamsNone") : tp("overview.jams", count);
  if (els.stateJam.textContent !== text) {
    els.stateJam.textContent = text;
  }
}

function renderDrumLine(pagesLeft: number | null): void {
  els.stateDrum.hidden = pagesLeft === null;
  if (pagesLeft === null) {
    return;
  }
  const text = t("overview.drumPagesLeft", { pages: pagesLeft });
  if (els.stateDrum.textContent !== text) {
    els.stateDrum.textContent = text;
  }
}

function renderStateCard(status: DeviceStatus | null): void {
  renderJam(status === null ? null : status.jamCount);
  renderDrumLine(status === null ? null : status.drumPagesLeft);
  els.card.hidden = els.stateJam.hidden && els.stateDrum.hidden;
}

function renderTonerNote(tonerPercent: number | null): void {
  const estimate =
    yieldPages !== null && tonerPercent !== null
      ? t("overview.tonerPagesLeft", {
          pages: Math.floor((yieldPages * tonerPercent) / 100),
        })
      : null;

  els.tonerEstimate.hidden = estimate === null;
  if (estimate !== null && els.tonerEstimate.textContent !== estimate) {
    els.tonerEstimate.textContent = estimate;
  }

  els.tonerHint.hidden = yieldPages !== null;
}

export function renderOverviewSupplies(state: AppState): void {
  els.stateTitle.textContent = t("overview.stateTitle");
  els.consumablesTitle.textContent = t("overview.consumablesTitle");
  els.countersTitle.textContent = t("overview.countersTitle");
  els.tonerLabel.textContent = t("overview.tonerLabel");
  els.drumLabel.textContent = t("overview.drumLabel");
  els.tonerHint.textContent = t("overview.tonerYieldHint");
  els.pagesLabel.textContent = t("overview.pagesLabel");
  els.duplexLabel.textContent = t("overview.duplexLabel");
  els.drumLeftLabel.textContent = t("overview.drumLeftLabel");
  const status = state.status;
  const toner = status === null ? null : status.tonerPercent;
  const drum = status === null ? null : status.drumPercent;

  els.toner.textContent = percentText(toner);
  renderBar(els.tonerBar, els.tonerFill, toner);
  els.drum.textContent = percentText(drum);
  renderBar(els.drumBar, els.drumFill, drum);
  renderTonerNote(toner);

  els.pages.textContent = countText(status === null ? null : status.pagesTotal);
  els.duplex.textContent = countText(status === null ? null : status.duplexPages);
  els.drumLeft.textContent = countText(
    status === null ? null : status.drumPagesLeft,
  );

  renderStateCard(status);
}

export async function reloadSuppliesConfig(): Promise<void> {
  try {
    const config = await getConfig();
    yieldPages = config.toner.yieldPages;
  } catch (error) {
    console.warn("failed to load the config", error);
  }
  if (getState !== null) {
    renderOverviewSupplies(getState());
  }
}

export function initOverviewSupplies(stateGetter: () => AppState): void {
  getState = stateGetter;
  els.tonerHint.addEventListener("click", () => {
    setScreen("settings");
  });
  void reloadSuppliesConfig();
}
