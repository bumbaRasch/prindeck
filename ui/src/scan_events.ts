import type { AppState } from "./state";
import { scanErrorText, scanFailureText } from "./scan_errors";
import type { ScanDone, ScanError, ScanProgress } from "./types";

export interface ScanEventHandlers {
  onProgress: (payload: ScanProgress) => void;
  onDone: (payload: ScanDone) => void;
  onError: (payload: ScanError) => void;
}

export function createScanEventHandlers(
  state: AppState,
  render: () => void,
): ScanEventHandlers {
  function acceptEvent(jobId: string): boolean {
    if (state.jobId !== null) {
      return state.jobId === jobId;
    }
    if (!state.starting) {
      return false;
    }
    return state.lastFinishedJob !== jobId;
  }

  function adoptJob(jobId: string): void {
    if (state.jobId === null) {
      state.jobId = jobId;
    }
  }

  function finishJob(jobId: string): void {
    state.lastFinishedJob = jobId;
    state.jobId = null;
    state.canceling = false;
    state.jobStartedAt = null;
  }

  function onProgress(payload: ScanProgress): void {
    if (!acceptEvent(payload.jobId)) {
      return;
    }
    adoptJob(payload.jobId);
    state.pages = payload.pages;

    switch (payload.phase) {
      case "created":
      case "scanning":
        state.phase = payload.phase;
        state.errorText = null;
        state.esclFailed = false;
        break;
      case "done":
        state.phase = "done";
        break;
      case "canceled":
        finishJob(payload.jobId);
        state.phase = "canceled";
        state.errorText = null;
        break;
    }

    render();
  }

  function onDone(payload: ScanDone): void {
    if (!acceptEvent(payload.jobId)) {
      return;
    }
    finishJob(payload.jobId);
    state.phase = "done";
    state.lastFiles = payload.files;
    state.lastPartial = payload.partial;
    state.lastWarning =
      payload.warning === null ? null : scanFailureText(payload.warning);
    state.errorText = null;
    state.esclFailed = false;
    render();
  }

  function onError(payload: ScanError): void {
    if (!acceptEvent(payload.jobId)) {
      return;
    }
    finishJob(payload.jobId);
    state.phase = null;
    state.errorText = scanErrorText(payload.kind);
    if (payload.kind === "transport") {
      state.esclFailed = true;
    }
    render();
  }

  return { onProgress, onDone, onError };
}
