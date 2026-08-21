import { useEffect, useState } from "react";

import { cn } from "@/lib/cn";

/** Последние столько секунд кольцо краснеет. */
const URGENT_SECONDS = 5;

/** Как часто перерисовывается кольцо: плавно, но без лишней работы. */
const TICK_MS = 100;

type BlitzRingProps = {
  /** Момент, когда время карточки выйдет, ISO-8601. */
  deadline: string;
  /** Сколько секунд даётся на карточку целиком. */
  seconds: number;
  /** Вызывается один раз, когда время кончилось. */
  onExpire: () => void;
};

const SIZE = 44;
const RADIUS = 19;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/**
 * Убывающее кольцо блица.
 *
 * Остаток считается от отметки, которую выдал бэкенд, а не тиками: свёрнутое
 * окно или уснувшая машина не подарят студенту лишних секунд, а вернувшись,
 * он увидит правду — и, скорее всего, уже истёкшее время.
 */
export function BlitzRing({ deadline, seconds, onExpire }: BlitzRingProps) {
  const [left, setLeft] = useState(() => remaining(deadline));

  // Сброс остатка не нужен: экран пересоздаёт кольцо на каждой карточке
  // (`key` — сама отметка), поэтому начальное состояние всегда своё.
  useEffect(() => {
    let expired = false;

    const id = setInterval(() => {
      const value = remaining(deadline);
      setLeft(value);

      if (value <= 0 && !expired) {
        expired = true;
        onExpire();
      }
    }, TICK_MS);

    return () => clearInterval(id);
  }, [deadline, onExpire]);

  const share = Math.max(0, Math.min(1, left / (seconds * 1000)));
  const urgent = left <= URGENT_SECONDS * 1000;

  return (
    <div
      className="flex items-center gap-2"
      role="timer"
      aria-label={`Осталось секунд: ${Math.ceil(left / 1000)}`}
    >
      <svg
        viewBox={`0 0 ${SIZE} ${SIZE}`}
        className="size-9 -rotate-90"
        aria-hidden="true"
      >
        <circle
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          fill="none"
          strokeWidth="3"
          className="stroke-border"
        />
        <circle
          cx={SIZE / 2}
          cy={SIZE / 2}
          r={RADIUS}
          fill="none"
          strokeWidth="3"
          strokeLinecap="round"
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={CIRCUMFERENCE * (1 - share)}
          className={cn(urgent ? "stroke-danger" : "stroke-accent")}
        />
      </svg>

      <span
        className={cn(
          "font-mono text-15 tabular-nums",
          urgent ? "text-danger-text" : "text-text-dim",
        )}
      >
        {Math.ceil(left / 1000)}
      </span>
    </div>
  );
}

/** Сколько миллисекунд осталось до отметки. Никогда не меньше нуля. */
function remaining(deadline: string): number {
  return Math.max(0, new Date(deadline).getTime() - Date.now());
}
