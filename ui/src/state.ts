import { PAGE_FORMS } from "./labels";
import type {
  ColorMode,
  Connectivity,
  CupsJob,
  DeviceStatus,
  DocFormat,
  ScanCaps,
  ScanParams,
  ScanPhase,
  Sides,
  Source,
  Tone,
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
  esclFailed: boolean;
  refreshingStatus: boolean;
  printFile: string | null;
  printCopies: number;
  printSides: Sides;
  printSending: boolean;
  printError: string | null;
  queue: CupsJob[];
  queueLoading: boolean;
  queueLoaded: boolean;
  queueError: string | null;
  cancelingJobId: string | null;
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
    esclFailed: false,
    refreshingStatus: false,
    printFile: null,
    printCopies: 1,
    printSides: "one-sided",
    printSending: false,
    printError: null,
    queue: [],
    queueLoading: false,
    queueLoaded: false,
    queueError: null,
    cancelingJobId: null,
  };
}

export const CONNECTIVITY_TONES: Record<Connectivity, Tone> = {
  checking: "idle",
  online: "ok",
  offline: "error",
  scanning: "busy",
};

export function connectivity(state: AppState): Connectivity {
  if (state.jobId !== null) {
    return "scanning";
  }
  const status = state.status;
  if (status === null) {
    return "checking";
  }
  if (status.ok) {
    return "online";
  }
  return status.polledAt === null && status.error === null
    ? "checking"
    : "offline";
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
