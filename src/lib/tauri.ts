/**
 * Typed wrappers around the Tauri command layer.
 *
 * Every `invoke` call in the app goes through this module, so the Rust command
 * signatures in `src-tauri/src/commands/` are mirrored in exactly one place
 * and the rest of the frontend never touches stringly-typed command names.
 */
import { invoke } from "@tauri-apps/api/core";

/**
 * Health check for the Rust ↔ TypeScript bridge. Resolves to `"pong"`.
 */
export function ping(): Promise<string> {
  return invoke<string>("ping");
}

/** Как команда сообщает об отказе. Зеркало `commands::CommandError`. */
export type CommandError = {
  kind: "validation" | "not_found" | "database";
  message: string;
};

/**
 * Сообщение из отказа команды.
 *
 * `invoke` отклоняет промис тем, что вернул Rust, — это обычный объект, а не
 * `Error`, поэтому `catch (e) { e.message }` молча даёт `undefined`. Разбор
 * живёт здесь, чтобы каждый экран не переизобретал его заново.
 */
export function errorMessage(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const { message } = error as { message: unknown };
    if (typeof message === "string") return message;
  }
  if (error instanceof Error) return error.message;
  return String(error);
}

// ------------------------------------------------------------------ предметы

export type Subject = {
  id: string;
  name: string;
  /** Слаг палитры (`subject-1` … `subject-8`), не hex. */
  color: string | null;
  icon: string | null;
  position: number;
};

export type SubjectInput = {
  name: string;
  /** `null` при создании — цвет выберет бэкенд. */
  color: string | null;
  icon: string | null;
};

export function listSubjects(): Promise<Subject[]> {
  return invoke<Subject[]>("list_subjects");
}

export function createSubject(input: SubjectInput): Promise<Subject> {
  return invoke<Subject>("create_subject", { input });
}

export function updateSubject(
  id: string,
  input: SubjectInput,
): Promise<Subject> {
  return invoke<Subject>("update_subject", { id, input });
}

export function deleteSubject(id: string): Promise<void> {
  return invoke<void>("delete_subject", { id });
}

// ------------------------------------------------------------------- пресеты

export type PresetMode = "countup" | "countdown" | "pomodoro";

export type Preset = {
  id: string;
  /** `null` — глобальный пресет, доступный всем предметам. */
  subject_id: string | null;
  name: string;
  mode: PresetMode;
  work_seconds: number;
  break_seconds: number | null;
  long_break_seconds: number | null;
  cycles_before_long: number | null;
  auto_start_next: boolean;
  is_default: boolean;
};

export type PresetInput = Omit<Preset, "id">;

export function listPresets(): Promise<Preset[]> {
  return invoke<Preset[]>("list_presets");
}

export function createPreset(input: PresetInput): Promise<Preset> {
  return invoke<Preset>("create_preset", { input });
}

export function updatePreset(id: string, input: PresetInput): Promise<Preset> {
  return invoke<Preset>("update_preset", { id, input });
}

export function deletePreset(id: string): Promise<void> {
  return invoke<void>("delete_preset", { id });
}

// ------------------------------------------------------------------- сегодня

export type TodayTotals = {
  /** Учебный день, за который посчитаны суммы, `'YYYY-MM-DD'`. */
  day_key: string;
  /** `[id предмета, секунды]` — только для предметов с временем за сегодня. */
  seconds_by_subject: [string, number][];
};

export function todayTotals(): Promise<TodayTotals> {
  return invoke<TodayTotals>("today_totals");
}
