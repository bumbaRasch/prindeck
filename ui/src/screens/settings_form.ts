import { showNotification } from "../api";
import { t } from "../i18n/runtime";
import { errorText } from "../scan_errors";
import { configFieldError, setConfig, settingsErrorKey } from "../settings_ipc";
import type { Config, Tone } from "../types";
import { configFromValues, validateSettingsValues } from "./settings_values";
import type { SettingsFormValues } from "./settings_values";
import { els, inputs, renderFieldErrors } from "./settings_form_dom";

let baseline: Config | null = null;
let saving = false;
let onApplied: ((config: Config) => void) | null = null;

function refreshControls(): void {
  const disabled = saving || baseline === null;
  els.save.disabled = disabled;
  els.test.disabled = disabled;
  els.save.textContent = saving ? t("settings.saving") : t("settings.save");
  for (const input of Object.values(inputs)) {
    input.disabled = disabled;
  }
}

export function settingsFormMessage(text: string, tone: Tone): void {
  els.status.textContent = text;
  els.status.dataset["tone"] = tone;
}

export function loadSettingsForm(config: Config): void {
  baseline = config;
  inputs.host.value = config.device.host;
  inputs.poll.value = String(config.pollSeconds);
  inputs.scanDir.value = config.scanDir;
  inputs.tonerModel.value = config.toner.model ?? "";
  inputs.tonerYield.value =
    config.toner.yieldPages === null ? "" : String(config.toner.yieldPages);
  inputs.notify.checked = config.notify.enabled;
  inputs.tonerPercent.value = String(config.notify.tonerPercent);
  inputs.drumPercent.value = String(config.notify.drumPercent);
  inputs.autostart.checked = config.autostart;
  renderFieldErrors([]);
  settingsFormMessage("", "idle");
  refreshControls();
}

function readValues(): SettingsFormValues {
  return {
    host: inputs.host.value,
    pollSeconds: inputs.poll.value,
    notifyEnabled: inputs.notify.checked,
    tonerPercent: inputs.tonerPercent.value,
    drumPercent: inputs.drumPercent.value,
    scanDir: inputs.scanDir.value,
    tonerModel: inputs.tonerModel.value,
    tonerYieldPages: inputs.tonerYield.value,
    autostart: inputs.autostart.checked,
  };
}

async function handleSave(): Promise<void> {
  const config = baseline;
  if (config === null || saving) {
    return;
  }

  const values = readValues();
  const errors = validateSettingsValues(values);
  renderFieldErrors(errors);
  if (errors.length > 0) {
    settingsFormMessage(t("settings.saveFailed"), "error");
    return;
  }

  saving = true;
  refreshControls();
  settingsFormMessage(t("settings.saving"), "busy");
  try {
    const applied = await setConfig(configFromValues(config, values));
    baseline = applied;
    renderFieldErrors([]);
    settingsFormMessage(t("settings.saved"), "ok");
    if (onApplied !== null) {
      onApplied(applied);
    }
  } catch (error) {
    const failure = configFieldError(error);
    const fieldError =
      failure === null ? null : settingsErrorKey(failure.field, failure.kind);
    if (fieldError !== null) {
      renderFieldErrors([fieldError]);
      settingsFormMessage(t("settings.saveFailed"), "error");
    } else if (failure !== null) {
      settingsFormMessage(t("settings.error.io"), "error");
    } else {
      settingsFormMessage(
        `${t("settings.saveFailed")}: ${errorText(error)}`,
        "error",
      );
    }
  } finally {
    saving = false;
    refreshControls();
  }
}

async function handleTest(): Promise<void> {
  try {
    await showNotification(t("notify.test.title"), t("notify.test.body"));
  } catch (error) {
    settingsFormMessage(errorText(error), "error");
  }
}

export function initSettingsForm(applied: (config: Config) => void): void {
  onApplied = applied;

  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    void handleSave();
  });
  els.test.addEventListener("click", () => {
    void handleTest();
  });
  refreshControls();
}
