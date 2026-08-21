import { cn } from "@/lib/cn";
import { formatClock, formatClockMinutes } from "@/lib/format";
import type { ZenFontSize } from "@/lib/tauri";

/**
 * Шаги размера цифр из `docs/designs/tokens.md`. Мобильный размер и
 * десктопный — одна настройка: на телефоне 212px не существует в принципе, а
 * на мониторе 52px не читаются с другого конца комнаты. Ширину окна цифры
 * всё равно не переполнят — за этим следит `zen-fit`.
 */
const DIGIT_SIZE: Record<ZenFontSize, string> = {
  small: "text-52 sm:text-112",
  normal: "text-76 sm:text-212",
  large: "text-88 sm:text-256",
};

type ZenDisplayProps = {
  /** Время учёбы с начала сессии, в секундах. */
  seconds: number;
  subjectName: string;
  minutesOnly: boolean;
  fontSize: ZenFontSize;
  /** Прошло 5 секунд без единого движения — гасим. */
  dimmed: boolean;
  paused: boolean;
};

/**
 * Всё, что есть на чёрном экране: время и под ним название предмета.
 *
 * Единственное состояние — приглушённость. Переход между ярким и приглушённым
 * длится 1.6 секунды (`glow-fade`): гаснуть должно так, чтобы этого не
 * заметить, иначе экран сам себя и отвлекает.
 */
export function ZenDisplay({
  seconds,
  subjectName,
  minutesOnly,
  fontSize,
  dimmed,
  paused,
}: ZenDisplayProps) {
  const clock = minutesOnly
    ? formatClockMinutes(seconds)
    : formatClock(seconds);

  return (
    <div className="flex flex-col items-center gap-4 sm:gap-7">
      <div className={cn("font-mono leading-none", DIGIT_SIZE[fontSize])}>
        <span
          className={cn(
            "zen-fit glow-fade block font-medium tracking-timer tabular-nums",
            dimmed
              ? "glow-timer-dim motion-safe:animate-breathe-dim"
              : "glow-timer motion-safe:animate-breathe",
          )}
        >
          {clock}
        </span>
      </div>

      <p
        className={cn(
          "glow-fade text-center text-12 font-medium tracking-zen-subject-sm uppercase sm:text-16 sm:tracking-zen-subject",
          dimmed ? "text-text-zen-dim-2" : "text-text-zen-dim",
        )}
      >
        {subjectName}
      </p>

      {/* Пауза на чёрном экране иначе неотличима от остановившихся часов. */}
      {paused && (
        <p className="glow-fade text-11 tracking-label-3 text-text-zen-dim-2 uppercase">
          Пауза
        </p>
      )}
    </div>
  );
}
