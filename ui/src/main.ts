import {
  cancelScan,
  getCapabilities,
  getStartupError,
  onScanDone,
  onScanError,
  onScanProgress,
  openScanFolder,
  startScan,
  type UnlistenFn,
} from "./api";
import {
  COLOR_LABELS,
  FORMAT_LABELS,
  PAGE_FORMS,
  SOURCE_LABELS,
  TEXT,
} from "./labels";
import type {
  ColorMode,
  DocFormat,
  InputCaps,
  ScanCaps,
  ScanDone,
  ScanError,
  ScanParams,
  ScanPhase,
  ScanProgress,
  Source,
} from "./types";

function mustQuery<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (element === null) {
    throw new Error(`element ${selector} not found`);
  }
  return element;
}

const els = {
  deviceModel: mustQuery<HTMLParagraphElement>("#device-model"),
  startupBanner: mustQuery<HTMLDivElement>("#startup-banner"),
  startupBannerText: mustQuery<HTMLParagraphElement>("#startup-banner-text"),
  capsBanner: mustQuery<HTMLDivElement>("#caps-banner"),
  capsBannerText: mustQuery<HTMLParagraphElement>("#caps-banner-text"),
  capsRetry: mustQuery<HTMLButtonElement>("#caps-retry"),
  form: mustQuery<HTMLFormElement>("#scan-form"),
  source: mustQuery<HTMLSelectElement>("#source"),
  dpi: mustQuery<HTMLSelectElement>("#dpi"),
  color: mustQuery<HTMLSelectElement>("#color"),
  format: mustQuery<HTMLSelectElement>("#format"),
  formatHint: mustQuery<HTMLParagraphElement>("#format-hint"),
  start: mustQuery<HTMLButtonElement>("#start"),
  cancel: mustQuery<HTMLButtonElement>("#cancel"),
  status: mustQuery<HTMLParagraphElement>("#status"),
  result: mustQuery<HTMLElement>("#result"),
  resultPath: mustQuery<HTMLElement>("#result-path"),
  openFolder: mustQuery<HTMLButtonElement>("#open-folder"),
};

type Tone = "idle" | "busy" | "ok" | "warn" | "error";

interface ScreenState {
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
}

const state: ScreenState = {
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
};

const DPI_PREFERENCE = [300, 200, 600, 100];
const COLOR_PREFERENCE: ColorMode[] = [
  "Grayscale8",
  "RGB24",
  "BlackAndWhite1",
];
const FORMAT_PREFERENCE: DocFormat[] = ["application/pdf", "image/jpeg"];

function isSource(value: string): value is Source {
  return value === "Platen" || value === "Feeder";
}

function isColorMode(value: string): value is ColorMode {
  return (
    value === "BlackAndWhite1" ||
    value === "Grayscale8" ||
    value === "RGB24"
  );
}

function isDocFormat(value: string): value is DocFormat {
  return value === "application/pdf" || value === "image/jpeg";
}

function errorText(error: unknown): string {
  if (typeof error === "string" && error.trim() !== "") {
    return error;
  }
  if (error instanceof Error && error.message !== "") {
    return error.message;
  }
  return TEXT.requestFailed;
}

function pickFirst<T>(available: readonly T[], preferred: readonly T[]): T | null {
  for (const value of preferred) {
    if (available.includes(value)) {
      return value;
    }
  }
  return available.length > 0 ? available[0] : null;
}

function activeInput(caps: ScanCaps, source: Source | null): InputCaps | null {
  return source === "Feeder" ? caps.adf : caps.platen;
}

