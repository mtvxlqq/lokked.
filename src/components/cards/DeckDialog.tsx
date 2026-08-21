import { useState, type FormEvent } from "react";

import { Button, Dialog, Input, Select } from "@/components/ui";
import {
  createDeck,
  deleteDeck,
  errorMessage,
  updateDeck,
  type Deck,
  type DeckInput,
  type Subject,
} from "@/lib/tauri";

type DeckDialogProps = {
  open: boolean;
  /** `null` — создание новой колоды. */
  deck: Deck | null;
  subjects: Subject[];
  onClose: () => void;
  onSaved: (deck: Deck | null) => void;
};

/**
 * Диалог создания и правки колоды.
 *
 * Правила ввода проверяет Rust — здесь только форма и его сообщение об
 * отказе. Удаление мягкое: карточки и их история остаются, колода просто
 * перестаёт показываться.
 */
export function DeckDialog({
  open,
  deck,
  subjects,
  onClose,
  onSaved,
}: DeckDialogProps) {
  const [name, setName] = useState(deck?.name ?? "");
  const [subjectId, setSubjectId] = useState(deck?.subject_id ?? "");
  const [description, setDescription] = useState(deck?.description ?? "");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const input: DeckInput = {
    name,
    subject_id: subjectId === "" ? null : subjectId,
    description: description.trim() === "" ? null : description,
  };

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);

    try {
      const saved = deck
        ? await updateDeck(deck.id, input)
        : await createDeck(input);
      onSaved(saved);
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setSaving(false);
    }
  }

  async function remove() {
    if (!deck) return;
    setSaving(true);
    setError(null);

    try {
      await deleteDeck(deck.id);
      onSaved(null);
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
      title={deck ? "Колода" : "Новая колода"}
      footer={
        <>
          {deck &&
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
            form="deck-form"
            disabled={saving}
          >
            Сохранить
          </Button>
        </>
      }
    >
      <form id="deck-form" onSubmit={submit} className="flex flex-col gap-3.5">
        <Input
          label="Название"
          value={name}
          onChange={(event) => setName(event.target.value)}
          autoFocus
          required
        />

        <Select
          label="Предмет"
          value={subjectId}
          onChange={(event) => setSubjectId(event.target.value)}
          hint="Колода без предмета живёт сама по себе."
        >
          <option value="">Без предмета</option>
          {subjects.map((subject) => (
            <option key={subject.id} value={subject.id}>
              {subject.name}
            </option>
          ))}
        </Select>

        <Input
          label="Описание"
          value={description}
          onChange={(event) => setDescription(event.target.value)}
          placeholder="Источник, охват, что угодно"
        />

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </form>
    </Dialog>
  );
}
