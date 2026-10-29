import type { MessageKey } from "../locales/en";
export type { MessageKey } from "../locales/en";

export type LocaleTag =
  | "en"
  | "ru"
  | "uk"
  | "de"
  | "fr"
  | "es"
  | "pt-BR"
  | "zh-CN"
  | "ja"
  | "ar";

export type PluralForms = { other: string } & Partial<
  Record<"zero" | "one" | "two" | "few" | "many", string>
>;

export type Message = string | PluralForms;

export type Catalog = { [K in MessageKey]: Message };

export const SUPPORTED: readonly LocaleTag[] = [
  "en",
  "ru",
  "uk",
  "de",
  "fr",
  "es",
  "pt-BR",
  "zh-CN",
  "ja",
  "ar",
];

const RTL_LANGUAGES: ReadonlySet<string> = new Set(["ar", "he", "fa", "ur"]);

export function direction(tag: LocaleTag): "ltr" | "rtl" {
  return RTL_LANGUAGES.has(tag.split("-")[0] ?? tag) ? "rtl" : "ltr";
}
