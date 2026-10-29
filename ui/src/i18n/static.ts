import { en } from "../locales/en";
import type { MessageKey } from "../locales/en";
import { t } from "./runtime";

type StaticHook = {
  attribute: "data-i18n" | "data-i18n-title" | "data-i18n-aria";
  target: "textContent" | "title" | "aria-label";
};

const STATIC_HOOKS: readonly StaticHook[] = [
  { attribute: "data-i18n", target: "textContent" },
  { attribute: "data-i18n-title", target: "title" },
  { attribute: "data-i18n-aria", target: "aria-label" },
];

function applyHook(hook: StaticHook): void {
  const nodes = document.querySelectorAll<HTMLElement>(`[${hook.attribute}]`);
  for (const node of nodes) {
    const key = node.getAttribute(hook.attribute);
    if (key === null || !(key in en)) {
      continue;
    }
    if (hook.target === "textContent") {
      node.textContent = t(key as MessageKey);
    } else {
      node.setAttribute(hook.target, t(key as MessageKey));
    }
  }
}

export function applyStaticTexts(): void {
  for (const hook of STATIC_HOOKS) {
    applyHook(hook);
  }
}
