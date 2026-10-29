import { SUPPORTED, type LocaleTag } from "./catalog";

export function matchTag(candidate: string): LocaleTag | null {
  const normalized = candidate.trim().toLowerCase().replace(/_/g, "-");
  const exact = SUPPORTED.find((tag) => tag.toLowerCase() === normalized);
  if (exact !== undefined) {
    return exact;
  }
  const base = normalized.split("-")[0] ?? normalized;
  const byBase = SUPPORTED.find(
    (tag) => (tag.split("-")[0] ?? tag) === base,
  );
  return byBase ?? null;
}

export function detectLocale(hint: string | null): LocaleTag {
  if (hint !== null && hint.trim() !== "") {
    const matched = matchTag(hint);
    if (matched !== null) {
      return matched;
    }
  }
  for (const language of navigator.languages) {
    const matched = matchTag(language);
    if (matched !== null) {
      return matched;
    }
  }
  return "en";
}
