import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router";

import { Screen } from "@/components/Screen";
import { AwayDialog } from "@/components/timer/AwayDialog";
import { PhaseRing } from "@/components/timer/PhaseRing";
import { SessionControls } from "@/components/timer/SessionControls";
import { Button, Card } from "@/components/ui";
import { chime, notify } from "@/lib/announce";
import { formatClock } from "@/lib/format";
import {
  discardAway,
  errorMessage,
  markInterruption,
  pauseSession,
  reportReturn,
  resumeSession,
  sessionSnapshot,
  skipPhase,
  startSession,
  stopSession,
  type SessionPhase,
  type SessionSnapshot,
} from "@/lib/tauri";

/** Как часто перечитывается состояние сессии. */
const TICK_MS = 250;

const PHASE_LABELS: Record<SessionPhase, string> = {
  work: "Работа",
  break: "Перерыв",
  long_break: "Длинный перерыв",
};

/** Подпись под цифрами: «Работа 2/4» у помодоро, просто фаза у остальных. */
function phaseCaption(session: SessionSnapshot): string {
  const label = PHASE_LABELS[session.phase];

  if (session.mode !== "pomodoro" || session.phase !== "work") return label;
  if (session.cycles_before_long === null) return label;

  return `${label} ${session.cycle}/${session.cycles_before_long}`;
}

/** Что показывают крупные цифры: остаток, если он есть, иначе набранное время. */
function bigNumber(session: SessionSnapshot): number {
  return session.remaining_seconds ?? session.elapsed_seconds;
}

function progress(session: SessionSnapshot): number {
  if (!session.target_seconds) return 0;
  return session.elapsed_seconds / session.target_seconds;
}

/**
 * Экран активной сессии.
 *
 * Своего счётчика у экрана нет: раз в 250 мс он спрашивает бэкенд, что
 * сейчас, и рисует ответ. Поэтому свёрнутое на час окно, уснувшая машина или
 * выгруженное из памяти приложение не сбивают время — оно всё равно
 * вычисляется из отметок начала и пауз.
 */
export function Timer() {
  const { subjectId } = useParams<{ subjectId: string }>();
  const navigate = useNavigate();

  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [away, setAway] = useState<{ seconds: number; since: Date } | null>(
    null,
  );

  /** Фаза, о которой уже сообщили, — чтобы не звенеть на каждом тике. */
  const announced = useRef<string | null>(null);
  /** Когда экран последний раз был на виду. */
  const lastSeen = useRef<Date>(new Date());

  const apply = useCallback((next: SessionSnapshot) => {
    const mark = `${next.phase}-${next.cycle}`;

    if (announced.current !== null && announced.current !== mark) {
      chime();
      void notify(
        "Lokked",
        `${PHASE_LABELS[next.phase]} — ${next.subject_name}`,
      );
    }
    announced.current = mark;
    setSession(next);
  }, []);

  // Первый заход: показываем уже идущую сессию или начинаем новую. Сессия
  // одна на всё приложение, поэтому чужой предмет в адресе — повод уйти
  // к тому, что действительно идёт.
  useEffect(() => {
    if (!subjectId) return;
    let cancelled = false;

    sessionSnapshot()
      .then((existing) => {
        if (cancelled) return null;
        if (existing && existing.subject_id !== subjectId) {
          void navigate(`/timer/${existing.subject_id}`, { replace: true });
          return null;
        }
        return existing ?? startSession(subjectId);
      })
      .then((next) => {
        if (!cancelled && next) apply(next);
      })
      .catch((failure: unknown) => {
        if (!cancelled) setError(errorMessage(failure));
      });

    return () => {
      cancelled = true;
    };
  }, [subjectId, navigate, apply]);

  // Тик. Сессия могла закончиться в другом окне — тогда возвращаемся к списку.
  useEffect(() => {
    const id = setInterval(() => {
      sessionSnapshot()
        .then((next) => {
          if (next) apply(next);
          else void navigate("/");
        })
        .catch((failure: unknown) => setError(errorMessage(failure)));
    }, TICK_MS);

    return () => clearInterval(id);
  }, [apply, navigate]);

  // Возвращение из фона: спрашиваем бэкенд, было ли отсутствие достаточно
  // долгим, чтобы вообще о нём говорить.
  useEffect(() => {
    function onVisibility() {
      if (document.hidden) {
        lastSeen.current = new Date();
        return;
      }

      const since = lastSeen.current;
      reportReturn(since)
        .then((report) => {
          if (report.needs_decision) {
            setAway({ seconds: report.away_seconds, since });
          }
        })
        .catch((failure: unknown) => setError(errorMessage(failure)));
    }

    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  /** Выполняет действие сессии, показывая отказ вместо того, чтобы его глотать. */
  function run(action: () => Promise<SessionSnapshot>) {
    setBusy(true);
    action()
      .then(apply)
      .catch((failure: unknown) => setError(errorMessage(failure)))
      .finally(() => setBusy(false));
  }

  function stop() {
    setBusy(true);
    stopSession()
      .then(() => navigate("/"))
      .catch((failure: unknown) => {
        setError(errorMessage(failure));
        setBusy(false);
      });
  }

  if (!session) {
    return (
      <Screen title="Сессия">
        <Card title={error ? "Не удалось начать сессию" : "Готовим таймер"}>
          {error ? (
            <>
              <p className="text-14 text-danger-text" role="alert">
                {error}
              </p>
              <div>
                <Button variant="secondary" onClick={() => void navigate("/")}>
                  К предметам
                </Button>
              </div>
            </>
          ) : (
            <p className="text-14 text-text-dim">Загрузка…</p>
          )}
        </Card>
      </Screen>
    );
  }

  return (
    <Screen title={session.subject_name}>
      <div className="flex flex-col gap-7">
        <PhaseRing
          progress={progress(session)}
          dimmed={session.status === "paused"}
        >
          <span className="font-mono text-40 tabular-nums text-text sm:text-58">
            {formatClock(bigNumber(session))}
          </span>
          <span className="text-12 tracking-label text-text-dim-2 uppercase">
            {phaseCaption(session)}
          </span>
          {session.status === "paused" && (
            <span className="text-12 tracking-label text-accent-text uppercase">
              Пауза
            </span>
          )}
        </PhaseRing>

        {session.phase_finished && !session.auto_start_next && (
          <p className="text-center text-14 text-accent-text">
            Фаза закончилась.
            {session.mode === "pomodoro"
              ? " Можно переходить дальше."
              : " Время вышло — но счёт идёт, пока не нажмёшь «Стоп»."}
          </p>
        )}

        <SessionControls
          session={session}
          busy={busy}
          onPause={() => run(pauseSession)}
          onResume={() => run(resumeSession)}
          onInterruption={() => run(markInterruption)}
          onSkip={() => run(skipPhase)}
          onStop={stop}
          onZen={() => void navigate("/zen")}
        />

        {session.interruptions > 0 && (
          <p className="text-center text-12.5 text-text-dim">
            Отвлекался: {session.interruptions}
          </p>
        )}

        {error && (
          <p className="text-center text-13 text-danger-text" role="alert">
            {error}
          </p>
        )}
      </div>

      {away && (
        <AwayDialog
          awaySeconds={away.seconds}
          onKeep={() => setAway(null)}
          onDiscard={() => {
            const since = away.since;
            setAway(null);
            run(() => discardAway(since));
          }}
        />
      )}
    </Screen>
  );
}
