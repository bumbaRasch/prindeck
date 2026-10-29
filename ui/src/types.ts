
export type Source = "Platen" | "Feeder";

export type ColorMode = "BlackAndWhite1" | "Grayscale8" | "RGB24";

export type DocFormat = "application/pdf" | "image/jpeg";

export type Screen = "scan" | "overview" | "print" | "settings";

export interface DeviceConfig {
  host: string;
}

export interface SnmpConfig {
  community: string;
}

export interface NotifyConfig {
  enabled: boolean;
  tonerPercent: number;
  drumPercent: number;
}

export interface TonerConfig {
  model: string | null;
  yieldPages: number | null;
}

export interface Config {
  device: DeviceConfig;
  scanDir: string;
  cupsQueue: string;
  snmp: SnmpConfig;
  pollSeconds: number;
  autostart: boolean;
  notify: NotifyConfig;
  toner: TonerConfig;
}

export type Sides = "one-sided" | "two-sided-long-edge" | "two-sided-short-edge";

export interface CupsJob {
  id: string;
  owner: string;
  sizeKb: number;
}

export type Connectivity = "checking" | "online" | "offline" | "scanning";

export type Tone = "idle" | "busy" | "ok" | "warn" | "error";

export interface DeviceStatus {
  ok: boolean;
  host: string;
  tonerPercent: number | null;
  drumPercent: number | null;
  pagesTotal: number | null;
  duplexPages: number | null;
  drumPagesLeft: number | null;
  jamCount: number | null;
  polledAt: string | null;
  error: string | null;
}

export interface HistorySnapshot {
  ts: string;
  tonerPercent: number | null;
  drumPercent: number | null;
  pagesTotal: number | null;
  duplexPages: number | null;
  drumPagesLeft: number | null;
}

export interface DeviceConfig {
  host: string;
}

export interface SnmpConfig {
  community: string;
}

export interface NotifyConfig {
  enabled: boolean;
  tonerPercent: number;
  drumPercent: number;
}

export interface TonerConfig {
  model: string | null;
  yieldPages: number | null;
}

export interface Config {
  device: DeviceConfig;
  scanDir: string;
  cupsQueue: string;
  snmp: SnmpConfig;
  pollSeconds: number;
  autostart: boolean;
  notify: NotifyConfig;
  toner: TonerConfig;
}

export interface ScanParams {
  source: Source;
  dpi: number;
  color: ColorMode;
  format: DocFormat;
}

export interface InputCaps {
  maxWidth: number;
  maxHeight: number;
  resolutions: number[];
  colorModes: ColorMode[];
  formats: DocFormat[];
}

export interface ScanCaps {
  makeAndModel: string;
  platen: InputCaps;
  adf: InputCaps | null;
  feederCapacity: number | null;
}

export type ScanPhase = "created" | "scanning" | "done" | "canceled";

export type ScanErrorKind =
  | "transport"
  | "busy"
  | "invalid"
  | "device"
  | "storage"
  | "canceled";

export interface ScanProgress {
  jobId: string;
  phase: ScanPhase;
  pages: number;
}

export interface ScanWarning {
  kind: ScanErrorKind;
  message: string;
}

export interface ScanDone {
  jobId: string;
  files: string[];
  partial: boolean;
  warning: ScanWarning | null;
}

export interface ScanError {
  jobId: string;
  kind: ScanErrorKind;
  message: string;
}

export interface AlertPayload {
  kind: string;
  percent: number | null;
}

export interface TrayLabels {
  show: string;
  scan: string;
  panel: string;
  quit: string;
}
