import { formatDuration } from "@/lib/format";
import type { StreakDay, StreakDayState, StreakMonth } from "@/lib/tauri";
import { cn } from "@/lib/cn";

/**
 * Заливка и цвет текста по состоянию дня. Карта классов, а не склейка в
 * рантайме: Tailwind собирает классы, читая исходники.
 */
const CELL_STYLES: Record<StreakDayState, string> = {
  counted: "bg-streak-done text-streak-done-text",
  frozen: "bg-streak-frozen text-streak-frozen-text",
  missed: "bg-streak-missed text-streak-missed-text",
  pending: "bg-streak-future text-streak-done-text border border-accent",
  future: "bg-streak-future text-streak-future-text",
};

const STATE_NAMES: Record<StreakDayState, string> = {
  counted: "зачтено",
  frozen: "заморозка",
  missed: "пропуск",
  pending: "сегодня",
  future: "впереди",
};

/** Подписи столбцов. Неделя начинается с понедельника, как в календаре. */
const WEEKDAYS = ["пн", "вт", "ср", "чт", "пт", "сб", "вс"];

/** На какой столбец приходится день недели: понедельник — нулевой. */
function column(day: string): number {
  return (new Date(`${day}T00:00:00`).getDay() + 6) % 7;
}

function dayNumber(day: string): number {
  return Number(day.slice(8, 10));
}

/**
 * Календарь месяца с отмеченными днями.
 *
 * Клетки — кнопки-заглушки без действия, поэтому это обычный список с
 * подписью: нажимать здесь не на что, а вот прочитать, сколько было в
 * конкретный день, полезно — это `title`.
 */
export function StreakCalendar({
  month,
  today,
}: {
  month: StreakMonth;
  today: string;
}) {
  if (month.days.length === 0) return null;

  const offset = column(month.days[0].day);

  return (
    <div className="flex flex-col gap-3.5">
      <div className="grid grid-cols-7 gap-1.5">
        {WEEKDAYS.map((weekday) => (
          <span
            key={weekday}
            className="text-center text-11 tracking-label text-text-faint uppercase"
          >
            {weekday}
          </span>
        ))}

        {Array.from({ length: offset }, (_, index) => (
          <span key={`before-${index}`} aria-hidden="true" />
        ))}

        {month.days.map((day) => (
          <Cell key={day.day} day={day} today={today} />
        ))}
      </div>

      <ul className="flex flex-wrap gap-x-4 gap-y-2">
        {(["counted", "frozen", "missed", "pending"] as StreakDayState[]).map(
          (state) => (
            <li
              key={state}
              className="flex items-center gap-2 text-12.5 text-text-dim"
            >
              <span
                aria-hidden="true"
                className={cn("size-3 rounded", CELL_STYLES[state])}
              />
              {STATE_NAMES[state]}
            </li>
          ),
        )}
      </ul>
    </div>
  );
}

function Cell({ day, today }: { day: StreakDay; today: string }) {
  const studied =
    day.seconds > 0 ? formatDuration(day.seconds) : "ничего не записано";

  return (
    <span
      title={`${dayNumber(day.day)} число — ${STATE_NAMES[day.state]}, ${studied}`}
      aria-current={day.day === today ? "date" : undefined}
      className={cn(
        "flex aspect-square min-h-9 items-center justify-center rounded-lg text-13 tabular-nums sm:min-h-11 sm:text-15",
        CELL_STYLES[day.state],
      )}
    >
      {dayNumber(day.day)}
    </span>
  );
}
