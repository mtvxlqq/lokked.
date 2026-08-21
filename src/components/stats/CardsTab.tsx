import { AccuracyChart } from "@/components/stats/AccuracyChart";
import { LoadFrame } from "@/components/stats/LoadFrame";
import { ProblemCards } from "@/components/stats/ProblemCards";
import { StatTiles } from "@/components/stats/StatTiles";
import { useStatsData } from "@/components/stats/useStatsData";
import { Card, EmptyState } from "@/components/ui";
import { CardsIcon } from "@/components/nav/icons";
import { statsCards, type ProblemCard, type StatsRange } from "@/lib/tauri";

/** Запрос вкладки — вне компонента, чтобы эффект зависел только от периода. */
function load(range: string) {
  return statsCards(range as StatsRange);
}

type CardsTabProps = {
  range: StatsRange;
  /** Открыть историю карточки — по ней кликнули в списке проблемных. */
  onOpenCard: (card: ProblemCard) => void;
};

/**
 * Вкладка «Карточки»: сколько отвечено за период, с какой точностью и какие
 * карточки не даются.
 */
export function CardsTab({ range, onOpenCard }: CardsTabProps) {
  const { state, data, error, reload } = useStatsData(load, range);

  return (
    <LoadFrame state={state} error={error} onRetry={reload}>
      {data && (
        <div className="flex flex-col gap-5 sm:gap-6">
          <StatTiles
            tiles={[
              { label: "Ответов", value: String(data.answered) },
              { label: "Верно", value: String(data.correct) },
              {
                label: "Точность",
                value: data.answered === 0 ? "—" : `${data.accuracy_percent} %`,
              },
            ]}
          />

          {data.answered === 0 ? (
            <EmptyState
              icon={<CardsIcon className="size-8" />}
              title="За этот период карточек не было"
              description="Пройди колоду — точность и проблемные карточки появятся здесь."
            />
          ) : (
            <>
              <Card title="Точность по дням">
                <AccuracyChart days={data.by_day} />
              </Card>

              <Card title="Проблемные карточки" aside="худшие двадцать">
                {data.problems.length === 0 ? (
                  <p className="text-14 text-text-dim">
                    Пока не по чему судить: карточка попадает сюда после трёх
                    показов.
                  </p>
                ) : (
                  <ProblemCards
                    cards={data.problems}
                    onOpen={(cardId) => {
                      const card = data.problems.find(
                        (problem) => problem.card_id === cardId,
                      );
                      if (card) onOpenCard(card);
                    }}
                  />
                )}
              </Card>
            </>
          )}
        </div>
      )}
    </LoadFrame>
  );
}
