/**
 * Колоды, карточки и массовый импорт с экспортом.
 */

import { invoke } from "@tauri-apps/api/core";

export type Deck = {
  id: string;
  subject_id: string | null;
  name: string;
  description: string | null;
  /** Сколько живых карточек в колоде. */
  card_count: number;
};

export type DeckInput = {
  subject_id: string | null;
  name: string;
  description: string | null;
};

export function listDecks(): Promise<Deck[]> {
  return invoke<Deck[]>("list_decks");
}

export function createDeck(input: DeckInput): Promise<Deck> {
  return invoke<Deck>("create_deck", { input });
}

export function updateDeck(id: string, input: DeckInput): Promise<Deck> {
  return invoke<Deck>("update_deck", { id, input });
}

export function deleteDeck(id: string): Promise<void> {
  return invoke<void>("delete_deck", { id });
}

export type Card = {
  id: string;
  deck_id: string;
  front: string;
  back: string;
  hint: string | null;
  tags: string[];
};

export type CardInput = {
  front: string;
  back: string;
  hint: string | null;
  tags: string[];
};

export function listCards(deckId: string): Promise<Card[]> {
  return invoke<Card[]>("list_cards", { deckId });
}

export function createCard(deckId: string, input: CardInput): Promise<Card> {
  return invoke<Card>("create_card", { deckId, input });
}

export function updateCard(id: string, input: CardInput): Promise<Card> {
  return invoke<Card>("update_card", { id, input });
}

export function moveCard(id: string, deckId: string): Promise<Card> {
  return invoke<Card>("move_card", { id, deckId });
}

export function deleteCard(id: string): Promise<void> {
  return invoke<void>("delete_card", { id });
}

/** Карточка, разобранная из текста или файла, но ещё не записанная. */
export type ParsedCard = {
  front: string;
  back: string;
  hint: string | null;
  tags: string[];
};

export type ImportProblem = {
  /** Номер блока в исходном тексте, начиная с единицы. */
  block: number;
  kind: "missing_back" | "blank_side" | "too_many_sides";
  /** Сколько частей нашлось — только у `too_many_sides`. */
  found?: number;
};

export type ImportReport = {
  format: "text" | "lecture_json";
  cards: ParsedCard[];
  problems: ImportProblem[];
  /** Название колоды, если формат его знает. */
  suggested_deck: string | null;
  suggested_description: string | null;
};

export type Separators = {
  cardSeparator: string;
  sideSeparator: string;
};

export function previewImport(
  text: string,
  separators?: Separators,
): Promise<ImportReport> {
  return invoke<ImportReport>("preview_import", {
    text,
    cardSeparator: separators?.cardSeparator ?? null,
    sideSeparator: separators?.sideSeparator ?? null,
  });
}

export function importCards(
  deckId: string,
  cards: ParsedCard[],
): Promise<number> {
  return invoke<number>("import_cards", { deckId, cards });
}

export function exportDeck(
  deckId: string,
  separators?: Separators,
): Promise<string> {
  return invoke<string>("export_deck", {
    deckId,
    cardSeparator: separators?.cardSeparator ?? null,
    sideSeparator: separators?.sideSeparator ?? null,
  });
}
