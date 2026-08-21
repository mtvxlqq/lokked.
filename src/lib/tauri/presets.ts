/**
 * Пресеты таймера: режим, длительности, «по умолчанию».
 */

import { invoke } from "@tauri-apps/api/core";

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
