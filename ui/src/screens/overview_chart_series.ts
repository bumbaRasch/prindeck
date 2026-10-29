import type { HistorySnapshot } from "../types";

export interface ChartPoint {
  ts: string;
  tonerPercent: number | null;
  pagesTotal: number | null;
}

export const CHART_CAP = 90;
export const CHART_WIDTH = 600;
export const CHART_HEIGHT = 160;

const PAD = 8;
const DOT_RADIUS = 3;
const TONER_SCALE = 100;

function toPoint(row: HistorySnapshot): ChartPoint {
  return {
    ts: row.ts,
    tonerPercent: row.tonerPercent,
    pagesTotal: row.pagesTotal,
  };
}

export function toPoints(
  rows: HistorySnapshot[],
  cap: number = CHART_CAP,
): ChartPoint[] {
  if (rows.length <= cap) {
    return rows.map(toPoint);
  }

  const last = rows.length - 1;
  const points: ChartPoint[] = [];
  for (let slot = 0; slot < cap; slot += 1) {
    const index = Math.round((slot * last) / (cap - 1));
    const row = rows[index];
    if (row !== undefined) {
      points.push(toPoint(row));
    }
  }
  return points;
}

export function canChart(points: ChartPoint[]): boolean {
  return points.length >= 2;
}

type ValuePicker = (point: ChartPoint) => number | null;

function xAt(index: number, count: number): number {
  return PAD + (index * (CHART_WIDTH - PAD * 2)) / (count - 1);
}

function yAt(value: number, scale: number): number {
  return CHART_HEIGHT - PAD - (Math.max(0, value) / scale) * (CHART_HEIGHT - PAD * 2);
}

function linePath(points: ChartPoint[], pick: ValuePicker, scale: number): string {
  let path = "";
  let open = false;
  points.forEach((point, index) => {
    const value = pick(point);
    if (value === null) {
      open = false;
      return;
    }
    const command = open ? "L" : "M";
    path += `${command}${xAt(index, points.length).toFixed(1)} ${yAt(value, scale).toFixed(1)}`;
    open = true;
  });
  return path;
}

function lastDot(
  points: ChartPoint[],
  pick: ValuePicker,
  scale: number,
  className: string,
): string {
  for (let index = points.length - 1; index >= 0; index -= 1) {
    const point = points[index];
    if (point === undefined) {
      continue;
    }
    const value = pick(point);
    if (value !== null) {
      const x = xAt(index, points.length).toFixed(1);
      const y = yAt(value, scale).toFixed(1);
      return `<circle class="chart-dot ${className}" cx="${x}" cy="${y}" r="${DOT_RADIUS}" />`;
    }
  }
  return "";
}

export function renderChart(
  el: SVGSVGElement,
  points: ChartPoint[],
  max: number,
): void {
  if (!canChart(points)) {
    el.innerHTML = "";
    return;
  }

  const pages = max > 0;
  el.innerHTML = [
    pages
      ? `<path class="chart-line chart-line--pages" d="${linePath(points, (point) => point.pagesTotal, max)}" />`
      : "",
    `<path class="chart-line chart-line--toner" d="${linePath(points, (point) => point.tonerPercent, TONER_SCALE)}" />`,
    pages ? lastDot(points, (point) => point.pagesTotal, max, "chart-dot--pages") : "",
    lastDot(points, (point) => point.tonerPercent, TONER_SCALE, "chart-dot--toner"),
  ].join("");
}

export function pagesScale(points: ChartPoint[]): number {
  let max = 0;
  for (const point of points) {
    if (point.pagesTotal !== null && point.pagesTotal > max) {
      max = point.pagesTotal;
    }
  }
  return max;
}
