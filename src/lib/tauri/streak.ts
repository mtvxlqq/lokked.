/**
 * Страница серии: дни подряд, рекорд, заморозки, календарь и вехи.
 */

import { invoke } from "@tauri-apps/api/core";

/** Что стало с одним днём. */
export type StreakDayState =
  /** Занимался достаточно — день в серии. */
  | "counted"
  /** Пропущен, но закрыт заморозкой: серия не оборвалась. */
  | "frozen"
  /** Пропущен, и закрыть было нечем. */
  | "missed"
  /** Сегодня: минимум ещё не набран, но день не кончился. */
  | "pending"
  /** День календаря, который ещё не наступил. */
  | "future";

export type StreakDay = {
  /** `YYYY-MM-DD`. */
  day: string;
  seconds: number;
  state: StreakDayState;
};

export type StreakMilestone = {
  /** Сколько дней подряд нужно. */
  target: number;
  reached: boolean;
  /** День, когда веха была взята, если была. */
  reached_on: string | null;
  /** Сколько дней осталось, или 0 у взятой. */
  remaining: number;
};

export type StreakMonth = {
  year: number;
  /** 1–12. */
  month: number;
  days: StreakDay[];
};

export type StreakView = {
  /** Учебный день по границе студента, `YYYY-MM-DD`. */
  today: string;
  today_seconds: number;
  min_seconds: number;
  /** Смещение границы дня от полуночи, в секундах. */
  day_start_seconds: number;
  current: number;
  longest: number;
  longest_from: string | null;
  longest_to: string | null;
  freezes: number;
  max_freezes: number;
  /** Сколько дней подряд приносят ещё одну заморозку. */
  freeze_every: number;
  /** Сколько заморозок потрачено внутри текущей серии. */
  frozen_days: number;
  milestones: StreakMilestone[];
  month: StreakMonth;
};

export function streakView(): Promise<StreakView> {
  return invoke<StreakView>("streak_view");
}

/**
 * Сохраняет картинку серии рядом с остальными изображениями и возвращает
 * путь, куда она легла. `png` — то, что вернул `canvas.toDataURL`.
 */
export function saveStreakImage(png: string): Promise<string> {
  return invoke<string>("streak_save_image", { png });
}

export type StreakSettings = {
  /** Сколько секунд учёбы засчитывают день в серию. */
  min_seconds: number;
};

export function streakSettings(): Promise<StreakSettings> {
  return invoke<StreakSettings>("streak_settings");
}

export function saveStreakSettings(
  minSeconds: number,
): Promise<StreakSettings> {
  return invoke<StreakSettings>("set_streak_settings", { minSeconds });
}
