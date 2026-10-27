import { cancelPrint, errorText, getQueue } from "../api";
import { mustQuery } from "../dom";
import { TEXT, formatSizeKb } from "../labels";
import type { AppState } from "../state";
import { renderShell } from "../shell";
import type { CupsJob } from "../types";

const QUEUE_REFRESH_MS = 5000;

const TEMPLATE = `
  <div class="overview-head">
    <h2 class="overview-title">${TEXT.queueTitle}</h2>
    <button id="print-queue-refresh" class="button button--ghost" type="button">${TEXT.refresh}</button>
  </div>
  <p id="print-queue-note" class="queue-note" hidden></p>
  <div id="print-queue-scroll" class="queue-scroll" hidden>
    <table class="queue-table">
      <thead>
        <tr>
          <th>${TEXT.queueJobLabel}</th>
          <th>${TEXT.queueOwnerLabel}</th>
          <th>${TEXT.queueSizeLabel}</th>
          <th>${TEXT.queueStateLabel}</th>
          <th></th>
        </tr>
      </thead>
      <tbody id="print-queue-body"></tbody>
    </table>
  </div>
`;

mustQuery<HTMLElement>("#print-queue-mount").innerHTML = TEMPLATE;

const els = {
  refresh: mustQuery<HTMLButtonElement>("#print-queue-refresh"),
  note: mustQuery<HTMLParagraphElement>("#print-queue-note"),
  scroll: mustQuery<HTMLDivElement>("#print-queue-scroll"),
  body: mustQuery<HTMLTableSectionElement>("#print-queue-body"),
};

let getState: (() => AppState) | null = null;
let queueTimer: number | null = null;

function render(state: AppState): void {
  renderShell(state);
  renderPrintQueue(state);
}

function noteText(state: AppState): string {
  if (!state.queueLoaded && state.queueLoading) {
    return TEXT.queueLoading;
  }
  if (state.queueLoaded && state.queue.length === 0 && state.queueError === null) {
    return TEXT.queueEmpty;
  }
  return "";
}

function textCell(text: string, className: string): HTMLTableCellElement {
  const cell = document.createElement("td");
  cell.className = className;
  cell.textContent = text;
  return cell;
}

function queueRow(job: CupsJob): HTMLTableRowElement {
  const row = document.createElement("tr");
  const cancel = document.createElement("button");
  cancel.type = "button";
  cancel.className = "button button--ghost";
  cancel.dataset["jobId"] = job.id;
  cancel.textContent = TEXT.cancelJob;
  const action = document.createElement("td");
  action.appendChild(cancel);
  row.append(
    textCell(job.id, "queue-cell-id"),
    textCell(job.owner, "queue-cell-owner"),
    textCell(formatSizeKb(job.sizeKb), "queue-cell-size"),
    textCell(TEXT.jobPendingLabel, "queue-cell-state"),
    action,
  );
  return row;
}

function updateCancelButtons(state: AppState): void {
  for (const job of state.queue) {
    const button = els.body.querySelector<HTMLButtonElement>(
      `[data-job-id="${CSS.escape(job.id)}"]`,
    );
    if (button === null) {
      continue;
    }
    const busy = state.cancelingJobId === job.id;
    button.disabled = busy;
    button.textContent = busy ? TEXT.cancelingJob : TEXT.cancelJob;
  }
}

export function renderPrintQueue(state: AppState): void {
  const text = noteText(state);
  els.note.hidden = text === "";
  if (els.note.textContent !== text) {
    els.note.textContent = text;
  }
  els.scroll.hidden = state.queue.length === 0;

  const signature = state.queue.map((job) => job.id).join(" ");
  if (els.body.dataset["jobs"] !== signature) {
    els.body.dataset["jobs"] = signature;
    els.body.replaceChildren(...state.queue.map(queueRow));
  }
  updateCancelButtons(state);
}

export async function refreshQueue(): Promise<void> {
  if (getState === null) {
    return;
  }
  const state = getState();
  state.queueLoading = true;
  render(state);
  try {
    state.queue = await getQueue();
    state.queueLoaded = true;
    state.queueError = null;
  } catch (error) {
    state.queueError = errorText(error);
  } finally {
    state.queueLoading = false;
    render(state);
  }
}

async function handleCancel(jobId: string): Promise<void> {
  if (getState === null) {
    return;
  }
  const state = getState();
  if (state.cancelingJobId !== null) {
    return;
  }

  state.cancelingJobId = jobId;
  render(state);
  try {
    await cancelPrint(jobId);
  } catch (error) {
    state.printError = errorText(error);
  } finally {
    state.cancelingJobId = null;
    render(state);
    await refreshQueue();
  }
}

function stopQueueTimer(): void {
  if (queueTimer !== null) {
    window.clearInterval(queueTimer);
    queueTimer = null;
  }
}

export function activatePrintQueue(): void {
  stopQueueTimer();
  queueTimer = window.setInterval(() => {
    void refreshQueue();
  }, QUEUE_REFRESH_MS);
  void refreshQueue();
}

export function deactivatePrintQueue(): void {
  stopQueueTimer();
}

export function initPrintQueue(stateGetter: () => AppState): void {
  getState = stateGetter;

  els.refresh.addEventListener("click", () => {
    void refreshQueue();
  });

  els.body.addEventListener("click", (event) => {
    const target = event.target;
    if (!(target instanceof HTMLButtonElement)) {
      return;
    }
    const jobId = target.dataset["jobId"];
    if (jobId !== undefined) {
      void handleCancel(jobId);
    }
  });
}
