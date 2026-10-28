import type { ColorMode, DocFormat, Source } from "../types";

export function pluralPages(count: number): string {
  const mod10 = count % 10;
  const mod100 = count % 100;
  if (mod10 === 1 && mod100 !== 11) {
    return PAGE_FORMS.one;
  }
  if (mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)) {
    return PAGE_FORMS.few;
  }
  return PAGE_FORMS.many;
}

export function receivedPages(count: number): string {
  return `Получено страниц: ${count}`;
}

export function elapsedSeconds(seconds: number): string {
  return `прошло ${seconds} с`;
}

export function pageEntry(page: number): string {
  return `Страница ${page}`;
}

export function partialPages(count: number): string {
  return `неполный: ${count} ${pluralPages(count)}`;
}

export function filesReady(count: number): string {
  return count === 1 ? "Файл готов" : "Файлы готовы";
}

export const SOURCE_LABELS: Record<Source, string> = {
  Platen: "Планшет",
  Feeder: "Автоподатчик",
};

export const COLOR_LABELS: Record<ColorMode, string> = {
  BlackAndWhite1: "Чёрно-белый",
  Grayscale8: "Оттенки серого",
  RGB24: "Цветной",
};

export const FORMAT_LABELS: Record<DocFormat, string> = {
  "application/pdf": "PDF",
  "image/jpeg": "JPEG",
};

export const PAGE_FORMS = {
  one: "страница",
  few: "страницы",
  many: "страниц",
};
