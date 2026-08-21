/**
 * Настройки: начало учебного дня и вид чёрного экрана.
 */

import { invoke } from "@tauri-apps/api/core";

export type DaySettings = {
  /** Смещение начала учебного дня от местной полуночи, в секундах. */
  start_offset_seconds: number;
};

export function daySettings(): Promise<DaySettings> {
  return invoke<DaySettings>("day_settings");
}

export function saveDaySettings(
  startOffsetSeconds: number,
): Promise<DaySettings> {
  return invoke<DaySettings>("set_day_settings", { startOffsetSeconds });
}

/** Шаг размера цифр на чёрном экране. */
export type ZenFontSize = "small" | "normal" | "large";

export type ZenSettings = {
  /** Показывать «1:12» вместо «1:12:24». */
  minutes_only: boolean;
  font_size: ZenFontSize;
};

export function zenSettings(): Promise<ZenSettings> {
  return invoke<ZenSettings>("zen_settings");
}

export function saveZenSettings(settings: ZenSettings): Promise<ZenSettings> {
  return invoke<ZenSettings>("set_zen_settings", {
    minutesOnly: settings.minutes_only,
    fontSize: settings.font_size,
  });
}
