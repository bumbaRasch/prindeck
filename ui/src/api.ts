import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import type {
  AlertPayload,
  Config,
  DeviceStatus,
  HistorySnapshot,
  ScanCaps,
  ScanDone,
  ScanError,
  ScanParams,
  ScanProgress,
  TrayLabels,
} from "./types";
import { toScanFailure } from "./scan_errors";

export type UnlistenFn = () => void;

export * from "./scan_errors";
export * from "./print_ipc";
export { getStartupError, setConfig } from "./settings_ipc";

export function getLocaleHint(): Promise<string | null> {
  return invoke<string | null>("get_locale_hint");
}

export function getCapabilities(): Promise<ScanCaps> {
  return invoke<ScanCaps>("get_capabilities");
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

export function onAlert(
  callback: (payload: AlertPayload) => void,
): Promise<UnlistenFn> {
  return listen<AlertPayload>("alert", (event) => callback(event.payload));
}

export async function showNotification(
  title: string,
  body: string,
): Promise<void> {
  let granted = await isPermissionGranted();
  if (!granted) {
    granted = (await requestPermission()) === "granted";
  }
  if (granted) {
    sendNotification({ title, body });
  }
}

export function setTrayLabels(labels: TrayLabels): Promise<void> {
  return invoke<void>("set_tray_labels", { labels });
}