function pluralPages(count: number): string {
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

function pickDpi(input: InputCaps, current: number | null): number | null {
  if (current !== null && input.resolutions.includes(current)) {
    return current;
  }
  return pickFirst(input.resolutions, DPI_PREFERENCE);
}

function pickFormat(
  input: InputCaps,
  color: ColorMode | null,
  current: DocFormat | null,
): DocFormat | null {
  if (color === "BlackAndWhite1" && input.formats.includes("application/pdf")) {
    return "application/pdf";
  }
  if (current !== null && input.formats.includes(current)) {
    return current;
  }
  return pickFirst(input.formats, FORMAT_PREFERENCE);
}

function normalizeSelections(): void {
  const caps = state.caps;
  if (caps === null) {
    return;
  }

  if (state.source === null || (state.source === "Feeder" && caps.adf === null)) {
    state.source = "Platen";
  }

  const input = activeInput(caps, state.source);
  if (input === null) {
    return;
  }

  state.dpi = pickDpi(input, state.dpi);
  if (state.color === null || !input.colorModes.includes(state.color)) {
    state.color = pickFirst(input.colorModes, COLOR_PREFERENCE);
  }
  state.format = pickFormat(input, state.color, state.format);
}

function currentParams(): ScanParams | null {
  const { source, dpi, color, format } = state;
  if (source === null || dpi === null || color === null || format === null) {
    return null;
  }
  return { source, dpi, color, format };
}

interface OptionSpec {
  value: string;
  label: string;
  disabled?: boolean;
}

function fillSelect(
  select: HTMLSelectElement,
  options: readonly OptionSpec[],
  selected: string | null,
): void {
  const signature = options
    .map((option) => `${option.value}${option.disabled === true ? "!" : ""}`)
    .join("\u0000");

  if (select.dataset.options !== signature) {
    select.replaceChildren(
      ...options.map((option) => {
        const element = document.createElement("option");
        element.value = option.value;
        element.textContent = option.label;
        element.disabled = option.disabled === true;
        return element;
      }),
    );
    select.dataset.options = signature;
  }

  if (selected !== null) {
    select.value = selected;
  }
}

function statusView(): { text: string; tone: Tone } {
  if (state.capsError !== null) {
    return { text: TEXT.deviceUnavailable, tone: "error" };
  }
  if (state.errorText !== null) {
    return { text: state.errorText, tone: "error" };
  }
  if (state.starting) {
    return { text: TEXT.creatingJob, tone: "busy" };
  }
  if (state.capsLoading && state.caps === null) {
    return { text: TEXT.checkingDevice, tone: "busy" };
  }

  const phase = state.phase;
  switch (phase) {
    case "created":
      return { text: TEXT.jobCreated, tone: "busy" };
    case "scanning": {
      const pages =
        state.pages > 0
          ? ` · ${state.pages} ${pluralPages(state.pages)}`
          : "";
      return { text: `${TEXT.scanning}${pages}`, tone: "busy" };
    }
    case "done":
      return { text: TEXT.done, tone: "ok" };
    case "canceled":
      return { text: TEXT.canceled, tone: "warn" };
    case null:
      return state.caps === null
        ? { text: "", tone: "idle" }
        : { text: TEXT.ready, tone: "idle" };
  }
}

function render(): void {
  normalizeSelections();

  const caps = state.caps;
  const input = caps === null ? null : activeInput(caps, state.source);
  const busy = state.starting || state.jobId !== null;
  const controlsDisabled = busy || state.capsLoading || caps === null;

  els.deviceModel.textContent = caps === null ? "" : caps.makeAndModel;

  const sourceOptions: OptionSpec[] = [
    { value: "Platen", label: SOURCE_LABELS.Platen },
  ];
  if (caps !== null && caps.adf !== null) {
    sourceOptions.push({ value: "Feeder", label: SOURCE_LABELS.Feeder });
  }
  fillSelect(els.source, caps === null ? [] : sourceOptions, state.source);
  els.source.disabled = controlsDisabled;

  const dpiOptions: OptionSpec[] =
    input === null
      ? []
      : input.resolutions.map((dpi) => ({ value: String(dpi), label: `${dpi} dpi` }));
  fillSelect(
    els.dpi,
    dpiOptions,
    state.dpi === null ? null : String(state.dpi),
  );
  els.dpi.disabled = controlsDisabled || dpiOptions.length === 0;

  const colorOptions: OptionSpec[] =
    input === null
      ? []
      : input.colorModes.map((mode) => ({
          value: mode,
          label: COLOR_LABELS[mode],
        }));
  fillSelect(els.color, colorOptions, state.color);
  els.color.disabled = controlsDisabled || colorOptions.length === 0;

  const blackAndWhite = state.color === "BlackAndWhite1";
  const jpegAvailable = input !== null && input.formats.includes("image/jpeg");
  const formatOptions: OptionSpec[] =
    input === null
      ? []
      : input.formats.map((format) => ({
          value: format,
          label: FORMAT_LABELS[format],
          disabled: blackAndWhite && format === "image/jpeg",
        }));
  fillSelect(els.format, formatOptions, state.format);
  els.format.disabled = controlsDisabled || formatOptions.length === 0;

  const showFormatHint = blackAndWhite && jpegAvailable;
  els.formatHint.hidden = !showFormatHint;
  els.formatHint.textContent = showFormatHint ? TEXT.formatHint : "";

  els.start.disabled =
    caps === null ||
    busy ||
    state.capsLoading ||
    state.capsError !== null ||
    currentParams() === null;
  els.cancel.hidden = state.jobId === null;
  els.cancel.disabled = state.canceling;
  els.cancel.textContent = state.canceling ? TEXT.canceling : TEXT.cancel;

  els.startupBanner.hidden = state.startupError === null;
  els.startupBannerText.textContent = state.startupError ?? "";
  els.capsBanner.hidden = state.capsError === null;
  els.capsBannerText.textContent = state.capsError ?? "";
  els.capsRetry.disabled = state.capsLoading;

  const status = statusView();
  els.status.textContent = status.text;
  els.status.dataset.tone = status.tone;

  els.result.hidden = state.lastFile === null;
  els.resultPath.textContent = state.lastFile ?? "";
}

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

function handleProgress(payload: ScanProgress): void {
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

  render();
}

function handleDone(payload: ScanDone): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = "done";
  state.lastFile = payload.file;
  state.errorText = null;
  render();
}

