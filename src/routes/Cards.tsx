import { useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { CardDialog } from "@/components/cards/CardDialog";
import { CardTable } from "@/components/cards/CardTable";
import { DeckDialog } from "@/components/cards/DeckDialog";
import { DeckList } from "@/components/cards/DeckList";
import { ExportDialog } from "@/components/cards/ExportDialog";
import { ImportDialog } from "@/components/cards/ImportDialog";
import { CardsIcon } from "@/components/nav/icons";
import { MODE_NAMES, STUDY_MODES } from "@/components/study/modes";
import { Screen } from "@/components/Screen";
import { Button, Card as Panel, EmptyState } from "@/components/ui";
import { withNonBreakingMarkers } from "@/lib/format";
import {
  errorMessage,
  listCards,
  listDecks,
  listSubjects,
  type Card,
  type Deck,
  type Subject,
} from "@/lib/tauri";

type LoadState = "loading" | "ready" | "failed";

/**
 * Раздел «Карточки»: колоды слева, карточки выбранной колоды справа.
 *
 * На узком экране колонки складываются одна под другую — сначала колоды,
 * потом карточки той, что выбрана.
 */
export function Cards() {
  const navigate = useNavigate();

  const [decks, setDecks] = useState<Deck[]>([]);
  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [cards, setCards] = useState<Card[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [state, setState] = useState<LoadState>("loading");
  const [error, setError] = useState<string | null>(null);

  const [deckDialog, setDeckDialog] = useState<{
    open: boolean;
    deck: Deck | null;
  }>({ open: false, deck: null });
  const [cardDialog, setCardDialog] = useState<{
    open: boolean;
    card: Card | null;
  }>({ open: false, card: null });
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  /** Список карточек по умолчанию свёрнут: экран открывают, чтобы начать
      прогон, а не листать сотню карточек — облако тегов и таблица разворачиваются
      по кнопке. */
  const [listOpen, setListOpen] = useState(false);

  const [reloads, setReloads] = useState(0);
  const reload = () => setReloads((count) => count + 1);

  const selected = decks.find((deck) => deck.id === selectedId) ?? null;
  // Карточки принадлежат колоде, поэтому фильтруются, а не затираются при
  // переключении: иначе на миг показывались бы чужие.
  const deckCards = cards.filter((card) => card.deck_id === selectedId);

  useEffect(() => {
    let cancelled = false;

    Promise.all([listDecks(), listSubjects()])
      .then(([loadedDecks, loadedSubjects]) => {
        if (cancelled) return;

        setDecks(loadedDecks);
        setSubjects(loadedSubjects);
        // Первая колода открывается сама: экран без выбранной колоды
        // показывал бы пустоту, за которой на самом деле есть карточки.
        setSelectedId((current) =>
          current && loadedDecks.some((deck) => deck.id === current)
            ? current
            : (loadedDecks[0]?.id ?? null),
        );
        setError(null);
        setState("ready");
      })
      .catch((failure: unknown) => {
        if (cancelled) return;
        setError(errorMessage(failure));
        setState("failed");
      });

    return () => {
      cancelled = true;
    };
  }, [reloads]);

  useEffect(() => {
    if (!selectedId) return;

    let cancelled = false;

    listCards(selectedId)
      .then((loaded) => {
        if (!cancelled) setCards(loaded);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [selectedId, reloads]);

  return (
    <Screen
      title="Карточки"
      actions={
        state === "ready" && (
          <div className="flex flex-wrap gap-2.5">
            <Button size="sm" onClick={() => setImporting(true)}>
              Импорт
            </Button>
            {selected && (
              <Button size="sm" onClick={() => setExporting(true)}>
                Экспорт
              </Button>
            )}
            <Button
              size="sm"
              variant="primary"
              onClick={() => setDeckDialog({ open: true, deck: null })}
            >
              Новая колода
            </Button>
          </div>
        )
      }
    >
      {state === "loading" && (
        <p className="text-14 text-text-dim">Загрузка…</p>
      )}

      {state === "failed" && (
        <Panel title="Не удалось загрузить карточки">
          <p className="text-14 text-danger-text" role="alert">
            {error}
          </p>
          <div>
            <Button variant="secondary" onClick={reload}>
              Повторить
            </Button>
          </div>
        </Panel>
      )}

      {state === "ready" && decks.length === 0 && (
        <EmptyState
          icon={<CardsIcon className="size-8" />}
          title="Колод пока нет"
          description="Создай колоду и добавь карточки — или вставь готовые списком через импорт."
          action={
            <div className="flex flex-wrap justify-center gap-2.5">
              <Button variant="primary" onClick={() => setImporting(true)}>
                Импортировать
              </Button>
              <Button onClick={() => setDeckDialog({ open: true, deck: null })}>
                Новая колода
              </Button>
            </div>
          }
        />
      )}

      {state === "ready" && decks.length > 0 && (
        <div className="grid gap-5 lg:grid-cols-3 lg:items-start">
          <Panel title="Колоды">
            <DeckList
              decks={decks}
              subjects={subjects}
              selectedId={selectedId}
              onSelect={(deck) => setSelectedId(deck.id)}
              onEdit={(deck) => setDeckDialog({ open: true, deck })}
            />
          </Panel>

          {selected && (
            <Panel className="lg:col-span-2">
              {/* Заголовок свой, а не через `title`/`aside`: описание колоды
                  бывает в несколько строк и рядом с названием сплющивало его
                  в столбик. */}
              <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-2">
                <h2 className="text-14.5 font-medium text-text-1">
                  {withNonBreakingMarkers(selected.name)}
                </h2>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-expanded={listOpen}
                  onClick={() => setListOpen((open) => !open)}
                >
                  {listOpen ? "Скрыть карточки" : "Показать карточки"}
                </Button>
              </div>

              {selected.description && (
                <p className="text-12.5 leading-text text-text-dim-2">
                  {selected.description}
                </p>
              )}

              <div className="flex flex-wrap gap-2.5">
                {STUDY_MODES.map((mode) => (
                  <Button
                    key={mode}
                    size="sm"
                    disabled={deckCards.length === 0}
                    onClick={() =>
                      void navigate(`/study/${selected.id}?mode=${mode}`)
                    }
                  >
                    {MODE_NAMES[mode]}
                  </Button>
                ))}

                {/* Дуэль стоит отдельно от режимов: это не способ пройти
                    колоду, а игра на двоих — и колоду в ней выбирают
                    заново, хоть барабаном. */}
                <Button
                  size="sm"
                  variant="secondary"
                  onClick={() => void navigate("/duel")}
                >
                  Дуэль
                </Button>
              </div>

              {listOpen &&
                (deckCards.length === 0 ? (
                  <p className="text-14 text-text-dim">
                    В колоде пока нет карточек.
                  </p>
                ) : (
                  <CardTable
                    cards={deckCards}
                    onEdit={(card) => setCardDialog({ open: true, card })}
                  />
                ))}
              {/* Правка колоды — отдельно от прогонов и последней: сюда
                  заходят учить, а не пополнять. */}
              <div className="flex border-t border-border pt-3.5">
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setCardDialog({ open: true, card: null })}
                >
                  + Новая карточка
                </Button>
              </div>
            </Panel>
          )}
        </div>
      )}

      {deckDialog.open && (
        <DeckDialog
          open
          deck={deckDialog.deck}
          subjects={subjects}
          onClose={() => setDeckDialog({ open: false, deck: null })}
          onSaved={(deck) => {
            if (deck) setSelectedId(deck.id);
            else setSelectedId(null);
            reload();
          }}
        />
      )}

      {cardDialog.open && selected && (
        <CardDialog
          open
          card={cardDialog.card}
          deckId={selected.id}
          decks={decks}
          onClose={() => setCardDialog({ open: false, card: null })}
          onSaved={reload}
        />
      )}

      {importing && (
        <ImportDialog
          open
          decks={decks}
          currentDeckId={selectedId}
          onClose={() => setImporting(false)}
          onImported={(deckId) => {
            setSelectedId(deckId);
            reload();
          }}
        />
      )}

      {exporting && selected && (
        <ExportDialog
          open
          deck={selected}
          onClose={() => setExporting(false)}
        />
      )}
    </Screen>
  );
}
