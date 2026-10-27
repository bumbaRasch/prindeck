import type {
  ColorMode,
  Connectivity,
  DocFormat,
  Screen,
  Sides,
  Source,
} from "./types";

export const SCREEN_TITLES: Record<Screen, string> = {
  scan: "Скан",
  overview: "Обзор",
  print: "Печать",
  settings: "Настройки",
};

export const SIDES_LABELS: Record<Sides, string> = {
  "one-sided": "односторонняя",
  "two-sided-long-edge": "по длинному краю",
  "two-sided-short-edge": "по короткому краю",
};

export function formatSizeKb(sizeKb: number): string {
  if (sizeKb < 1024) {
    return `${sizeKb} КБ`;
  }
  return `${(sizeKb / 1024).toFixed(1)} МБ`;
}

export const CHIP_LABELS: Record<Connectivity, string> = {
  checking: "Проверяем…",
  online: "На связи",
  offline: "Нет связи",
  scanning: "Сканируем",
};

export function updatedSecondsAgo(seconds: number): string {
  return `обновлено ${seconds} с назад`;
}

export function jamCountText(count: number): string {
  return count === 0
    ? "замятия: нет"
    : `замятия: ${count} — проверьте тракт бумаги`;
}

export function drumPagesLeftText(pages: number): string {
  return `барабан: осталось ≈${pages} стр.`;
}

export function tonerPagesLeftText(pages: number): string {
  return `≈ ${pages} стр.`;
}

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
  stateTitle: "Состояние",
  consumablesTitle: "Расходные материалы",
  countersTitle: "Счётчики",
  actionsTitle: "Действия",
  tonerLabel: "Тонер",
  drumLabel: "Барабан",
  pagesLabel: "Напечатано страниц",
  duplexLabel: "Дуплекс",
  drumLeftLabel: "Ресурс барабана",
  tonerYieldHint: "укажите ресурс картриджа в настройках",
  historyTitle: "История",
  historyEmpty: "данных пока нет — копим",
  historyTonerLine: "Тонер, %",
  historyPagesLine: "Напечатано",
  historyChartLabel: "график тонера и счётчика страниц за 90 дней",
  scanAction: "Скан",
  webPanelAction: "Веб-панель",
  firmwareAction: "Обновление прошивки",
  firmwareHint: "страница прошивки в веб-панели устройства",
  chooseFile: "Выбрать файл…",
  dropzoneTitle: "Перетащите файл для печати",
  dropzoneHint: "PDF, изображение или другой документ",
  copiesLabel: "Копии",
  sidesLabel: "Стороны",
  printAction: "Печать",
  printSending: "Печать…",
  printReady: "готов к печати",
  printPickFile: "выберите файл для печати",
  sendingPrint: "отправляем на печать…",
  cancelingPrint: "отменяем задание…",
  queueTitle: "Очередь печати",
  queueEmpty: "Очередь пуста",
  queueLoading: "Обновляем…",
  queueJobLabel: "Задание",
  queueOwnerLabel: "Владелец",
  queueSizeLabel: "Размер",
  queueStateLabel: "Состояние",
  jobPendingLabel: "в ожидании",
  cancelJob: "Отменить",
  cancelingJob: "Отменяем…",
};
