import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TEXT } from "./labels";
import type {
  DeviceStatus,
  ScanCaps,
  ScanDone,
  ScanError,
  ScanParams,
  ScanProgress,
} from "./types";

export type UnlistenFn = () => void;

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

export function startScan(params: ScanParams): Promise<string> {
  return invoke<string>("start_scan", { params });
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

export function onStatus(
  callback: (payload: DeviceStatus) => void,
): Promise<UnlistenFn> {
  return listen<DeviceStatus>("status", (event) => callback(event.payload));
}
