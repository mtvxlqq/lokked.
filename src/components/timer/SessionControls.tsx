import { Button } from "@/components/ui";
import type { SessionSnapshot } from "@/lib/tauri";

type SessionControlsProps = {
  session: SessionSnapshot;
  busy: boolean;
  onPause: () => void;
  onResume: () => void;
  onInterruption: () => void;
  onSkip: () => void;
  onStop: () => void;
  onZen: () => void;
};

/**
 * Кнопки сессии.
 *
 * Одна главная — пауза или продолжение; остальные рядом равноправны.
 * На узком экране складываются в колонку: три кнопки по 44px в ряд на 380px
 * не помещаются, а ужимать их нельзя.
 */
export function SessionControls({
  session,
  busy,
  onPause,
  onResume,
  onInterruption,
  onSkip,
  onStop,
  onZen,
}: SessionControlsProps) {
  const paused = session.status === "paused";

  return (
    <div className="flex flex-col items-center gap-3">
      <Button
        variant="primary"
        size="lg"
        disabled={busy}
        onClick={paused ? onResume : onPause}
        className="w-full sm:w-auto"
      >
        {paused ? "Продолжить" : "Пауза"}
      </Button>

      <div className="flex w-full flex-col gap-2.5 sm:w-auto sm:flex-row sm:justify-center">
        <Button size="sm" disabled={busy} onClick={onInterruption}>
          Отвлёкся
        </Button>

        {session.mode === "pomodoro" && (
          <Button size="sm" variant="ghost" disabled={busy} onClick={onSkip}>
            {session.phase === "work" ? "На перерыв" : "К работе"}
          </Button>
        )}

        <Button size="sm" disabled={busy} onClick={onZen}>
          Чёрный экран
        </Button>

        <Button size="sm" variant="danger" disabled={busy} onClick={onStop}>
          Стоп
        </Button>
      </div>
    </div>
  );
}
