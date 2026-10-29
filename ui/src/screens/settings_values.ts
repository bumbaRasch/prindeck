import type { MessageKey } from "../locales/en";
import type { Config } from "../types";

export const MAX_PERCENT = 100;
export const MAX_YIELD_PAGES = 100_000;
export const MIN_POLL_SECONDS = 60;
export const MAX_POLL_SECONDS = 3600;

export type SettingsField =
  | "host"
  | "pollSeconds"
  | "tonerPercent"
  | "drumPercent"
  | "scanDir"
  | "tonerYieldPages";

export interface SettingsFormValues {
  host: string;
  pollSeconds: string;
  notifyEnabled: boolean;
  tonerPercent: string;
  drumPercent: string;
  scanDir: string;
  tonerModel: string;
  tonerYieldPages: string;
  autostart: boolean;
}

export interface SettingsFieldError {
  field: SettingsField;
  key: MessageKey;
}

function parseWhole(raw: string): number | null {
  const text = raw.trim();
  if (!/^\d+$/.test(text)) {
    return null;
  }
  return Number.parseInt(text, 10);
}

function hostProblem(host: string): MessageKey | null {
  const trimmed = host.trim();
  if (trimmed === "") {
    return "settings.error.hostRequired";
  }
  if (trimmed.includes("://") || /[\s/\\@?#]/.test(trimmed)) {
    return "settings.error.hostFormat";
  }
  return null;
}

function percentProblem(
  raw: string,
  field: SettingsField,
): SettingsFieldError | null {
  const value = parseWhole(raw);
  return value !== null && value <= MAX_PERCENT
    ? null
    : { field, key: "settings.error.percentRange" };
}

export function validateSettingsValues(
  values: SettingsFormValues,
): SettingsFieldError[] {
  const errors: SettingsFieldError[] = [];

  const host = hostProblem(values.host);
  if (host !== null) {
    errors.push({ field: "host", key: host });
  }

  const poll = parseWhole(values.pollSeconds);
  if (poll === null || poll < MIN_POLL_SECONDS || poll > MAX_POLL_SECONDS) {
    errors.push({ field: "pollSeconds", key: "settings.error.pollRange" });
  }

  const toner = percentProblem(values.tonerPercent, "tonerPercent");
  if (toner !== null) {
    errors.push(toner);
  }

  const drum = percentProblem(values.drumPercent, "drumPercent");
  if (drum !== null) {
    errors.push(drum);
  }

  if (values.scanDir.trim() === "") {
    errors.push({ field: "scanDir", key: "settings.error.scanDirRequired" });
  }

  const yieldPages = parseWhole(values.tonerYieldPages);
  const yieldGiven = values.tonerYieldPages.trim() !== "";
  if (
    yieldGiven &&
    (yieldPages === null || yieldPages < 1 || yieldPages > MAX_YIELD_PAGES)
  ) {
    errors.push({ field: "tonerYieldPages", key: "settings.error.yieldRange" });
  }

  return errors;
}

export function configFromValues(
  baseline: Config,
  values: SettingsFormValues,
): Config {
  const yieldPages = parseWhole(values.tonerYieldPages);

  return {
    ...baseline,
    device: { host: values.host.trim() },
    scanDir: values.scanDir.trim(),
    pollSeconds: parseWhole(values.pollSeconds) ?? baseline.pollSeconds,
    autostart: values.autostart,
    notify: {
      enabled: values.notifyEnabled,
      tonerPercent:
        parseWhole(values.tonerPercent) ?? baseline.notify.tonerPercent,
      drumPercent:
        parseWhole(values.drumPercent) ?? baseline.notify.drumPercent,
    },
    toner: {
      model: values.tonerModel.trim() === "" ? null : values.tonerModel.trim(),
      yieldPages: yieldPages === null ? null : yieldPages,
    },
  };
}
