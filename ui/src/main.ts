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
import { reloadCapabilities } from "./caps";
import { initRouter } from "./router";
import { createState } from "./state";
import { initShell, renderShell } from "./shell";
import type { DeviceStatus, ScanDone, ScanError, ScanProgress } from "./types";
import { initOverviewScreen, renderOverviewScreen } from "./screens/overview";
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
  state.lastFile = payload.file;
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
  state.esclFailed = true;
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
  } catch {
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
  } catch {
  } finally {
    state.refreshingStatus = false;
    render();
  }
}

async function loadCapabilities(): Promise<void> {
  state.capsLoading = true;
  state.capsError = null;
  render();
  await reloadCapabilities(state);
  render();
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
  } catch {
  }
}

function render(): void {
  renderScanScreen(state);
  renderOverviewScreen(state);
}

function init(): void {
  initShell(
    () => {
      void loadCapabilities();
    },
    () => {
      void refreshStatusNow();
    },
  );
  initRouter(render);
  initScanScreen(() => state);
  initOverviewScreen(() => state, () => refreshStatusNow());

  void registerListeners();
  void loadStartupError();
  void loadCapabilities();
  void loadStatus();
  render();
}

init();
