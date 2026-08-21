import { CardText } from "@/components/cards/CardText";
import type { ProblemCard } from "@/lib/tauri";

type ProblemCardsProps = {
  cards: ProblemCard[];
  onOpen: (cardId: string) => void;
};

/**
 * Карточки с худшей точностью: та, что чаще всего не вспоминается, сверху.
 *
 * Строка целиком — кнопка: она открывает историю этой карточки, и попасть по
 * ней пальцем должно быть так же легко, как прочитать.
 */
export function ProblemCards({ cards, onOpen }: ProblemCardsProps) {
  return (
    <ul className="flex flex-col">
      {cards.map((card) => (
        <li
          key={card.card_id}
          className="border-b border-border last:border-b-0"
        >
          <button
            type="button"
            onClick={() => onOpen(card.card_id)}
            className="flex min-h-11 w-full flex-wrap items-center gap-x-4 gap-y-1.5 py-3 text-left transition-colors duration-150 ease-standard hover:bg-row focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
          >
            <CardText
              text={card.front}
              className="min-w-0 flex-1 text-14 text-text-2"
            />

            <span className="shrink-0 font-mono text-13 tabular-nums text-text-dim">
              {card.correct} из {card.shown}
            </span>
            <span className="w-12 shrink-0 text-right font-mono text-13 tabular-nums text-danger-text">
              {card.accuracy_percent}%
            </span>
          </button>
        </li>
      ))}
    </ul>
  );
}
