import type { ColorMode, DocFormat, Source } from "./types";

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
  requestFailed: "request to the device failed",
  deviceUnavailable: "device unavailable — check the power and the network",
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
};
