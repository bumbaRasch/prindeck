import { errorText, setConfig, testNotification } from "../api";
import { mustQuery } from "../dom";
import { TEXT } from "../labels";
import type { Config, Tone } from "../types";
import {
  configFromValues,
  fieldFromErrorText,
  validateSettingsValues,
} from "./settings_values";
import type { SettingsField, SettingsFieldError, SettingsFormValues } from "./settings_values";

const TEMPLATE = `
  <form id="settings-form" class="panel" novalidate>
    <h2 class="settings-title">${TEXT.settingsFormTitle}</h2>
    <div class="fields">
      <label class="field"><span class="field-label">${TEXT.hostLabel}</span>
        <input id="settings-host" class="input" type="text" autocomplete="off" spellcheck="false" />
        <span class="field-hint">${TEXT.hostHint}</span>
        <p class="field-error" data-error="host" hidden></p>
      </label>
      <label class="field"><span class="field-label">${TEXT.pollSecondsLabel}</span>
        <input id="settings-poll" class="input" type="number" min="60" max="3600" step="1" inputmode="numeric" />
        <p class="field-error" data-error="pollSeconds" hidden></p>
      </label>
      <label class="field"><span class="field-label">${TEXT.scanDirLabel}</span>
        <input id="settings-scan-dir" class="input" type="text" autocomplete="off" spellcheck="false" />
        <p class="field-error" data-error="scanDir" hidden></p>
      </label>
      <label class="field"><span class="field-label">${TEXT.tonerModelLabel}</span>
        <input id="settings-toner-model" class="input" type="text" autocomplete="off" spellcheck="false" />
      </label>
      <label class="field"><span class="field-label">${TEXT.tonerYieldLabel}</span>
        <input id="settings-toner-yield" class="input" type="number" min="1" max="100000" step="1" inputmode="numeric" />
        <p class="field-error" data-error="tonerYieldPages" hidden></p>
      </label>
      <label class="field checkbox-field">
        <input id="settings-notify" class="checkbox" type="checkbox" />
        <span class="checkbox-label">${TEXT.notifyEnabledLabel}</span>
      </label>
      <label class="field"><span class="field-label">${TEXT.tonerThresholdLabel}</span>
        <input id="settings-toner-percent" class="input" type="number" min="0" max="100" step="1" inputmode="numeric" />
        <p class="field-error" data-error="tonerPercent" hidden></p>
      </label>
      <label class="field"><span class="field-label">${TEXT.drumThresholdLabel}</span>
        <input id="settings-drum-percent" class="input" type="number" min="0" max="100" step="1" inputmode="numeric" />
        <p class="field-error" data-error="drumPercent" hidden></p>
      </label>
    </div>
    <p id="settings-status" class="status" role="status" aria-live="polite" data-tone="idle"></p>
    <div class="actions">
      <button id="settings-save" class="button button--primary" type="submit" disabled>${TEXT.saveSettings}</button>
      <button id="settings-test-notify" class="button button--secondary" type="button" disabled>${TEXT.testNotification}</button>
    </div>
  </form>
`;

mustQuery<HTMLElement>("#settings-form-mount").innerHTML = TEMPLATE;

const els = {
  form: mustQuery<HTMLFormElement>("#settings-form"),
  save: mustQuery<HTMLButtonElement>("#settings-save"),
  test: mustQuery<HTMLButtonElement>("#settings-test-notify"),
  status: mustQuery<HTMLParagraphElement>("#settings-status"),
};

const inputs = {
  host: mustQuery<HTMLInputElement>("#settings-host"),
  poll: mustQuery<HTMLInputElement>("#settings-poll"),
  scanDir: mustQuery<HTMLInputElement>("#settings-scan-dir"),
  tonerModel: mustQuery<HTMLInputElement>("#settings-toner-model"),
  tonerYield: mustQuery<HTMLInputElement>("#settings-toner-yield"),
  notify: mustQuery<HTMLInputElement>("#settings-notify"),
  tonerPercent: mustQuery<HTMLInputElement>("#settings-toner-percent"),
  drumPercent: mustQuery<HTMLInputElement>("#settings-drum-percent"),
};

const errorEls = new Map<SettingsField, HTMLElement>(
  Array.from(document.querySelectorAll<HTMLElement>("[data-error]")).map(
    (node) => [node.dataset["error"] as SettingsField, node],
  ),
);

let baseline: Config | null = null;
let saving = false;
let onApplied: ((config: Config) => void) | null = null;

function refreshControls(): void {
  const disabled = saving || baseline === null;
  els.save.disabled = disabled;
  els.test.disabled = disabled;
  els.save.textContent = saving ? TEXT.settingsSaving : TEXT.saveSettings;
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
  inputs.tonerYield.value = config.toner.yieldPages === null ? "" : String(config.toner.yieldPages);
  inputs.notify.checked = config.notify.enabled;
  inputs.tonerPercent.value = String(config.notify.tonerPercent);
  inputs.drumPercent.value = String(config.notify.drumPercent);
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
  };
}

function renderFieldErrors(errors: readonly SettingsFieldError[]): void {
  const byField = new Map(errors.map((error) => [error.field, error.message]));
  for (const [field, node] of errorEls) {
    const message = byField.get(field);
    node.hidden = message === undefined;
    node.textContent = message ?? "";
  }
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
    settingsFormMessage(TEXT.settingsSaveFailed, "error");
    return;
  }

  saving = true;
  refreshControls();
  settingsFormMessage(TEXT.settingsSaving, "busy");
  try {
    const applied = await setConfig(configFromValues(config, values));
    baseline = applied;
    renderFieldErrors([]);
    settingsFormMessage(TEXT.settingsSaved, "ok");
    if (onApplied !== null) {
      onApplied(applied);
    }
  } catch (error) {
    const message = errorText(error);
    const field = fieldFromErrorText(message);
    if (field !== null) {
      renderFieldErrors([{ field, message }]);
      settingsFormMessage(TEXT.settingsSaveFailed, "error");
    } else {
      settingsFormMessage(`${TEXT.settingsSaveFailed}: ${message}`, "error");
    }
  } finally {
    saving = false;
    refreshControls();
  }
}

async function handleTest(): Promise<void> {
  try {
    await testNotification();
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
