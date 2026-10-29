import { mustQuery } from "./dom";
import { t, tp } from "./i18n/runtime";
import { currentScreen } from "./router";
import {
  CONNECTIVITY_TONES,
  connectivity,
  type AppState,
} from "./state";
import type { Tone } from "./types";
import { initSettingsScreen } from "./screens/settings";

const els = {
  deviceDot: mustQuery<HTMLElement>(".device-status-dot"),
  deviceModel: mustQuery<HTMLParagraphElement>("#device-model"),
  startupBanner: mustQuery<HTMLDivElement>("#startup-banner"),
  startupBannerText: mustQuery<HTMLParagraphElement>("#startup-banner-text"),
  capsBanner: mustQuery<HTMLDivElement>("#caps-banner"),
  capsBannerText: mustQuery<HTMLParagraphElement>("#caps-banner-text"),
  capsRetry: mustQuery<HTMLButtonElement>("#caps-retry"),
  connBanner: mustQuery<HTMLDivElement>("#conn-banner"),
  connRetry: mustQuery<HTMLButtonElement>("#conn-retry"),
  scanNav: mustQuery<HTMLButtonElement>('[data-nav="scan"]'),
  status: mustQuery<HTMLParagraphElement>("#status"),
};

let connBannerShown: boolean | null = null;

function statusView(state: AppState): { text: string; tone: Tone } {
  if (currentScreen() === "settings") {
    return { text: "", tone: "idle" };
  }
  if (currentScreen() === "print") {
    return printStatusView(state);
  }
  if (state.capsError !== null) {
    return { text: t("scan.deviceUnavailable"), tone: "error" };
  }
  if (state.errorText !== null) {
    return { text: state.errorText, tone: "error" };
  }
  if (state.starting) {
    return { text: t("scan.creatingJob"), tone: "busy" };
  }
  if (state.capsLoading && state.caps === null) {
    return { text: t("scan.checkingDevice"), tone: "busy" };
  }

  const phase = state.phase;
  switch (phase) {
    case "created":
      return { text: t("scan.jobCreated"), tone: "busy" };
    case "scanning": {
      const pages =
        state.pages > 0 ? ` · ${tp("scan.pages", state.pages)}` : "";
      return { text: `${t("scan.scanning")}${pages}`, tone: "busy" };
    }
    case "done":
      return { text: t("scan.done"), tone: "ok" };
    case "canceled":
      return { text: t("scan.canceled"), tone: "warn" };
    case null:
      return state.caps === null
        ? { text: "", tone: "idle" }
        : { text: t("scan.ready"), tone: "idle" };
  }
}

function printStatusView(state: AppState): { text: string; tone: Tone } {
  if (state.printError !== null) {
    return { text: state.printError, tone: "error" };
  }
  if (state.queueError !== null) {
    return { text: state.queueError, tone: "error" };
  }
  if (state.printSending) {
    return { text: t("print.sendingPrint"), tone: "busy" };
  }
  if (state.cancelingJobId !== null) {
    return { text: t("print.cancelingPrint"), tone: "busy" };
  }
  return {
    text:
      state.printFile === null
        ? t("print.printPickFile")
        : t("print.printReady"),
    tone: "idle",
  };
}

function renderConnBanner(state: AppState): void {
  const shown = connectivity(state) === "offline" && state.esclFailed;
  if (shown === connBannerShown) {
    return;
  }
  connBannerShown = shown;
  els.connBanner.hidden = !shown;
}

export function renderShell(state: AppState): void {
  const caps = state.caps;
  els.deviceModel.textContent = caps === null ? "" : caps.makeAndModel;
  els.deviceDot.dataset["tone"] = CONNECTIVITY_TONES[connectivity(state)];
  els.scanNav.toggleAttribute("data-active", state.jobId !== null);

  els.startupBanner.hidden = state.startupError === null;
  els.startupBannerText.textContent = state.startupError ?? "";
  const capsBannerText =
    state.capsError ?? (state.esclFailed ? t("scan.esclUnavailable") : null);
  els.capsBanner.hidden = capsBannerText === null;
  const capsBannerValue = capsBannerText ?? "";
  if (els.capsBannerText.textContent !== capsBannerValue) {
    els.capsBannerText.textContent = capsBannerValue;
  }
  els.capsRetry.disabled = state.capsLoading;
  renderConnBanner(state);
  els.connRetry.disabled = state.refreshingStatus;

  const status = statusView(state);
  els.status.textContent = status.text;
  els.status.dataset["tone"] = status.tone;
}

export function initShell(
  onCapsRetry: () => void,
  onStatusRetry: () => void,
): void {
  els.capsRetry.addEventListener("click", () => {
    onCapsRetry();
  });
  els.connRetry.addEventListener("click", () => {
    onStatusRetry();
  });
  initSettingsScreen();
}