function handleError(payload: ScanError): void {
  if (!acceptEvent(payload.jobId)) {
    return;
  }
  finishJob(payload.jobId);
  state.phase = null;
  state.errorText = payload.message;
  render();
}

async function loadCapabilities(): Promise<void> {
  state.capsLoading = true;
  state.capsError = null;
  render();

  try {
    state.caps = await getCapabilities();
  } catch (error) {
    state.capsError = errorText(error);
  } finally {
    state.capsLoading = false;
    render();
  }
}

async function handleStart(): Promise<void> {
  const params = currentParams();
  if (params === null || state.starting || state.jobId !== null) {
    return;
  }

  state.starting = true;
  state.errorText = null;
  state.phase = null;
  state.pages = 0;
  state.lastFile = null;
  state.canceling = false;
  render();

  try {
    const jobId = await startScan(params);
    if (state.jobId === null && state.lastFinishedJob !== jobId) {
      state.jobId = jobId;
    }
  } catch (error) {
    state.errorText = errorText(error);
    state.jobId = null;
    state.phase = null;
    state.pages = 0;
  } finally {
    state.starting = false;
    render();
  }
}

async function handleCancel(): Promise<void> {
  const jobId = state.jobId;
  if (jobId === null || state.canceling) {
    return;
  }

  state.canceling = true;
  render();

  try {
    await cancelScan(jobId);
  } catch (error) {
    state.canceling = false;
    state.errorText = errorText(error);
    render();
  }
}

async function handleOpenFolder(): Promise<void> {
  try {
    await openScanFolder();
  } catch (error) {
    state.errorText = errorText(error);
    render();
  }
}

async function loadStartupError(): Promise<void> {
  try {
    const message = await getStartupError();
    state.startupError = message !== null && message !== "" ? message : null;
    render();
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
    unlisteners.push(await onScanProgress(handleProgress));
    unlisteners.push(await onScanDone(handleDone));
    unlisteners.push(await onScanError(handleError));
  } catch {
  }
}

function init(): void {
  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    void handleStart();
  });

  els.cancel.addEventListener("click", () => {
    void handleCancel();
  });

  els.capsRetry.addEventListener("click", () => {
    void loadCapabilities();
  });

  els.openFolder.addEventListener("click", () => {
    void handleOpenFolder();
  });

  els.source.addEventListener("change", () => {
    const value = els.source.value;
    if (isSource(value)) {
      state.source = value;
    }
    render();
  });

  els.dpi.addEventListener("change", () => {
    const parsed = Number.parseInt(els.dpi.value, 10);
    if (Number.isFinite(parsed)) {
      state.dpi = parsed;
    }
    render();
  });

  els.color.addEventListener("change", () => {
    const value = els.color.value;
    if (isColorMode(value)) {
      state.color = value;
    }
    render();
  });

  els.format.addEventListener("change", () => {
    const value = els.format.value;
    if (isDocFormat(value)) {
      state.format = value;
    }
    render();
  });

  void registerListeners();
  void loadStartupError();
  void loadCapabilities();
  render();
}

init();
