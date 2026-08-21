import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router";

import { useIdle } from "@/components/zen/useIdle";
import { ZenDisplay } from "@/components/zen/ZenDisplay";
import { cn } from "@/lib/cn";
import { setFullscreen } from "@/lib/fullscreen";
import {
  pauseSession,
  resumeSession,
  sessionSnapshot,
  stopSession,
  zenSettings,
  type SessionSnapshot,
  type ZenSettings,
} from "@/lib/tauri";

/** Как часто перечитывается состояние сессии — как на экране таймера. */
const TICK_MS = 250;

/** Свайп короче этого — просто касание, а не жест выхода. */
const SWIPE_PX = 60;

const DEFAULT_SETTINGS: ZenSettings = {
  minutes_only: false,
  font_size: "normal",
};

/**
 * Чёрный экран.
 *
 * Ничего, кроме времени и предмета: весь смысл экрана в том, чего на нём нет.
 * Время, как и везде, спрашивается у бэкенда, а не считается тиками, — свёрнутое
 * окно и уснувшая машина на него не влияют.
 *
 * Управление разное на разных устройствах, потому что разные устройства: на
 * десктопе клавиши (Esc — выход, пробел — пауза, Q — завершить), на телефоне
 * касание по центру и свайп вниз. Первое касание погасшего экрана только
 * возвращает свет — иначе поднести палец, чтобы посмотреть время, значило бы
 * поставить сессию на паузу.
 */
export function Zen() {
  const navigate = useNavigate();
  const dimmed = useIdle();

  const [session, setSession] = useState<SessionSnapshot | null>(null);
  const [settings, setSettings] = useState<ZenSettings>(DEFAULT_SETTINGS);

  /** Откуда начался текущий свайп, чтобы отличить жест от касания. */
  const touchStartY = useRef<number | null>(null);

  /**
   * Сессия для обработчиков. Снимок приходит четыре раза в секунду, и если бы
   * обработчики зависели от него напрямую, слушатель клавиш переподписывался
   * бы с той же частотой.
   */
  const current = useRef<SessionSnapshot | null>(null);
  useEffect(() => {
    current.current = session;
  }, [session]);

  // Разворачиваем окно на вход и возвращаем как было на выход. На мобилке
  // и в браузере вызов просто ничего не делает.
  useEffect(() => {
    void setFullscreen(true);
    return () => {
      void setFullscreen(false);
    };
  }, []);

  // Настройки читаем один раз: меняются они на другом экране, а сюда студент
  // приходит уже с готовым выбором.
  useEffect(() => {
    let cancelled = false;

    zenSettings()
      .then((loaded) => {
        if (!cancelled) setSettings(loaded);
      })
      .catch(() => {
        // Настройки — украшение экрана, а не его работа: не прочитались,
        // показываем со значениями по умолчанию.
      });

    return () => {
      cancelled = true;
    };
  }, []);

  // Сессия могла закончиться где угодно — в другом окне, по завершении
  // последней фазы; тогда на чёрном экране делать нечего.
  useEffect(() => {
    let cancelled = false;

    function poll() {
      sessionSnapshot()
        .then((next) => {
          if (cancelled) return;
          if (next) setSession(next);
          else void navigate("/", { replace: true });
        })
        .catch(() => {
          // Один неудавшийся опрос ничего не значит: следующий через 250 мс.
        });
    }

    poll();
    const id = setInterval(poll, TICK_MS);

    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [navigate]);

  /** Назад к экрану сессии — туда, откуда сюда обычно и приходят. */
  const leave = useCallback(() => {
    const active = current.current;
    void navigate(active ? `/timer/${active.subject_id}` : "/", {
      replace: true,
    });
  }, [navigate]);

  const togglePause = useCallback(() => {
    const active = current.current;
    if (!active) return;

    const action = active.status === "paused" ? resumeSession : pauseSession;
    action()
      .then(setSession)
      .catch(() => {
        // Отказ пришёл бы только от гонки с другим окном; следующий опрос
        // всё равно покажет настоящее состояние.
      });
  }, []);

  const finish = useCallback(() => {
    stopSession()
      .then(() => navigate("/", { replace: true }))
      .catch(() => {
        // Не остановилось — экран остаётся, и попробовать можно ещё раз.
      });
  }, [navigate]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      // По `code`, а не по `key`: на русской раскладке Q — это «й», и выход
      // по клавише не должен зависеть от того, что сейчас включено.
      switch (event.code) {
        case "Escape":
          leave();
          break;
        case "Space":
          // Иначе пробел прокрутит страницу или нажмёт кнопку под фокусом.
          event.preventDefault();
          togglePause();
          break;
        case "KeyQ":
          finish();
          break;
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [leave, togglePause, finish]);

  function onTouchStart(event: React.TouchEvent) {
    touchStartY.current = event.touches[0]?.clientY ?? null;
  }

  function onTouchEnd(event: React.TouchEvent) {
    const start = touchStartY.current;
    touchStartY.current = null;
    if (start === null) return;

    const end = event.changedTouches[0]?.clientY;
    if (end !== undefined && end - start > SWIPE_PX) leave();
  }

  return (
    <div
      onTouchStart={onTouchStart}
      onTouchEnd={onTouchEnd}
      className={cn(
        "flex min-h-screen flex-col items-center justify-center overflow-hidden px-4",
        dimmed ? "halo-zen-dim cursor-none" : "halo-zen",
      )}
    >
      {session && (
        <button
          type="button"
          // Погасший экран сначала возвращает свет: нажатие в темноте — это
          // «покажи», а не «останови».
          onClick={() => {
            if (!dimmed) togglePause();
          }}
          aria-label={
            session.status === "paused" ? "Продолжить сессию" : "Пауза"
          }
          className="flex flex-col items-center focus-visible:outline-2 focus-visible:outline-offset-8 focus-visible:outline-text-zen-dim-2"
        >
          <ZenDisplay
            seconds={session.session_seconds}
            subjectName={session.subject_name}
            minutesOnly={settings.minutes_only}
            fontSize={settings.font_size}
            dimmed={dimmed}
            paused={session.status === "paused"}
          />
        </button>
      )}

      <p className="sr-only">
        Esc — выйти из чёрного экрана, пробел — пауза, Q — завершить сессию. На
        телефоне: касание по центру — пауза, свайп вниз — выход.
      </p>
    </div>
  );
}
