import { useState, type FormEvent } from "react";

import { CardText } from "@/components/cards/CardText";
import { Button, Dialog, Input, Select, Textarea } from "@/components/ui";
import {
  createCard,
  deleteCard,
  errorMessage,
  moveCard,
  updateCard,
  type Card,
  type CardInput,
  type Deck,
} from "@/lib/tauri";

type CardDialogProps = {
  open: boolean;
  /** `null` — создание новой карточки. */
  card: Card | null;
  deckId: string;
  decks: Deck[];
  onClose: () => void;
  onSaved: () => void;
};

/**
 * Диалог создания и правки карточки.
 *
 * Под каждой стороной — то, как она покажется на самом деле: формулы в
 * карточках пишут руками, и увидеть `\dfrac` разобранным лучше сразу,
 * а не на середине прогона.
 */
export function CardDialog({
  open,
  card,
  deckId,
  decks,
  onClose,
  onSaved,
}: CardDialogProps) {
  const [front, setFront] = useState(card?.front ?? "");
  const [back, setBack] = useState(card?.back ?? "");
  const [hint, setHint] = useState(card?.hint ?? "");
  const [tags, setTags] = useState((card?.tags ?? []).join(", "));
  const [targetDeck, setTargetDeck] = useState(card?.deck_id ?? deckId);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const input: CardInput = {
    front,
    back,
    hint: hint.trim() === "" ? null : hint,
    tags: tags
      .split(",")
      .map((tag) => tag.trim())
      .filter((tag) => tag !== ""),
  };

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);

    try {
      if (card) {
        await updateCard(card.id, input);
        // Переезд отдельной командой: у него своя проверка, что колода
        // на месте, и своя запись в истории карточки.
        if (targetDeck !== card.deck_id) await moveCard(card.id, targetDeck);
      } else {
        await createCard(targetDeck, input);
      }
      onSaved();
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setSaving(false);
    }
  }

  async function remove() {
    if (!card) return;
    setSaving(true);
    setError(null);

    try {
      await deleteCard(card.id);
      onSaved();
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setSaving(false);
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={card ? "Карточка" : "Новая карточка"}
      className="max-w-2xl"
      footer={
        <>
          {card &&
            (confirmingDelete ? (
              <Button variant="danger" disabled={saving} onClick={remove}>
                Точно удалить
              </Button>
            ) : (
              <Button
                variant="ghost"
                disabled={saving}
                onClick={() => setConfirmingDelete(true)}
                className="sm:mr-auto"
              >
                Удалить
              </Button>
            ))}
          <Button variant="secondary" disabled={saving} onClick={onClose}>
            Отмена
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="card-form"
            disabled={saving}
          >
            Сохранить
          </Button>
        </>
      }
    >
      <form
        id="card-form"
        onSubmit={submit}
        className="flex max-h-[60vh] flex-col gap-3.5 overflow-y-auto"
      >
        <Textarea
          label="Лицевая сторона"
          value={front}
          onChange={(event) => setFront(event.target.value)}
          rows={2}
          autoFocus
          required
        />
        {front.trim() !== "" && (
          <CardText text={front} className="rounded-lg bg-raised px-4 py-3" />
        )}

        <Textarea
          label="Оборотная сторона"
          value={back}
          onChange={(event) => setBack(event.target.value)}
          rows={6}
          required
          hint="Формулы: $…$ в строке и $$…$$ отдельной строкой. Выделение: **жирный**, *курсив*."
        />
        {back.trim() !== "" && (
          <CardText text={back} className="rounded-lg bg-raised px-4 py-3" />
        )}

        <Input
          label="Подсказка"
          value={hint}
          onChange={(event) => setHint(event.target.value)}
          placeholder="Необязательно"
        />

        <Input
          label="Теги"
          value={tags}
          onChange={(event) => setTags(event.target.value)}
          placeholder="через запятую"
        />

        <Select
          label="Колода"
          value={targetDeck}
          onChange={(event) => setTargetDeck(event.target.value)}
        >
          {decks.map((deck) => (
            <option key={deck.id} value={deck.id}>
              {deck.name}
            </option>
          ))}
        </Select>

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
