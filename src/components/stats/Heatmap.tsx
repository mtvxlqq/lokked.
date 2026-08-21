import { formatDay, formatDuration } from "@/lib/format";
import type { HeatCell } from "@/lib/tauri";

/**
 * Заливки по уровням. Карта классов, а не склейка в рантайме: Tailwind
 * собирает классы, читая исходники, и `fill-heat-${level}` в сборку не
 * попадёт.
 */
const FILLS = [
  "fill-heat-0",
  "fill-heat-1",
  "fill-heat-2",
  "fill-heat-3",
  "fill-heat-4",
];

/** Сторона клетки и шаг сетки вместе с зазором, в единицах viewBox. */
const CELL = 11;
const STEP = 14;

/**
 * Активность по дням в духе GitHub: колонка — неделя, строка — день недели.
 *
 * Рисуется обычным SVG, без библиотек. Уровень клетки считает бэкенд —
 * относительно лучшего дня того же периода, — здесь остаётся только цвет.
 *
 * На узком экране карта не сжимается, а прокручивается: клетка меньше
 * восьми пикселей перестаёт читаться, и лучше показать половину картинки
 * целиком, чем всю нечитаемой.
 */
export function Heatmap({ cells }: { cells: HeatCell[] }) {
  if (cells.length === 0) return null;

  const offset = cells[0].weekday;
  const columns = Math.ceil((offset + cells.length) / 7);
  const width = columns * STEP - (STEP - CELL);
  const height = 7 * STEP - (STEP - CELL);

  return (
    <div className="flex flex-col gap-3">
      <div className="overflow-x-auto">
        <svg
          viewBox={`0 0 ${width} ${height}`}
          width={width}
          height={height}
          role="img"
          aria-label={`Активность с ${formatDay(cells[0].day_key)} по ${formatDay(
            cells[cells.length - 1].day_key,
          )}`}
          className="max-w-none"
        >
          {cells.map((cell, index) => {
            const column = Math.floor((index + offset) / 7);

            return (
              <rect
                key={cell.day_key}
                x={column * STEP}
                y={cell.weekday * STEP}
                width={CELL}
                height={CELL}
                rx={2}
                className={FILLS[Math.min(cell.level, FILLS.length - 1)]}
              >
                <title>{`${formatDay(cell.day_key)}: ${formatDuration(cell.seconds)}`}</title>
              </rect>
            );
          })}
        </svg>
      </div>

      <div className="flex items-center gap-2 text-11 tracking-label text-text-faint uppercase">
        <span>меньше</span>
        <svg
          viewBox={`0 0 ${5 * STEP - (STEP - CELL)} ${CELL}`}
          width={5 * STEP - (STEP - CELL)}
          height={CELL}
          aria-hidden="true"
        >
          {FILLS.map((fill, level) => (
            <rect
              key={fill}
              x={level * STEP}
              y={0}
              width={CELL}
              height={CELL}
              rx={2}
              className={fill}
            />
          ))}
        </svg>
        <span>больше</span>
      </div>
    </div>
  );
}
