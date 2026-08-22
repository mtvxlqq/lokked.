import { useState } from "react";

import { ReelSpinner, SPIN_MS } from "@/components/study/ReelSpinner";
import { Button } from "@/components/ui";
import type { Deck } from "@/lib/tauri";

type DeckReelProps = {
  decks: Deck[];
  /** Колода, на которой барабан остановится. Выбрал её бэкенд. */
  picked: Deck;
  onDone: () => void;
};

/**
 * Барабан выбора колоды: мимо проносятся названия, одно выпадает.
 *
 * Это первый барабан дуэли — тот самый выбор «рандом» перед боем. Второй
 * крутится уже внутри хода, на каждой карточке.
 */
export function DeckReel({ decks, picked, onDone }: DeckReelProps) {
  const [spinning, setSpinning] = useState(true);

  return (
    <div className="flex min-h-dvh flex-col items-center justify-center gap-10 bg-bg-zen px-4 py-8">
      <p className="text-11 tracking-label text-text-zen-dim-2 uppercase">
        Барабан выбирает колоду
      </p>

      <ReelSpinner
        labels={decks.map((deck) => deck.name)}
        target={picked.name}
        spinKey={picked.id}
        onSettled={() => setSpinning(false)}
      />

      <div className="flex min-h-11 flex-col items-center gap-3">
        {!spinning && (
          <>
            <p className="text-13 text-text-zen-dim">
              {picked.card_count} карточек в колоде
            </p>
            <Button onClick={onDone}>Дальше</Button>
          </>
        )}
      </div>
    </div>
  );
}

/** Сколько крутится барабан колоды — столько же, сколько и карточный. */
export const DECK_SPIN_MS = SPIN_MS;
