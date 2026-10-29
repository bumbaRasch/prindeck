import { invoke } from "@tauri-apps/api/core";
import { t } from "./i18n/runtime";
import type { MessageKey } from "./locales/en";
import type {
  SettingsField,
  SettingsFieldError,
} from "./screens/settings_values";
import type { Config } from "./types";

export type ConfigErrorField =
  | "host"
  | "pollSeconds"
  | "scanDir"
  | "notifyTonerPercent"
  | "notifyDrumPercent"
  | "tonerYieldPages"
  | "cupsQueue";

export type ConfigErrorKind = "required" | "format" | "range" | "io";

export interface ConfigFieldError {
  field: ConfigErrorField | null;
  kind: ConfigErrorKind;
}

type FormErrorField = Exclude<ConfigErrorField, "cupsQueue">;

const FORM_ERROR_FIELDS: readonly FormErrorField[] = [
  "host",
  "pollSeconds",
  "scanDir",
  "notifyTonerPercent",
  "notifyDrumPercent",
  "tonerYieldPages",
];

const SETTINGS_FORM_FIELDS: Record<FormErrorField, SettingsField> = {
  host: "host",
  pollSeconds: "pollSeconds",
  scanDir: "scanDir",
  notifyTonerPercent: "tonerPercent",
  notifyDrumPercent: "drumPercent",
  tonerYieldPages: "tonerYieldPages",
};

const SETTINGS_ERROR_KEYS: Record<
  FormErrorField,
  Record<ConfigErrorKind, MessageKey>
> = {
  host: {
    required: "settings.error.hostRequired",
    format: "settings.error.hostFormat",
    range: "settings.error.hostFormat",
    io: "settings.error.io",
  },
  pollSeconds: {
    required: "settings.error.pollRange",
    format: "settings.error.pollRange",
    range: "settings.error.pollRange",
    io: "settings.error.io",
  },
  scanDir: {
    required: "settings.error.scanDirRequired",
    format: "settings.error.scanDirFormat",
    range: "settings.error.scanDirFormat",
    io: "settings.error.io",
  },
  notifyTonerPercent: {
    required: "settings.error.percentRange",
    format: "settings.error.percentRange",
    range: "settings.error.percentRange",
    io: "settings.error.io",
  },
  notifyDrumPercent: {
    required: "settings.error.percentRange",
    format: "settings.error.percentRange",
    range: "settings.error.percentRange",
    io: "settings.error.io",
  },
  tonerYieldPages: {
    required: "settings.error.yieldRange",
    format: "settings.error.yieldRange",
    range: "settings.error.yieldRange",
    io: "settings.error.io",
  },
};

function isFormErrorField(value: string): value is FormErrorField {
  return FORM_ERROR_FIELDS.includes(value as FormErrorField);
}

export function settingsErrorKey(
  field: ConfigErrorField | null,
  kind: ConfigErrorKind,
): SettingsFieldError | null {
  if (field === null || !isFormErrorField(field)) {
    return null;
  }
  const formField = SETTINGS_FORM_FIELDS[field];
  const key = SETTINGS_ERROR_KEYS[field]?.[kind];
  if (formField === undefined || key === undefined) {
    return null;
  }
  return { field: formField, key };
}

function isConfigErrorKind(value: unknown): value is ConfigErrorKind {
  return (
    value === "required" ||
    value === "format" ||
    value === "range" ||
    value === "io"
  );
}

function isConfigErrorField(value: unknown): value is ConfigErrorField {
  return (
    typeof value === "string" && (value === "cupsQueue" || isFormErrorField(value))
  );
}

export function configFieldError(error: unknown): ConfigFieldError | null {
  if (
    typeof error === "object" &&
    error !== null &&
    "field" in error &&
    "kind" in error
  ) {
    const { field, kind } = error as { field: unknown; kind: unknown };
    if (
      (field === null || isConfigErrorField(field)) &&
      isConfigErrorKind(kind)
    ) {
      return { field, kind };
    }
  }
  return null;
}

export async function setConfig(config: Config): Promise<Config> {
  try {
    return await invoke<Config>("set_config", { config });
  } catch (error) {
    throw configFieldError(error) ?? error;
  }
}

export type StartupProblemKind = "loadFailed" | "invalid" | "fallback";

export interface StartupProblem {
  kind: StartupProblemKind;
  field: ConfigErrorField | null;
  detail: string;
}

const STARTUP_KEYS: Record<StartupProblemKind, MessageKey> = {
  loadFailed: "startup.loadFailed",
  invalid: "startup.invalid",
  fallback: "startup.fallback",
};

const STARTUP_FIELD_KEYS: Record<FormErrorField, MessageKey> = {
  host: "settings.error.hostFormat",
  pollSeconds: "settings.error.pollRange",
  scanDir: "settings.error.scanDirFormat",
  notifyTonerPercent: "settings.error.percentRange",
  notifyDrumPercent: "settings.error.percentRange",
  tonerYieldPages: "settings.error.yieldRange",
};

function startupKey(problem: StartupProblem): MessageKey {
  if (
    problem.kind === "invalid" &&
    problem.field !== null &&
    isFormErrorField(problem.field)
  ) {
    const fieldKey = STARTUP_FIELD_KEYS[problem.field];
    if (fieldKey !== undefined) {
      return fieldKey;
    }
  }
  return STARTUP_KEYS[problem.kind] ?? "startup.invalid";
}

function startupLine(problem: StartupProblem): string {
  const text = t(startupKey(problem));
  return problem.detail === "" ? text : `${text} — ${problem.detail}`;
}

export async function getStartupError(): Promise<string | null> {
  const problems = await invoke<StartupProblem[] | null>("get_startup_error");
  return problems === null ? null : problems.map(startupLine).join("; ");
}
