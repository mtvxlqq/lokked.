import { useEffect, useRef, useState } from "react";

import { Button, Dialog } from "@/components/ui";
import { formatClock, plural } from "@/lib/format";
import {
  drawShareImage,
  SHARE_HEIGHT,
  SHARE_WIDTH,
  type ShareContext,
} from "@/lib/shareImage";
import { errorMessage, saveStreakImage } from "@/lib/tauri";

type ShareDialogProps = {
  open: boolean;
  days: number;
  seconds: number;
  onClose: () => void;
};

/**
 * «Поделиться серией»: картинка в стиле чёрного экрана, которую можно
 * сохранить и отправить.
 *
 * Холст рисуется в полном размере 1080×1350, а на экран показывается
 * уменьшенным через CSS: сохранять надо то, что пойдёт в сторис, а не то,
 * что поместилось в диалог.
 */
export function ShareDialog({
  open,
  days,
  seconds,
  onClose,
}: ShareDialogProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [saved, setSaved] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;

    const context = canvas.current?.getContext("2d");
    if (!context) return;

    drawShareImage(context as ShareContext, {
      days,
      seconds,
      daysLabel: `${plural(days, ["день", "дня", "дней"])} подряд`,
      clock: formatClock(seconds),
    });
  }, [open, days, seconds]);

  /** Закрытие забывает, что было сохранено: в следующий раз серия другая. */
  function close() {
    setSaved(null);
    setError(null);
    onClose();
  }

  function save() {
    const png = canvas.current?.toDataURL("image/png");
    if (!png) {
      setError("Картинка не нарисовалась");
      return;
    }

    setError(null);
    saveStreakImage(png)
      .then(setSaved)
      .catch((failure: unknown) => setError(errorMessage(failure)));
  }

  return (
    <Dialog
      open={open}
      onClose={close}
      title="Поделиться серией"
      description="Картинка в стиле чёрного экрана: серия, время за сегодня и больше ничего."
      footer={
        <>
          <Button variant="ghost" onClick={close}>
            Закрыть
          </Button>
          <Button onClick={save}>Сохранить</Button>
        </>
      }
    >
      <canvas
        ref={canvas}
        width={SHARE_WIDTH}
        height={SHARE_HEIGHT}
        aria-label={`Серия ${days} ${plural(days, ["день", "дня", "дней"])} подряд`}
        role="img"
        className="mx-auto w-full max-w-64 rounded-xl border border-border bg-bg-zen"
      />

      {saved && (
        <p className="text-12.5 break-all text-text-dim">
          Сохранено: <span className="text-text-2">{saved}</span>
        </p>
      )}
      {error && (
        <p className="text-13 text-danger-text" role="alert">
          {error}
        </p>
      )}
    </Dialog>
  );
}
