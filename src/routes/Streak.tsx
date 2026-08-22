import { useCallback, useEffect, useState } from "react";

import { Screen } from "@/components/Screen";
import { Freezes } from "@/components/streak/Freezes";
import { Milestones } from "@/components/streak/Milestones";
import { ShareDialog } from "@/components/streak/ShareDialog";
import { monthTitle } from "@/components/streak/monthTitle";
import { StreakCalendar } from "@/components/streak/StreakCalendar";
import { Button, Card } from "@/components/ui";
import { formatDay, formatDuration, plural } from "@/lib/format";
import { errorMessage, streakView, type StreakView } from "@/lib/tauri";

/** Граница учебного дня словами: «04:00». */
function boundary(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`;
}

/** Когда шёл рекорд: «15 марта — 10 апреля». */
function recordSpan(view: StreakView): string | null {
  if (!view.longest_from || !view.longest_to) return null;
  if (view.longest_from === view.longest_to)
    return formatDay(view.longest_from);

  return `${formatDay(view.longest_from)} — ${formatDay(view.longest_to)}`;
}

/**
 * Раздел «Серия»: сколько дней подряд, рекорд, заморозки, календарь месяца и
 * ближайшие вехи.
 *
 * Отдельный экран, а не вкладка статистики: серия — это не отчёт, а то,
 * ради чего открывают приложение в день, когда заниматься не хочется.
 */
export function Streak() {
  const [view, setView] = useState<StreakView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sharing, setSharing] = useState(false);

  const load = useCallback(() => {
    streakView()
      .then((loaded) => {
        setView(loaded);
        setError(null);
      })
      .catch((failure: unknown) => setError(errorMessage(failure)));
  }, []);

  useEffect(load, [load]);

  if (error && !view) {
    return (
      <Screen title="Серия">
        <Card title="Не удалось загрузить серию">
          <p className="text-14 text-danger-text" role="alert">
            {error}
          </p>
          <div>
            <Button variant="secondary" onClick={load}>
              Повторить
            </Button>
          </div>
        </Card>
      </Screen>
    );
  }

  if (!view) {
    return (
      <Screen title="Серия">
        <p className="text-14 text-text-dim">Загрузка…</p>
      </Screen>
    );
  }

  const record = recordSpan(view);

  return (
    <Screen
      title="Серия"
      actions={
        <Button variant="secondary" onClick={() => setSharing(true)}>
          Поделиться серией
        </Button>
      }
    >
      <div className="grid gap-2.5 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
        <Card title="Текущая серия" role="group" aria-label="Текущая серия">
          <p className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
            <span className="glow-streak font-mono text-58 leading-none tracking-timer-2 tabular-nums text-text sm:text-72">
              {view.current}
            </span>
            <span className="text-15.5 text-text-dim">
              {plural(view.current, ["день", "дня", "дней"])} подряд
            </span>
          </p>
          <p className="text-12.5 text-text-dim">
            Сегодня зачтено — {formatDuration(view.today_seconds)}, минимум{" "}
            {formatDuration(view.min_seconds)}.
          </p>
        </Card>

        <div className="flex flex-col gap-2.5">
          <Card title="Рекорд" role="group" aria-label="Рекорд">
            <p className="flex items-baseline gap-3">
              <span className="font-mono text-30 leading-none tabular-nums text-text-1">
                {view.longest}
              </span>
              <span className="text-14.5 text-text-dim">
                {plural(view.longest, ["день", "дня", "дней"])}
              </span>
            </p>
            {record && <p className="text-12.5 text-text-dim">{record}</p>}
          </Card>

          <Card title="Заморозки" role="group" aria-label="Заморозки">
            <Freezes
              freezes={view.freezes}
              max={view.max_freezes}
              every={view.freeze_every}
            />
          </Card>
        </div>
      </div>

      <div className="grid gap-2.5 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
        <Card
          title={monthTitle(view.month)}
          aside={`граница дня — ${boundary(view.day_start_seconds)}`}
          role="group"
          aria-label={monthTitle(view.month)}
        >
          <StreakCalendar month={view.month} today={view.today} />
        </Card>

        <Card title="Ближайшие вехи" role="group" aria-label="Ближайшие вехи">
          <Milestones milestones={view.milestones} current={view.current} />
        </Card>
      </div>

      <ShareDialog
        open={sharing}
        days={view.current}
        seconds={view.today_seconds}
        onClose={() => setSharing(false)}
      />
    </Screen>
  );
}
