/**
 * Статистика: время по предметам, точность карточек и история одной карточки.
 */

import { invoke } from "@tauri-apps/api/core";
import type { Grade } from "@/lib/tauri/study";

/** Период, за который показаны цифры. */
export type StatsRange = "day" | "week" | "month" | "all";

/** Общее для всех вкладок: какие дни попали в период. */
type Period = {
  range: StatsRange;
  /** Первый день периода, `'YYYY-MM-DD'`. */
  from: string;
  /** Последний — всегда сегодняшний учебный день. */
  to: string;
};

export type SubjectTotal = {
  subject_id: string;
  seconds: number;
  /** Доля от самого длинного столбика, 0..100 — длина бара. */
  share_percent: number;
};

export type HeatCell = {
  day_key: string;
  seconds: number;
  /** 0 — не занимался, дальше 1..4 по нарастающей. */
  level: number;
  /** Понедельник — 0. Строка, в которую попадает клетка. */
  weekday: number;
};

export type TimeStats = Period & {
  /** Всё время учёбы за период, без перерывов. */
  total_seconds: number;
  pomodoros: number;
  /** Дней подряд — считается от сегодня и не зависит от периода. */
  streak_days: number;
  subjects: SubjectTotal[];
  heatmap: HeatCell[];
};

export type DayAccuracy = {
  day_key: string;
  answered: number;
  correct: number;
  /** Ноль в день без ответов: точности у такого дня нет. */
  accuracy_percent: number;
};

export type ProblemCard = {
  card_id: string;
  shown: number;
  correct: number;
  accuracy_percent: number;
  front: string;
  deck_id: string;
};

export type CardsStats = Period & {
  answered: number;
  correct: number;
  accuracy_percent: number;
  by_day: DayAccuracy[];
  problems: ProblemCard[];
};

export type CardReport = {
  card_id: string;
  deck_id: string;
  front: string;
  back: string;
  shown: number;
  correct: number;
  accuracy_percent: number;
  /** Последние десять ответов, самый старый из них первым. */
  recent: Grade[];
  /** Среднее время припоминания, мс; `null`, если его ни разу не мерили. */
  average_think_ms: number | null;
  current_streak: number;
};

export function statsTime(range: StatsRange): Promise<TimeStats> {
  return invoke<TimeStats>("stats_time", { range });
}

export function statsCards(range: StatsRange): Promise<CardsStats> {
  return invoke<CardsStats>("stats_cards", { range });
}

export function statsCard(cardId: string): Promise<CardReport> {
  return invoke<CardReport>("stats_card", { cardId });
}

/** Период таблицей: строка на день. */
export function statsExportCsv(range: StatsRange): Promise<string> {
  return invoke<string>("stats_export_csv", { range });
}
