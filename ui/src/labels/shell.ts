import type { Connectivity, Screen } from "../types";

export const SCREEN_TITLES: Record<Screen, string> = {
  scan: "Скан",
  overview: "Обзор",
  print: "Печать",
  settings: "Настройки",
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
