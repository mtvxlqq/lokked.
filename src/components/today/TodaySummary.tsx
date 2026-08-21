import { formatClock, plural } from "@/lib/format";
import type { TodayTotals } from "@/lib/tauri";

type TileProps = {
  label: string;
  value: string;
};

/** Одна цифра сбоку от главной: помодоро или серия. */
function Tile({ label, value }: TileProps) {
  return (
    <div className="flex min-w-28 flex-col gap-1 rounded-lg border border-border bg-surface px-4 py-3">
      <span className="text-11 tracking-label text-text-faint uppercase">
        {label}
      </span>
      <span className="font-mono text-17 tabular-nums text-text-1">
        {value}
      </span>
    </div>
  );
}

/**
 * Сводка за учебный день над списком предметов: сколько всего изучено,
 * сколько помодоро доведено до конца и сколько дней подряд идёт серия.
 *
 * Все три числа приходят одной командой и относятся к одному дню — тому,
 * который сейчас идёт по настройке «начало учебного дня», а не календарному.
 */
export function TodaySummary({ totals }: { totals: TodayTotals }) {
  return (
    <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
      <div className="flex flex-col gap-1.5">
        <span className="text-11 tracking-label text-text-faint uppercase">
          Сегодня
        </span>
        <span className="glow-today font-mono text-40 leading-none tracking-timer-2 tabular-nums sm:text-58">
          {formatClock(totals.total_seconds)}
        </span>
      </div>

      <div className="flex flex-wrap gap-2.5">
        <Tile label="Помодоро" value={String(totals.pomodoros)} />
        <Tile
          label="Серия"
          value={`${totals.streak_days} ${plural(totals.streak_days, [
            "день",
            "дня",
            "дней",
          ])}`}
        />
      </div>
    </div>
  );
}
