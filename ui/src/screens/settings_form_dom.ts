import { mustQuery } from "../dom";
import { t } from "../i18n/runtime";
import type {
  SettingsField,
  SettingsFieldError,
} from "./settings_values";

const TEMPLATE = `
  <form id="settings-form" class="panel" novalidate>
    <h2 class="settings-title" data-i18n="settings.formTitle"></h2>
    <div class="fields">
      <label class="field"><span class="field-label" data-i18n="settings.hostLabel"></span>
        <input id="settings-host" class="input" type="text" autocomplete="off" spellcheck="false" />
        <span class="field-hint" data-i18n="settings.hostHint"></span>
        <p class="field-error" data-error="host" hidden></p>
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.pollSecondsLabel"></span>
        <input id="settings-poll" class="input" type="number" min="60" max="3600" step="1" inputmode="numeric" />
        <p class="field-error" data-error="pollSeconds" hidden></p>
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.scanDirLabel"></span>
        <input id="settings-scan-dir" class="input" type="text" autocomplete="off" spellcheck="false" />
        <p class="field-error" data-error="scanDir" hidden></p>
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.tonerModelLabel"></span>
        <input id="settings-toner-model" class="input" type="text" autocomplete="off" spellcheck="false" />
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.tonerYieldLabel"></span>
        <input id="settings-toner-yield" class="input" type="number" min="1" max="100000" step="1" inputmode="numeric" />
        <p class="field-error" data-error="tonerYieldPages" hidden></p>
      </label>
      <label class="field checkbox-field">
        <input id="settings-notify" class="checkbox" type="checkbox" />
        <span class="checkbox-label" data-i18n="settings.notifyEnabledLabel"></span>
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.tonerThresholdLabel"></span>
        <input id="settings-toner-percent" class="input" type="number" min="0" max="100" step="1" inputmode="numeric" />
        <p class="field-error" data-error="tonerPercent" hidden></p>
      </label>
      <label class="field"><span class="field-label" data-i18n="settings.drumThresholdLabel"></span>
        <input id="settings-drum-percent" class="input" type="number" min="0" max="100" step="1" inputmode="numeric" />
        <p class="field-error" data-error="drumPercent" hidden></p>
      </label>
      <label class="field checkbox-field">
        <input id="settings-autostart" class="checkbox" type="checkbox" />
        <span class="checkbox-text">
          <span class="checkbox-label" data-i18n="settings.autostartLabel"></span>
          <span class="field-hint" data-i18n="settings.autostartHint"></span>
        </span>
      </label>
    </div>
    <p id="settings-status" class="status" role="status" aria-live="polite" data-tone="idle"></p>
    <div class="actions">
      <button id="settings-save" class="button button--primary" type="submit" disabled data-i18n="settings.save"></button>
      <button id="settings-test-notify" class="button button--secondary" type="button" disabled data-i18n="settings.testNotification"></button>
    </div>
  </form>
`;

mustQuery<HTMLElement>("#settings-form-mount").innerHTML = TEMPLATE;

export const els = {
  form: mustQuery<HTMLFormElement>("#settings-form"),
  save: mustQuery<HTMLButtonElement>("#settings-save"),
  test: mustQuery<HTMLButtonElement>("#settings-test-notify"),
  status: mustQuery<HTMLParagraphElement>("#settings-status"),
};

export const inputs = {
  host: mustQuery<HTMLInputElement>("#settings-host"),
  poll: mustQuery<HTMLInputElement>("#settings-poll"),
  scanDir: mustQuery<HTMLInputElement>("#settings-scan-dir"),
  tonerModel: mustQuery<HTMLInputElement>("#settings-toner-model"),
  tonerYield: mustQuery<HTMLInputElement>("#settings-toner-yield"),
  notify: mustQuery<HTMLInputElement>("#settings-notify"),
  tonerPercent: mustQuery<HTMLInputElement>("#settings-toner-percent"),
  drumPercent: mustQuery<HTMLInputElement>("#settings-drum-percent"),
  autostart: mustQuery<HTMLInputElement>("#settings-autostart"),
};

const errorEls = new Map<SettingsField, HTMLElement>(
  Array.from(document.querySelectorAll<HTMLElement>("[data-error]")).map(
    (node) => [node.dataset["error"] as SettingsField, node],
  ),
);

export function renderFieldErrors(errors: readonly SettingsFieldError[]): void {
  const byField = new Map(errors.map((error) => [error.field, t(error.key)]));
  for (const [field, node] of errorEls) {
    const message = byField.get(field);
    node.hidden = message === undefined;
    node.textContent = message ?? "";
  }
}
