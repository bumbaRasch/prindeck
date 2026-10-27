import { PAGE_FORMS } from "./labels";
import type {
  ColorMode,
  DeviceStatus,
  DocFormat,
  ScanCaps,
  ScanParams,
  ScanPhase,
  Source,
} from "./types";

export interface AppState {
  caps: ScanCaps | null;
  capsLoading: boolean;
  capsError: string | null;
  startupError: string | null;
  source: Source | null;
  dpi: number | null;
  color: ColorMode | null;
  format: DocFormat | null;
  jobId: string | null;
  lastFinishedJob: string | null;
  starting: boolean;
  canceling: boolean;
  phase: ScanPhase | null;
  pages: number;
  errorText: string | null;
  lastFile: string | null;
  status: DeviceStatus | null;
}

export function createState(): AppState {
  return {
    caps: null,
    capsLoading: false,
    capsError: null,
    startupError: null,
    source: null,
    dpi: null,
    color: null,
    format: null,
    jobId: null,
    lastFinishedJob: null,
    starting: false,
    canceling: false,
    phase: null,
    pages: 0,
    errorText: null,
    lastFile: null,
    status: null,
  };
}

export function pluralPages(count: number): string {
  const mod10 = count % 10;
  const mod100 = count % 100;
  if (mod10 === 1 && mod100 !== 11) {
    return PAGE_FORMS.one;
  }
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
    return PAGE_FORMS.few;
  }
  return PAGE_FORMS.many;
}

export function currentParams(state: AppState): ScanParams | null {
  const { source, dpi, color, format } = state;
  if (source === null || dpi === null || color === null || format === null) {
    return null;
  }
  return { source, dpi, color, format };
}
