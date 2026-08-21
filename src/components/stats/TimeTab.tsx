import { Heatmap } from "@/components/stats/Heatmap";
import { LoadFrame } from "@/components/stats/LoadFrame";
import { StatTiles } from "@/components/stats/StatTiles";
import { SubjectBars } from "@/components/stats/SubjectBars";
import { useStatsData } from "@/components/stats/useStatsData";
import { Card, EmptyState } from "@/components/ui";
import { TimerIcon } from "@/components/nav/icons";
import { formatDuration, plural } from "@/lib/format";
import { listSubjects, statsTime, type StatsRange } from "@/lib/tauri";

const HEATMAP_WEEKS = 30;

/**
 * Запрос вкладки. Объявлен вне компонента, чтобы у эффекта загрузки была
 * ровно одна причина сработать — смена периода.
 */
function load(range: string) {
  return Promise.all([statsTime(range as StatsRange), listSubjects()]);
}

/**
 * Вкладка «Время»: сколько изучено за период, по каким предметам и в какие
 * дни.
 *
 * Предметы приходят отдельной командой: у времени в базе есть только их
 * идентификаторы, а названия и цвета живут в своей таблице.
 */
export function TimeTab({ range }: { range: StatsRange }) {
  const { state, data, error, reload } = useStatsData(load, range);

  const [stats, subjects] = data ?? [null, []];

  return (
    <LoadFrame state={state} error={error} onRetry={reload}>
      {stats && (
        <div className="flex flex-col gap-5 sm:gap-6">
          <StatTiles
            tiles={[
              { label: "Время", value: formatDuration(stats.total_seconds) },
              { label: "Помодоро", value: String(stats.pomodoros) },
              {
                label: "Серия",
                value: `${stats.streak_days} ${plural(stats.streak_days, [
                  "день",
                  "дня",
                  "дней",
                ])}`,
              },
            ]}
          />

          <Card title="Время по предметам">
            {stats.subjects.length === 0 ? (
              <EmptyState
                icon={<TimerIcon className="size-8" />}
                title="За этот период занятий не было"
                description="Запусти таймер на любом предмете — время появится здесь."
              />
            ) : (
              <SubjectBars totals={stats.subjects} subjects={subjects} />
            )}
          </Card>

          <Card title="Активность" aside={`${HEATMAP_WEEKS} недель`}>
            <Heatmap cells={stats.heatmap} />
          </Card>
        </div>
      )}
    </LoadFrame>
  );
}
