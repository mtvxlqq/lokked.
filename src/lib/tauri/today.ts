/**
 * Сводка за учебный день для главного экрана.
 */

import { invoke } from "@tauri-apps/api/core";

export type TodayTotals = {
  /** Учебный день, за который посчитаны суммы, `'YYYY-MM-DD'`. */
  day_key: string;
  /** `[id предмета, секунды]` — только для предметов с временем за сегодня. */
  seconds_by_subject: [string, number][];
  /** Всё время учёбы за день, без перерывов. */
  total_seconds: number;
  /** Доведённые до конца рабочие фазы помодоро. */
  pomodoros: number;
  /** Дней подряд с достаточным временем учёбы. */
  streak_days: number;
  /** Когда сменится учебный день, ISO-8601 в UTC. */
  next_boundary: string;
};

export function todayTotals(): Promise<TodayTotals> {
  return invoke<TodayTotals>("today_totals");
}
