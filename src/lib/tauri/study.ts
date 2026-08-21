/**
 * Прогон по колоде: какая карточка на экране, ответы и итоги.
 */
import { invoke } from "@tauri-apps/api/core";

export type Grade = "again" | "hard" | "good" | "easy";

/** Режим прогона: обычный, на время, вся колода, только слабые, барабан. */
export type StudyMode = "classic" | "blitz" | "marathon" | "weak" | "reel";

export type StudyCard = {
  id: string;
  front: string;
  /** `null`, пока ответ не раскрыт: бэкенд его не отдаёт. */
  back: string | null;
  hint: string | null;
  tags: string[];
};

export type StudyView = {
  deck_id: string;
  deck_name: string;
  mode: StudyMode;
  total: number;
  /** Номер карточки на экране, начиная с единицы. */
  position: number;
  answered: number;
  revealed: boolean;
  /** `null`, когда прогон закончен. */
  card: StudyCard | null;
  finished: boolean;
  /** Когда истекает время карточки, ISO-8601. Только блиц. */
  deadline: string | null;
  /** Сколько всего секунд даётся на карточку. Только блиц. */
  seconds_per_card: number | null;
  /** Очки и текущая серия. Только блиц. */
  points: number | null;
  streak: number | null;
};

export type StudySummary = {
  deck_id: string;
  deck_name: string;
  mode: StudyMode;
  answered: number;
  correct: number;
  accuracy_percent: number;
  total_ms: number;
  average_ms: number;
  /** Идентификаторы ошибочных карточек, по порядку. */
  mistakes: string[];
  /** Они же целиком, чтобы показать разбор. */
  mistake_cards: StudyCard[];
  /** Очки прогона, лучшая серия и рекорд колоды. Только блиц. */
  points: number | null;
  best_streak: number | null;
  record: number | null;
  record_beaten: boolean;
};

export function studyStart(
  deckId: string,
  mode: StudyMode,
): Promise<StudyView> {
  return invoke<StudyView>("study_start", { deckId, mode });
}

export function studyCurrent(): Promise<StudyView | null> {
  return invoke<StudyView | null>("study_current");
}

export function studyReveal(): Promise<StudyView> {
  return invoke<StudyView>("study_reveal");
}

export function studyAnswer(grade: Grade): Promise<StudyView> {
  return invoke<StudyView>("study_answer", { grade });
}

/** Время карточки вышло: то же, что ответить «не помню». */
export function studyTimeout(): Promise<StudyView> {
  return invoke<StudyView>("study_timeout");
}

export function studySummary(): Promise<StudySummary> {
  return invoke<StudySummary>("study_summary");
}

export function studyRepeatMistakes(): Promise<StudyView> {
  return invoke<StudyView>("study_repeat_mistakes");
}

export function studyStop(): Promise<void> {
  return invoke<void>("study_stop");
}
