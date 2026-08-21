import { Button } from "@/components/ui";
import { cn } from "@/lib/cn";
import { plural, withNonBreakingMarkers } from "@/lib/format";
import type { Deck, Subject } from "@/lib/tauri";

type DeckListProps = {
  decks: Deck[];
  subjects: Subject[];
  selectedId: string | null;
  onSelect: (deck: Deck) => void;
  onEdit: (deck: Deck) => void;
};

/**
 * Список колод: название, предмет и число карточек.
 *
 * Выбранная колода подсвечена — она же определяет, что показано в таблице
 * справа (на узком экране — ниже).
 */
export function DeckList({
  decks,
  subjects,
  selectedId,
  onSelect,
  onEdit,
}: DeckListProps) {
  return (
    <ul className="flex flex-col gap-1.5">
      {decks.map((deck) => {
        const subject = subjects.find(
          (candidate) => candidate.id === deck.subject_id,
        );
        const selected = deck.id === selectedId;

        return (
          <li key={deck.id} className="flex items-stretch gap-1.5">
            <button
              type="button"
              onClick={() => onSelect(deck)}
              aria-current={selected || undefined}
              className={cn(
                "flex min-h-11 flex-1 flex-col gap-0.5 rounded-lg px-3 py-2 text-left",
                "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent",
                selected ? "bg-raised text-text-1" : "text-text-2",
              )}
            >
              <span className="text-14.5 font-medium">
                {withNonBreakingMarkers(deck.name)}
              </span>
              <span className="text-12.5 text-text-dim-2">
                {deck.card_count}{" "}
                {plural(deck.card_count, ["карточка", "карточки", "карточек"])}
                {subject ? ` · ${subject.name}` : ""}
              </span>
            </button>

            <Button
              size="sm"
              variant="ghost"
              onClick={() => onEdit(deck)}
              aria-label={`Изменить колоду «${deck.name}»`}
            >
              Изм.
            </Button>
          </li>
        );
      })}
    </ul>
  );
}
