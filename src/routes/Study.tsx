import { useCallback, useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";

import { CardText } from "@/components/cards/CardText";
import { GradeBar } from "@/components/study/GradeBar";
import { RunSummary } from "@/components/study/RunSummary";
import { Button } from "@/components/ui";
import {
  errorMessage,
  studyAnswer,
  studyCurrent,
  studyRepeatMistakes,
  studyReveal,
  studyStart,
  studyStop,
  studySummary,
  type Grade,
  type StudySummary,
  type StudyView,
} from "@/lib/tauri";

/** Клавиша — оценка. Порядок тот же, что и у кнопок. */
const GRADE_KEYS: Record<string, Grade> = {
  Digit1: "again",
  Digit2: "hard",
  Digit3: "good",
  Digit4: "easy",
};

/**
 * Классический прогон по колоде.
 *
 * Экран ничего не решает: какая карточка следующая, засчитан ли ответ и
 * сколько времени он занял — всё это знает бэкенд, где прогон и живёт. Сюда
 * приходит готовое состояние, отсюда уходят «покажи ответ» и «вот оценка».
 *
 * Оборот не загружается заранее: до раскрытия его на экране нет вообще,
 * поэтому «время до ответа» измеряет именно припоминание.
 */
export function Study() {
  const { deckId } = useParams<{ deckId: string }>();
  const navigate = useNavigate();

  const [view, setView] = useState<StudyView | null>(null);
  const [summary, setSummary] = useState<StudySummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  // Прогон продолжается, а не начинается заново: вернуться на экран — не то же
  // самое, что взять колоду сначала.
  useEffect(() => {
    if (!deckId) return;
    let cancelled = false;

    studyCurrent()
      .then((running) =>
        running && running.deck_id === deckId && !running.finished
          ? running
          : studyStart(deckId),
      )
      .then((next) => {
        if (!cancelled) setView(next);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [deckId]);

  // Прогон дошёл до конца — показываем итоги.
  useEffect(() => {
    if (!view?.finished) return;
    let cancelled = false;

    studySummary()
      .then((result) => {
        if (!cancelled) setSummary(result);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [view?.finished]);

  const leave = useCallback(() => {
    void studyStop().finally(() => void navigate("/cards"));
  }, [navigate]);

  const reveal = useCallback(() => {
    setBusy(true);
    studyReveal()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, []);

  const grade = useCallback((value: Grade) => {
    setBusy(true);
    studyAnswer(value)
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, []);

  function repeat() {
    setBusy(true);
    studyRepeatMistakes()
      .then((next) => {
        setSummary(null);
        setView(next);
      })
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.code === "Escape") {
        leave();
        return;
      }
      if (!view || view.finished) return;

      if (!view.revealed) {
        // Пробел — главный жест прогона: проговорил вслух, нажал, сверился.
        if (event.code === "Space" || event.code === "Enter") {
          event.preventDefault();
          reveal();
        }
        return;
      }

      const chosen = GRADE_KEYS[event.code];
      if (chosen) {
        event.preventDefault();
        grade(chosen);
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [view, leave, reveal, grade]);

  if (error && !view) {
    return (
      <Shell onLeave={leave}>
        <p className="text-14 text-danger-text" role="alert">
          {error}
        </p>
      </Shell>
    );
  }

  if (summary) {
    return (
      <Shell onLeave={leave} deckName={summary.deck_name}>
        <RunSummary
          summary={summary}
          onRepeat={repeat}
          onLeave={leave}
          busy={busy}
        />
      </Shell>
    );
  }

  if (!view?.card) {
    return (
      <Shell onLeave={leave}>
        <p className="text-14 text-text-dim">Готовим карточки…</p>
      </Shell>
    );
  }

  const card = view.card;

  return (
    <Shell
      onLeave={leave}
      deckName={view.deck_name}
      progress={`${view.position} / ${view.total}`}
    >
      <div className="flex w-full flex-col gap-5">
        <button
          type="button"
          onClick={() => !view.revealed && reveal()}
          aria-label={
            view.revealed
              ? "Карточка с ответом"
              : "Карточка, нажми, чтобы увидеть ответ"
          }
          className="flex min-h-40 w-full flex-col justify-center gap-4 rounded-2xl border border-border bg-surface px-5 py-7 text-left focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent sm:min-h-56 sm:gap-5 sm:px-10 sm:py-10"
        >
          {card.tags.length > 0 && (
            <span className="text-11 tracking-label text-text-faint uppercase">
              {card.tags[0]}
            </span>
          )}

          <CardText
            text={card.front}
            className="text-center text-22 text-text sm:text-30"
          />

          {card.back && (
            <>
              <span className="h-px w-full bg-border" aria-hidden="true" />
              <CardText text={card.back} className="text-15.5 text-text-2" />
            </>
          )}

          {!view.revealed && card.hint && (
            <span className="text-center text-13 text-text-dim-2">
              {card.hint}
            </span>
          )}
        </button>

        {view.revealed ? (
          <GradeBar onGrade={grade} disabled={busy} />
        ) : (
          <div className="flex flex-col items-center gap-3">
            <p className="text-13 text-text-dim">
              Проговори формулировку вслух, затем пробел
            </p>
            <Button variant="secondary" disabled={busy} onClick={reveal}>
              Показать ответ
            </Button>
          </div>
        )}

        {error && (
          <p className="text-center text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </div>
    </Shell>
  );
}

/** Рамка экрана: колода и счётчик сверху, содержимое по центру. */
function Shell({
  onLeave,
  deckName,
  progress,
  children,
}: {
  onLeave: () => void;
  deckName?: string;
  progress?: string;
  children: React.ReactNode;
}) {
  return (
    // Высота окна бывает меньше карточки: тогда экран должен прокручиваться,
    // а не обрезать кнопку снизу. `m-auto` на содержимом центрирует его, пока
    // место есть, и перестаёт, когда места нет.
    <div className="flex h-dvh flex-col gap-5 px-4 py-4 sm:px-8 sm:py-6">
      <header className="flex shrink-0 items-center justify-between gap-4">
        <button
          type="button"
          onClick={onLeave}
          className="min-h-11 text-13.5 text-text-dim focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
        >
          ← {deckName ? `${deckName} · Классика` : "К карточкам"}
        </button>
        {progress && (
          <span className="font-mono text-13 tabular-nums text-text-dim-2">
            {progress}
          </span>
        )}
      </header>

      <main className="flex flex-1 flex-col overflow-y-auto">
        <div className="m-auto flex w-full max-w-2xl flex-col items-center py-2">
          {children}
        </div>
      </main>
    </div>
  );
}
