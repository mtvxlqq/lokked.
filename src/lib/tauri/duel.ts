/**
 * Дуэль: блиц на одном устройстве, по очереди, на одних и тех же карточках.
 */

import { invoke } from "@tauri-apps/api/core";

import type { Deck } from "@/lib/tauri/cards";
import type { Grade, StudyCard } from "@/lib/tauri/study";

export type DuelPlayer = {
  name: string;
  /** Владелец устройства — тот, чьи ответы идут в личную статистику. */
  is_owner: boolean;
  /** Ход уже сделан. Счёт до конца дуэли скрыт. */
  played: boolean;
};

export type DuelView = {
  duel_id: string;
  deck_id: string;
  deck_name: string;
  players: DuelPlayer[];
  /** Чей ход, 0-based, и его имя. */
  current_player: number;
  current_name: string;
  /** Номер хода с единицы и сколько их всего. */
  turn: number;
  turns: number;
  /** Карточек в ходе и где ход сейчас. */
  total: number;
  position: number;
  answered: number;
  revealed: boolean;
  /** `null` на экране передачи устройства и после конца дуэли. */
  card: StudyCard | null;
  deadline: string | null;
  seconds_per_card: number;
  /** Свой счёт текущего игрока. Чужих здесь нет — они скрыты до конца. */
  points: number;
  streak: number;
  /** Ждём, пока следующий игрок скажет, что готов. */
  handover: boolean;
  finished: boolean;
};

export type DuelResult = {
  name: string;
  is_owner: boolean;
  points: number;
  correct: number;
  answered: number;
  best_streak: number;
  winner: boolean;
};

export type DuelCard = {
  card_id: string;
  front: string;
  back: string;
  /** По записи на игрока в порядке ходов; `null` — не дошёл. */
  answers: (Grade | null)[];
};

export type DuelSummary = {
  duel_id: string;
  deck_id: string;
  deck_name: string;
  cards: number;
  seconds_per_card: number;
  players: DuelResult[];
  breakdown: DuelCard[];
};

/** Колода, которую выбрал барабан. Решает бэкенд, барабан только показывает. */
export function duelPickDeck(): Promise<Deck> {
  return invoke<Deck>("duel_pick_deck");
}

export function duelStart(options: {
  deckId: string;
  players: string[];
  cards: number;
  secondsPerCard: number;
}): Promise<DuelView> {
  return invoke<DuelView>("duel_start", {
    deckId: options.deckId,
    players: options.players,
    cards: options.cards,
    secondsPerCard: options.secondsPerCard,
  });
}

export function duelCurrent(): Promise<DuelView | null> {
  return invoke<DuelView | null>("duel_current");
}

/** Следующий игрок взял устройство: ход начинается. */
export function duelBeginTurn(): Promise<DuelView> {
  return invoke<DuelView>("duel_begin_turn");
}

/** Барабан встал — с этого мгновения идёт время карточки. */
export function duelSettled(): Promise<DuelView> {
  return invoke<DuelView>("duel_settled");
}

export function duelReveal(): Promise<DuelView> {
  return invoke<DuelView>("duel_reveal");
}

export function duelAnswer(grade: Grade): Promise<DuelView> {
  return invoke<DuelView>("duel_answer", { grade });
}

export function duelTimeout(): Promise<DuelView> {
  return invoke<DuelView>("duel_timeout");
}

export function duelSummary(): Promise<DuelSummary> {
  return invoke<DuelSummary>("duel_summary");
}

export function duelStop(): Promise<void> {
  return invoke<void>("duel_stop");
}
