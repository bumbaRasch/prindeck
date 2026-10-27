import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TEXT } from "./labels";
import type {
  Config,
  CupsJob,
  DeviceStatus,
  HistorySnapshot,
  ScanCaps,
  ScanDone,
  ScanError,
  ScanErrorKind,
  ScanParams,
  ScanProgress,
  Sides,
} from "./types";

export type UnlistenFn = () => void;

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

export class ScanFailureError extends Error {
  readonly kind: ScanErrorKind;

  constructor(kind: ScanErrorKind, message: string) {
    super(message);
    this.kind = kind;
  }
}

function toScanFailure(error: unknown): ScanFailureError {
  if (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error
  ) {
    const payload = error as { kind: unknown; message: unknown };
    const kind = isScanErrorKind(payload.kind) ? payload.kind : "device";
    const message =
      typeof payload.message === "string" && payload.message !== ""
        ? payload.message
        : TEXT.requestFailed;
    return new ScanFailureError(kind, message);
  }
  return new ScanFailureError("device", errorText(error));
}

export function scanFailureKind(error: unknown): ScanErrorKind | null {
  return error instanceof ScanFailureError ? error.kind : null;
}

export function errorText(error: unknown): string {
  if (typeof error === "string" && error.trim() !== "") {
    return error;
  }
  if (error instanceof Error && error.message !== "") {
    return error.message;
  }
  return TEXT.requestFailed;
}

export function getCapabilities(): Promise<ScanCaps> {
  return invoke<ScanCaps>("get_capabilities");
}

export function getStartupError(): Promise<string | null> {
  return invoke<string | null>("get_startup_error");
}

export async function startScan(params: ScanParams): Promise<string> {
  try {
    return await invoke<string>("start_scan", { params });
  } catch (error) {
    throw toScanFailure(error);
  }
}

export function cancelScan(jobId: string): Promise<void> {
  return invoke<void>("cancel_scan", { jobId });
}

export function openScanFolder(): Promise<void> {
  return invoke<void>("open_scan_folder");
}

export function onScanProgress(
  callback: (payload: ScanProgress) => void,
): Promise<UnlistenFn> {
  return listen<ScanProgress>("scan-progress", (event) => callback(event.payload));
}

export function onScanDone(
  callback: (payload: ScanDone) => void,
): Promise<UnlistenFn> {
  return listen<ScanDone>("scan-done", (event) => callback(event.payload));
}

export function onScanError(
  callback: (payload: ScanError) => void,
): Promise<UnlistenFn> {
  return listen<ScanError>("scan-error", (event) => callback(event.payload));
}

export function getStatus(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("get_status");
}

export function refreshStatus(): Promise<DeviceStatus> {
  return invoke<DeviceStatus>("refresh_status");
}

export function openPrinterPanel(): Promise<void> {
  return invoke<void>("open_printer_panel");
}

export function openFirmwarePage(): Promise<void> {
  return invoke<void>("open_firmware_page");
}

export function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

export function getHistory(days: number): Promise<HistorySnapshot[]> {
  return invoke<HistorySnapshot[]>("get_history", { days });
}

export function printFile(
  path: string,
  copies: number,
  sides: Sides,
): Promise<string> {
  return invoke<string>("print_file", { path, copies, sides });
}

export function getQueue(): Promise<CupsJob[]> {
  return invoke<CupsJob[]>("get_queue");
}

export function cancelPrint(jobId: string): Promise<void> {
  return invoke<void>("cancel_print", { jobId });
}

export function onStatus(
  callback: (payload: DeviceStatus) => void,
): Promise<UnlistenFn> {
  return listen<DeviceStatus>("status", (event) => callback(event.payload));
}

export function onNavigate(
  callback: (screen: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("navigate", (event) => callback(event.payload));
}

export function setConfig(config: Config): Promise<Config> {
  return invoke<Config>("set_config", { config });
}

export function testNotification(): Promise<void> {
  return invoke<void>("test_notification");
}
