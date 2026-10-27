import { mustQuery } from "./dom";
import { TEXT } from "./labels";
import { pluralPages, type AppState } from "./state";

export const capsRetry = mustQuery<HTMLButtonElement>("#caps-retry");

const els = {
  deviceModel: mustQuery<HTMLParagraphElement>("#device-model"),
  startupBanner: mustQuery<HTMLDivElement>("#startup-banner"),
  startupBannerText: mustQuery<HTMLParagraphElement>("#startup-banner-text"),
  capsBanner: mustQuery<HTMLDivElement>("#caps-banner"),
  capsBannerText: mustQuery<HTMLParagraphElement>("#caps-banner-text"),
  capsRetry,
  status: mustQuery<HTMLParagraphElement>("#status"),
};

type Tone = "idle" | "busy" | "ok" | "warn" | "error";

function statusView(state: AppState): { text: string; tone: Tone } {
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

export function renderShell(state: AppState): void {
  const caps = state.caps;
  els.deviceModel.textContent = caps === null ? "" : caps.makeAndModel;

  els.startupBanner.hidden = state.startupError === null;
  els.startupBannerText.textContent = state.startupError ?? "";
  els.capsBanner.hidden = state.capsError === null;
  els.capsBannerText.textContent = state.capsError ?? "";
  els.capsRetry.disabled = state.capsLoading;

  const status = statusView(state);
  els.status.textContent = status.text;
  els.status.dataset["tone"] = status.tone;
}

export function initShell(): void {}
