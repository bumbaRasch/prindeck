
export type Source = "Platen" | "Feeder";

export type ColorMode = "BlackAndWhite1" | "Grayscale8" | "RGB24";

export type DocFormat = "application/pdf" | "image/jpeg";

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
  message: string;
}
