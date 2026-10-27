import {
  getStatus,
  getStartupError,
  onScanDone,
  onScanError,
  onScanProgress,
  onStatus,
  refreshStatus,
  type UnlistenFn,
} from "./api";
import { bindCapabilitiesRetry, runCapabilitiesRefresh } from "./caps";
import { initRouter } from "./router";
import { createState } from "./state";
import { initShell, renderShell } from "./shell";
import type { DeviceStatus, ScanDone, ScanError, ScanProgress } from "./types";
import { initOverviewScreen, renderOverviewScreen } from "./screens/overview";
import { initPrintScreen, renderPrintScreen, setPrintScreenActive } from "./screens/print";
import { initScanScreen, renderScanScreen } from "./screens/scan";

const state = createState();

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
}

function handleScanProgress(payload: ScanProgress): void {
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

function handleScanDone(payload: ScanDone): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = "done";
  state.lastFiles = payload.files;
  state.lastPartial = payload.partial;
  state.lastWarning = payload.warning;
  state.errorText = null;
  state.esclFailed = false;
  render();
}

function handleScanError(payload: ScanError): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = null;
  state.errorText = payload.message;
  if (payload.kind === "transport") {
    state.esclFailed = true;
  }
  render();
}

function handleStatus(payload: DeviceStatus): void {
  state.status = payload;
  renderShell(state);
  renderOverviewScreen(state);
}

async function loadStatus(): Promise<void> {
  try {
    state.status = await getStatus();
  } catch (error) {
    console.warn("failed to load the device status", error);
  }
  renderShell(state);
  renderOverviewScreen(state);
}

async function refreshStatusNow(): Promise<void> {
  if (state.refreshingStatus) {
    return;
  }
  state.refreshingStatus = true;
  render();
  try {
    state.status = await refreshStatus();
  } catch (error) {
    console.warn("failed to refresh the device status", error);
  } finally {
    state.refreshingStatus = false;
    render();
  }
}

async function loadStartupError(): Promise<void> {
  try {
    const message = await getStartupError();
    state.startupError = message !== null && message !== "" ? message : null;
    renderShell(state);
  } catch {
  }
}

async function registerListeners(): Promise<void> {
  const unlisteners: UnlistenFn[] = [];

  window.addEventListener("beforeunload", () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  });

  try {
    unlisteners.push(await onScanProgress(handleScanProgress));
    unlisteners.push(await onScanDone(handleScanDone));
    unlisteners.push(await onScanError(handleScanError));
    unlisteners.push(await onStatus(handleStatus));
  } catch (error) {
    console.warn("failed to register the event listeners", error);
  }
}

function render(): void {
  renderShell(state);
  renderScanScreen(state);
  renderOverviewScreen(state);
  renderPrintScreen(state);
}

function init(): void {
  initShell(
    () => {
      void runCapabilitiesRefresh(state, render);
    },
    () => {
      void refreshStatusNow();
    },
  );
  initRouter((screen) => {
    render();
    setPrintScreenActive(screen === "print");
  });
  initScanScreen(() => state);
  initOverviewScreen(() => state, () => refreshStatusNow());
  initPrintScreen(() => state);
  bindCapabilitiesRetry(() => state, render);

  void registerListeners();
  void loadStartupError();
  void runCapabilitiesRefresh(state, render);
  void loadStatus();
  render();
}

init();
