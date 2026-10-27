
export type Source = "Platen" | "Feeder";

export type ColorMode = "BlackAndWhite1" | "Grayscale8" | "RGB24";

export type DocFormat = "application/pdf" | "image/jpeg";

export type Screen = "scan" | "overview" | "print";

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
  polledAt: string | null;
  error: string | null;
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

export interface ScanDone {
  jobId: string;
  file: string;
}

export interface ScanError {
  jobId: string;
  kind: ScanErrorKind;
  message: string;
}
