import { CardText } from "@/components/cards/CardText";
import { Button, Card } from "@/components/ui";
import { formatClock, plural } from "@/lib/format";
import type { StudySummary } from "@/lib/tauri";

type RunSummaryProps = {
  summary: StudySummary;
  onRepeat: () => void;
  onLeave: () => void;
  busy: boolean;
};

/**
 * Экран после прогона: сколько, насколько точно, сколько времени — и разбор
 * ошибок сразу здесь.
 *
 * Ошибки показываются целиком, с ответом: смотреть их надо, пока карточка
 * ещё в голове, а не через переход в редактор.
 */
export function RunSummary({
  summary,
  onRepeat,
  onLeave,
  busy,
}: RunSummaryProps) {
  return (
    <div className="flex w-full flex-col gap-5">
      <Card title="Прогон закончен" aside={summary.deck_name}>
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          <Figure label="Карточек" value={String(summary.answered)} />
          <Figure label="Точность" value={`${summary.accuracy_percent}%`} />
          <Figure
            label="Время"
            value={formatClock(Math.round(summary.total_ms / 1000))}
          />
          <Figure
            label="В среднем"
            value={`${(summary.average_ms / 1000).toFixed(1)} с`}
          />
        </div>
      </Card>

      {summary.mistake_cards.length > 0 && (
        <Card
          title="Ошибки"
          aside={`${summary.mistake_cards.length} ${plural(
            summary.mistake_cards.length,
            ["карточка", "карточки", "карточек"],
          )}`}
        >
          <ul className="flex flex-col divide-y divide-border">
            {summary.mistake_cards.map((card) => (
              <li key={card.id} className="flex flex-col gap-2 py-3.5">
                <CardText text={card.front} className="text-15 text-text-1" />
                {card.back && (
                  <CardText text={card.back} className="text-14 text-text-2" />
                )}
              </li>
            ))}
          </ul>
        </Card>
      )}

      <div className="flex flex-col gap-2.5 sm:flex-row sm:justify-end">
        <Button variant="secondary" disabled={busy} onClick={onLeave}>
          К карточкам
        </Button>
        {summary.mistake_cards.length > 0 && (
          <Button variant="primary" disabled={busy} onClick={onRepeat}>
            Повторить ошибки
          </Button>
        )}
      </div>
    </div>
  );
}

function Figure({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-1">
      <span className="text-11 tracking-label text-text-faint uppercase">
        {label}
      </span>
      <span className="font-mono text-24 tabular-nums text-text-1">
        {value}
      </span>
    </div>
  );
}
