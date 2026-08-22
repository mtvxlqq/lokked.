import { useEffect, useMemo, useState } from "react";

import { CardLine } from "@/components/cards/CardText";
import { cn } from "@/lib/cn";
import { plainText } from "@/lib/markdown";
import { playSpin } from "@/lib/reelSound";

/** Сколько строк видно в окне барабана. Центральная — та, что выпала. */
const VISIBLE = 5;

/** Индекс центральной строки в окне. */
const CENTER = Math.floor(VISIBLE / 2);

/**
 * Высота строки — она же шаг прокрутки.
 *
 * Живёт в CSS-переменной, а не в числе: на узком экране строка ниже, чем на
 * широком, и переменная держит высоту строки, высоту окна и сдвиг ленты в
 * согласии — сдвинуть ленту на «полторы строки» невозможно по построению.
 *
 * Высоты хватает на две строки текста самой крупной, центральной надписи:
 * длинная формулировка переносится по словам, а не обрывается на первом же
 * слове.
 */
const ROW = "var(--reel-row)";

/** Сколько строк проносится мимо до остановки. */
const RUN_UP = 24;

/** Где в ленте стоит выпавшая карточка. */
const TARGET_INDEX = RUN_UP;

/** Сколько крутится, мс. Столько же ждёт `onSettled`. */
export const SPIN_MS = 1300;

type ReelSpinnerProps = {
  /**
   * Надписи для прокрутки: чем их больше, тем меньше повторов мелькает.
   * Размеченный текст лицевых сторон — формулы в ленте рисуются, а не
   * показываются исходником.
   */
  labels: string[];
  /** Надпись, на которой барабан остановится. Тоже размеченная. */
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

  // Лента сдвигается так, чтобы выпавшая карточка встала в середину окна, а
  // не в его низ: под ней должны остаться соседи, иначе барабан выглядит
  // оборвавшимся.
  const stop = Math.max(TARGET_INDEX - CENTER, 0);

  // Кадром позже, чем лента встала в исходное положение: иначе браузеру
  // нечего анимировать — он сразу увидит конечное состояние.
  useEffect(() => {
    const frame = requestAnimationFrame(() => setRunningKey(spinKey));
    return () => cancelAnimationFrame(frame);
  }, [spinKey]);

  useEffect(() => {
    if (!running) return;

    // Звук расписывается там же, где стартует анимация, и теми же числами:
    // щелчки должны совпасть со строками, а не идти рядом с ними.
    const silence = playSpin(stop, SPIN_MS);

    const id = setTimeout(() => {
      setSettledKey(spinKey);
      onSettled();
    }, SPIN_MS);

    return () => {
      clearTimeout(id);
      silence();
    };
  }, [running, spinKey, onSettled, stop]);

  return (
    <div
      // Маска сверху и снизу: строки не обрываются, а растворяются в чёрном.
      className="w-full overflow-hidden [--reel-row:5.5rem] [mask-image:linear-gradient(to_bottom,transparent,black_28%,black_72%,transparent)] sm:[--reel-row:7rem]"
      style={{ height: `calc(${ROW} * ${VISIBLE})` }}
      role="status"
      aria-live="polite"
      aria-label={settled ? `Выпало: ${plainText(target)}` : "Барабан крутится"}
    >
      <div
        className={cn(
          "flex flex-col motion-reduce:transition-none",
          running && "transition-transform duration-1300 ease-reel",
        )}
        style={{
          transform: `translateY(calc(${ROW} * -${running ? stop : 0}))`,
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
                // Строка фиксированной высоты и с обрезкой: высокая формула
                // не должна наезжать на соседей и сбивать шаг прокрутки.
                "flex shrink-0 items-center justify-center overflow-hidden px-4 text-center sm:px-8",
                "transition-colors duration-300 ease-standard",
                distance === 0 && "font-semibold text-text",
                distance === 1 && "text-text-zen-dim",
                distance > 1 && "text-text-zen-dim-2",
              )}
              style={{ height: ROW }}
            >
              <CardLine
                text={label}
                className={cn(
                  // Две строки, дальше многоточие: надпись переносится по
                  // словам, но в свою строку барабана обязана уместиться —
                  // иначе она поехала бы на соседей.
                  "w-full break-words line-clamp-2 leading-title",
                  distance === 0 && "text-30 sm:text-40",
                  distance === 1 && "text-20 sm:text-26",
                  distance > 1 && "text-16 sm:text-21",
                )}
              />
            </span>
          );
        })}
      </div>
    </div>
  );
}

/**
 * Лента прокрутки: разбег из надписей колоды, выпавшая карточка, хвост.
 *
 * Хвост нужен ровно затем, чтобы под остановившейся карточкой были соседние
 * варианты: барабан кончается не на ней, он на ней стоит.
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
    ...Array.from({ length: TARGET_INDEX }, pick),
    target,
    // Столько же, сколько видно сверху: окно симметрично относительно
    // центральной строки.
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
