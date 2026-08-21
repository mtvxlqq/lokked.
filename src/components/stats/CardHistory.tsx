import { CardText } from "@/components/cards/CardText";
import { RecentAnswers } from "@/components/stats/RecentAnswers";
import { StatTiles } from "@/components/stats/StatTiles";
import { Card } from "@/components/ui";
import { formatThinkTime, plural } from "@/lib/format";
import type { CardReport } from "@/lib/tauri";

/**
 * История одной карточки: что на ней написано и как она идёт.
 */
export function CardHistory({ report }: { report: CardReport }) {
  return (
    <div className="flex flex-col gap-5 sm:gap-6">
      <StatTiles
        tiles={[
          { label: "Показов", value: String(report.shown) },
          {
            label: "Точность",
            value: report.shown === 0 ? "—" : `${report.accuracy_percent} %`,
          },
          {
            label: "Серия",
            value: `${report.current_streak} ${plural(report.current_streak, [
              "ответ",
              "ответа",
              "ответов",
            ])}`,
          },
        ]}
      />

      <Card title="Карточка">
        <CardText text={report.front} className="text-text-1" />
        <div className="border-t border-border pt-4">
          <CardText text={report.back} className="text-text-dim" />
        </div>
      </Card>

      <Card
        title="Последние ответы"
        aside={
          report.average_think_ms === null
            ? "время не замерялось"
            : `в среднем ${formatThinkTime(report.average_think_ms)}`
        }
      >
        <RecentAnswers grades={report.recent} />
        <p className="text-13 text-text-dim">
          Угадано {report.correct} из {report.shown}.
        </p>
      </Card>
    </div>
  );
}
