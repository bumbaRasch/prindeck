import { convertFileSrc } from "@tauri-apps/api/core";
import { errorText, openScanFolder } from "../api";
import { mustQuery } from "../dom";
import {
  elapsedSeconds,
  filesReady,
  pageEntry,
  partialPages,
  receivedPages,
} from "../labels";
import { renderShell } from "../shell";
import type { AppState } from "../state";

const ELAPSED_TICK_MS = 10_000;
const JPEG_PATTERN = /\.(jpe?g)$/i;

const els = {
  progress: mustQuery<HTMLElement>("#scan-progress"),
  counter: mustQuery<HTMLParagraphElement>("#pages-counter"),
  elapsed: mustQuery<HTMLParagraphElement>("#scan-elapsed"),
  pageList: mustQuery<HTMLUListElement>("#page-list"),
  result: mustQuery<HTMLElement>("#result"),
  title: mustQuery<HTMLHeadingElement>("#result-title"),
  partial: mustQuery<HTMLParagraphElement>("#result-partial"),
  warning: mustQuery<HTMLParagraphElement>("#result-warning"),
  files: mustQuery<HTMLUListElement>("#result-files"),
  openFolder: mustQuery<HTMLButtonElement>("#open-folder"),
};

function baseName(path: string): string {
  const index = path.lastIndexOf("/");
  return index === -1 ? path : path.slice(index + 1);
}

function elapsedSecondsValue(state: AppState): number {
  if (state.jobStartedAt === null) {
    return 0;
  }
  return Math.max(0, Math.floor((Date.now() - state.jobStartedAt) / 1000));
}

function renderElapsed(state: AppState): void {
  els.elapsed.textContent = elapsedSeconds(elapsedSecondsValue(state));
}

function pageItem(page: number): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "page-item";
  item.textContent = pageEntry(page);
  return item;
}

function renderPageList(pages: number): void {
  els.pageList.hidden = pages === 0;
  const signature = String(pages);
  if (els.pageList.dataset["pages"] === signature) {
    return;
  }
  els.pageList.dataset["pages"] = signature;
  els.pageList.replaceChildren(
    ...Array.from({ length: pages }, (_, index) => pageItem(index + 1)),
  );
}

function fileItem(path: string): HTMLLIElement {
  const item = document.createElement("li");
  item.className = "result-file";
  item.title = path;

  if (JPEG_PATTERN.test(path)) {
    const preview = document.createElement("img");
    preview.className = "result-preview";
    preview.alt = "";
    preview.src = convertFileSrc(path);
    item.appendChild(preview);
  }

  const name = document.createElement("span");
  name.className = "result-file-name";
  name.textContent = baseName(path);
  item.appendChild(name);
  return item;
}

function renderFiles(state: AppState): void {
  const files = state.lastFiles;
  els.files.hidden = files.length === 0;
  const signature = files.join("\u0000");
  if (els.files.dataset["files"] === signature) {
    return;
  }
  els.files.dataset["files"] = signature;
  els.files.replaceChildren(...files.map(fileItem));
}

function renderProgress(state: AppState): void {
  const running = state.starting || state.jobId !== null;
  els.progress.hidden = !running;
  if (!running) {
    return;
  }
  els.counter.textContent = receivedPages(state.pages);
  renderElapsed(state);
  renderPageList(state.pages);
}

function renderResult(state: AppState): void {
  const hasFiles = state.lastFiles.length > 0;
  els.result.hidden = !hasFiles;
  if (!hasFiles) {
    return;
  }
  els.title.textContent = filesReady(state.lastFiles.length);
  els.partial.hidden = !state.lastPartial;
  els.partial.textContent = state.lastPartial ? partialPages(state.pages) : "";
  els.warning.hidden = state.lastWarning === null;
  els.warning.textContent = state.lastWarning ?? "";
  renderFiles(state);
}

export function renderScanResult(state: AppState): void {
  renderProgress(state);
  renderResult(state);
}

async function handleOpenFolder(state: AppState): Promise<void> {
  try {
    await openScanFolder();
  } catch (error) {
    state.errorText = errorText(error);
    renderShell(state);
    renderScanResult(state);
  }
}

export function initScanResult(getState: () => AppState): void {
  els.openFolder.addEventListener("click", () => {
    void handleOpenFolder(getState());
  });

  window.setInterval(() => {
    const state = getState();
    if (state.starting || state.jobId !== null) {
      renderElapsed(state);
    }
  }, ELAPSED_TICK_MS);
}
