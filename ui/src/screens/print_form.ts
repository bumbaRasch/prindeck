import { printErrorText, printFile } from "../api";
import { fillSelect, mustQuery, type OptionSpec } from "../dom";
import { t } from "../i18n/runtime";
import type { AppState } from "../state";
import { renderShell } from "../shell";
import { choosePrintFile, registerFileDrop } from "./print_file";
import { refreshQueue } from "./print_queue";
import type { Sides } from "../types";

const MIN_COPIES = 1;
const MAX_COPIES = 99;

function sidesOptions(): OptionSpec[] {
  return [
    { value: "one-sided", label: t("print.sides.oneSided") },
    { value: "two-sided-long-edge", label: t("print.sides.longEdge") },
    { value: "two-sided-short-edge", label: t("print.sides.shortEdge") },
  ];
}

const TEMPLATE = `
  <form id="print-form" class="panel" novalidate>
    <div id="print-dropzone" class="dropzone">
      <p class="dropzone-title" data-i18n="print.dropzoneTitle"></p>
      <p class="dropzone-hint" data-i18n="print.dropzoneHint"></p>
      <button id="print-choose" class="button button--secondary" type="button" data-i18n="print.chooseFile"></button>
    </div>
    <p id="print-file" class="print-file" hidden></p>
    <div class="fields">
      <div class="field">
        <span class="field-label" data-i18n="print.copiesLabel"></span>
        <div class="stepper">
          <button id="print-copies-minus" class="stepper-button" type="button">−</button>
          <input id="print-copies" class="stepper-input" type="number" inputmode="numeric" min="${MIN_COPIES}" max="${MAX_COPIES}" step="1" value="1" />
          <button id="print-copies-plus" class="stepper-button" type="button">+</button>
        </div>
      </div>
      <label class="field">
        <span class="field-label" data-i18n="print.sidesLabel"></span>
        <select id="print-sides" class="select"></select>
      </label>
    </div>
    <div class="actions">
      <button id="print-start" class="button button--primary" type="submit" disabled data-i18n="print.printAction"></button>
    </div>
  </form>
`;

mustQuery<HTMLElement>("#print-form-mount").innerHTML = TEMPLATE;

const els = {
  form: mustQuery<HTMLFormElement>("#print-form"),
  dropzone: mustQuery<HTMLElement>("#print-dropzone"),
  choose: mustQuery<HTMLButtonElement>("#print-choose"),
  file: mustQuery<HTMLParagraphElement>("#print-file"),
  copies: mustQuery<HTMLInputElement>("#print-copies"),
  copiesMinus: mustQuery<HTMLButtonElement>("#print-copies-minus"),
  copiesPlus: mustQuery<HTMLButtonElement>("#print-copies-plus"),
  sides: mustQuery<HTMLSelectElement>("#print-sides"),
  start: mustQuery<HTMLButtonElement>("#print-start"),
};

let getState: (() => AppState) | null = null;

function render(state: AppState): void {
  renderShell(state);
  renderPrintForm(state);
}

function requireState(): AppState {
  if (getState === null) {
    throw new Error("the print screen is not initialized");
  }
  return getState();
}

function isSides(value: string): value is Sides {
  return (
    value === "one-sided" ||
    value === "two-sided-long-edge" ||
    value === "two-sided-short-edge"
  );
}

export function renderPrintForm(state: AppState): void {
  els.file.hidden = state.printFile === null;
  const file = state.printFile ?? "";
  if (els.file.textContent !== file) {
    els.file.textContent = file;
  }

  const copies = String(state.printCopies);
  if (els.copies.value !== copies && document.activeElement !== els.copies) {
    els.copies.value = copies;
  }
  els.copies.disabled = state.printSending;
  els.copiesMinus.disabled = state.printSending || state.printCopies <= MIN_COPIES;
  els.copiesPlus.disabled = state.printSending || state.printCopies >= MAX_COPIES;

  fillSelect(els.sides, sidesOptions(), state.printSides);
  els.sides.disabled = state.printSending;
  els.choose.disabled = state.printSending;
  els.start.disabled = state.printFile === null || state.printSending;
  els.start.textContent = state.printSending
    ? t("print.printSending")
    : t("print.printAction");
}

function clampCopies(value: number): number {
  if (!Number.isFinite(value)) {
    return MIN_COPIES;
  }
  return Math.min(MAX_COPIES, Math.max(MIN_COPIES, Math.trunc(value)));
}

function stepCopies(by: number): void {
  const state = requireState();
  state.printCopies = clampCopies(state.printCopies + by);
  render(state);
}

function selectFile(path: string): void {
  const state = requireState();
  state.printFile = path;
  state.printError = null;
  render(state);
}

async function handleChoose(): Promise<void> {
  const picked = await choosePrintFile();
  if (picked !== null) {
    selectFile(picked);
  }
}

async function handlePrint(): Promise<void> {
  const state = requireState();
  const file = state.printFile;
  if (file === null || state.printSending) {
    return;
  }

  state.printSending = true;
  state.printError = null;
  render(state);
  try {
    await printFile(file, state.printCopies, state.printSides);
  } catch (error) {
    state.printError = printErrorText(error);
  } finally {
    state.printSending = false;
    render(state);
    await refreshQueue();
  }
}

export function initPrintForm(stateGetter: () => AppState): void {
  getState = stateGetter;

  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    void handlePrint();
  });
  els.choose.addEventListener("click", () => {
    void handleChoose();
  });
  els.copiesMinus.addEventListener("click", () => stepCopies(-1));
  els.copiesPlus.addEventListener("click", () => stepCopies(1));
  els.copies.addEventListener("change", () => {
    const state = requireState();
    state.printCopies = clampCopies(els.copies.valueAsNumber);
    render(state);
  });
  els.sides.addEventListener("change", () => {
    const value = els.sides.value;
    if (isSides(value)) {
      requireState().printSides = value;
    }
  });

  registerFileDrop(els.dropzone, selectFile);
}
