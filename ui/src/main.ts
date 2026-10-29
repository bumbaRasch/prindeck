import {
  getStatus,
  getStartupError,
  onAlert,
  onNavigate,
  onScanDone,
  onScanError,
  onScanProgress,
  onStatus,
  refreshStatus,
  setTrayLabels,
  showNotification,
  type UnlistenFn,
} from "./api";
import { bindCapabilitiesRetry, runCapabilitiesRefresh } from "./caps";
import { initLocale, t } from "./i18n/runtime";
import { applyStaticTexts } from "./i18n/static";
import type { MessageKey } from "./locales/en";
import { createScanEventHandlers } from "./scan_events";
import { initRouter, setScreen } from "./router";
import { createState } from "./state";
import { initShell, renderShell } from "./shell";
import type { AlertPayload, DeviceStatus } from "./types";
import { initOverviewScreen, renderOverviewScreen } from "./screens/overview";
import { initPrintScreen, renderPrintScreen, setPrintScreenActive } from "./screens/print";
import { initScanScreen, renderScanScreen } from "./screens/scan";

const state = createState();
const scanEvents = createScanEventHandlers(state, render);

const NOTIFY_TEXTS: Record<string, { title: MessageKey; body: MessageKey }> = {
  "toner-low": { title: "notify.tonerLow.title", body: "notify.tonerLow.body" },
  "drum-low": { title: "notify.drumLow.title", body: "notify.drumLow.body" },
  silent: { title: "notify.silent.title", body: "notify.silent.body" },
  "back-online": {
    title: "notify.backOnline.title",
    body: "notify.backOnline.body",
  },
};

function handleAlert(payload: AlertPayload): void {
  const texts = NOTIFY_TEXTS[payload.kind];
  if (texts === undefined) {
    console.warn("unknown alert kind", payload.kind);
    return;
  }
  const params =
    payload.percent === null ? undefined : { percent: payload.percent };
  void showNotification(t(texts.title), t(texts.body, params));
}

function applyTrayLabels(): void {
  void setTrayLabels({
    show: t("tray.show"),
    scan: t("tray.scan"),
    panel: t("tray.panel"),
    quit: t("tray.quit"),
  });
}

function handleStatus(payload: DeviceStatus): void {
  state.status = payload;
  renderShell(state);
  renderOverviewScreen(state);
}

async function loadStatus(): Promise<void> {
  try {
    state.status = await getStatus();
  } catch (error) {
    console.warn("failed to load the device status", error);
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
  } catch (error) {
    console.warn("failed to refresh the device status", error);
  } finally {
    state.refreshingStatus = false;
    render();
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
    unlisteners.push(await onScanProgress(scanEvents.onProgress));
    unlisteners.push(await onScanDone(scanEvents.onDone));
    unlisteners.push(await onScanError(scanEvents.onError));
    unlisteners.push(await onStatus(handleStatus));
    unlisteners.push(await onAlert(handleAlert));
    unlisteners.push(
      await onNavigate((screen) => {
        if (screen === "scan") {
          setScreen(screen);
        }
      }),
    );
  } catch (error) {
    console.warn("failed to register the event listeners", error);
  }
}

function render(): void {
  renderShell(state);
  renderScanScreen(state);
  renderOverviewScreen(state);
  renderPrintScreen(state);
}

async function init(): Promise<void> {
  await initLocale();
  applyStaticTexts();
  applyTrayLabels();

  initShell(
    () => {
      void runCapabilitiesRefresh(state, render);
    },
    () => {
      void refreshStatusNow();
    },
  );
  initRouter((screen) => {
    render();
    setPrintScreenActive(screen === "print");
  });
  initScanScreen(() => state);
  initOverviewScreen(() => state, () => refreshStatusNow());
  initPrintScreen(() => state);
  bindCapabilitiesRetry(() => state, render);

  void registerListeners();
  void loadStartupError();
  void runCapabilitiesRefresh(state, render);
  void loadStatus();
  render();
}

void init();
