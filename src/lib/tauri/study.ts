/**
 * Прогон по колоде: какая карточка на экране, ответы и итоги.
 */
import { invoke } from "@tauri-apps/api/core";

export type Grade = "again" | "hard" | "good" | "easy";

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
  mode: string;
  total: number;
  /** Номер карточки на экране, начиная с единицы. */
  position: number;
  answered: number;
  revealed: boolean;
  /** `null`, когда прогон закончен. */
  card: StudyCard | null;
  finished: boolean;
};

export type StudySummary = {
  deck_id: string;
  deck_name: string;
  answered: number;
  correct: number;
  accuracy_percent: number;
  total_ms: number;
  average_ms: number;
  /** Идентификаторы ошибочных карточек, по порядку. */
  mistakes: string[];
  /** Они же целиком, чтобы показать разбор. */
  mistake_cards: StudyCard[];
};

export function studyStart(deckId: string): Promise<StudyView> {
  return invoke<StudyView>("study_start", { deckId });
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

export function studySummary(): Promise<StudySummary> {
  return invoke<StudySummary>("study_summary");
}

export function studyRepeatMistakes(): Promise<StudyView> {
  return invoke<StudyView>("study_repeat_mistakes");
}

export function studyStop(): Promise<void> {
  return invoke<void>("study_stop");
}
