import { useEffect } from "react";

import { CardText } from "@/components/cards/CardText";
import { GradeBar } from "@/components/study/GradeBar";
import { ReelSpinner } from "@/components/study/ReelSpinner";
import { Button } from "@/components/ui";
import { setFullscreen } from "@/lib/fullscreen";
import { plainText } from "@/lib/markdown";
import type { Grade, StudyView } from "@/lib/tauri";

type ReelStageProps = {
  view: StudyView;
  /** Надписи колоды для прокрутки — уже без разметки. */
  labels: string[];
  /** Барабан ещё крутится: раскрывать нечего. */
  spinning: boolean;
  busy: boolean;
  onSettled: () => void;
  onReveal: () => void;
  onGrade: (grade: Grade) => void;
  onLeave: () => void;
  error: string | null;
};

/**
 * Барабан: чёрный экран, мимо проносятся карточки колоды, одна из них
 * выпадает.
 *
 * Смысл тот же, что у чёрного экрана: на экране нет ничего, кроме того, о чём
 * сейчас думаешь. Поэтому здесь нет ни навигации, ни полосы прогресса — номер
 * карточки и выход спрятаны в приглушённую строку сверху.
 *
 * Что выпадет, решает бэкенд, как и в любом другом режиме. Барабан — способ
 * это показать, а не способ выбрать.
 */
export function ReelStage({
  view,
  labels,
  spinning,
  busy,
  onSettled,
  onReveal,
  onGrade,
  onLeave,
  error,
}: ReelStageProps) {
  // Разворачиваем окно на вход и возвращаем как было на выход — как на
  // чёрном экране: барабан живёт по тем же правилам, на экране только он.
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
      <header className="flex shrink-0 items-center justify-between gap-4">
        <button
          type="button"
          onClick={onLeave}
          className="min-h-11 text-13 text-text-zen-dim-2 transition-colors duration-300 ease-standard hover:text-text-zen-dim focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          ← {view.deck_name} · Барабан
        </button>

        <span className="font-mono text-12.5 tabular-nums text-text-zen-dim-2">
          {view.position} / {view.total}
        </span>
      </header>

      <main className="flex flex-1 flex-col items-center justify-center gap-8">
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
              target={plainText(card.front)}
              spinKey={String(view.position)}
              onSettled={onSettled}
            />

            {card.hint && !spinning && (
              <span className="text-center text-13 text-text-zen-dim-2">
                {card.hint}
              </span>
            )}
          </>
        )}
      </main>

      <footer className="flex shrink-0 flex-col items-center gap-3 pb-2">
        {view.revealed ? (
          <div className="w-full max-w-2xl">
            <GradeBar onGrade={onGrade} disabled={busy} />
          </div>
        ) : (
          // Кнопка появляется, только когда барабан встал: жать по крутящейся
          // ленте нечего, а мигающая кнопка отвлекала бы от неё.
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
