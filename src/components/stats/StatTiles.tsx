export type Tile = {
  label: string;
  value: string;
};

/**
 * Ряд крупных цифр над содержимым вкладки: сколько времени, какая точность,
 * какая серия.
 *
 * Плашки равной ширины и переносятся: на 380px их три в столбик, на десктопе
 * — в строку.
 */
export function StatTiles({ tiles }: { tiles: Tile[] }) {
  return (
    <div className="grid gap-2.5 sm:grid-cols-3">
      {tiles.map((tile) => (
        <div
          key={tile.label}
          className="flex flex-col gap-2 rounded-xl border border-border bg-surface px-4 py-3.5 sm:px-6 sm:py-5"
        >
          <span className="text-11 tracking-label text-text-faint uppercase">
            {tile.label}
          </span>
          <span className="font-mono text-24 leading-none tracking-timer-2 tabular-nums text-text-1 sm:text-34">
            {tile.value}
          </span>
        </div>
      ))}
    </div>
  );
}
