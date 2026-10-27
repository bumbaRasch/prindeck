import { cancelScan, errorText, scanFailureKind, startScan } from "../api";
import {
  activeInput,
  colorOptions,
  dpiOptions,
  formatOptions,
  isColorMode,
  isDocFormat,
  isSource,
  normalizeSelections,
  sourceOptions,
} from "../caps";
import { fillSelect, mustQuery } from "../dom";
import { TEXT } from "../labels";
import { currentParams, type AppState } from "../state";
import { renderShell } from "../shell";
import { initScanResult, renderScanResult } from "./scan_result";

const els = {
  form: mustQuery<HTMLFormElement>("#scan-form"),
  source: mustQuery<HTMLSelectElement>("#source"),
  dpi: mustQuery<HTMLSelectElement>("#dpi"),
  color: mustQuery<HTMLSelectElement>("#color"),
  format: mustQuery<HTMLSelectElement>("#format"),
  formatHint: mustQuery<HTMLParagraphElement>("#format-hint"),
  start: mustQuery<HTMLButtonElement>("#start"),
  cancel: mustQuery<HTMLButtonElement>("#cancel"),
};

function render(state: AppState): void {
  renderShell(state);
  renderScanScreen(state);
}

export function renderScanScreen(state: AppState): void {
  normalizeSelections(state);
  renderScanResult(state);

  const caps = state.caps;
  const input = caps === null ? null : activeInput(caps, state.source);
  const busy = state.starting || state.jobId !== null;
  const controlsDisabled = busy || state.capsLoading || caps === null;

  const sources = sourceOptions(caps);
  fillSelect(els.source, sources, state.source);
  els.source.disabled = controlsDisabled;

  const dpi = input === null ? [] : dpiOptions(input);
  fillSelect(els.dpi, dpi, state.dpi === null ? null : String(state.dpi));
  els.dpi.disabled = controlsDisabled || dpi.length === 0;

  const colors = input === null ? [] : colorOptions(input);
  fillSelect(els.color, colors, state.color);
  els.color.disabled = controlsDisabled || colors.length === 0;

  const blackAndWhite = state.color === "BlackAndWhite1";
  const jpegAvailable = input !== null && input.formats.includes("image/jpeg");
  const formats = input === null ? [] : formatOptions(input, blackAndWhite);
  fillSelect(els.format, formats, state.format);
  els.format.disabled = controlsDisabled || formats.length === 0;

  const showFormatHint = blackAndWhite && jpegAvailable;
  els.formatHint.hidden = !showFormatHint;
  els.formatHint.textContent = showFormatHint ? TEXT.formatHint : "";

  els.start.disabled =
    caps === null ||
    busy ||
    state.capsLoading ||
    state.capsError !== null ||
    currentParams(state) === null;
  els.cancel.hidden = state.jobId === null;
  els.cancel.disabled = state.canceling;
  els.cancel.textContent = state.canceling ? TEXT.canceling : TEXT.cancel;
}

async function handleStart(state: AppState): Promise<void> {
  const params = currentParams(state);
  if (params === null || state.starting || state.jobId !== null) {
    return;
  }

  state.starting = true;
  state.errorText = null;
  state.phase = null;
  state.pages = 0;
  state.jobStartedAt = Date.now();
  state.lastFiles = [];
  state.lastPartial = false;
  state.lastWarning = null;
  state.canceling = false;
  render(state);

  try {
    const jobId = await startScan(params);
    state.esclFailed = false;
    if (state.jobId === null && state.lastFinishedJob !== jobId) {
      state.jobId = jobId;
    }
  } catch (error) {
    state.errorText = errorText(error);
    if (scanFailureKind(error) === "transport") {
      state.esclFailed = true;
    }
    state.jobId = null;
    state.phase = null;
    state.pages = 0;
    state.jobStartedAt = null;
  } finally {
    state.starting = false;
    render(state);
  }
}

async function handleCancel(state: AppState): Promise<void> {
  const jobId = state.jobId;
  if (jobId === null || state.canceling) {
    return;
  }

  state.canceling = true;
  render(state);

  try {
    await cancelScan(jobId);
  } catch (error) {
    state.canceling = false;
    state.errorText = errorText(error);
    render(state);
  }
}

export function initScanScreen(getState: () => AppState): void {
  initScanResult(getState);

  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    void handleStart(getState());
  });

  els.cancel.addEventListener("click", () => {
    void handleCancel(getState());
  });

  els.source.addEventListener("change", () => {
    const value = els.source.value;
    if (isSource(value)) {
      getState().source = value;
    }
    render(getState());
  });

  els.dpi.addEventListener("change", () => {
    const parsed = Number.parseInt(els.dpi.value, 10);
    if (Number.isFinite(parsed)) {
      getState().dpi = parsed;
    }
    render(getState());
  });

  els.color.addEventListener("change", () => {
    const value = els.color.value;
    if (isColorMode(value)) {
      getState().color = value;
    }
    render(getState());
  });

  els.format.addEventListener("change", () => {
    const value = els.format.value;
    if (isDocFormat(value)) {
      getState().format = value;
    }
    render(getState());
  });
}
