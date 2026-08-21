import { useEffect, useState } from "react";

import { Button, Dialog, Textarea } from "@/components/ui";
import { errorMessage, statsExportCsv, type StatsRange } from "@/lib/tauri";

type ExportDialogProps = {
  open: boolean;
  range: StatsRange;
  onClose: () => void;
};

/**
 * Экспорт периода в CSV: строка на день, со временем и карточками.
 *
 * Как и экспорт колоды, текст показывается на экране и копируется в буфер, а
 * не сохраняется файлом: диалог сохранения — отдельное платформенное
 * разрешение ради того же результата, только с поисками файла после.
 */
export function ExportDialog({ open, range, onClose }: ExportDialogProps) {
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    let cancelled = false;

    statsExportCsv(range)
      .then((csv) => {
        if (!cancelled) setText(csv);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [range]);

  async function copy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      setError("Скопировать не вышло — выдели текст и скопируй вручную");
    }
  }

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title="Экспорт в CSV"
      description="Строка на день: время учёбы и карточки за него."
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
        label="Таблица"
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
