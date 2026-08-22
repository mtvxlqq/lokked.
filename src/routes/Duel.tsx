import { useCallback, useEffect, useState } from "react";
import { useNavigate } from "react-router";

import { DeckReel } from "@/components/duel/DeckReel";
import { DuelResults } from "@/components/duel/DuelResults";
import {
  DuelSetupForm,
  type DuelChoice,
} from "@/components/duel/DuelSetupForm";
import { DuelStage } from "@/components/duel/DuelStage";
import { HandOver } from "@/components/duel/HandOver";
import { Screen } from "@/components/Screen";
import {
  duelAnswer,
  duelBeginTurn,
  duelCurrent,
  duelPickDeck,
  duelReveal,
  duelSettled,
  duelStart,
  duelStop,
  duelSummary,
  duelTimeout,
  errorMessage,
  listCards,
  listDecks,
  type Deck,
  type DuelSummary,
  type DuelView,
  type Grade,
} from "@/lib/tauri";

/** Клавиша — оценка. Тот же порядок, что и у кнопок. */
const GRADE_KEYS: Record<string, Grade> = {
  Digit1: "again",
  Digit2: "hard",
  Digit3: "good",
  Digit4: "easy",
};

/**
 * Дуэль: блиц на одном устройстве, по очереди.
 *
 * Экран проходит четыре состояния — настройка, передача устройства, ход и
 * итоги, — но решает всё бэкенд: чей ход, какая карточка выпала, сколько
 * осталось времени и кто победил. Здесь только показ и жесты.
 *
 * Барабан крутится дважды: один раз выбирает колоду, если её не выбрали
 * руками, и потом на каждой карточке хода. Часы карточки заводятся не в
 * момент раздачи, а когда барабан встал, — иначе прокрут съедал бы время.
 */
export function Duel() {
  const navigate = useNavigate();

  const [decks, setDecks] = useState<Deck[]>([]);
  const [picked, setPicked] = useState<Deck | null>(null);
  const [spinningDeck, setSpinningDeck] = useState(false);
  const [view, setView] = useState<DuelView | null>(null);
  const [summary, setSummary] = useState<DuelSummary | null>(null);
  const [labels, setLabels] = useState<string[]>([]);
  const [settledAt, setSettledAt] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  /** Ключ карточки, на которой барабан остановился. */
  const cardKey = view ? `${view.current_player}-${view.position}` : null;
  const spinning =
    view !== null &&
    !view.handover &&
    !view.finished &&
    !view.revealed &&
    cardKey !== settledAt;

  // Колоды нужны и форме, и барабану — как список названий для прокрутки.
  useEffect(() => {
    let cancelled = false;

    listDecks()
      .then((loaded) => {
        if (!cancelled) setDecks(loaded);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, []);

  // Дуэль продолжается, а не начинается заново: вернуться на экран — не то
  // же самое, что начать новую.
  useEffect(() => {
    let cancelled = false;

    duelCurrent()
      .then((running) => {
        if (!cancelled && running) setView(running);
      })
      .catch(() => {
        // Нечего продолжать — открывается настройка.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  // Лента барабана: лицевые стороны той же колоды, без ответов.
  const deckId = view?.deck_id ?? null;
  useEffect(() => {
    if (!deckId) return;
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
  }, [deckId]);

  // Дуэль дошла до конца — показываем таблицу.
  useEffect(() => {
    if (!view?.finished) return;
    let cancelled = false;

    duelSummary()
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
    void duelStop().finally(() => void navigate("/cards"));
  }, [navigate]);

  function spin() {
    setBusy(true);
    setError(null);

    duelPickDeck()
      .then((deck) => {
        setPicked(deck);
        setSpinningDeck(true);
      })
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }

  function start(choice: DuelChoice) {
    setBusy(true);
    setError(null);

    duelStart(choice)
      .then((next) => {
        setSummary(null);
        setView(next);
      })
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }

  function ready() {
    setBusy(true);
    duelBeginTurn()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }

  /** Барабан встал: с этого мгновения идёт время карточки. */
  const settle = useCallback(() => {
    if (!cardKey) return;
    setSettledAt(cardKey);

    duelSettled()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)));
  }, [cardKey]);

  const reveal = useCallback(() => {
    // По крутящемуся барабану жать нечего: карточка ещё не выпала.
    if (spinning) return;

    setBusy(true);
    duelReveal()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, [spinning]);

  const grade = useCallback((value: Grade) => {
    setBusy(true);
    duelAnswer(value)
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }, []);

  // Время карточки вышло. Решает всё равно бэкенд: он сверит отметку со
  // своими часами и запишет «не помню».
  const expire = useCallback(() => {
    duelTimeout()
      .then(setView)
      .catch((failure: unknown) => setError(errorMessage(failure)));
  }, []);

  function again() {
    if (!summary) return;

    start({
      deckId: summary.deck_id,
      players: summary.players.map((player) => player.name),
      cards: summary.cards,
      secondsPerCard: summary.seconds_per_card,
    });
  }

  function backToSetup() {
    setBusy(true);
    void duelStop().finally(() => {
      setView(null);
      setSummary(null);
      setPicked(null);
      setBusy(false);
    });
  }

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!view || view.finished) return;

      if (view.handover) {
        if (event.code === "Space" || event.code === "Enter") {
          event.preventDefault();
          ready();
        }
        return;
      }

      if (!view.revealed) {
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
  }, [view, reveal, grade]);

  if (spinningDeck && picked) {
    return (
      <DeckReel
        decks={decks}
        picked={picked}
        onDone={() => setSpinningDeck(false)}
      />
    );
  }

  if (view && !view.finished) {
    return view.handover ? (
      <HandOver view={view} busy={busy} onReady={ready} onLeave={leave} />
    ) : (
      <DuelStage
        view={view}
        labels={labels}
        spinning={spinning}
        busy={busy}
        onSettled={settle}
        onReveal={reveal}
        onGrade={grade}
        onExpire={expire}
        onLeave={leave}
        error={error}
      />
    );
  }

  // Настройка и итоги живут вне общей оболочки, как и сам ход: дуэль
  // занимает экран целиком, навигация в ней только мешает.
  return (
    <div className="mx-auto flex min-h-dvh w-full max-w-app flex-col px-5 py-6.5 sm:px-14 sm:py-11">
      <Screen
        title="Дуэль"
        actions={
          <span className="text-12.5 text-text-dim">
            Блиц по очереди на одном устройстве
          </span>
        }
      >
        {summary ? (
          <DuelResults
            summary={summary}
            onAgain={again}
            onChangeDeck={backToSetup}
            onLeave={leave}
          />
        ) : (
          <DuelSetupForm
            decks={decks}
            picked={picked}
            busy={busy}
            error={error}
            onSpin={spin}
            onStart={start}
            onLeave={leave}
          />
        )}
      </Screen>
    </div>
  );
}
