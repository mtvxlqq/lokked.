import { useEffect, useState } from "react";

import { CardHistory } from "@/components/stats/CardHistory";
import { LoadFrame } from "@/components/stats/LoadFrame";
import { useStatsData } from "@/components/stats/useStatsData";
import { EmptyState, Select } from "@/components/ui";
import { CardsIcon } from "@/components/nav/icons";
import {
  errorMessage,
  listCards,
  listDecks,
  statsCard,
  type Card,
  type Deck,
} from "@/lib/tauri";

/** Какая карточка сейчас разбирается. */
export type CardSelection = {
  deckId: string;
  /** `null`, пока в колоде не выбрана карточка — например, если она пуста. */
  cardId: string | null;
};

/** Отчёт по карточке; пустой выбор — пустой ответ, без обращения к команде. */
function load(cardId: string) {
  return cardId ? statsCard(cardId) : Promise.resolve(null);
}

type CardTabProps = {
  selection: CardSelection | null;
  onSelect: (selection: CardSelection) => void;
};

/**
 * Вкладка «Карточка»: колода, карточка и её история.
 *
 * Выбор живёт в экране, а не здесь: на эту вкладку попадают ещё и из списка
 * проблемных карточек, и тогда она открывается сразу на нужной.
 */
export function CardTab({ selection, onSelect }: CardTabProps) {
  const [decks, setDecks] = useState<Deck[]>([]);
  const [cards, setCards] = useState<Card[]>([]);
  const [error, setError] = useState<string | null>(null);

  const deckId = selection?.deckId ?? decks[0]?.id ?? null;
  const cardId = selection?.cardId ?? null;

  useEffect(() => {
    let cancelled = false;

    listDecks()
      .then((loaded) => {
        if (!cancelled) setDecks(loaded);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  // Карточки выбранной колоды — и сразу первая из них, если выбирать ещё
  // нечего: пустой список с пустым отчётом рядом выглядел бы поломкой.
  useEffect(() => {
    if (!deckId) return;
    let cancelled = false;

    listCards(deckId)
      .then((loaded) => {
        if (cancelled) return;

        setCards(loaded);
        const known = loaded.some((card) => card.id === cardId);
        if (!known) {
          onSelect({ deckId, cardId: loaded[0]?.id ?? null });
        }
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
    // `onSelect` и `cardId` намеренно вне зависимостей: список карточек
    // зависит только от колоды, а перезапрашивать его на каждый выбор
    // карточки незачем.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [deckId]);

  const report = useStatsData(load, cardId ?? "");

  if (error) {
    return (
      <p className="text-14 text-danger-text" role="alert">
        {error}
      </p>
    );
  }

  if (decks.length === 0) {
    return (
      <EmptyState
        icon={<CardsIcon className="size-8" />}
        title="Колод пока нет"
        description="Создай колоду и пройди её — здесь появится история каждой карточки."
      />
    );
  }

  return (
    <div className="flex flex-col gap-5 sm:gap-6">
      <div className="grid gap-4 sm:grid-cols-2">
        <Select
          label="Колода"
          value={deckId ?? ""}
          onChange={(event) =>
            onSelect({ deckId: event.target.value, cardId: null })
          }
        >
          {decks.map((deck) => (
            <option key={deck.id} value={deck.id}>
              {deck.name}
            </option>
          ))}
        </Select>

        <Select
          label="Карточка"
          value={cardId ?? ""}
          disabled={cards.length === 0}
          onChange={(event) =>
            deckId && onSelect({ deckId, cardId: event.target.value })
          }
        >
          {cards.length === 0 && (
            <option value="">В колоде нет карточек</option>
          )}
          {cards.map((card) => (
            <option key={card.id} value={card.id}>
              {card.front}
            </option>
          ))}
        </Select>
      </div>

      {cardId === null ? (
        <EmptyState
          icon={<CardsIcon className="size-8" />}
          title="В этой колоде нет карточек"
          description="Добавь карточки на экране «Карточки» — тогда их будет что разбирать."
        />
      ) : (
        <LoadFrame
          state={report.state}
          error={report.error}
          onRetry={report.reload}
        >
          {report.data && <CardHistory report={report.data} />}
        </LoadFrame>
      )}
    </div>
  );
}
