import { getLocaleHint } from "../api";
import { CATALOGS } from "../locales";
import type { MessageKey } from "../locales/en";
import { direction, type LocaleTag } from "./catalog";
import { detectLocale } from "./detect";

let current: LocaleTag = "en";

export async function initLocale(): Promise<LocaleTag> {
  let hint: string | null = null;
  try {
    hint = await getLocaleHint();
  } catch {
    hint = null;
  }
  current = detectLocale(hint);
  document.documentElement.lang = current;
  document.documentElement.dir = direction(current);
  return current;
}

export function localeTag(): LocaleTag {
  return current;
}

function interpolate(
  template: string,
  params: Record<string, string | number>,
): string {
  return template.replace(/\{([a-zA-Z]+)\}/g, (match, token: string) =>
    token in params ? String(params[token]) : match,
  );
}

export function t(
  key: MessageKey,
  params?: Record<string, string | number>,
): string {
  const message = CATALOGS[current][key];
  const template = typeof message === "string" ? message : message.other;
  return interpolate(template, params ?? {});
}

export function tp(
  key: MessageKey,
  count: number,
  params?: Record<string, string | number>,
): string {
  const message = CATALOGS[current][key];
  const forms = typeof message === "string" ? { other: message } : message;
  const category = new Intl.PluralRules(current).select(count);
  const template = forms[category] ?? forms.other;
  return interpolate(template, { count, ...params });
}

export function formatRelativeSeconds(seconds: number): string {
  return new Intl.RelativeTimeFormat(current, { numeric: "auto" }).format(
    -seconds,
    "second",
  );
}

export function formatSizeKb(sizeKb: number): string {
  const kilobytes = sizeKb < 1024;
  return new Intl.NumberFormat(current, {
    style: "unit",
    unit: kilobytes ? "kilobyte" : "megabyte",
    maximumFractionDigits: 1,
  }).format(kilobytes ? sizeKb : sizeKb / 1024);
}
