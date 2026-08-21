import { formatDay } from "@/lib/format";
import type { DayAccuracy } from "@/lib/tauri";

/**
 * Точность по дням: колонка на день, высота — процент верных ответов.
 *
 * День без ответов остаётся пустой дорожкой: нулевая колонка означала бы,
 * что всё отвечено неверно, а это совсем другая история.
 *
 * Колонки не сжимаются ниже читаемого: длинный период прокручивается вбок
 * внутри своего блока, а не расползается по всему экрану.
 */
export function AccuracyChart({ days }: { days: DayAccuracy[] }) {
  if (days.length === 0) return null;

  return (
    <div className="flex flex-col gap-2">
      <div className="overflow-x-auto">
        <div className="flex h-32 min-w-full items-end gap-0.5">
          {days.map((day) => (
            <div
              key={day.day_key}
              className="flex h-full min-w-1 flex-1 flex-col justify-end rounded-sm bg-track"
              title={
                day.answered === 0
                  ? `${formatDay(day.day_key)}: карточек не было`
                  : `${formatDay(day.day_key)}: ${day.answered} ответов, ${day.accuracy_percent}%`
              }
            >
              {day.answered > 0 && (
                <div
                  className="rounded-sm bg-accent"
                  // Высота — это и есть данные: процент из команды.
                  style={{
                    height: `${Math.max(day.accuracy_percent, 2)}%`,
                  }}
                />
              )}
            </div>
          ))}
        </div>
      </div>

      <div className="flex justify-between text-11.5 text-text-faint">
        <span>{formatDay(days[0].day_key)}</span>
        <span>{formatDay(days[days.length - 1].day_key)}</span>
      </div>
    </div>
  );
}
