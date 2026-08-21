import { useMemo, useState } from "react";

import { CardLine } from "@/components/cards/CardText";
import { Input } from "@/components/ui";
import { cn } from "@/lib/cn";
import { plainText } from "@/lib/markdown";
import type { Card } from "@/lib/tauri";

type CardTableProps = {
  cards: Card[];
  onEdit: (card: Card) => void;
};

/**
 * Карточки колоды с поиском и фильтром по тегам.
 *
 * Поиск и фильтр работают на клиенте: в колоде сотни карточек, они уже
 * загружены, и запрос на каждое нажатие клавиши сделал бы отклик хуже,
 * а не лучше.
 */
export function CardTable({ cards, onEdit }: CardTableProps) {
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState<string | null>(null);

  const tags = useMemo(() => {
    const seen = new Map<string, number>();
    for (const card of cards) {
      for (const value of card.tags) {
        seen.set(value, (seen.get(value) ?? 0) + 1);
      }
    }
    return [...seen.entries()].sort((a, b) => b[1] - a[1]);
  }, [cards]);

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return cards.filter((card) => {
      if (tag !== null && !card.tags.includes(tag)) return false;
      if (needle === "") return true;

      return (
        card.front.toLowerCase().includes(needle) ||
        card.back.toLowerCase().includes(needle) ||
        card.tags.some((value) => value.toLowerCase().includes(needle))
      );
    });
  }, [cards, query, tag]);

  return (
    <div className="flex flex-col gap-4">
      <Input
        label="Поиск"
        type="search"
        value={query}
        onChange={(event) => setQuery(event.target.value)}
        placeholder="По тексту карточки или тегу"
      />

      {tags.length > 0 && (
        <div className="flex flex-wrap gap-2">
          {tags.map(([value, count]) => {
            const active = tag === value;
            return (
              <button
                key={value}
                type="button"
                aria-pressed={active}
                onClick={() => setTag(active ? null : value)}
                className={cn(
                  "flex min-h-11 items-baseline rounded-full border px-3 py-1.5 text-12.5 sm:min-h-0 sm:py-1",
                  "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                  active
                    ? "border-border-accent bg-raised text-text-1"
                    : "border-border text-text-dim",
                )}
                // Имя задаётся явно: в теме лекции живёт формула, и без
                // этого имя чипа собиралось бы из разметки KaTeX.
                aria-label={`${plainText(value)}, карточек: ${count}`}
              >
                <CardLine text={value} />
                <span className="ml-1.5 text-text-faint">{count}</span>
              </button>
            );
          })}
        </div>
      )}

      <p className="text-12.5 text-text-dim-2">
        Показано {shown.length} из {cards.length}
      </p>

      <ul className="flex flex-col divide-y divide-border">
        {shown.map((card) => (
          <li key={card.id}>
            <button
              type="button"
              onClick={() => onEdit(card)}
              // Имя строки задаётся явно: иначе оно собиралось бы из разметки
              // формул, а вслух это не читается.
              aria-label={`Карточка: ${plainText(card.front)}`}
              className={cn(
                "flex min-h-11 w-full flex-col gap-1 py-3 text-left",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
              )}
            >
              <CardLine
                text={card.front}
                className="line-clamp-1 text-14.5 text-text-1"
              />
              <CardLine
                text={card.back}
                className="line-clamp-1 text-12.5 text-text-dim-2"
              />
              {card.tags.length > 0 && (
                <span className="flex flex-wrap gap-x-1.5 text-11.5 text-text-faint">
                  {card.tags.map((value, index) => (
                    <span key={value} className="flex gap-x-1.5">
                      {index > 0 && <span aria-hidden="true">·</span>}
                      <CardLine text={value} />
                    </span>
                  ))}
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>

      {shown.length === 0 && cards.length > 0 && (
        <p className="text-14 text-text-dim">
          Ничего не нашлось. Попробуй другой запрос или сними фильтр по тегу.
        </p>
      )}
    </div>
  );
}
