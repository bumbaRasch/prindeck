import type { AppState } from "../state";
import { initPrintForm, renderPrintForm } from "./print_form";
import {
  activatePrintQueue,
  deactivatePrintQueue,
  initPrintQueue,
  renderPrintQueue,
} from "./print_queue";

export function renderPrintScreen(state: AppState): void {
  renderPrintForm(state);
  renderPrintQueue(state);
}

export function setPrintScreenActive(active: boolean): void {
  if (active) {
    activatePrintQueue();
  } else {
    deactivatePrintQueue();
  }
}

export function initPrintScreen(stateGetter: () => AppState): void {
  initPrintForm(stateGetter);
  initPrintQueue(stateGetter);
}
