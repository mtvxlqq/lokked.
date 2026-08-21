import { useEffect, useState } from "react";
import { Outlet, useNavigate } from "react-router";
import { listen } from "@tauri-apps/api/event";

import { WokeDialog } from "@/components/timer/WokeDialog";
import {
  cliPendingZen,
  resumeSession,
  WOKE_EVENT,
  ZEN_EVENT,
  type WokeUp,
} from "@/lib/tauri";

/**
 * Мост между рабочим столом и экранами.
 *
 * Живёт корнем маршрутов, выше и `AppShell`, и чёрного экрана: горячая
 * клавиша может прийти на любом экране, а увести должна на нужный.
 *
 * Ошибки подписки глотаются намеренно: во фронтенде, открытом просто в
 * браузере (`npm run dev`), никакого Tauri нет, и это законный режим
 * работы — приложение должно рисоваться, а не падать.
 */
export function DesktopEvents() {
  const navigate = useNavigate();
  const [woke, setWoke] = useState<WokeUp | null>(null);

  useEffect(() => {
    const subscriptions = [
      listen(ZEN_EVENT, () => void navigate("/zen")),
      listen<WokeUp>(WOKE_EVENT, (event) => setWoke(event.payload)),
    ];

    return () => {
      for (const subscription of subscriptions) {
        subscription.then((unlisten) => unlisten()).catch(() => {});
      }
    };
  }, [navigate]);

  // Запуск вида `lokked --zen`, когда приложения ещё не было: команда
  // ждала, пока появится кому её показать.
  useEffect(() => {
    cliPendingZen()
      .then((pending) => {
        if (pending) void navigate("/zen");
      })
      .catch(() => {});
  }, [navigate]);

  return (
    <>
      <Outlet />

      {woke && (
        <WokeDialog
          asleepSeconds={woke.asleep_seconds}
          onResume={() => {
            setWoke(null);
            void resumeSession().catch(() => {});
          }}
          onKeepPaused={() => setWoke(null)}
        />
      )}
    </>
  );
}
