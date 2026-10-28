import { getConfig } from "../api";
import { mustQuery } from "../dom";
import {
  TEXT,
  drumPagesLeftText,
  jamCountText,
  tonerPagesLeftText,
} from "../labels";
import { setScreen } from "../router";
import type { AppState } from "../state";
import type { DeviceStatus } from "../types";

const TEMPLATE = `
  <article class="panel overview-card" id="overview-state">
    <h2 class="overview-title">${TEXT.stateTitle}</h2>
    <ul class="state-rows">
      <li class="state-row" id="overview-state-jam" data-tone="idle" hidden></li>
      <li class="state-row" id="overview-state-drum" data-tone="idle" hidden></li>
    </ul>
  </article>
  <article class="panel overview-card">
    <h2 class="overview-title">${TEXT.consumablesTitle}</h2>
    <div class="metrics">
      <div class="metric">
        <div class="metric-head">
          <span class="metric-name">${TEXT.tonerLabel}</span>
          <span class="metric-value" id="overview-toner">${TEXT.unreported}</span>
        </div>
        <p class="metric-note" id="overview-toner-estimate" hidden></p>
        <div class="bar" id="overview-toner-bar" hidden><div class="bar-fill" id="overview-toner-fill"></div></div>
        <button class="metric-hint" id="overview-toner-hint" type="button" hidden>${TEXT.tonerYieldHint}</button>
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
`;

mustQuery<HTMLElement>("#overview-supplies-mount").innerHTML = TEMPLATE;

const els = {
  card: mustQuery<HTMLElement>("#overview-state"),
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

function renderJam(count: number | null): void {
  els.stateJam.hidden = count === null;
  if (count === null) {
    return;
  }
  const text = jamCountText(count);
  if (els.stateJam.textContent !== text) {
    els.stateJam.textContent = text;
  }
  els.stateJam.dataset["tone"] = count > 0 ? "warn" : "ok";
}

function renderDrumLine(pagesLeft: number | null): void {
  els.stateDrum.hidden = pagesLeft === null;
  if (pagesLeft === null) {
    return;
  }
  const text = drumPagesLeftText(pagesLeft);
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
      ? tonerPagesLeftText(Math.floor((yieldPages * tonerPercent) / 100))
      : null;

  els.tonerEstimate.hidden = estimate === null;
  if (estimate !== null && els.tonerEstimate.textContent !== estimate) {
    els.tonerEstimate.textContent = estimate;
  }

  els.tonerHint.hidden = yieldPages !== null;
}

export function renderOverviewSupplies(state: AppState): void {
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
