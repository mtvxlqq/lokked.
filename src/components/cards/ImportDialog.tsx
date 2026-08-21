import { useState, type ChangeEvent } from "react";

import { Button, Dialog, Input, Select, Textarea } from "@/components/ui";
import {
  createDeck,
  errorMessage,
  importCards,
  previewImport,
  type Deck,
  type ImportProblem,
  type ImportReport,
} from "@/lib/tauri";

type ImportDialogProps = {
  open: boolean;
  decks: Deck[];
  /** Колода, открытая на экране, — она же предлагается по умолчанию. */
  currentDeckId: string | null;
  onClose: () => void;
  /** Вызывается после импорта с колодой, в которую он прошёл. */
  onImported: (deckId: string) => void;
};

const NEW_DECK = "new";

/** Человеческое объяснение того, почему блок не стал карточкой. */
function problemText(problem: ImportProblem): string {
  switch (problem.kind) {
    case "missing_back":
      return "нет оборотной стороны";
    case "blank_side":
      return "одна из сторон пустая";
    case "too_many_sides":
      return `частей ${problem.found ?? "больше трёх"}, а у карточки их не больше трёх`;
  }
}

/**
 * Импорт карточек: сначала разбор, потом запись.
 *
 * Разбор ничего не сохраняет — он только показывает, что получилось, и
 * сколько блоков не разобралось. Записывается ровно то, что видно в
 * предпросмотре, и одной транзакцией: колода либо наполняется целиком,
 * либо остаётся как была.
 */
export function ImportDialog({
  open,
  decks,
  currentDeckId,
  onClose,
  onImported,
}: ImportDialogProps) {
  const [text, setText] = useState("");
  const [cardSeparator, setCardSeparator] = useState("===");
  const [sideSeparator, setSideSeparator] = useState("---");
  const [report, setReport] = useState<ImportReport | null>(null);
  const [target, setTarget] = useState(currentDeckId ?? NEW_DECK);
  const [deckName, setDeckName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function readFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    if (!file) return;

    setError(null);
    try {
      const content = await file.text();
      setText(content);
      await parse(content);
    } catch (failure) {
      setError(errorMessage(failure));
    }
  }

  async function parse(source = text) {
    setBusy(true);
    setError(null);

    try {
      const parsed = await previewImport(source, {
        cardSeparator,
        sideSeparator,
      });
      setReport(parsed);
      if (parsed.suggested_deck && deckName === "") {
        setDeckName(parsed.suggested_deck);
      }
    } catch (failure) {
      setReport(null);
      setError(errorMessage(failure));
    } finally {
      setBusy(false);
    }
  }

  async function run() {
    if (!report) return;
    setBusy(true);
    setError(null);

    try {
      const deckId =
        target === NEW_DECK
          ? (
              await createDeck({
                name: deckName,
                subject_id: null,
                description: report.suggested_description,
              })
            ).id
          : target;

      await importCards(deckId, report.cards);
      onImported(deckId);
      onClose();
    } catch (failure) {
      setError(errorMessage(failure));
      setBusy(false);
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Импорт карточек"
      description="Текстом или файлом. Ничего не записывается, пока не нажат «Импортировать»."
      className="max-w-2xl"
      footer={
        <>
          <Button variant="secondary" disabled={busy} onClick={onClose}>
            Отмена
          </Button>
          <Button variant="ghost" disabled={busy} onClick={() => void parse()}>
            Разобрать
          </Button>
          <Button
            variant="primary"
            disabled={busy || !report || report.cards.length === 0}
            onClick={() => void run()}
          >
            Импортировать
          </Button>
        </>
      }
    >
      <div className="flex max-h-[60vh] flex-col gap-3.5 overflow-y-auto">
        <label className="flex min-h-11 flex-col gap-1.5">
          <span className="text-11 tracking-label text-text-faint uppercase">
            Файл
          </span>
          <input
            type="file"
            accept=".json,.txt,.md,text/plain,application/json"
            onChange={(event) => void readFile(event)}
            className="text-14 text-text-dim file:mr-3 file:rounded-md file:border file:border-border file:bg-raised file:px-3 file:py-2 file:text-14 file:text-text-1"
          />
        </label>

        <Textarea
          label="Или вставь текст"
          value={text}
          onChange={(event) => setText(event.target.value)}
          rows={6}
          hint="Карточки разделяются одним разделителем, стороны — другим. JSON с карточками распознаётся сам."
        />

        <div className="flex flex-col gap-3.5 sm:flex-row">
          <Input
            label="Разделитель карточек"
            value={cardSeparator}
            onChange={(event) => setCardSeparator(event.target.value)}
            wrapperClassName="flex-1"
          />
          <Input
            label="Разделитель сторон"
            value={sideSeparator}
            onChange={(event) => setSideSeparator(event.target.value)}
            wrapperClassName="flex-1"
          />
        </div>

        {report && (
          <div className="flex flex-col gap-2 rounded-lg border border-border bg-raised px-4 py-3">
            <p className="text-14 text-text-1">
              Распознано карточек: {report.cards.length}
              {report.format === "lecture_json" &&
                " (файл с карточками лекций)"}
            </p>

            {report.problems.length > 0 && (
              <ul className="flex flex-col gap-1 text-12.5 text-danger-text">
                {report.problems.slice(0, 5).map((problem) => (
                  <li key={problem.block}>
                    блок {problem.block}: {problemText(problem)}
                  </li>
                ))}
                {report.problems.length > 5 && (
                  <li>…и ещё {report.problems.length - 5}</li>
                )}
              </ul>
            )}

            {report.cards.length > 0 && (
              <p className="text-12.5 text-text-dim-2">
                Первая: {report.cards[0].front.slice(0, 80)}
              </p>
            )}
          </div>
        )}

        <Select
          label="Куда импортировать"
          value={target}
          onChange={(event) => setTarget(event.target.value)}
        >
          <option value={NEW_DECK}>Новая колода</option>
          {decks.map((deck) => (
            <option key={deck.id} value={deck.id}>
              {deck.name}
            </option>
          ))}
        </Select>

        {target === NEW_DECK && (
          <Input
            label="Название новой колоды"
            value={deckName}
            onChange={(event) => setDeckName(event.target.value)}
            required
          />
        )}

        {error && (
          <p className="text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </div>
    </Dialog>
  );
}
