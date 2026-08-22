import { useCallback, useEffect, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router";

import { CardText } from "@/components/cards/CardText";
import { BlitzRing } from "@/components/study/BlitzRing";
import { ReelStage } from "@/components/study/ReelStage";
import { GradeBar } from "@/components/study/GradeBar";
import { RunShell as Shell } from "@/components/study/RunShell";
import { RunSummary } from "@/components/study/RunSummary";
import { Button } from "@/components/ui";
import {
  errorMessage,
  listCards,
  studyAnswer,
  studyCurrent,
  studyRepeatMistakes,
  studyReveal,
  studyStart,
  studyStop,
  studySummary,
  studyTimeout,
  type Grade,
  type StudyMode,
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
  const [params] = useSearchParams();
  const navigate = useNavigate();

  const mode = (params.get("mode") ?? "classic") as StudyMode;

  const [view, setView] = useState<StudyView | null>(null);
  const [summary, setSummary] = useState<StudySummary | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  /**
   * Надписи колоды для барабана — только лицевые стороны, без ответов.
   * Разметка сохраняется: формулы в ленте рисует KaTeX.
   */
  const [labels, setLabels] = useState<string[]>([]);
  /**
   * Номер карточки, на которой барабан уже остановился.
   *
   * Состояние выражено так, а не флагом «крутится»: флаг пришлось бы
   * поднимать в эффекте на каждую новую карточку, а это лишний каскад
   * перерисовок. Здесь же «крутится» — вывод из того, что позиция сменилась,
   * а барабан об остановке ещё не сообщил.
   */
  const [settledAt, setSettledAt] = useState<number | null>(null);

  const position = view?.position ?? null;
  const spinning =
    mode === "reel" &&
    view !== null &&
    !view.revealed &&
    position !== settledAt;

  // Прогон продолжается, а не начинается заново: вернуться на экран — не то же
  // самое, что взять колоду сначала.
  useEffect(() => {
    if (!deckId) return;
    let cancelled = false;

    studyCurrent()
      .then((running) =>
        running &&
        running.deck_id === deckId &&
        running.mode === mode &&
        !running.finished
          ? running
          : studyStart(deckId, mode),
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
  }, [deckId, mode]);

  // Лента барабана: лицевые стороны той же колоды. Оборотов среди них нет —
  // подсмотреть ответ в проносящихся мимо надписях нельзя.
  useEffect(() => {
    if (!deckId || mode !== "reel") return;
    let cancelled = false;

    listCards(deckId)
      .then((cards) => {
        if (!cancelled) setLabels(cards.map((card) => card.front));
      })
      .catch(() => {
        // Барабан переживёт: без ленты он покажет одну выпавшую надпись.
      });

    return () => {
      cancelled = true;
    };
  }, [deckId, mode]);

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
    // По крутящемуся барабану жать нечего: карточка ещё не выпала.
    if (spinning) return;

    setBusy(true);
    studyReveal()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, [spinning]);

  /** Барабан встал на этой карточке. Стабилен, пока карточка не сменится. */
  const settle = useCallback(() => setSettledAt(position), [position]);

  const grade = useCallback((value: Grade) => {
    setBusy(true);
    studyAnswer(value)
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, []);

  // Время карточки вышло. Решает всё равно бэкенд: он сверит отметку со своими
  // часами и запишет «не помню».
  const expire = useCallback(() => {
    studyTimeout()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)));
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
      <Shell onLeave={leave} deckName={summary.deck_name} mode={summary.mode}>
        <RunSummary
          summary={summary}
          onRepeat={repeat}
          onLeave={leave}
          busy={busy}
        />
      </Shell>
    );
  }

  if (mode === "reel" && view?.card) {
    return (
      <ReelStage
        view={view}
        labels={labels}
        spinning={spinning}
        busy={busy}
        onSettled={settle}
        onReveal={reveal}
        onGrade={grade}
        onLeave={leave}
        error={error}
      />
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
      mode={view.mode}
      progress={`${view.position} / ${view.total}`}
      answered={view.answered}
      total={view.total}
      aside={
        view.deadline && view.seconds_per_card ? (
          <BlitzRing
            key={view.deadline}
            deadline={view.deadline}
            seconds={view.seconds_per_card}
            onExpire={expire}
          />
        ) : null
      }
      score={
        view.points !== null ? (
          <span className="font-mono text-15 tabular-nums text-text-1">
            {view.points}
            {view.streak !== null && view.streak >= 5 && (
              <span className="ml-2 text-12.5 text-accent-text">
                ×{view.streak >= 10 ? 2 : 1.5}
              </span>
            )}
          </span>
        ) : null
      }
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
