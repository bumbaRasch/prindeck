import { errorText, getHistory } from "../api";
import { mustQuery } from "../dom";
import { t } from "../i18n/runtime";
import type { HistorySnapshot } from "../types";
import {
  canChart,
  CHART_HEIGHT,
  CHART_WIDTH,
  pagesScale,
  renderChart,
  toPoints,
} from "./overview_chart_series";

const HISTORY_DAYS = 90;

const TEMPLATE = `
  <article class="panel overview-card">
    <h2 class="overview-title" id="overview-history-title"></h2>
    <svg class="chart" id="overview-chart" width="${CHART_WIDTH}" height="${CHART_HEIGHT}" viewBox="0 0 ${CHART_WIDTH} ${CHART_HEIGHT}" role="img"></svg>
    <p class="chart-note" id="overview-chart-note" hidden></p>
    <div class="chart-legend" id="overview-chart-legend">
      <span class="chart-legend-item" id="overview-history-toner-line"><span class="chart-key chart-key--toner"></span></span>
      <span class="chart-legend-item" id="overview-history-pages-line"><span class="chart-key chart-key--pages"></span></span>
    </div>
  </article>
`;

mustQuery<HTMLElement>("#overview-chart-mount").innerHTML = TEMPLATE;

const els = {
  title: mustQuery<HTMLHeadingElement>("#overview-history-title"),
  chart: mustQuery<SVGSVGElement>("#overview-chart"),
  tonerLine: mustQuery<HTMLSpanElement>("#overview-history-toner-line"),
  pagesLine: mustQuery<HTMLSpanElement>("#overview-history-pages-line"),
  note: mustQuery<HTMLParagraphElement>("#overview-chart-note"),
  legend: mustQuery<HTMLElement>("#overview-chart-legend"),
};

function renderChartLabels(): void {
  els.title.textContent = t("overview.historyTitle");
  els.chart.setAttribute("aria-label", t("overview.historyChartLabel"));
  renderLegendLabel(els.tonerLine, t("overview.historyTonerLine"));
  renderLegendLabel(els.pagesLine, t("overview.historyPagesLine"));
}

function renderLegendLabel(element: HTMLElement, text: string): void {
  const label = element.childNodes.item(1);
  if (label === null) {
    element.append(document.createTextNode(text));
  } else {
    label.textContent = text;
  }
}

function setHidden(element: Element, hidden: boolean): void {
  if (hidden) {
    element.setAttribute("hidden", "");
  } else {
    element.removeAttribute("hidden");
  }
}

function showNote(text: string): void {
  setHidden(els.chart, true);
  setHidden(els.legend, true);
  setHidden(els.note, false);
  if (els.note.textContent !== text) {
    els.note.textContent = text;
  }
}

function renderCard(rows: HistorySnapshot[]): void {
  renderChartLabels();
  const points = toPoints(rows);
  if (!canChart(points)) {
    showNote(t("overview.historyEmpty"));
    return;
  }

  setHidden(els.note, true);
  setHidden(els.chart, false);
  setHidden(els.legend, false);
  renderChart(els.chart, points, pagesScale(points));
}

export async function reloadOverviewChart(): Promise<void> {
  try {
    renderCard(await getHistory(HISTORY_DAYS));
  } catch (error) {
    showNote(errorText(error));
  }
}

export function initOverviewChart(): void {
  renderChartLabels();
  void reloadOverviewChart();
}
