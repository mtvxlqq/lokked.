import { useEffect, useState } from "react";

import { Button, Dialog, Textarea } from "@/components/ui";
import { errorMessage, exportDeck, type Deck } from "@/lib/tauri";

type ExportDialogProps = {
  open: boolean;
  deck: Deck;
  onClose: () => void;
};

/**
 * Экспорт колоды в тот же текстовый формат, из которого идёт импорт.
 *
 * Текст показывается как есть, а не сохраняется в файл: диалог сохранения —
 * это отдельное платформенное разрешение ради того, что и так делается
 * копированием, а скопированное можно вставить куда угодно, включая импорт
 * в другую колоду.
 */
export function ExportDialog({ open, deck, onClose }: ExportDialogProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let cancelled = false;

    exportDeck(deck.id)
      .then((exported) => {
        if (!cancelled) setText(exported);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [deck.id]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      // Буфер может быть недоступен — текст на экране, его можно выделить.
      setError("Скопировать не вышло — выдели текст и скопируй вручную");
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={`Экспорт: ${deck.name}`}
      description="Тот же формат, что понимает импорт."
      className="max-w-2xl"
      footer={
        <>
          <Button variant="secondary" onClick={onClose}>
            Закрыть
          </Button>
          <Button variant="primary" onClick={() => void copy()}>
            {copied ? "Скопировано" : "Скопировать"}
          </Button>
        </>
      }
    >
      <Textarea
        label="Карточки"
        value={text}
        readOnly
        rows={12}
        className="font-mono text-13"
      />

      {error && (
        <p className="text-13 text-danger-text" role="alert">
          {error}
        </p>
      )}
    </Dialog>
  );
}
