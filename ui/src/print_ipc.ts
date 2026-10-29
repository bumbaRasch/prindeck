import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n/runtime";
import type { MessageKey } from "./i18n/catalog";
import type { CupsJob, Sides } from "./types";

export type PrintErrorKind =
  | "copies-range"
  | "sides-unknown"
  | "job-outside-queue"
  | "cups-missing"
  | "queue-missing"
  | "send-failed"
  | "queue-failed"
  | "cancel-failed";

export type PrintFailure = { kind: PrintErrorKind };

const PRINT_ERROR_KEYS: Record<string, MessageKey> = {
  "copies-range": "printError.copies-range",
  "sides-unknown": "printError.sides-unknown",
  "job-outside-queue": "printError.job-outside-queue",
  "cups-missing": "printError.cups-missing",
  "queue-missing": "printError.queue-missing",
  "send-failed": "printError.send-failed",
  "queue-failed": "printError.queue-failed",
  "cancel-failed": "printError.cancel-failed",
};

const PRINT_ERROR_KINDS: readonly string[] = Object.keys(PRINT_ERROR_KEYS);

function isPrintErrorKind(value: unknown): value is PrintErrorKind {
  return typeof value === "string" && PRINT_ERROR_KINDS.includes(value);
}

function printFailure(error: unknown): unknown {
  if (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    isPrintErrorKind(error.kind)
  ) {
    return { kind: error.kind } satisfies PrintFailure;
  }
  return error;
}

async function invokePrint<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw printFailure(error);
  }
}

export function printFile(
  path: string,
  copies: number,
  sides: Sides,
): Promise<string> {
  return invokePrint<string>("print_file", { path, copies, sides });
}

export function getQueue(): Promise<CupsJob[]> {
  return invokePrint<CupsJob[]>("get_queue");
}

export function cancelPrint(jobId: string): Promise<void> {
  return invokePrint<void>("cancel_print", { jobId });
}

function stringifyError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message || String(error);
  }
  try {
    return JSON.stringify(error) ?? t("print.requestFailed");
  } catch {
    return t("print.requestFailed");
  }
}

export function printErrorText(error: unknown): string {
  if (typeof error === "object" && error !== null && "kind" in error) {
    const kind = error.kind;
    if (typeof kind === "string") {
      const key = PRINT_ERROR_KEYS[kind];
      if (key !== undefined) {
        return t(key);
      }
    }
  }
  return stringifyError(error);
}
