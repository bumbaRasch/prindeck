import type { Catalog, LocaleTag } from "../i18n/catalog";
import { ar } from "./ar";
import { de } from "./de";
import { en } from "./en";
import { es } from "./es";
import { fr } from "./fr";
import { ja } from "./ja";
import { ptBR } from "./pt_br";
import { ru } from "./ru";
import { uk } from "./uk";
import { zhCN } from "./zh_cn";

export const CATALOGS: Record<LocaleTag, Catalog> = {
  en,
  ru,
  uk,
  de,
  fr,
  es,
  "pt-BR": ptBR,
  "zh-CN": zhCN,
  ja,
  ar,
};
