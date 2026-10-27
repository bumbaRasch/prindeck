import {
  errorText,
  getCapabilities,
  getStartupError,
  onScanDone,
  onScanError,
  onScanProgress,
  type UnlistenFn,
} from "./api";
import { initRouter } from "./router";
import { createState } from "./state";
import { initShell, renderShell } from "./shell";
import type { ScanDone, ScanError, ScanProgress } from "./types";
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

  renderScanScreen(state);
}

function handleScanDone(payload: ScanDone): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = "done";
  state.lastFile = payload.file;
  state.errorText = null;
  renderScanScreen(state);
}

function handleScanError(payload: ScanError): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = null;
  state.errorText = payload.message;
  renderScanScreen(state);
}

async function loadCapabilities(): Promise<void> {
  state.capsLoading = true;
  state.capsError = null;
  renderScanScreen(state);

  try {
    state.caps = await getCapabilities();
  } catch (error) {
    state.capsError = errorText(error);
  } finally {
    state.capsLoading = false;
    renderScanScreen(state);
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
  } catch {
  }
}

function render(): void {
  renderScanScreen(state);
  renderOverviewScreen();
}

function init(): void {
  initShell(() => {
    void loadCapabilities();
  });
  initRouter(render);
  initScanScreen(() => state);
  initOverviewScreen();

  void registerListeners();
  void loadStartupError();
  void loadCapabilities();
  render();
}

init();
