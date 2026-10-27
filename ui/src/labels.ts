import type { ColorMode, Connectivity, DocFormat, Screen, Source } from "./types";

export const SCREEN_TITLES: Record<Screen, string> = {
  scan: "Скан",
  overview: "Обзор",
};

export const CHIP_LABELS: Record<Connectivity, string> = {
  checking: "Проверяем…",
  online: "На связи",
  offline: "Нет связи",
  scanning: "Сканируем",
};

export function updatedSecondsAgo(seconds: number): string {
  return `обновлено ${seconds} с назад`;
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

export const TEXT = {
  requestFailed: "не удалось обратиться к устройству",
  deviceUnavailable: "устройство недоступно — проверьте питание и сеть",
  creatingJob: "создаём задание…",
  checkingDevice: "проверяем устройство…",
  jobCreated: "задание создано…",
  scanning: "сканирует…",
  done: "готово",
  canceled: "отменено",
  ready: "готов к сканированию",
  formatHint: "ч/б — только PDF",
  cancel: "Отмена",
  canceling: "Отмена…",
  unreported: "—",
  refresh: "Обновить",
  retry: "Повторить",
  esclUnavailable: "сканирование недоступно",
  connectionTitle: "Подключение",
  consumablesTitle: "Расходные материалы",
  countersTitle: "Счётчики",
  actionsTitle: "Действия",
  tonerLabel: "Тонер",
  drumLabel: "Барабан",
  pagesLabel: "Напечатано страниц",
  duplexLabel: "Дуплекс",
  drumLeftLabel: "Ресурс барабана",
  scanAction: "Скан",
  webPanelAction: "Веб-панель",
};
