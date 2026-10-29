import { t } from "./i18n/runtime";
import type { MessageKey } from "./i18n/catalog";
import type { ScanErrorKind } from "./types";

const SCAN_ERROR_KEYS: Record<string, MessageKey> = {
  transport: "scanError.transport",
  busy: "scanError.busy",
  invalid: "scanError.invalid",
  device: "scanError.device",
  storage: "scanError.storage",
  canceled: "scanError.canceled",
};

const SCAN_ERROR_KINDS: readonly string[] = [
  "transport",
  "busy",
  "invalid",
  "device",
  "storage",
  "canceled",
];

function isScanErrorKind(value: unknown): value is ScanErrorKind {
  return typeof value === "string" && SCAN_ERROR_KINDS.includes(value);
}

export interface FailurePayload {
  kind: unknown;
  message: unknown;
}

function isFailurePayload(error: unknown): error is FailurePayload {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error
  );
}

export class ScanFailureError extends Error {
  readonly kind: ScanErrorKind;

  constructor(kind: ScanErrorKind, message: string) {
    super(message);
    this.kind = kind;
  }
}

export function toScanFailure(error: unknown): ScanFailureError | Error {
  if (isFailurePayload(error)) {
    const kind = isScanErrorKind(error.kind) ? error.kind : "device";
    const message =
      typeof error.message === "string" && error.message !== ""
        ? error.message
        : t("scan.requestFailed");
    return new ScanFailureError(kind, message);
  }
  return new Error(t("scan.requestFailed"));
}

export function scanFailureKind(error: unknown): ScanErrorKind | null {
  return error instanceof ScanFailureError ? error.kind : null;
}

export function scanFailureText(failure: FailurePayload): string {
  const key =
    typeof failure.kind === "string" ? SCAN_ERROR_KEYS[failure.kind] : undefined;
  if (key !== undefined) {
    return t(key);
  }
  return typeof failure.message === "string" && failure.message !== ""
    ? failure.message
    : t("scan.requestFailed");
}

export function scanErrorText(kind: ScanErrorKind): string {
  return scanFailureText({ kind, message: null });
}

export function rejectionText(error: unknown): string {
  return isFailurePayload(error) ? scanFailureText(error) : errorText(error);
}

export function errorText(error: unknown): string {
  if (error instanceof ScanFailureError) {
    return scanErrorText(error.kind);
  }
  if (typeof error === "string" && error.trim() !== "") {
    return error;
  }
  if (error instanceof Error && error.message !== "") {
    return error.message;
  }
  return t("scan.requestFailed");
}
