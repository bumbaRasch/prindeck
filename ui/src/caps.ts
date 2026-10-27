import type { OptionSpec } from "./dom";
import { COLOR_LABELS, FORMAT_LABELS, SOURCE_LABELS } from "./labels";
import type { AppState } from "./state";
import type { ColorMode, DocFormat, InputCaps, ScanCaps, Source } from "./types";

const DPI_PREFERENCE = [300, 200, 600, 100];
const COLOR_PREFERENCE: ColorMode[] = [
  "Grayscale8",
  "RGB24",
  "BlackAndWhite1",
];
const FORMAT_PREFERENCE: DocFormat[] = ["application/pdf", "image/jpeg"];

export function isSource(value: string): value is Source {
  return value === "Platen" || value === "Feeder";
}

export function isColorMode(value: string): value is ColorMode {
  return (
    value === "BlackAndWhite1" ||
    value === "Grayscale8" ||
    value === "RGB24"
  );
}

export function isDocFormat(value: string): value is DocFormat {
  return value === "application/pdf" || value === "image/jpeg";
}

export function pickFirst<T>(available: readonly T[], preferred: readonly T[]): T | null {
  for (const value of preferred) {
    if (available.includes(value)) {
      return value;
    }
  }
  return available[0] ?? null;
}

export function activeInput(caps: ScanCaps, source: Source | null): InputCaps | null {
  return source === "Feeder" ? caps.adf : caps.platen;
}

export function pickDpi(input: InputCaps, current: number | null): number | null {
  if (current !== null && input.resolutions.includes(current)) {
    return current;
  }
  return pickFirst(input.resolutions, DPI_PREFERENCE);
}

export function pickFormat(
  input: InputCaps,
  color: ColorMode | null,
  current: DocFormat | null,
): DocFormat | null {
  if (color === "BlackAndWhite1" && input.formats.includes("application/pdf")) {
    return "application/pdf";
  }
  if (current !== null && input.formats.includes(current)) {
    return current;
  }
  return pickFirst(input.formats, FORMAT_PREFERENCE);
}

export function normalizeSelections(state: AppState): void {
  const caps = state.caps;
  if (caps === null) {
    return;
  }

  if (state.source === null || (state.source === "Feeder" && caps.adf === null)) {
    state.source = "Platen";
  }

  const input = activeInput(caps, state.source);
  if (input === null) {
    return;
  }

  state.dpi = pickDpi(input, state.dpi);
  if (state.color === null || !input.colorModes.includes(state.color)) {
    state.color = pickFirst(input.colorModes, COLOR_PREFERENCE);
  }
  state.format = pickFormat(input, state.color, state.format);
}

export function sourceOptions(caps: ScanCaps | null): OptionSpec[] {
  if (caps === null) {
    return [];
  }
  const options: OptionSpec[] = [
    { value: "Platen", label: SOURCE_LABELS.Platen },
  ];
  if (caps.adf !== null) {
    options.push({ value: "Feeder", label: SOURCE_LABELS.Feeder });
  }
  return options;
}

export function dpiOptions(input: InputCaps): OptionSpec[] {
  return input.resolutions.map((dpi) => ({ value: String(dpi), label: `${dpi} dpi` }));
}

export function colorOptions(input: InputCaps): OptionSpec[] {
  return input.colorModes.map((mode) => ({
    value: mode,
    label: COLOR_LABELS[mode],
  }));
}

export function formatOptions(input: InputCaps, blackAndWhite: boolean): OptionSpec[] {
  return input.formats.map((format) => ({
    value: format,
    label: FORMAT_LABELS[format],
    disabled: blackAndWhite && format === "image/jpeg",
  }));
}
