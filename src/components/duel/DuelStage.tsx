import { useEffect } from "react";

import { CardText } from "@/components/cards/CardText";
import { BlitzRing } from "@/components/study/BlitzRing";
import { GradeBar } from "@/components/study/GradeBar";
import { ReelSpinner } from "@/components/study/ReelSpinner";
import { Button } from "@/components/ui";
import { setFullscreen } from "@/lib/fullscreen";
import type { DuelView, Grade } from "@/lib/tauri";

type DuelStageProps = {
  view: DuelView;
  /** Надписи колоды для прокрутки — размеченные, как в самой карточке. */
  labels: string[];
  /** Барабан ещё крутится: ни ответа, ни часов. */
  spinning: boolean;
  busy: boolean;
  onSettled: () => void;
  onReveal: () => void;
  onGrade: (grade: Grade) => void;
  onExpire: () => void;
  onLeave: () => void;
  error: string | null;
};

/**
 * Ход дуэли: тот же барабан, что и в одиночном режиме, только сверху видно,
 * чей ход и сколько он уже набрал.
 *
 * Часы заводятся, когда барабан встал, а не когда карточка выпала: время
 * должно уходить на припоминание, а не на анимацию.
 */
export function DuelStage({
  view,
  labels,
  spinning,
  busy,
  onSettled,
  onReveal,
  onGrade,
  onExpire,
  onLeave,
  error,
}: DuelStageProps) {
  // Разворачиваем окно на вход и возвращаем как было на выход — как на
  // чёрном экране и в барабане: на экране только дуэль.
  useEffect(() => {
    void setFullscreen(true);
    return () => {
      void setFullscreen(false);
    };
  }, []);

  const card = view.card;
  if (!card) return null;

  return (
    <div className="flex h-dvh flex-col bg-bg-zen px-4 py-4 sm:px-8 sm:py-6">
      <header className="flex shrink-0 flex-wrap items-center justify-between gap-x-4 gap-y-2">
        <button
          type="button"
          onClick={onLeave}
          className="min-h-11 text-13 text-text-zen-dim-2 transition-colors duration-300 ease-standard hover:text-text-zen-dim focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          ← {view.current_name} · ход {view.turn} из {view.turns}
        </button>

        <div className="flex items-center gap-4">
          <span className="font-mono text-15.5 tabular-nums text-text-zen-dim">
            {view.points}
          </span>
          {view.deadline && !spinning && (
            <BlitzRing
              key={view.deadline}
              deadline={view.deadline}
              seconds={view.seconds_per_card}
              onExpire={onExpire}
            />
          )}
          <span className="font-mono text-12.5 tabular-nums text-text-zen-dim-2">
            {view.position} / {view.total}
          </span>
        </div>
      </header>

      <main className="flex flex-1 flex-col overflow-y-auto">
        <div className="m-auto flex w-full flex-col items-center gap-8 py-2">
          {view.revealed ? (
            <div className="flex w-full max-w-2xl flex-col gap-6">
              <CardText
                text={card.front}
                className="text-center text-22 text-text sm:text-30"
              />
              <span className="h-px w-full bg-border-soft" aria-hidden="true" />
              <CardText
                text={card.back ?? ""}
                className="text-center text-16 text-text-4"
              />
            </div>
          ) : (
            <>
              <ReelSpinner
                labels={labels}
                target={card.front}
                spinKey={`${view.current_player}-${view.position}`}
                onSettled={onSettled}
              />

              {card.hint && !spinning && (
                <span className="text-center text-13 text-text-zen-dim-2">
                  {card.hint}
                </span>
              )}
            </>
          )}
        </div>
      </main>

      <footer className="flex shrink-0 flex-col items-center gap-3 pb-2">
        {view.revealed ? (
          <div className="w-full max-w-2xl">
            <GradeBar onGrade={onGrade} disabled={busy} />
          </div>
        ) : (
          <div className="flex min-h-11 flex-col items-center gap-3">
            {!spinning && (
              <>
                <p className="text-12.5 text-text-zen-dim-2">
                  Вспомни ответ, затем пробел
                </p>
                <Button variant="secondary" disabled={busy} onClick={onReveal}>
                  Показать ответ
                </Button>
              </>
            )}
          </div>
        )}

        {error && (
          <p className="text-center text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </footer>
    </div>
  );
}
