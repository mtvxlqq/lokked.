import type { ReactNode } from "react";

type PhaseRingProps = {
  /** Доля пройденного, 0…1. Больше единицы обрезается — фаза может перестоять. */
  progress: number;
  /** Приглушить кольцо: на паузе оно не должно тянуть на себя внимание. */
  dimmed?: boolean;
  children: ReactNode;
};

const RADIUS = 46;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;

/**
 * Круговой индикатор фазы с цифрами внутри.
 *
 * Обычный SVG без библиотек: круг рисуется штрихом длиной в свою окружность,
 * а прогресс — сдвигом штриха. Перерисовывается четыре раза в секунду,
 * поэтому анимаций и фильтров здесь нет — только смена одного числа.
 */
export function PhaseRing({
  progress,
  dimmed = false,
  children,
}: PhaseRingProps) {
  const clamped = Math.min(1, Math.max(0, progress));

  return (
    <div className="relative mx-auto aspect-square w-full max-w-70 sm:max-w-88">
      <svg
        viewBox="0 0 100 100"
        aria-hidden="true"
        className="size-full -rotate-90"
      >
        <circle
          cx="50"
          cy="50"
          r={RADIUS}
          fill="none"
          strokeWidth="1.5"
          className="stroke-border"
        />
        <circle
          cx="50"
          cy="50"
          r={RADIUS}
          fill="none"
          strokeWidth="1.5"
          strokeLinecap="round"
          strokeDasharray={CIRCUMFERENCE}
          strokeDashoffset={CIRCUMFERENCE * (1 - clamped)}
          className={dimmed ? "stroke-border-strong" : "stroke-accent"}
        />
      </svg>

      <div className="absolute inset-0 flex flex-col items-center justify-center gap-2">
        {children}
      </div>
    </div>
  );
}
