import { Button, Card } from "@/components/ui";
import { CardLine } from "@/components/cards/CardText";
import { cn } from "@/lib/cn";
import type { DuelSummary, Grade } from "@/lib/tauri";

/** Как называется оценка в разборе. */
const GRADE_NAMES: Record<Grade, string> = {
  again: "Не помню",
  hard: "С трудом",
  good: "Знаю",
  easy: "Легко",
};

type DuelResultsProps = {
  summary: DuelSummary;
  onAgain: () => void;
  onChangeDeck: () => void;
  onLeave: () => void;
};

/**
 * Итоги дуэли: счёт каждого и разбор по карточкам — кто что знал.
 *
 * Здесь и только здесь чужие результаты становятся видны: до последнего хода
 * они прятались, иначе играть по очереди не имело бы смысла.
 */
export function DuelResults({
  summary,
  onAgain,
  onChangeDeck,
  onLeave,
}: DuelResultsProps) {
  return (
    <div className="flex flex-col gap-2.5">
      <div className="grid gap-2.5 sm:grid-cols-2">
        {summary.players.map((player, index) => (
          <Card
            key={player.name}
            role="group"
            aria-label={player.name}
            className={cn(player.winner && "border-border-accent")}
          >
            <div className="flex items-baseline justify-between gap-3">
              <span className="flex items-baseline gap-2.5">
                <span className="font-mono text-12.5 text-text-faint tabular-nums">
                  {index + 1}
                </span>
                <span className="text-15.5 text-text-1">{player.name}</span>
              </span>
              <span className="font-mono text-30 leading-none tabular-nums text-text sm:text-34">
                {player.points}
              </span>
            </div>
            <p className="text-12.5 text-text-dim">
              {player.winner ? "победа · " : ""}
              {player.correct} из {player.answered} · серия ×
              {player.best_streak}
            </p>
          </Card>
        ))}
      </div>

      <Card title="Разбор по карточкам" aside={`${summary.cards} карточек`}>
        <div className="overflow-x-auto">
          <table className="w-full min-w-md border-collapse text-left">
            <thead>
              <tr className="text-11 tracking-label text-text-faint uppercase">
                <th scope="col" className="py-2 pr-4 font-normal">
                  Карточка
                </th>
                {summary.players.map((player) => (
                  <th
                    key={player.name}
                    scope="col"
                    className="py-2 pr-4 font-normal"
                  >
                    {player.name}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {summary.breakdown.map((row) => (
                <tr key={row.card_id} className="border-t border-border-soft">
                  <th scope="row" className="py-2.5 pr-4 font-normal">
                    <CardLine
                      text={row.front}
                      className="text-14 text-text-2"
                    />
                  </th>
                  {row.answers.map((grade, index) => (
                    <td
                      key={index}
                      className={cn(
                        "py-2.5 pr-4 text-13",
                        grade === "again" ? "text-danger-text" : "text-text-3",
                      )}
                    >
                      {grade ? GRADE_NAMES[grade] : "—"}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>

      <div className="flex flex-wrap gap-2.5">
        <Button onClick={onAgain}>Ещё раз</Button>
        <Button variant="secondary" onClick={onChangeDeck}>
          Сменить колоду
        </Button>
        <Button variant="ghost" onClick={onLeave}>
          Выйти
        </Button>
      </div>
    </div>
  );
}
