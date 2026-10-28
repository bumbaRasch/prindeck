import type { Sides } from "../types";

export const SIDES_LABELS: Record<Sides, string> = {
  "one-sided": "односторонняя",
  "two-sided-long-edge": "по длинному краю",
  "two-sided-short-edge": "по короткому краю",
};

export function formatSizeKb(sizeKb: number): string {
  if (sizeKb < 1024) {
    return `${sizeKb} КБ`;
  }
  return `${(sizeKb / 1024).toFixed(1)} МБ`;
}
