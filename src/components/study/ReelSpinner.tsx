import { useEffect, useMemo, useState } from "react";

import { cn } from "@/lib/cn";

/** Сколько строк видно в окне барабана. Центральная — та, что выпала. */
const VISIBLE = 5;

/** Индекс центральной строки в окне. */
const CENTER = Math.floor(VISIBLE / 2);

/** Высота строки в rem — она же шаг прокрутки. */
const ROW_REM = 3;

/** Сколько строк проносится мимо до остановки. */
const RUN_UP = 24;

/** Сколько крутится, мс. Столько же ждёт `onSettled`. */
export const SPIN_MS = 1300;

type ReelSpinnerProps = {
  /** Надписи для прокрутки: чем их больше, тем меньше повторов мелькает. */
  labels: string[];
  /** Надпись, на которой барабан остановится. */
  target: string;
  /** Меняется — барабан крутится заново. */
  spinKey: string;
  /** Вызывается один раз, когда барабан встал. */
  onSettled: () => void;
};

/**
 * Барабан: строки проносятся мимо и замедляются, пока в центре не встанет
 * выпавшая карточка.
 *
 * Ничего не решает — что выпадет, уже решил бэкенд; здесь только показ.
 * Проносящиеся мимо надписи взяты из той же колоды, поэтому барабан не врёт:
 * всё, что мелькнуло, там действительно есть.
 *
 * Лента строится детерминированно от `spinKey`: один и тот же прокрут всегда
 * выглядит одинаково, и лишняя перерисовка не дёргает картинку.
 */
export function ReelSpinner({
  labels,
  target,
  spinKey,
  onSettled,
}: ReelSpinnerProps) {
  const strip = useMemo(
    () => buildStrip(labels, target, spinKey),
    [labels, target, spinKey],
  );

  /**
   * Прокрут, для которого анимация уже запущена. Ключ, а не флаг: смена
   * карточки сама возвращает барабан в начало, без сброса состояния в
   * эффекте.
   */
  const [runningKey, setRunningKey] = useState<string | null>(null);
  const running = runningKey === spinKey;

  /**
   * Прокрут, который уже доехал. Пока лента едет, объявлять выпавшую
   * карточку рано — скринридер должен услышать её тогда же, когда её видно.
   */
  const [settledKey, setSettledKey] = useState<string | null>(null);
  const settled = settledKey === spinKey;

  // Кадром позже, чем лента встала в исходное положение: иначе браузеру
  // нечего анимировать — он сразу увидит конечное состояние.
  useEffect(() => {
    const frame = requestAnimationFrame(() => setRunningKey(spinKey));
    return () => cancelAnimationFrame(frame);
  }, [spinKey]);

  useEffect(() => {
    if (!running) return;

    const id = setTimeout(() => {
      setSettledKey(spinKey);
      onSettled();
    }, SPIN_MS);

    return () => clearTimeout(id);
  }, [running, spinKey, onSettled]);

  const stop = Math.max(strip.length - 1 - CENTER, 0);

  return (
    <div
      // Маска сверху и снизу: строки не обрываются, а растворяются в чёрном.
      className="w-full overflow-hidden [mask-image:linear-gradient(to_bottom,transparent,black_28%,black_72%,transparent)]"
      style={{ height: `${VISIBLE * ROW_REM}rem` }}
      role="status"
      aria-live="polite"
      aria-label={settled ? `Выпало: ${target}` : "Барабан крутится"}
    >
      <div
        className={cn(
          "flex flex-col motion-reduce:transition-none",
          running && "transition-transform duration-1300 ease-reel",
        )}
        style={{
          transform: `translateY(-${(running ? stop : 0) * ROW_REM}rem)`,
        }}
      >
        {strip.map((label, index) => {
          // Расстояние до центра окна: центральная строка яркая, соседние
          // приглушены, дальние почти не видны.
          const distance = Math.abs(index - (running ? stop + CENTER : CENTER));

          return (
            <span
              key={`${index}-${label}`}
              aria-hidden={distance === 0 ? undefined : "true"}
              className={cn(
                "flex h-12 shrink-0 items-center justify-center truncate px-4 text-center",
                "transition-colors duration-300 ease-standard",
                distance === 0 && "text-20 font-medium text-text sm:text-24",
                distance === 1 && "text-15 text-text-zen-dim",
                distance > 1 && "text-13 text-text-zen-dim-2",
              )}
            >
              {label}
            </span>
          );
        })}
      </div>
    </div>
  );
}

/**
 * Лента прокрутки: разбег из надписей колоды, выпавшая карточка в конце.
 *
 * Хвост после неё нужен, чтобы под центральной строкой были соседи, а не
 * пустота: окно показывает пять строк, а останавливается на третьей.
 */
function buildStrip(
  labels: string[],
  target: string,
  spinKey: string,
): string[] {
  const pool = labels.length > 0 ? labels : [target];
  let seed = hash(spinKey);
  const pick = () => {
    // Линейный конгруэнтный генератор: лента должна выглядеть случайной и
    // при этом одинаково при каждом пересчёте одного и того же прокрута.
    seed = (seed * 1664525 + 1013904223) >>> 0;
    return pool[seed % pool.length];
  };

  return [
    ...Array.from({ length: RUN_UP }, pick),
    target,
    ...Array.from({ length: CENTER }, pick),
  ];
}

function hash(value: string): number {
  let result = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    result = ((result ^ value.charCodeAt(index)) * 16777619) >>> 0;
  }

  return result;
}
